#!/bin/sh
# User-local installation of a portable preview. Existing installs are preserved.
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
[ "$(uname -s)" = Darwin ] && [ "$(uname -m)" = arm64 ] || { echo 'This preview supports macOS arm64 only.' >&2; exit 1; }
[ -d "$root/SpriteLab.app" ] || { echo 'Run from an extracted portable release, not a source checkout.' >&2; exit 1; }
version=$(node -p 'JSON.parse(require("fs").readFileSync(process.argv[1],"utf8")).version' "$root/adapters/mcp/package.json")
install_base="${SPRITELAB_INSTALL_ROOT:-$HOME/Applications}"
mkdir -p "$install_base"
dest="$install_base/SpriteLab-$version"
[ ! -e "$dest" ] || { echo "Already installed at $dest; preserved. Use that installation or choose SPRITELAB_INSTALL_ROOT." >&2; exit 1; }
codesign --verify --deep --strict "$root/SpriteLab.app"
ditto "$root" "$dest"
codesign --verify --deep --strict "$dest/SpriteLab.app"
printf 'Installed: %s\nMCP: node "%s/scripts/configure-mcp.mjs" --workspace YOUR_WORKSPACE\n' "$dest/SpriteLab.app" "$dest"
