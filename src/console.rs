//! Opening the AWS console for a selected profile.
//!
//! We run `assumego <profile> -c` with `GRANTED_ALIAS_CONFIGURED=true`. That is
//! the environment granted's own shell wrapper (`assume`) sets: it makes
//! `assumego` emit its structured output on stdout rather than depending on an
//! interactive shell. For console mode that output is:
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
/// Returns the opened URL on success so the caller can report it to Alfred.
pub fn open_console(runner: &dyn CommandRunner, profile: &str) -> Result<String> {
    let stdout = runner
        .run(
            ASSUMEGO_BIN,
            &[profile, "-c"],
            &[("GRANTED_ALIAS_CONFIGURED", "true")],
        )
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
