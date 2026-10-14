#!/bin/sh
# Use the delivered isolated toolchain when present, otherwise the user's Cargo.
set -eu
repo_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_dir"
local_tools="$repo_dir/work/toolchain"
if [ -x "$local_tools/cargo/bin/cargo" ]; then
  export CARGO_HOME="$local_tools/cargo"
  export RUSTUP_HOME="$local_tools/rustup"
  export PATH="$CARGO_HOME/bin:$PATH"
  exec "$CARGO_HOME/bin/cargo" "$@"
fi
command -v cargo >/dev/null 2>&1 || { echo 'Install Rust/Cargo: https://rustup.rs' >&2; exit 1; }
exec cargo "$@"
