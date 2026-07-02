//! Translation of profile names into Alfred script filter items.

use powerpack::Item;

use crate::services::Service;

/// Build the Alfred items shown in the dropdown for the given profiles.
///
/// The `arg` carries the profile name, which the connected action passes to
/// `alfred-granted console`. `autocomplete` lets the user press <kbd>Tab</kbd>
/// to complete the query to the highlighted profile name. We deliberately do
/// not set a `uid`: Alfred would otherwise re-sort items by its own usage
/// knowledge and override the frecency ranking we already computed.
pub fn build_items(profiles: Vec<String>) -> Vec<Item> {
    if profiles.is_empty() {
        return vec![Item::new("No matching AWS profile")
            .subtitle("Check your ~/.aws/config or granted setup")
            .valid(false)];
    }

    profiles
        .into_iter()
        .map(|profile| {
            Item::new(profile.clone())
                .subtitle(format!("Open the AWS console for “{profile}”"))
                .arg(profile.clone())
                .autocomplete(profile)
        })
        .collect()
}

/// Build the Alfred items shown once a profile is selected and the user is
/// typing a service alias.
///
/// Each item carries `"<profile> <alias>"` as its `arg`, which the connected
/// action passes to `alfred-granted console` and which that subcommand splits
/// back into the profile and service (neither ever contains whitespace).
/// `autocomplete` completes the query to `"<profile> <alias>"`. As with
/// profiles, no `uid` is set so Alfred keeps our ordering.
pub fn build_service_items(profile: &str, services: Vec<Service>) -> Vec<Item> {
    if services.is_empty() {
        return vec![Item::new("No matching AWS service")
            .subtitle(format!("No granted service alias matches for “{profile}”"))
            .valid(false)];
    }

    services
        .into_iter()
        .map(|service| {
            let arg = format!("{profile} {}", service.alias);

            Item::new(service.alias)
                .subtitle(format!(
                    "Open the AWS console for “{profile}” → {}",
                    service.destination
                ))
                .arg(arg.clone())
                .autocomplete(arg)
        })
        .collect()
}
