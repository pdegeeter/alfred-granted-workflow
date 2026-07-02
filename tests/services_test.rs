//! Tests for the static granted service-alias table and its filtering.

use alfred_granted::services::{self, SERVICES};

#[test]
fn service_table_is_non_empty_and_has_no_blank_alias() {
    assert!(!SERVICES.is_empty());
    assert!(SERVICES
        .iter()
        .all(|service| !service.alias.is_empty() && !service.destination.is_empty()));
}

#[test]
fn service_table_is_sorted_by_alias_and_has_unique_aliases() {
    let aliases: Vec<&str> = SERVICES.iter().map(|service| service.alias).collect();

    let mut sorted = aliases.clone();
    sorted.sort_unstable();
    assert_eq!(aliases, sorted, "SERVICES must stay sorted by alias");

    let mut deduped = sorted.clone();
    deduped.dedup();
    assert_eq!(deduped.len(), aliases.len(), "aliases must be unique");
}

#[test]
fn filter_empty_query_returns_everything() {
    assert_eq!(services::filter("").len(), SERVICES.len());
    assert_eq!(services::filter("   ").len(), SERVICES.len());
}

#[test]
fn filter_matches_alias_as_case_insensitive_substring() {
    let aliases: Vec<&str> = services::filter("EC")
        .into_iter()
        .map(|service| service.alias)
        .collect();

    // "ec" appears in ec2, ecr, ecs (and any destination containing "ec").
    assert!(aliases.contains(&"ec2"));
    assert!(aliases.contains(&"ecr"));
    assert!(aliases.contains(&"ecs"));
}

#[test]
fn filter_matches_against_destination_too() {
    // "dynamo" is not in the aliases "ddb"/"dynamodb"'s… well, "dynamodb" has it,
    // and both map to the "dynamodbv2" destination, so both are returned.
    let aliases: Vec<&str> = services::filter("dynamo")
        .into_iter()
        .map(|service| service.alias)
        .collect();
    assert!(aliases.contains(&"ddb"));
    assert!(aliases.contains(&"dynamodb"));

    // "cost" only exists in the destination of the "ce" alias.
    let by_destination: Vec<&str> = services::filter("cost")
        .into_iter()
        .map(|service| service.alias)
        .collect();
    assert_eq!(by_destination, vec!["ce"]);
}

#[test]
fn filter_requires_all_whitespace_separated_terms() {
    // Both terms must match (alias or destination) for the same entry.
    let aliases: Vec<&str> = services::filter("secrets manager")
        .into_iter()
        .map(|service| service.alias)
        .collect();
    assert!(aliases.contains(&"secretsmanager"));
    // "sm" alias → "secretsmanager" destination also matches both terms.
    assert!(aliases.contains(&"sm"));
}

#[test]
fn filter_no_match_returns_empty() {
    assert!(services::filter("definitely-not-a-service").is_empty());
}
