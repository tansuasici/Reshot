#!/usr/bin/env bash
# Compile the Icon Composer source (assets/brand/Reshot.icon) into the asset
# catalog macOS 26+ uses for app icons. Needs Xcode 26+ (actool).
# Re-run after editing the icon; commit src-tauri/icons/Assets.car.
set -euo pipefail
cd "$(dirname "$0")/.."
out="$(mktemp -d)"
trap 'rm -rf "$out"' EXIT
xcrun actool assets/brand/Reshot.icon \
  --compile "$out" \
  --platform macosx \
  --minimum-deployment-target 11.0 \
  --app-icon Reshot \
  --output-partial-info-plist "$out/partial.plist" >/dev/null
cp "$out/Assets.car" src-tauri/icons/Assets.car
echo "wrote src-tauri/icons/Assets.car"
