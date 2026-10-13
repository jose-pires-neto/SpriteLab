#!/bin/sh
# Convenience for this delivered workspace; normal contributors use cargo directly.
set -eu
repo_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
local_tools="$repo_dir/work/toolchain"
export CARGO_HOME="$local_tools/cargo"
export RUSTUP_HOME="$local_tools/rustup"
cd "$repo_dir"
exec "$CARGO_HOME/bin/cargo" "$@"
