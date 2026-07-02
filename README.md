# Alfred Granted Workflow

An [Alfred](https://www.alfredapp.com/) workflow, written in Rust, that lets you
assume an AWS profile with [granted](https://granted.dev) and open the AWS
console straight from Alfred.

Type `assume`, start typing a profile name, and Alfred suggests matching AWS
profiles (ranked by how often you use them). Press <kbd>Enter</kbd> to open the
AWS console for that profile in your browser.

```
assume prod
 ┌─────────────────────────────────────────────────────────┐
 │ prod-admin                                                │
 │   Open the AWS console for "prod-admin"                   │
 │ pre-prod-admin                                            │
 │   Open the AWS console for "pre-prod-admin"               │
 └─────────────────────────────────────────────────────────┘
```

Optionally, after a full profile name and a space, keep typing to pick an AWS
service — the console then opens straight to it (granted's `assume -s`):

```
assume prod-admin ec
 ┌─────────────────────────────────────────────────────────┐
 │ ec2                                                       │
 │   Open the AWS console for "prod-admin" → ec2/v2          │
 │ ecr                                                       │
 │   Open the AWS console for "prod-admin" → ecr             │
 │ ecs                                                       │
 │   Open the AWS console for "prod-admin" → ecs             │
 └─────────────────────────────────────────────────────────┘
```

## How it works

- **Profile suggestions** come from granted itself
  (`assumego --generate-bash-completion`, the same source its shell completion
  uses), then get re-ranked using granted's frecency database
  (`~/.granted/aws_profiles_frecency`) so your most-used profiles float to the
  top.
- **Service suggestions** mirror granted's own `-s` aliases (its `ServiceMap`):
  `ec2`, `s3`, `lambda`, `ddb`, `sqs`, `sfn`, … Matching is a case-insensitive
  substring against both the alias and the AWS service it opens, so `dynamo`
  finds `ddb`/`dynamodb` and `cost` finds `ce`.
- **Opening the console** runs `assumego <profile> -c` — or
  `assumego <profile> -s <service>` when a service is selected — with
  `GRANTED_ALIAS_CONFIGURED=true` (the environment granted's shell wrapper sets)
  to obtain the federated console URL from stdout, then opens it with macOS
  `open`. Extracting the URL ourselves means the workflow works even when
  granted's `DefaultBrowser` is set to `STDOUT`.

## Requirements

- macOS (Apple Silicon / arm64)
- [Alfred 5](https://www.alfredapp.com/) with the **Powerpack** license
  (workflows are a paid feature)
- [granted](https://granted.dev) installed and on `PATH` (`assumego` must be
  available)

## Install

Download the latest `alfred-granted.alfredworkflow` from the
[Releases](https://github.com/pdegeeter/alfred-granted-workflow/releases) page
and double-click it to import into Alfred.

Or build and deploy from source:

```sh
cargo install powerpack-cli   # one-time
./scripts/deploy.sh           # build + symlink into Alfred
```

## Usage

1. Trigger Alfred and type `assume` followed by a space.
2. Type part of a profile name — matching is a case-insensitive substring, so
   `sandbox` matches every profile containing "sandbox". Multiple
   whitespace-separated terms must all match (`sandbox admin` matches profiles
   containing both). Press <kbd>Tab</kbd> to complete the query to a profile.
3. Press <kbd>Enter</kbd> to open the AWS console home for the selected profile.
4. *Optional:* instead of pressing Enter, type a space after the full profile
   name and start typing a service (e.g. `assume prod-admin s3`). The list
   switches to matching services; Enter opens the console straight to that
   service. Leaving the service blank (just a trailing space) lists them all.

## Documentation

- [Architecture](docs/architecture.md) — components, data flow, design
  decisions.
- [Development](docs/development.md) — build, test, lint, deploy locally.
- [Release process](docs/release.md) — release-please and the CI/CD pipeline.

## Credits

The workflow icon is [granted](https://github.com/fwdcloudsec/granted)'s logo
([`docs/logo.svg`](https://github.com/fwdcloudsec/granted/blob/main/docs/logo.svg)),
rasterized to `workflow/icon.png`. granted is a project by fwd:cloudsec.

## License

MIT
