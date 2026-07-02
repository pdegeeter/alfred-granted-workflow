//! Parsing of granted's frecency database.
//!
//! granted keeps a `~/.granted/aws_profiles_frecency` file recording how often
//! and how recently each profile was assumed. We reuse it to surface the most
//! relevant profiles first, matching the ordering users already get in the
//! native `assume` fuzzy finder.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// A single entry in the frecency database.
#[derive(Debug, Deserialize)]
struct Entry {
    #[serde(rename = "Entry")]
    name: String,
    /// Combined frequency + recency score computed by granted. Higher is more
    /// relevant. Can be negative for stale-but-frequent entries.
    #[serde(rename = "FrecencySortingScore")]
    score: f64,
}

/// Root of the frecency database file.
#[derive(Debug, Deserialize)]
struct Database {
    #[serde(rename = "Entries", default)]
    entries: Vec<Entry>,
}

/// Default location of the granted frecency database, `~/.granted/aws_profiles_frecency`.
///
/// Returns `None` when the home directory cannot be resolved.
pub fn default_path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| Path::new(&home).join(".granted/aws_profiles_frecency"))
}

/// Parse the frecency database contents into a `profile name -> score` map.
///
/// Malformed JSON yields an empty map rather than an error: frecency is a
/// best-effort ranking signal and must never break profile listing.
pub fn parse_scores(contents: &str) -> HashMap<String, f64> {
    match serde_json::from_str::<Database>(contents) {
        Ok(db) => db
            .entries
            .into_iter()
            .map(|entry| (entry.name, entry.score))
            .collect(),
        Err(_) => HashMap::new(),
    }
}

/// Load and parse the frecency database from `path`.
///
/// A missing or unreadable file yields an empty map.
pub fn load(path: &Path) -> HashMap<String, f64> {
    match std::fs::read_to_string(path) {
        Ok(contents) => parse_scores(&contents),
        Err(_) => HashMap::new(),
    }
}
