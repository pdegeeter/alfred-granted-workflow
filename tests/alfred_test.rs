//! Tests for turning profiles/services into Alfred items.
//!
//! `powerpack::Item`'s fields are private, so we assert on its JSON shape — the
//! same JSON Alfred consumes.

use alfred_granted::alfred;
use alfred_granted::services::Service;

fn to_json(item: &powerpack::Item) -> serde_json::Value {
    serde_json::to_value(item).expect("item should serialize")
}

#[test]
fn build_service_items_encodes_profile_and_alias_in_arg() {
    let services = vec![
        Service {
            alias: "ec2",
            destination: "ec2/v2",
        },
        Service {
            alias: "ddb",
            destination: "dynamodbv2",
        },
    ];

    let items = alfred::build_service_items("prod-admin", services);
    assert_eq!(items.len(), 2);

    let ec2 = to_json(&items[0]);
    assert_eq!(ec2["title"], "ec2");
    // The action splits "<profile> <alias>" back into profile and service.
    assert_eq!(ec2["arg"], "prod-admin ec2");
    assert_eq!(ec2["autocomplete"], "prod-admin ec2");
    assert_eq!(
        ec2["subtitle"],
        "Open the AWS console for “prod-admin” → ec2/v2"
    );

    let ddb = to_json(&items[1]);
    assert_eq!(ddb["arg"], "prod-admin ddb");
    assert_eq!(
        ddb["subtitle"],
        "Open the AWS console for “prod-admin” → dynamodbv2"
    );
}

#[test]
fn build_service_items_returns_a_single_invalid_item_when_empty() {
    let items = alfred::build_service_items("prod-admin", Vec::new());
    assert_eq!(items.len(), 1);

    let item = to_json(&items[0]);
    assert_eq!(item["title"], "No matching AWS service");
    assert_eq!(item["valid"], false);
    // Nothing to action: no arg is emitted.
    assert!(item.get("arg").is_none());
}

#[test]
fn build_items_sets_profile_as_single_arg() {
    let items = alfred::build_items(vec!["prod-admin".to_string()]);
    let item = to_json(&items[0]);

    assert_eq!(item["title"], "prod-admin");
    assert_eq!(item["arg"], "prod-admin");
    assert_eq!(item["autocomplete"], "prod-admin");
}
