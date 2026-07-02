#!/usr/bin/env bash
#
# Regenerate the Alfred workflow icon (workflow/icon.png) from the source SVG
# (assets/granted-logo.svg). Alfred uses a PNG, so we rasterize the SVG.
#
# Requires: rsvg-convert (brew install librsvg).
#
# Usage: scripts/icon.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

if ! command -v rsvg-convert >/dev/null 2>&1; then
  echo "error: rsvg-convert not found. Install it with: brew install librsvg" >&2
  exit 1
fi

# Render at 512px tall, preserving the logo's aspect ratio on a transparent
# background. Alfred scales it to fit its square icon slot.
rsvg-convert -h 512 assets/granted-logo.svg -o workflow/icon.png

echo "==> Wrote workflow/icon.png ($(sips -g pixelWidth -g pixelHeight workflow/icon.png | awk '/pixel/{print $2}' | paste -sd x -))"
