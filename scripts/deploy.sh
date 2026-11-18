#!/usr/bin/env bash
#
# Build the workflow and install it into the local Alfred.
#
#   scripts/deploy.sh          # update the installed workflow in place, or
#                              # import it if it is not installed yet. Default.
#   scripts/deploy.sh --import # always open the .alfredworkflow for import.
#
# Updating in place: the installed workflow is found in Alfred's preferences
# by its bundle id, and the freshly built binary, info.plist and icon are copied
# over it (anything else in that folder, such as Alfred's prefs.plist, is kept).
# Alfred reloads the folder from disk. This replaces symlinking (`powerpack
# link`), which Alfred 6 turns into a stale copy, and re-importing, which Alfred
# 6 fails to apply over an existing install.
#
# ALFRED_PREFERENCES overrides the preferences folder (default: the `current`
# entry of ~/Library/Application Support/Alfred/prefs.json).
#
# Requires: Alfred with the Powerpack license, granted on PATH.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

PACKAGE="target/workflow/alfred-granted.alfredworkflow"
FILES=(alfred-granted info.plist icon.png)

MODE="update"
if [[ "${1:-}" == "--import" ]]; then
  MODE="import"
fi

# Alfred's active preferences folder.
alfred_preferences() {
  if [[ -n "${ALFRED_PREFERENCES:-}" ]]; then
    echo "$ALFRED_PREFERENCES"
    return
  fi

  local prefs_json="$HOME/Library/Application Support/Alfred/prefs.json"
  if [[ -f "$prefs_json" ]]; then
    plutil -extract current raw "$prefs_json" 2>/dev/null && return
  fi

  echo "$HOME/Library/Application Support/Alfred/Alfred.alfredpreferences"
}

# The installed workflow folder whose bundle id is $1, if any.
installed_workflow() {
  local bundle_id="$1"
  local workflows
  workflows="$(alfred_preferences)/workflows"

  [[ -d "$workflows" ]] || return 0

  local dir
  for dir in "$workflows"/*/; do
    dir="${dir%/}"
    if [[ "$(plutil -extract bundleid raw "$dir/info.plist" 2>/dev/null)" == "$bundle_id" ]]; then
      echo "$dir"
      return
    fi
  done
}

import_package() {
  echo "==> Opening $PACKAGE for import"
  open "$PACKAGE"
  echo "==> Confirm the import in Alfred, then type 'assume' to test."
}

# Always build first so the deployed workflow is current.
"$REPO_ROOT/scripts/build.sh"

if [[ "$MODE" == "import" ]]; then
  import_package
  exit 0
fi

BUNDLE_ID="$(plutil -extract bundleid raw workflow/info.plist)"
TARGET="$(installed_workflow "$BUNDLE_ID")"

if [[ -z "$TARGET" ]]; then
  echo "==> $BUNDLE_ID is not installed yet"
  import_package
  exit 0
fi

if [[ -L "$TARGET" ]]; then
  echo "error: $TARGET is a symlink (old 'powerpack link' install)." >&2
  echo "       Delete it in Alfred Preferences, then run: scripts/deploy.sh --import" >&2
  exit 1
fi

echo "==> Updating $TARGET"
for file in "${FILES[@]}"; do
  /bin/cp -f "workflow/$file" "$TARGET/$file"
done

echo "==> Updated. Open Alfred and type 'assume' to test."
