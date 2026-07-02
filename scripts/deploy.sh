#!/usr/bin/env bash
#
# Build and deploy the workflow into the local Alfred installation.
#
# Two modes:
#   scripts/deploy.sh          # symlink the workflow/ dir into Alfred (dev-friendly:
#                              # rebuilds are picked up live). Default.
#   scripts/deploy.sh --import # build the .alfredworkflow and open it so Alfred
#                              # imports it as a normal installed workflow.
#
# Requires: Alfred with the Powerpack license, granted on PATH.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

MODE="link"
if [[ "${1:-}" == "--import" ]]; then
  MODE="import"
fi

# Always build first so the deployed workflow is current.
"$REPO_ROOT/scripts/build.sh"

if [[ "$MODE" == "import" ]]; then
  echo "==> Opening .alfredworkflow for import"
  open target/workflow/alfred-granted.alfredworkflow
  echo "==> Alfred should now prompt to import the workflow."
else
  echo "==> Symlinking workflow into Alfred"
  powerpack link
  echo "==> Linked. Open Alfred and type 'assume' to test."
fi
