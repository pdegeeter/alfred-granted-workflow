# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Keep everything in sync (always)

Every code change must update its tests, the docs under `docs/`, the `README.md`,
and this `CLAUDE.md` in the **same** change — never as a follow-up. If you change
behaviour (filtering, flags, the `assumego`/`open` invocation, the plist, the
CLI, the release flow), find and fix every reference to the old behaviour across
`README.md`, `docs/`, `CLAUDE.md`, and doc comments before committing. `cargo test`,
`cargo clippy --all-targets -- -D warnings`, and `cargo fmt --all` must all pass.

## What this is

An Alfred (macOS) workflow written in Rust. A single binary, `alfred-granted`,
backs two objects in the Alfred workflow graph via two subcommands:

- `alfred-granted list [query]` — the **Script Filter** (keyword `assume`).
  Prints matching AWS profiles as Alfred JSON on every keystroke.
- `alfred-granted console <profile>` — the **Run Script** action. Opens the AWS
  console for the selected profile.

The binary ships inside `workflow/` (with `info.plist`) and is packaged into a
`.alfredworkflow` file for distribution.

## Commands

```sh
cargo test --all                          # full suite (offline, no AWS/granted needed)
cargo test --test profiles_test           # one test file
cargo test --test profiles_test filter_is_case_insensitive_contains  # one test
cargo clippy --all-targets -- -D warnings # lint; warnings are errors (CI gate)
cargo fmt --all                           # format (CI checks with --check)

./scripts/build.sh            # sync plist version + build release + package .alfredworkflow
./scripts/deploy.sh           # build + symlink workflow into Alfred (live dev)
./scripts/deploy.sh --import  # build + open the .alfredworkflow to import it

# Exercise the binary directly (it speaks Alfred JSON on stdout):
cargo build && ./target/debug/alfred-granted list prod | python3 -m json.tool
```

`powerpack-cli` is required for build/deploy/packaging: `cargo install powerpack-cli`.
`plutil` (macOS) is used by `build.sh` to sync the plist version.

## Architecture

The design principle is: **everything is a pure function except one trait.** The
only side-effecting boundary — spawning `assumego` and macOS `open` — lives
behind `runner::CommandRunner` (`SystemRunner` in prod). Tests inject
`FakeRunner` (`tests/common/mod.rs`) that returns canned stdout and records
invocations, so the whole suite runs offline. When adding behaviour, keep logic
pure and route any new process spawn through `CommandRunner`.

Data flow for `list`: `profiles::fetch` runs
`FORCE_NO_ALIAS=true assumego --generate-bash-completion` (the exact command
granted's own shell completion uses — this is deliberately *not* parsing
`~/.aws/config`) → `frecency::load` parses `~/.granted/aws_profiles_frecency`
into name→score → `profiles::order_by_frecency` ranks → `profiles::filter`
keeps profiles containing every whitespace-separated query term as a
case-insensitive **substring**, preserving frecency order → `alfred::build_items`
produces `powerpack::Item`s (with an `autocomplete` value for Tab completion).

Data flow for `console`: `console::open_console` runs `assumego <profile> -c`
with `GRANTED_ALIAS_CONFIGURED=true` (the env granted's shell wrapper sets, which
makes assumego print `GrantedOutput\n<url>` to stdout — works even when
`DefaultBrowser=STDOUT`), extracts the `https://` URL, and shells out to `open`.
Note the two paths use different env: listing uses `FORCE_NO_ALIAS=true`, console
uses `GRANTED_ALIAS_CONFIGURED=true`.

## Conventions that matter here

- **No `uid` on Alfred items** — Alfred would re-sort by its own usage knowledge
  and override our frecency ranking. Ranking/filtering is done in Rust, not by
  Alfred (`alfredfiltersresults = false` in the plist).
- **Frecency is best-effort**: a missing/malformed frecency file yields an empty
  score map, never an error — listing must not break.
- **`list` never hard-fails**: errors are surfaced as an Alfred item so the user
  sees a readable message in the dropdown.
- **Editing `workflow/info.plist`**: if you edit the workflow in Alfred's UI
  while symlinked, copy the updated `info.plist` back into `workflow/` before
  committing, and validate with `plutil -lint workflow/info.plist`. The binary
  in `workflow/` is a build artifact (gitignored); `info.plist` and `icon.png`
  are tracked.
- **Workflow icon**: `workflow/icon.png` is generated from
  `assets/granted-logo.svg` via `scripts/icon.sh` (needs `rsvg-convert`). Don't
  hand-edit the PNG; edit the SVG and regenerate. Both files are committed; it is
  not regenerated at build time, so CI needs no SVG tooling.
- **Protected `main`**: work via PRs only (enforced for admins). CI `test` and
  `package` must be green, and **all commits must be signed** (SSH signing) to
  merge. See `docs/development.md` for the signing setup.

## Releases

Automated via release-please (`release-type: rust`). Land Conventional Commits
on `main` (`feat:`/`fix:`/`chore:`…); release-please opens a Release PR that
bumps `Cargo.toml`/`Cargo.lock` and `CHANGELOG.md`. Merging it tags `vX.Y.Z`,
creates the GitHub Release, and the `build-and-upload` job attaches the
`.alfredworkflow` (arm64) as a release asset. Do not bump versions by hand.
See `docs/release.md`.

Target platform is **macOS arm64 only**; CI and release build on `macos-latest`.
