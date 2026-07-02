//! Translation of profile names into Alfred script filter items.

use powerpack::Item;

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
