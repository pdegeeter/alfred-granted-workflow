//! Discovery, ranking and filtering of AWS profiles.
//!
//! Profiles come from granted itself: `assumego --generate-bash-completion` is
//! the exact command granted's own shell completion uses, so we get the same
//! set of profiles the `assume` command would. We then rank them with the
//! frecency database and filter them against the user's Alfred query.

use std::cmp::Ordering;
use std::collections::HashMap;

use anyhow::Result;

use crate::runner::CommandRunner;

/// Program that exposes the profile completion (the real granted binary).
const ASSUMEGO_BIN: &str = "assumego";

/// Fetch the full list of AWS profiles known to granted.
///
/// Uses the same mechanism as granted's shell completion:
/// `FORCE_NO_ALIAS=true assumego --generate-bash-completion`. Flag completions
/// (lines starting with `-`) and blank lines are discarded.
pub fn fetch(runner: &dyn CommandRunner) -> Result<Vec<String>> {
    let stdout = runner.run(
        ASSUMEGO_BIN,
        &["--generate-bash-completion"],
        &[("FORCE_NO_ALIAS", "true")],
    )?;

    Ok(parse_completion(&stdout))
}

/// Parse the raw stdout of the completion command into profile names.
pub fn parse_completion(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('-'))
        .map(str::to_owned)
        .collect()
}

/// Reorder `profiles` so that the most frecent ones come first.
///
/// Profiles absent from the frecency map keep their original relative order and
/// sink below any profile that has ever been assumed.
pub fn order_by_frecency(profiles: Vec<String>, scores: &HashMap<String, f64>) -> Vec<String> {
    let mut ranked: Vec<(usize, String)> = profiles.into_iter().enumerate().collect();

    ranked.sort_by(|(a_idx, a_name), (b_idx, b_name)| {
        let a_score = scores.get(a_name);
        let b_score = scores.get(b_name);

        match (a_score, b_score) {
            (Some(a), Some(b)) => b
                .partial_cmp(a)
                .unwrap_or(Ordering::Equal)
                .then(a_idx.cmp(b_idx)),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            // Neither has a score: preserve the incoming (alphabetical) order.
            (None, None) => a_idx.cmp(b_idx),
        }
    });

    ranked.into_iter().map(|(_, name)| name).collect()
}

/// Decide whether `query` selects a profile and switches to *service mode*.
///
/// Service mode is entered as soon as the first whitespace-separated token is an
/// **exact** (case-insensitive) profile name *and* the query has whitespace
/// after it — i.e. the user has fully typed a profile and pressed space. In that
/// case we return `(canonical_profile, service_query)` where `service_query` is
/// the rest of the query (leading whitespace trimmed, possibly empty). Otherwise
/// we return `None` and the caller stays in profile-filtering mode.
///
/// Because the check requires an *exact* match, the multi-term profile filter
/// still works: `sandbox admin` stays in profile mode (neither "sandbox" is a
/// profile) and matches `sandbox-admin`. The only ambiguity is when one profile
/// is an exact prefix token of another intended search (e.g. both `prod` and
/// `prod-admin` exist and the user types `prod admin` meaning the latter) — a
/// rare collision we accept in exchange for a delimiter-free syntax.
pub fn parse_service_query(query: &str, profiles: &[String]) -> Option<(String, String)> {
    let (head, rest) = query.trim_start().split_once(char::is_whitespace)?;

    let canonical = profiles
        .iter()
        .find(|profile| profile.eq_ignore_ascii_case(head))?;

    Some((canonical.clone(), rest.trim_start().to_owned()))
}

/// Filter `profiles` against `query`, keeping only those that contain every
/// whitespace-separated term of the query as a case-insensitive substring.
/// Input order (i.e. the frecency ranking) is preserved.
///
/// Examples: `sandbox` keeps profiles containing "sandbox"; `sandbox admin`
/// keeps profiles containing both "sandbox" and "admin" (in any position).
/// An empty or whitespace-only query returns everything unchanged.
pub fn filter(profiles: Vec<String>, query: &str) -> Vec<String> {
    let terms: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();

    if terms.is_empty() {
        return profiles;
    }

    profiles
        .into_iter()
        .filter(|profile| {
            let haystack = profile.to_lowercase();

            terms.iter().all(|term| haystack.contains(term))
        })
        .collect()
}
