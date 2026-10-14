# Contributing to SpriteLab

This is an experimental editor. Bug reports and small tested improvements are welcome. Check existing issues and the coverage/roadmap before starting a large feature. Use original code/art; Aseprite research is evidence, not permission to copy source, icons, fonts, palettes or artwork.

## Local setup

Rust 1.88+, Node 22.19+, Python 3, and the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for desktop. Headless builds only need Cargo/Node/Python. See docs/QUICKSTART.md. No global MCP or tool installation is required to run tests.

```sh
npm ci --ignore-scripts --prefix apps/desktop
npm ci --ignore-scripts --prefix adapters/mcp
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --locked
npm test --prefix adapters/mcp
npm run build --prefix apps/desktop
node --test scripts/configure-mcp.test.mjs
target/debug/spritelab --workspace demo --catalog > /tmp/spritelab-commands.json
diff -u docs/commands.json /tmp/spritelab-commands.json
```

On Linux without Tauri dependencies, select `-p assetforge-command -p spritelab-core -p spritelab-cli` for checks/tests and `-p spritelab-cli` for build. Socket tests need local Unix socket access. CI uses fresh macOS/Linux runners; it does not certify Windows or every GUI interaction.

## Architectural gate

The Command Engine owns persistent edits, raster algorithms, semantic validation, history and revisions. React and MCP select parameters and present responses. New durable operations require a typed command, generated schema/catalog entry, declared effect/undo behavior, validation, documentation and meaningful tests. New read-only queries must preserve asset content/history/revision. Never duplicate domain rules in an adapter.

Tests should prove rollback, undo round trips, stale-revision rejection, renderer/export agreement and actual adapter parity. Recovery must preserve original files. Third-party formats need fixtures and explicit unsupported-feature reports. State measurements before making performance claims. Keep lockfiles committed.

## Pull requests

Describe the trigger, resulting behavior, validation and remaining limitations. Include before/after screenshots for visible changes and test instructions. Keep scope focused; do not hide failures by weakening tests. Use clear commits; Portuguese and English are welcome. Maintainers review compatibility and original provenance before merge.

Report security concerns privately per SECURITY.md. Release previews remain explicit about supported targets and incomplete features. Publication, release uploads and policy changes are maintainer operations, not automatic side effects of a local build.
