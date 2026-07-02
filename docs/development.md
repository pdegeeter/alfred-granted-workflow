# Development

## Prerequisites

- Rust (stable) — `rustup` recommended.
- [`powerpack-cli`](https://crates.io/crates/powerpack-cli): `cargo install powerpack-cli`.
- [granted](https://granted.dev) on `PATH` for end-to-end manual testing.
- Alfred 5 with Powerpack, to actually run the workflow.

## Common tasks

```sh
cargo test --all                          # run the test suite
cargo clippy --all-targets -- -D warnings # lint (warnings are errors)
cargo fmt --all                           # format

./scripts/build.sh    # build release binary + package the .alfredworkflow
./scripts/deploy.sh   # build + symlink the workflow into Alfred (live dev)
./scripts/deploy.sh --import  # build + open the .alfredworkflow to import it
./scripts/icon.sh     # regenerate workflow/icon.png from assets/granted-logo.svg
```

The workflow icon (`workflow/icon.png`) is a committed asset rasterized from
`assets/granted-logo.svg` (granted's logo). Regenerate it with `scripts/icon.sh`
after changing the SVG; that needs `rsvg-convert` (`brew install librsvg`). It is
not regenerated at build time, so CI has no SVG-conversion dependency.

`scripts/build.sh` syncs `workflow/info.plist`'s `<version>` from `Cargo.toml`
before packaging, so the version shown in Alfred always matches the crate.

## Manual smoke test (without Alfred)

The binary speaks Alfred's JSON on stdout, so you can exercise it directly:

```sh
cargo build

# List profiles (JSON, ranked by frecency, filtered by the query)
./target/debug/alfred-granted list prod | python3 -m json.tool

# Open the console for a profile (runs assumego + open)
./target/debug/alfred-granted console prod-admin
```

## Testing strategy

All logic is pure and unit-tested; the only side-effecting code path — spawning
`assumego` and `open` — is hidden behind the `CommandRunner` trait and mocked in
tests via `FakeRunner` (`tests/common/mod.rs`). This means the whole suite runs
offline, without granted, AWS credentials, or a network connection.

| Test file                 | Covers                                                        |
| ------------------------- | ------------------------------------------------------------- |
| `tests/frecency_test.rs`  | Parsing the frecency DB; malformed/missing files degrade to empty. |
| `tests/profiles_test.rs`  | Completion parsing, frecency ordering, substring/multi-term filtering, `fetch` invocation. |
| `tests/console_test.rs`   | URL extraction and the assumego → open sequence, including failure paths. |

When adding behaviour, keep new logic pure where possible and assert on
`FakeRunner` invocations for the process-spawning parts.

## Project layout

```
.
├── src/                    Rust sources (see docs/architecture.md)
├── tests/                  Integration tests + shared FakeRunner
├── assets/
│   └── granted-logo.svg    Source SVG for the workflow icon
├── workflow/
│   ├── info.plist          Alfred workflow definition (the binary is added at build time)
│   └── icon.png            Workflow icon, rasterized from assets/granted-logo.svg
├── scripts/
│   ├── build.sh            Build + package the .alfredworkflow
│   ├── deploy.sh           Build + install into Alfred
│   └── icon.sh             Regenerate workflow/icon.png from the SVG
├── docs/                   This documentation
├── .github/workflows/      CI and release-please pipelines
├── release-please-config.json
└── .release-please-manifest.json
```

## Editing the Alfred graph

`workflow/info.plist` defines three connected objects: the Script Filter
(keyword `assume`), the Run Script action, and a notification. If you edit the
workflow in Alfred's UI while it is symlinked (`./scripts/deploy.sh`), copy the
updated `info.plist` back into `workflow/` before committing. Validate it with:

```sh
plutil -lint workflow/info.plist
```
