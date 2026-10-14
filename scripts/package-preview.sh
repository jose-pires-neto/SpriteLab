#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$root"
[ "$(uname -s)" = Darwin ] && [ "$(uname -m)" = arm64 ] || { echo 'macOS arm64 preview packaging only' >&2; exit 1; }
./scripts/package-desktop.sh
./scripts/local-cargo.sh build -p spritelab-cli --locked
version=$(node -p 'JSON.parse(require("fs").readFileSync("adapters/mcp/package.json","utf8")).version')
stage=$(mktemp -d "${TMPDIR:-/tmp/}spritelab-preview.XXXXXX")
trap 'rm -rf "$stage"' EXIT HUP INT TERM
bundle="$stage/SpriteLab-$version-macOS-arm64"
mkdir -p "$bundle/bin" "$bundle/adapters/mcp" "$bundle/scripts" "$bundle/docs"
ditto --norsrc --noextattr target/debug/bundle/macos/SpriteLab.app "$bundle/SpriteLab.app"
codesign --force --deep --sign - "$bundle/SpriteLab.app"
codesign --verify --deep --strict "$bundle/SpriteLab.app"
cp target/debug/spritelab "$bundle/bin/"
cp adapters/mcp/server.mjs adapters/mcp/engine-client.mjs adapters/mcp/package.json adapters/mcp/package-lock.json "$bundle/adapters/mcp/"
(cd "$bundle/adapters/mcp" && npm ci --omit=dev --ignore-scripts)
cp scripts/configure-mcp.mjs scripts/install-macos.sh "$bundle/scripts/"
cp LICENSE THIRD_PARTY_NOTICES.md SECURITY.md "$bundle/"
cp docs/QUICKSTART.md "$bundle/README.md"
cp docs/DISTRIBUTION-LICENSES.txt docs/QUICKSTART.md docs/COMPATIBILITY.md "$bundle/docs/"
ditto examples/slime-menta "$bundle/examples/slime-menta"
printf '#!/bin/sh\nset -eu\nroot=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)\nopen "$root/SpriteLab.app"\n' > "$bundle/Open SpriteLab.command"
chmod +x "$bundle/Open SpriteLab.command"
ditto -c -k --keepParent --norsrc --noextattr "$bundle" "outputs/SpriteLab-$version-preview-macOS-arm64.zip"
shasum -a 256 "outputs/SpriteLab-$version-preview-macOS-arm64.zip" > "outputs/SpriteLab-$version-SHA256SUMS.txt"
printf 'Portable preview: outputs/SpriteLab-%s-preview-macOS-arm64.zip\n' "$version"
