//! Tests for opening the AWS console.

mod common;

use alfred_granted::console;
use common::FakeRunner;

const CONSOLE_URL: &str =
    "https://eu-west-1.signin.aws.amazon.com/federation?Action=login&Destination=abc";

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
        .with_success("open", "");

    let url = console::open_console(&runner, "prod-admin").expect("should succeed");

    assert_eq!(url, CONSOLE_URL);
    assert_eq!(runner.programs_called(), vec!["assumego", "open"]);

    let calls = runner.calls.borrow();
    // assumego is called in console mode with the wrapper's environment so it
    // prints its structured output (including the URL) to stdout.
    assert_eq!(calls[0].args, vec!["prod-admin", "-c"]);
    assert_eq!(
        calls[0].envs,
        vec![("GRANTED_ALIAS_CONFIGURED".to_string(), "true".to_string())]
    );
    // open receives the extracted URL.
    assert_eq!(calls[1].program, "open");
    assert_eq!(calls[1].args, vec![CONSOLE_URL]);
}

#[test]
fn open_console_fails_when_no_url_returned() {
    let runner = FakeRunner::new().with_success("assumego", "no url in this output");

    let result = console::open_console(&runner, "prod-admin");

    assert!(result.is_err());
    // `open` must not be called when there is no URL to open.
    assert_eq!(runner.programs_called(), vec!["assumego"]);
}

#[test]
fn open_console_propagates_assumego_failure() {
    let runner = FakeRunner::new().with_failure("assumego", "sso session expired");

    assert!(console::open_console(&runner, "prod-admin").is_err());
}
