# SpriteLab contributor/agent rules

The Command Engine is the heart of SpriteLab. MCP, CLI, Python and desktop UI are adapters. Keep original raster algorithms, semantic validation, history and persistent mutations in Rust domain/runtime code; never duplicate them in React/MCP.

Read README.md, docs/ARCHITECTURE.md and applicable ADRs before structural changes. Use explicit session mode: headless starts an isolated engine; --attach/--attach-latest uses the desktop-owned runtime. Never assume two independently opened projects share a session. New durable features need a typed command, generated schema, explicit effect/undo behavior, documentation and relevant behavioral tests. Update docs/commands.json from the binary catalog.

Aseprite source is research evidence only. Do not copy its code, icons, fonts, palettes or artwork into this permissively licensed project. Original detached pixel buffers may be built by a script, but persisted edits must be submitted to the engine.

Required validation: cargo fmt --all --check; cargo clippy --workspace --all-targets --locked -- -D warnings; cargo test --workspace --locked; cargo build --locked; npm ci --ignore-scripts and npm test in adapters/mcp. Add GUI runtime/parity validation when the desktop adapter exists. Preserve explicit distinction between tested behavior and planned capability.
