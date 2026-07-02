#!/usr/bin/env bash
#
# Build the Rust binary and package the distributable .alfredworkflow file.
#
# Output: target/workflow/alfred-granted.alfredworkflow
#
# Usage: scripts/build.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

if ! command -v powerpack >/dev/null 2>&1; then
  echo "error: powerpack-cli not found. Install it with: cargo install powerpack-cli" >&2
  exit 1
fi

# Keep the workflow version in sync with Cargo.toml so releases are labelled
# correctly in Alfred's workflow list.
VERSION="$(grep -m1 '^version' Cargo.toml | cut -d '"' -f 2)"
echo "==> Setting workflow version to ${VERSION}"
plutil -replace version -string "$VERSION" workflow/info.plist

echo "==> Building release binary"
powerpack build --release

echo "==> Packaging .alfredworkflow"
powerpack package

echo "==> Done: target/workflow/alfred-granted.alfredworkflow"
