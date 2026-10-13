#!/bin/sh
set -eu
repo_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
export CARGO_HOME="$repo_dir/work/toolchain/cargo"
export RUSTUP_HOME="$repo_dir/work/toolchain/rustup"
export PATH="$CARGO_HOME/bin:$PATH"
cd "$repo_dir/apps/desktop"
npm run tauri -- build --debug --bundles app
