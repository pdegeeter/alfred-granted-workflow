//! Detecting the AWS IAM Identity Center login link granted prints.
//!
//! When no valid SSO token is cached, `assumego` starts a login flow before it
//! can assume the profile. It writes this line to stderr (granted 0.40,
//! `pkg/idclogin/browser.go`):
//!
//! ```text
//! If the browser does not open automatically, please open this link: https://<...>
//! ```
//!
//! then a `Code: XXXX-XXXX` line for the device code flow, tries to open its
//! configured SSO browser, and polls until the login is approved. Once it is,
//! the same `assumego` process carries on and assumes the profile.
//!
//! [`login_url`] recognises that link so the workflow can open it itself: the
//! browser granted launches does not always come up when Alfred runs it.

use std::fmt;
use std::time::Duration;

/// Error reported when the SSO login was not approved before the timeout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginNotApproved {
    pub timeout: Duration,
}

impl fmt::Display for LoginNotApproved {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let seconds = self.timeout.as_secs();

        if seconds >= 60 && seconds.is_multiple_of(60) {
            write!(
                f,
                "AWS SSO login not approved within {} minutes",
                seconds / 60
            )
        } else {
            write!(f, "AWS SSO login not approved within {seconds} seconds")
        }
    }
}

impl std::error::Error for LoginNotApproved {}

/// Phrase granted puts in front of the login link.
const LINK_MARKER: &str = "open this link:";

/// Hosts a login link may point to: the IAM Identity Center portal
/// (`<id>.awsapps.com`, device code flow) and the regional OIDC and device
/// endpoints (`*.amazonaws.com`, authorization code flow and older portals).
const ALLOWED_HOST_SUFFIXES: [&str; 2] = [".awsapps.com", ".amazonaws.com"];

/// Extract the SSO login link from one line of `assumego` stderr.
///
/// Only an `https://` link on the line granted prints for the login, pointing
/// to an AWS host, is returned: the link is opened in the browser, so anything
/// else on stderr must never be.
pub fn login_url(line: &str) -> Option<String> {
    let lowercase = line.to_ascii_lowercase();
    let marker_end = lowercase.find(LINK_MARKER)? + LINK_MARKER.len();

    // Cut at the first control character so terminal colour codes stuck to the
    // link are left out.
    let url: String = line[marker_end..]
        .split_whitespace()
        .next()?
        .chars()
        .take_while(|c| !c.is_control())
        .collect();

    if is_allowed_login_url(&url) {
        Some(url)
    } else {
        None
    }
}

/// Whether `url` is an `https://` URL on an AWS sign-in host.
fn is_allowed_login_url(url: &str) -> bool {
    let Some(rest) = url.strip_prefix("https://") else {
        return false;
    };

    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..authority_end];

    // No credentials or port tricks: a bare host name only.
    if authority.contains(['@', ':', '\\']) {
        return false;
    }

    let host = authority.to_ascii_lowercase();

    ALLOWED_HOST_SUFFIXES
        .iter()
        .any(|suffix| host.len() > suffix.len() && host.ends_with(suffix))
}
