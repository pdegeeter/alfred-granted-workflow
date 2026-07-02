//! Tests for profile discovery, ranking and filtering.

mod common;

use std::collections::HashMap;

use alfred_granted::profiles;
use common::FakeRunner;

const COMPLETION_OUTPUT: &str = "\
prod-admin
sandbox-admin
pre-prod-admin
iam
";

#[test]
fn parse_completion_keeps_profile_names() {
    let profiles = profiles::parse_completion(COMPLETION_OUTPUT);

    assert_eq!(
        profiles,
        vec!["prod-admin", "sandbox-admin", "pre-prod-admin", "iam",]
    );
}

#[test]
fn parse_completion_drops_blank_and_flag_lines() {
    let raw = "\
prod-admin

--help
-c
iam
";

    let profiles = profiles::parse_completion(raw);

    assert_eq!(profiles, vec!["prod-admin", "iam"]);
}

#[test]
fn fetch_uses_assumego_completion_without_alias() {
    let runner = FakeRunner::new().with_success("assumego", COMPLETION_OUTPUT);

    let profiles = profiles::fetch(&runner).expect("fetch should succeed");

    assert_eq!(profiles.len(), 4);

    let calls = runner.calls.borrow();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].program, "assumego");
    assert_eq!(calls[0].args, vec!["--generate-bash-completion"]);
    assert_eq!(
        calls[0].envs,
        vec![("FORCE_NO_ALIAS".to_string(), "true".to_string())]
    );
}

#[test]
fn fetch_propagates_runner_errors() {
    let runner = FakeRunner::new().with_failure("assumego", "binary not found");

    assert!(profiles::fetch(&runner).is_err());
}

#[test]
fn order_by_frecency_puts_highest_score_first() {
    let profiles = vec![
        "prod-admin".to_string(),
        "sandbox-admin".to_string(),
        "iam".to_string(),
    ];
    let mut scores = HashMap::new();
    scores.insert("sandbox-admin".to_string(), 2.0);
    scores.insert("prod-admin".to_string(), 1.3);

    let ordered = profiles::order_by_frecency(profiles, &scores);

    assert_eq!(ordered, vec!["sandbox-admin", "prod-admin", "iam"]);
}

#[test]
fn order_by_frecency_sinks_unscored_profiles_below_scored_ones() {
    let profiles = vec![
        "never-used".to_string(),
        "used-once".to_string(),
        "also-never".to_string(),
    ];
    let mut scores = HashMap::new();
    scores.insert("used-once".to_string(), -0.5);

    let ordered = profiles::order_by_frecency(profiles, &scores);

    // Even a negative score outranks profiles that were never assumed, and the
    // unscored ones keep their original relative (alphabetical) order.
    assert_eq!(ordered, vec!["used-once", "never-used", "also-never"]);
}

#[test]
fn filter_empty_query_returns_everything() {
    let profiles = vec!["a".to_string(), "b".to_string()];

    assert_eq!(profiles::filter(profiles.clone(), ""), profiles);
    assert_eq!(profiles::filter(profiles.clone(), "   "), profiles);
}

#[test]
fn filter_is_case_insensitive_contains() {
    let profiles = vec![
        "prod-admin".to_string(),
        "sandbox-admin".to_string(),
        "iam".to_string(),
    ];

    assert_eq!(
        profiles::filter(profiles.clone(), "PROD"),
        vec!["prod-admin"]
    );

    assert_eq!(
        profiles::filter(profiles.clone(), "sandbox"),
        vec!["sandbox-admin"]
    );

    // A non-contiguous subsequence must NOT match (contains, not fuzzy).
    assert!(profiles::filter(profiles.clone(), "sba").is_empty());
}

#[test]
fn filter_requires_all_whitespace_separated_terms() {
    let profiles = vec![
        "sandbox-admin".to_string(),
        "sandbox-dev".to_string(),
        "prod-admin".to_string(),
    ];

    // Both "sandbox" and "admin" must appear.
    assert_eq!(
        profiles::filter(profiles.clone(), "sandbox admin"),
        vec!["sandbox-admin"]
    );

    // Order of terms does not matter.
    assert_eq!(
        profiles::filter(profiles, "admin   sandbox"),
        vec!["sandbox-admin"]
    );
}

#[test]
fn filter_preserves_input_order() {
    let profiles = vec!["sandbox-admin".to_string(), "prod-admin".to_string()];

    // Both contain "admin"; order must match the (already ranked) input.
    assert_eq!(
        profiles::filter(profiles.clone(), "admin"),
        vec!["sandbox-admin", "prod-admin"]
    );
}

#[test]
fn filter_no_match_returns_empty() {
    let profiles = vec!["prod-admin".to_string()];

    assert!(profiles::filter(profiles, "zzz").is_empty());
}
