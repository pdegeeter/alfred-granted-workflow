//! Tests for detecting the SSO login link granted prints.

use std::time::Duration;

use alfred_granted::sso;

const DEVICE_URL: &str = "https://d-1234567890.awsapps.com/start/#/device?user_code=ABCD-EFGH";

fn login_line(url: &str) -> String {
    format!("If the browser does not open automatically, please open this link: {url}")
}

#[test]
fn login_url_finds_the_device_code_link() {
    assert_eq!(
        sso::login_url(&login_line(DEVICE_URL)),
        Some(DEVICE_URL.to_owned())
    );
}

#[test]
fn login_url_finds_regional_endpoints() {
    let device = "https://device.sso.eu-west-1.amazonaws.com/?user_code=ABCD-EFGH";
    let authorize = "https://oidc.eu-west-1.amazonaws.com/authorize?response_type=code&client_id=x";

    assert_eq!(sso::login_url(&login_line(device)), Some(device.to_owned()));
    assert_eq!(
        sso::login_url(&login_line(authorize)),
        Some(authorize.to_owned())
    );
}

#[test]
fn login_url_tolerates_log_prefix_and_colour_codes() {
    let line = format!("\u{1b}[34m[i]\u{1b}[0m {}\u{1b}[0m", login_line(DEVICE_URL));

    assert_eq!(sso::login_url(&line), Some(DEVICE_URL.to_owned()));
}

#[test]
fn login_url_ignores_other_lines() {
    assert_eq!(sso::login_url("[i] Code: ABCD-EFGH"), None);
    assert_eq!(sso::login_url(&format!("see {DEVICE_URL}")), None);
    assert_eq!(sso::login_url(""), None);
}

#[test]
fn login_url_rejects_non_https_links() {
    assert_eq!(
        sso::login_url(&login_line(
            "http://d-1234567890.awsapps.com/start/#/device"
        )),
        None
    );
    assert_eq!(sso::login_url(&login_line("file:///etc/passwd")), None);
    assert_eq!(sso::login_url(&login_line("-a Calculator")), None);
}

#[test]
fn login_url_rejects_non_aws_hosts() {
    for url in [
        "https://evil.example.com/start",
        "https://awsapps.com.evil.example/start",
        "https://evilawsapps.com/start",
        "https://.awsapps.com/start",
        "https://d-1234567890.awsapps.com@evil.example/start",
        "https://d-1234567890.awsapps.com:8443/start",
    ] {
        assert_eq!(sso::login_url(&login_line(url)), None, "{url}");
    }
}

#[test]
fn login_not_approved_reads_in_minutes_or_seconds() {
    let minutes = sso::LoginNotApproved {
        timeout: Duration::from_secs(300),
    };
    let seconds = sso::LoginNotApproved {
        timeout: Duration::from_secs(90),
    };

    assert_eq!(
        minutes.to_string(),
        "AWS SSO login not approved within 5 minutes"
    );
    assert_eq!(
        seconds.to_string(),
        "AWS SSO login not approved within 90 seconds"
    );
}
