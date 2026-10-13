#!/bin/sh
# Package our local development app outside file-provider folders before signing.
set -eu
repo_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_dir"
./scripts/build-desktop.sh
spritelab_version=$(node -p 'JSON.parse(require("fs").readFileSync("apps/desktop/src-tauri/tauri.conf.json", "utf8")).version')
spritelab_stage=$(mktemp -d "${TMPDIR:-/tmp/}spritelab-package.XXXXXX")
trap 'rm -rf "$spritelab_stage"' EXIT HUP INT TERM
spritelab_app="SpriteLab $spritelab_version.app"
ditto --norsrc --noextattr target/debug/bundle/macos/SpriteLab.app "$spritelab_stage/$spritelab_app"
codesign --force --deep --sign - "$spritelab_stage/$spritelab_app"
codesign --verify --deep --strict "$spritelab_stage/$spritelab_app"
mkdir -p outputs
ditto -c -k --keepParent --norsrc --noextattr "$spritelab_stage/$spritelab_app" "outputs/SpriteLab-$spritelab_version-macOS-arm64.zip"
printf '%s\n' "Packaged outputs/SpriteLab-$spritelab_version-macOS-arm64.zip"
