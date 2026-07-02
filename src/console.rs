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

use anyhow::{bail, Context, Result};

use crate::runner::CommandRunner;

/// Program that produces the console URL (the real granted binary).
const ASSUMEGO_BIN: &str = "assumego";

/// macOS command that opens a URL in the default browser.
const OPEN_BIN: &str = "open";

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
/// home (`assumego <profile> -c`). Returns the opened URL on success so the
/// caller can report it to Alfred.
pub fn open_console(
    runner: &dyn CommandRunner,
    profile: &str,
    service: Option<&str>,
) -> Result<String> {
    let args: Vec<&str> = match service {
        Some(alias) => vec![profile, "-s", alias],
        None => vec![profile, "-c"],
    };

    let stdout = runner
        .run(ASSUMEGO_BIN, &args, &[("GRANTED_ALIAS_CONFIGURED", "true")])
        .with_context(|| format!("failed to get a console URL for profile `{profile}`"))?;

    let url = match extract_url(&stdout) {
        Some(url) => url,
        None => bail!("no console URL returned for profile `{profile}`"),
    };

    runner
        .run(OPEN_BIN, &[&url], &[])
        .with_context(|| format!("failed to open the console URL for profile `{profile}`"))?;

    Ok(url)
}
