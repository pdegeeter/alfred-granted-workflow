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
use alfred_granted::{alfred, console, frecency, profiles, services};

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
///
/// If the query has fully selected a profile followed by whitespace, switch to
/// service mode and list matching service aliases for that profile instead.
fn collect_items(runner: &dyn CommandRunner, query: &str) -> Result<Vec<Item>> {
    let profiles = profiles::fetch(runner)?;

    if let Some((profile, service_query)) = profiles::parse_service_query(query, &profiles) {
        let matching = services::filter(&service_query);

        return Ok(alfred::build_service_items(&profile, matching));
    }

    let scores = frecency::default_path()
        .map(|path| frecency::load(&path))
        .unwrap_or_default();

    let ranked = profiles::order_by_frecency(profiles, &scores);
    let matching = profiles::filter(ranked, query);

    Ok(alfred::build_items(matching))
}

/// Action: open the AWS console for the selected profile, optionally at a
/// service.
///
/// `arg` is the item's `arg` from the script filter: either `"<profile>"` or
/// `"<profile> <service>"`. Since neither a profile name nor a service alias
/// contains whitespace, we split on it to recover the two parts.
fn run_console(runner: &dyn CommandRunner, arg: &str) -> ExitCode {
    let mut parts = arg.split_whitespace();
    let profile = parts.next().unwrap_or("");
    let service = parts.next();

    let label = match service {
        Some(service) => format!("{profile} ({service})"),
        None => profile.to_owned(),
    };

    match open_console(runner, profile, service) {
        Ok(url) => {
            // Printed so a downstream "Post Notification" can show feedback.
            println!("Opened AWS console for {label}");
            eprintln!("opened {url}");

            ExitCode::SUCCESS
        }
        Err(err) => {
            println!("Failed to open console for {label}");
            eprintln!("{err:#}");

            ExitCode::FAILURE
        }
    }
}

fn open_console(
    runner: &dyn CommandRunner,
    profile: &str,
    service: Option<&str>,
) -> Result<String> {
    if profile.trim().is_empty() {
        bail!("no profile provided");
    }

    console::open_console(runner, profile, service)
}
