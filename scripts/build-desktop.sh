#!/bin/sh
set -eu
repo_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
if [ -x "$repo_dir/work/toolchain/cargo/bin/cargo" ]; then
  export CARGO_HOME="$repo_dir/work/toolchain/cargo"
  export RUSTUP_HOME="$repo_dir/work/toolchain/rustup"
  export PATH="$CARGO_HOME/bin:$PATH"
fi
command -v cargo >/dev/null 2>&1 || { echo 'Install Rust/Cargo: https://rustup.rs' >&2; exit 1; }
cd "$repo_dir/apps/desktop"
npm run tauri -- build --debug --bundles app
