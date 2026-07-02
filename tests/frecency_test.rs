//! Tests for parsing granted's frecency database.

use alfred_granted::frecency;

const SAMPLE: &str = r#"{
    "MaxFrequency": 11,
    "OldestDate": "2026-06-24T15:43:42.773411+02:00",
    "Entries": [
        { "Entry": "sandbox-admin", "Frequency": 11, "FrecencySortingScore": 1.9998 },
        { "Entry": "iam", "Frequency": 9, "FrecencySortingScore": 1.9128 },
        { "Entry": "iam-sandbox-tf", "Frequency": 10, "FrecencySortingScore": -1.7356 }
    ]
}"#;

#[test]
fn parses_scores_by_profile_name() {
    let scores = frecency::parse_scores(SAMPLE);

    assert_eq!(scores.len(), 3);
    assert_eq!(scores.get("sandbox-admin"), Some(&1.9998));
    assert_eq!(scores.get("iam"), Some(&1.9128));
}

#[test]
fn keeps_negative_scores() {
    let scores = frecency::parse_scores(SAMPLE);

    assert_eq!(scores.get("iam-sandbox-tf"), Some(&-1.7356));
}

#[test]
fn unknown_profile_has_no_score() {
    let scores = frecency::parse_scores(SAMPLE);

    assert_eq!(scores.get("does-not-exist"), None);
}

#[test]
fn malformed_json_yields_empty_map() {
    assert!(frecency::parse_scores("not json at all").is_empty());
    assert!(frecency::parse_scores("").is_empty());
}

#[test]
fn missing_entries_field_yields_empty_map() {
    assert!(frecency::parse_scores(r#"{"MaxFrequency": 3}"#).is_empty());
}

#[test]
fn missing_file_yields_empty_map() {
    let scores = frecency::load(std::path::Path::new("/nonexistent/aws_profiles_frecency"));

    assert!(scores.is_empty());
}
