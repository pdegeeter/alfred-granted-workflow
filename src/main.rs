//! `alfred-granted` — Alfred workflow entry point.
//!
//! Two subcommands, dispatched by Alfred:
//!   * `list [query]`      — script filter: print matching profiles as JSON.
//!   * `console <profile>` — action: open the AWS console for a profile.

use std::env;
use std::process::ExitCode;

use anyhow::{bail, Result};
use powerpack::Item;

use alfred_granted::runner::{CommandRunner, SystemRunner};
use alfred_granted::{alfred, console, frecency, profiles};

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_default();
    let rest: Vec<String> = args.collect();
    let runner = SystemRunner;

    match command.as_str() {
        "list" => run_list(&runner, rest.first().map(String::as_str).unwrap_or("")),
        "console" => run_console(&runner, rest.first().map(String::as_str).unwrap_or("")),
        other => {
            eprintln!("unknown command: `{other}` (expected `list` or `console`)");

            ExitCode::FAILURE
        }
    }
}

/// Script filter: list profiles matching `query`, ranked by frecency.
///
/// Any failure is surfaced as an Alfred item instead of a crash, so the user
/// sees a readable message in the dropdown.
fn run_list(runner: &dyn CommandRunner, query: &str) -> ExitCode {
    let items = match collect_items(runner, query) {
        Ok(items) => items,
        Err(err) => vec![Item::new("granted error")
            .subtitle(err.to_string())
            .valid(false)],
    };

    match powerpack::output(items) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("failed to write Alfred output: {err}");

            ExitCode::FAILURE
        }
    }
}

/// Gather, rank and filter profiles into Alfred items.
fn collect_items(runner: &dyn CommandRunner, query: &str) -> Result<Vec<Item>> {
    let profiles = profiles::fetch(runner)?;
    let scores = frecency::default_path()
        .map(|path| frecency::load(&path))
        .unwrap_or_default();

    let ranked = profiles::order_by_frecency(profiles, &scores);
    let matching = profiles::filter(ranked, query);

    Ok(alfred::build_items(matching))
}

/// Action: open the AWS console for `profile`.
fn run_console(runner: &dyn CommandRunner, profile: &str) -> ExitCode {
    match open_console(runner, profile) {
        Ok(url) => {
            // Printed so a downstream "Post Notification" can show feedback.
            println!("Opened AWS console for {profile}");
            eprintln!("opened {url}");

            ExitCode::SUCCESS
        }
        Err(err) => {
            println!("Failed to open console for {profile}");
            eprintln!("{err:#}");

            ExitCode::FAILURE
        }
    }
}

fn open_console(runner: &dyn CommandRunner, profile: &str) -> Result<String> {
    if profile.trim().is_empty() {
        bail!("no profile provided");
    }

    console::open_console(runner, profile)
}
