//! Tests for [`SystemRunner::run_streaming`] against fake `assumego`
//! executables (shell scripts), so no real granted or AWS call is made.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use alfred_granted::runner::{CommandRunner, SystemRunner, TimedOut};

const CONSOLE_URL: &str = "https://eu-west-1.signin.aws.amazon.com/federation?Action=login";

const LOGIN_LINE: &str = "If the browser does not open automatically, please open this link: https://d-1234567890.awsapps.com/start/#/device?user_code=ABCD-EFGH";

/// Write an executable `assumego` script running `body` into a directory of
/// its own, and return its path.
fn fake_assumego(test: &str, body: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("alfred-granted-{}-{test}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create the test directory");

    let path = dir.join("assumego");
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("write the fake assumego");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("make it executable");

    path
}

fn program(path: &Path) -> &str {
    path.to_str().expect("UTF-8 path")
}

#[test]
fn run_streaming_hands_stderr_lines_over_while_the_command_runs() {
    // The fake waits for the "approval" file that only the callback creates,
    // like assumego polling until the SSO login is approved.
    let approval =
        std::env::temp_dir().join(format!("alfred-granted-{}-approval", std::process::id()));
    let _ = fs::remove_file(&approval);
    let script = fake_assumego(
        "streaming",
        &format!(
            "echo '{LOGIN_LINE}' >&2\n\
             echo 'Code: ABCD-EFGH' >&2\n\
             while [ ! -f '{}' ]; do sleep 0.05; done\n\
             printf 'GrantedOutput\\n{CONSOLE_URL}\\n'",
            approval.display()
        ),
    );
    let mut lines = Vec::new();

    let stdout = SystemRunner
        .run_streaming(
            program(&script),
            &["prod-admin", "-c"],
            &[("GRANTED_ALIAS_CONFIGURED", "true")],
            Duration::from_secs(10),
            &mut |line| {
                lines.push(line.to_owned());

                if line.starts_with("Code:") {
                    fs::write(&approval, "").expect("approve");
                }
            },
        )
        .expect("should succeed once approved");

    assert_eq!(stdout, format!("GrantedOutput\n{CONSOLE_URL}\n"));
    assert_eq!(lines, vec![LOGIN_LINE, "Code: ABCD-EFGH"]);

    let _ = fs::remove_file(&approval);
}

#[test]
fn run_streaming_passes_args_and_env() {
    let script = fake_assumego("args", "echo \"$1 $2 $GRANTED_ALIAS_CONFIGURED\"");

    let stdout = SystemRunner
        .run_streaming(
            program(&script),
            &["prod-admin", "-c"],
            &[("GRANTED_ALIAS_CONFIGURED", "true")],
            Duration::from_secs(10),
            &mut |_| {},
        )
        .expect("should succeed");

    assert_eq!(stdout, "prod-admin -c true\n");
}

#[test]
fn run_streaming_kills_the_command_on_timeout() {
    let script = fake_assumego(
        "timeout",
        &format!("echo '{LOGIN_LINE}' >&2\nexec sleep 30"),
    );
    let mut lines = Vec::new();
    let started = Instant::now();

    let err = SystemRunner
        .run_streaming(
            program(&script),
            &[],
            &[],
            Duration::from_millis(500),
            &mut |line| lines.push(line.to_owned()),
        )
        .expect_err("should time out");

    assert!(started.elapsed() < Duration::from_secs(5));
    assert_eq!(lines, vec![LOGIN_LINE]);

    let timed_out = err
        .downcast_ref::<TimedOut>()
        .expect("the error should be a timeout");
    assert_eq!(timed_out.timeout, Duration::from_millis(500));
}

#[test]
fn run_streaming_reports_failures_with_stderr() {
    let script = fake_assumego("failure", "echo 'the SSO token expired' >&2\nexit 1");

    let err = SystemRunner
        .run_streaming(
            program(&script),
            &[],
            &[],
            Duration::from_secs(10),
            &mut |_| {},
        )
        .expect_err("should fail");

    assert!(err.to_string().contains("the SSO token expired"), "{err}");
    assert!(err.downcast_ref::<TimedOut>().is_none());
}

#[test]
fn run_streaming_does_not_wait_for_processes_holding_stderr() {
    // granted launches the SSO browser with stdout and stderr both pointing to
    // its stderr: a process left behind must not keep the workflow waiting
    // once assumego exited.
    let script = fake_assumego(
        "grandchild",
        &format!("sleep 5 >&2 2>&2 &\nprintf '{CONSOLE_URL}\\n'"),
    );
    let started = Instant::now();

    let stdout = SystemRunner
        .run_streaming(
            program(&script),
            &[],
            &[],
            Duration::from_secs(10),
            &mut |_| {},
        )
        .expect("should succeed");

    assert_eq!(stdout, format!("{CONSOLE_URL}\n"));
    assert!(started.elapsed() < Duration::from_secs(4));
}
