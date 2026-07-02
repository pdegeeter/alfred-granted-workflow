# Architecture

## Overview

The workflow is a single Rust binary, `alfred-granted`, with two subcommands
that map to the two objects in the Alfred workflow graph:

| Subcommand                    | Alfred object                    | Responsibility                                              |
| ----------------------------- | -------------------------------- | ----------------------------------------------------------- |
| `alfred-granted list [query]` | Script Filter (`input.scriptfilter`) | Print matching AWS profiles — or services, once a profile is selected — as Alfred JSON. |
| `alfred-granted console <arg>` | Run Script (`action.script`)  | Split `arg` into profile + optional service, resolve the console URL, open it. |

Alfred invokes `list` on every keystroke and `console` once, when the user hits
<kbd>Enter</kbd> on a result. The console action's `arg` is the selected item's
`arg`: `"<profile>"` for a profile item, `"<profile> <service>"` for a service
item (neither part contains whitespace, so `console` splits on it).

## Data flow

```mermaid
sequenceDiagram
    participant U as User
    participant A as Alfred
    participant B as alfred-granted
    participant G as granted (assumego)
    participant F as ~/.granted/aws_profiles_frecency
    participant OS as macOS open

    U->>A: type "assume prod"
    A->>B: list "prod"
    B->>G: assumego --generate-bash-completion
    G-->>B: profile names
    B->>F: read frecency scores
    F-->>B: name → score
    B-->>A: JSON items (ranked, filtered)
    A-->>U: dropdown of profiles

    Note over U,B: optional service mode
    U->>A: type "assume prod-admin s3"
    A->>B: list "prod-admin s3"
    B->>G: assumego --generate-bash-completion
    G-->>B: profile names (to confirm "prod-admin" is exact)
    B-->>A: JSON items (services matching "s3")
    A-->>U: dropdown of services

    U->>A: Enter on "prod-admin" (or "prod-admin s3")
    A->>B: console "prod-admin" (or "prod-admin s3")
    B->>G: GRANTED_ALIAS_CONFIGURED=true assumego prod-admin -c (or -s s3)
    G-->>B: "GrantedOutput\n<federated console URL>"
    B->>OS: open <url>
    OS-->>U: AWS console in browser
    B-->>A: "Opened AWS console for …" (notification)
```

## Module layout

```
src/
├── main.rs      Entry point: parse argv, dispatch to `list` / `console`.
├── lib.rs       Library root, re-exports the modules below.
├── runner.rs    CommandRunner trait + SystemRunner (the only impure boundary).
├── profiles.rs  Fetch (via assumego), rank (frecency), filter (substring), detect service mode.
├── services.rs  Static mirror of granted's service ServiceMap; substring filter.
├── frecency.rs  Parse ~/.granted/aws_profiles_frecency into name → score.
├── alfred.rs    Turn profile names / services into powerpack `Item`s.
└── console.rs   Resolve the console URL (optionally at a service) and open it.
```

## Design decisions

### Profiles come from granted, not from parsing `~/.aws/config`

`FORCE_NO_ALIAS=true assumego --generate-bash-completion` is exactly what
granted's own zsh completion calls. Reusing it means the workflow always sees
the same profile set as the native `assume` command — including profiles from
SSO sessions and profile registries — with zero config-parsing logic to keep in
sync with granted's behaviour.

### Frecency ranking

granted records usage frequency and recency per profile in
`~/.granted/aws_profiles_frecency`. We parse the `FrecencySortingScore` and sort
profiles by it (descending). Profiles that were never assumed have no score and
sink below the ones that were, keeping their original (alphabetical) order.
Frecency is best-effort: a missing or malformed file simply yields no scores and
the list falls back to alphabetical order — it never breaks listing.

### Opening the console ourselves

granted can open the browser itself, but its behaviour depends on the user's
`DefaultBrowser` setting — which may be `STDOUT` (print the URL instead of
opening it). To be robust regardless of that setting, we run `assumego -c` with
`GRANTED_ALIAS_CONFIGURED=true` — the same environment granted's `assume` shell
wrapper sets — so `assumego` prints its structured output (`GrantedOutput`
followed by the federated URL) to stdout. We extract that URL and call macOS
`open` ourselves, making the Enter action deterministic.

Note the environment difference between the two paths: listing uses
`FORCE_NO_ALIAS=true` (we only want completion output), while the console path
uses `GRANTED_ALIAS_CONFIGURED=true` (we need the wrapper-style structured
output that carries the URL).

### Substring filtering in Rust

The Script Filter sets `alfredfiltersresults = false`, so Alfred does not filter
results. We filter in Rust: a profile is kept only if it contains every
whitespace-separated term of the query as a case-insensitive substring (so
`sandbox admin` matches profiles containing both). This is more predictable than
fuzzy matching. Just as importantly, we **preserve the frecency order** of the
survivors, and do not set a `uid` on items — otherwise Alfred would re-sort by
its own usage knowledge and override our ranking. Items also carry an
`autocomplete` value so <kbd>Tab</kbd> completes the query to the profile name.

### Service mode: no delimiter, exact-profile trigger

The Script Filter has a single query string, yet it must serve two searches:
profiles, then services. Rather than introduce a delimiter (`prod-admin/ec2`),
we switch to service mode as soon as the first whitespace-separated token is an
**exact** profile name followed by whitespace — i.e. the user finished typing a
profile and pressed space (`parse_service_query`). Requiring an *exact* match is
what preserves the multi-term profile filter: `sandbox admin` keeps searching
profiles because "sandbox" is not itself a profile. The accepted trade-off is a
rare collision — if both `prod` and `prod-admin` exist and the user types
`prod admin` meaning the latter, we read it as service mode for `prod`.

The service list (`services::SERVICES`) is a static snapshot of granted's own
`ServiceMap` (`pkg/console/service_map.go`) compiled into the binary — it is a
fixed table upstream, not runtime data, so there is nothing to query at runtime.
We filter it by substring against **both** the alias and its console destination
(so `dynamo` finds `ddb`, `cost` finds `ce` → `cost-management`). Opening a
service uses granted's `-s <alias>` flag, which is documented as "like `-c` but
opens to a specified service" and emits the same structured stdout we already
parse.

### The `CommandRunner` boundary

Every external process (`assumego`, `open`) goes through the `CommandRunner`
trait. Production uses `SystemRunner`; tests inject a `FakeRunner` that returns
canned output and records invocations. Everything else — parsing, ranking,
filtering, URL extraction — is pure and unit-tested offline. This keeps the test
suite fast and independent of any AWS/granted setup.
