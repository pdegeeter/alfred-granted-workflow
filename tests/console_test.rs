//! Tests for opening the AWS console.

mod common;

use std::time::Duration;

use alfred_granted::{console, sso};
use common::FakeRunner;

const CONSOLE_URL: &str =
    "https://eu-west-1.signin.aws.amazon.com/federation?Action=login&Destination=abc";

const LOGIN_URL: &str = "https://d-1234567890.awsapps.com/start/#/device?user_code=ABCD-EFGH";

const OPEN: &str = "/usr/bin/open";

const TIMEOUT: Duration = Duration::from_secs(300);

/// The stderr lines granted 0.40 prints when the SSO session has expired.
fn sso_login_stderr() -> [String; 4] {
    [
        format!("[i] If the browser does not open automatically, please open this link: {LOGIN_URL}"),
        "[i] Awaiting AWS authentication in the browser".to_owned(),
        "[i] You will be prompted to authenticate with AWS in the browser, then you will be prompted to 'Allow'".to_owned(),
        "[i] Code: ABCD-EFGH".to_owned(),
    ]
}

#[test]
fn extract_url_finds_bare_url() {
    assert_eq!(
        console::extract_url(CONSOLE_URL),
        Some(CONSOLE_URL.to_string())
    );
}

#[test]
fn extract_url_ignores_surrounding_log_lines() {
    let output = format!("[granted] opening console\n{CONSOLE_URL}\ndone\n");

    assert_eq!(console::extract_url(&output), Some(CONSOLE_URL.to_string()));
}

#[test]
fn extract_url_returns_none_when_absent() {
    assert_eq!(console::extract_url("no url here"), None);
    assert_eq!(console::extract_url(""), None);
}

#[test]
fn extract_url_finds_url_after_granted_output_flag() {
    // The exact shape assumego emits in console mode.
    let output = format!("GrantedOutput\n{CONSOLE_URL}\n");

    assert_eq!(console::extract_url(&output), Some(CONSOLE_URL.to_string()));
}

#[test]
fn open_console_gets_url_then_opens_it() {
    let runner = FakeRunner::new()
        .with_success("assumego", &format!("GrantedOutput\n{CONSOLE_URL}\n"))
        .with_success(OPEN, "");

    let url = console::open_console(&runner, "prod-admin", None, TIMEOUT).expect("should succeed");

    assert_eq!(url, CONSOLE_URL);
    assert_eq!(runner.programs_called(), vec!["assumego", OPEN]);

    let calls = runner.calls.borrow();
    // assumego is called in console mode with the wrapper's environment so it
    // prints its structured output (including the URL) to stdout, bounded by
    // the timeout.
    assert_eq!(calls[0].args, vec!["prod-admin", "-c"]);
    assert_eq!(
        calls[0].envs,
        vec![("GRANTED_ALIAS_CONFIGURED".to_string(), "true".to_string())]
    );
    assert_eq!(calls[0].timeout, Some(TIMEOUT));
    // open receives the extracted URL.
    assert_eq!(calls[1].args, vec![CONSOLE_URL]);
}

#[test]
fn open_console_with_service_passes_dash_s() {
    let runner = FakeRunner::new()
        .with_success("assumego", &format!("GrantedOutput\n{CONSOLE_URL}\n"))
        .with_success(OPEN, "");

    let url =
        console::open_console(&runner, "prod-admin", Some("ec2"), TIMEOUT).expect("should succeed");

    assert_eq!(url, CONSOLE_URL);
    assert_eq!(runner.programs_called(), vec!["assumego", OPEN]);

    let calls = runner.calls.borrow();
    // A service opens with `-s <alias>` instead of `-c`.
    assert_eq!(calls[0].args, vec!["prod-admin", "-s", "ec2"]);
    assert_eq!(
        calls[0].envs,
        vec![("GRANTED_ALIAS_CONFIGURED".to_string(), "true".to_string())]
    );
    assert_eq!(calls[1].args, vec![CONSOLE_URL]);
}

#[test]
fn open_console_fails_when_no_url_returned() {
    let runner = FakeRunner::new().with_success("assumego", "no url in this output");

    let result = console::open_console(&runner, "prod-admin", None, TIMEOUT);

    assert!(result.is_err());
    // `open` must not be called when there is no URL to open.
    assert_eq!(runner.programs_called(), vec!["assumego"]);
}

#[test]
fn open_console_propagates_assumego_failure() {
    let runner = FakeRunner::new().with_failure("assumego", "sso session expired");

    assert!(console::open_console(&runner, "prod-admin", None, TIMEOUT).is_err());
}

#[test]
fn open_console_opens_sso_login_link_then_resumes() {
    let stderr = sso_login_stderr();
    let stderr: Vec<&str> = stderr.iter().map(String::as_str).collect();
    let runner = FakeRunner::new()
        .with_stderr("assumego", &stderr)
        .with_success("assumego", &format!("GrantedOutput\n{CONSOLE_URL}\n"))
        .with_success(OPEN, "");

    let url = console::open_console(&runner, "prod-admin", Some("s3"), TIMEOUT)
        .expect("should succeed once the login is approved");

    assert_eq!(url, CONSOLE_URL);
    // The login link is opened while assumego waits, then the console URL it
    // prints once the login is approved: one assumego run, same profile and
    // service, no second selection.
    assert_eq!(runner.programs_called(), vec!["assumego", OPEN, OPEN]);

    let calls = runner.calls.borrow();
    assert_eq!(calls[0].args, vec!["prod-admin", "-s", "s3"]);
    assert_eq!(calls[1].args, vec![LOGIN_URL]);
    assert_eq!(calls[2].args, vec![CONSOLE_URL]);
}

#[test]
fn open_console_opens_the_login_link_once() {
    let line =
        format!("If the browser does not open automatically, please open this link: {LOGIN_URL}");
    let runner = FakeRunner::new()
        .with_stderr("assumego", &[&line, &line])
        .with_success("assumego", &format!("GrantedOutput\n{CONSOLE_URL}\n"))
        .with_success(OPEN, "");

    console::open_console(&runner, "prod-admin", None, TIMEOUT).expect("should succeed");

    assert_eq!(runner.programs_called(), vec!["assumego", OPEN, OPEN]);
}

#[test]
fn open_console_ignores_links_outside_the_login_line() {
    let runner = FakeRunner::new()
        .with_stderr(
            "assumego",
            &["see https://d-1234567890.awsapps.com/start for details"],
        )
        .with_success("assumego", &format!("GrantedOutput\n{CONSOLE_URL}\n"))
        .with_success(OPEN, "");

    console::open_console(&runner, "prod-admin", None, TIMEOUT).expect("should succeed");

    assert_eq!(runner.programs_called(), vec!["assumego", OPEN]);
}

#[test]
fn open_console_keeps_waiting_when_the_login_link_fails_to_open() {
    let stderr = sso_login_stderr();
    let stderr: Vec<&str> = stderr.iter().map(String::as_str).collect();
    // Every `open` fails, so the console URL cannot be opened either; what
    // matters is that assumego still ran to completion after the login link.
    let runner = FakeRunner::new()
        .with_stderr("assumego", &stderr)
        .with_success("assumego", &format!("GrantedOutput\n{CONSOLE_URL}\n"))
        .with_failure(OPEN, "no browser");

    let result = console::open_console(&runner, "prod-admin", None, TIMEOUT);

    assert!(result.is_err());
    assert_eq!(runner.programs_called(), vec!["assumego", OPEN, OPEN]);
}

#[test]
fn open_console_reports_login_not_approved_on_timeout() {
    let stderr = sso_login_stderr();
    let stderr: Vec<&str> = stderr.iter().map(String::as_str).collect();
    let runner = FakeRunner::new()
        .with_stderr("assumego", &stderr)
        .with_timeout("assumego")
        .with_success(OPEN, "");

    let err =
        console::open_console(&runner, "prod-admin", None, TIMEOUT).expect_err("should time out");

    let reason = err
        .chain()
        .find_map(|cause| cause.downcast_ref::<sso::LoginNotApproved>())
        .expect("the error should say the login was not approved");
    assert_eq!(reason.timeout, TIMEOUT);
    assert_eq!(
        reason.to_string(),
        "AWS SSO login not approved within 5 minutes"
    );
    // The console is never opened.
    assert_eq!(runner.programs_called(), vec!["assumego", OPEN]);
}

#[test]
fn open_console_timeout_without_login_is_not_a_login_error() {
    let runner = FakeRunner::new().with_timeout("assumego");

    let err =
        console::open_console(&runner, "prod-admin", None, TIMEOUT).expect_err("should time out");

    assert!(err
        .chain()
        .all(|cause| cause.downcast_ref::<sso::LoginNotApproved>().is_none()));
    assert_eq!(runner.programs_called(), vec!["assumego"]);
}
