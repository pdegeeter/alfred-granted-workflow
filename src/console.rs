//! Opening the AWS console for a selected profile.
//!
//! We run `assumego <profile> -c` with `GRANTED_ALIAS_CONFIGURED=true`. That is
//! the environment granted's own shell wrapper (`assume`) sets: it makes
//! `assumego` emit its structured output on stdout rather than depending on an
//! interactive shell. When a service alias is given we pass `-s <service>`
//! instead of `-c` (granted's `-s` is "like `-c` but opens to a specified
//! service"), which yields the same structured output pointed at that service.
//! For console mode that output is:
//!
//! ```text
//! GrantedOutput
//! https://<region>.signin.aws.amazon.com/federation?...
//! ```
//!
//! We extract that URL and open it ourselves with macOS `open`, so the workflow
//! works consistently regardless of the user's granted `DefaultBrowser` setting
//! (which may be `STDOUT`, in which case the wrapper — not us — would normally
//! print the URL).
//!
//! When the AWS IAM Identity Center session has expired, `assumego` first
//! prints an SSO login link on stderr and waits for the login to be approved
//! (see [`crate::sso`]). We read its stderr while it runs, open that link in
//! the default browser, and let the same `assumego` process carry on: once the
//! login is approved it assumes the profile and prints the console URL, so the
//! request resumes without the user selecting the profile again. The whole run
//! is bounded by [`ASSUME_TIMEOUT`].

use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};

use crate::runner::{CommandRunner, TimedOut};
use crate::sso;

/// Program that produces the console URL (the real granted binary).
const ASSUMEGO_BIN: &str = "assumego";

/// macOS command that opens a URL in the default browser. Absolute so it does
/// not depend on the `PATH` Alfred gives the workflow.
const OPEN_BIN: &str = "/usr/bin/open";

/// How long `assumego` may run, SSO login approval included, before it is
/// killed. Shorter than the 10 minutes a device code stays valid.
pub const ASSUME_TIMEOUT: Duration = Duration::from_secs(300);

/// Extract the first `https://` URL found in `output`.
///
/// granted prints the federated sign-in URL on its own; we scan tokens so we
/// tolerate surrounding log lines or leading/trailing whitespace.
pub fn extract_url(output: &str) -> Option<String> {
    output
        .split_whitespace()
        .find(|token| token.starts_with("https://"))
        .map(str::to_owned)
}

/// Assume `profile` in console mode and open the resulting URL in the browser.
///
/// When `service` is `Some(alias)`, the console opens directly at that AWS
/// service (`assumego <profile> -s <alias>`); otherwise it opens the console
/// home (`assumego <profile> -c`). If `assumego` asks for an SSO login, the
/// login link is opened in the browser and the call waits for the login to be
/// approved, up to `timeout`. Returns the opened URL on success so the caller
/// can report it to Alfred.
pub fn open_console(
    runner: &dyn CommandRunner,
    profile: &str,
    service: Option<&str>,
    timeout: Duration,
) -> Result<String> {
    let args: Vec<&str> = match service {
        Some(alias) => vec![profile, "-s", alias],
        None => vec![profile, "-c"],
    };

    let mut login_url: Option<String> = None;
    let mut on_stderr_line = |line: &str| {
        // A single login is in flight per run: open its link once.
        if login_url.is_some() {
            return;
        }

        if let Some(url) = sso::login_url(line) {
            if let Err(err) = runner.run(OPEN_BIN, &[&url], &[]) {
                eprintln!("failed to open the SSO login link: {err:#}");
            }

            login_url = Some(url);
        }
    };

    let result = runner.run_streaming(
        ASSUMEGO_BIN,
        &args,
        &[("GRANTED_ALIAS_CONFIGURED", "true")],
        timeout,
        &mut on_stderr_line,
    );

    let stdout = match result {
        Ok(stdout) => stdout,
        Err(err) => return Err(assume_error(err, profile, login_url.is_some())),
    };

    let url = match extract_url(&stdout) {
        Some(url) => url,
        None => bail!("no console URL returned for profile `{profile}`"),
    };

    runner
        .run(OPEN_BIN, &[&url], &[])
        .with_context(|| format!("failed to open the console URL for profile `{profile}`"))?;

    Ok(url)
}

/// Turn an `assumego` failure into the error reported to the user.
///
/// A timeout after a login link was opened means the SSO login was not
/// approved in time; it becomes a [`sso::LoginNotApproved`] so the caller can
/// tell the user so.
fn assume_error(err: anyhow::Error, profile: &str, login_requested: bool) -> anyhow::Error {
    let timed_out = err
        .downcast_ref::<TimedOut>()
        .map(|timed_out| timed_out.timeout);

    match (login_requested, timed_out) {
        (true, Some(timeout)) => anyhow!(sso::LoginNotApproved { timeout }).context(format!(
            "failed to log in to AWS SSO for profile `{profile}`"
        )),
        (true, None) => err.context(format!(
            "failed to log in to AWS SSO for profile `{profile}`"
        )),
        (false, _) => err.context(format!(
            "failed to get a console URL for profile `{profile}`"
        )),
    }
}
