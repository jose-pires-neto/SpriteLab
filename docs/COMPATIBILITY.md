# Compatibility and preview status

- 0.5.0 is an experimental preview. macOS arm64 desktop is the validated runtime target; headless CI runs on macOS/Linux. Windows desktop and Windows local MCP attachment are unsupported (no named pipes).
- `.spritelab` format version 1 remains unchanged in 0.5.0. It stores asset content; undo, session revision and viewport are not saved. Unsupported versions fail explicitly. Keep project backups before upgrading previews.
- Command protocol version 1 remains unchanged. IDs and revision are session-specific, not durable asset IDs. Reopen/restore and independent headless sessions require inspection before editing.
- All 50 commands are available through the catalog and MCP. This means parity for implemented commands, not Aseprite feature parity. Changing MCP transport must not remove engine functionality.
- Recovery captures committed document snapshots every 15 seconds in desktop. The most recent interval and undo are not covered. Explicit checkpoints survive abrupt process termination after successful write; this is not a power-loss guarantee.
- macOS downloads are debug builds with ad-hoc signatures, no Developer ID certificate/notarization. Source builds require Rust 1.88+ and the current Tauri platform prerequisites. Node 22.19+ is needed by the MCP adapter. Python 3 runs parity examples/tests.
- Limits: 512×512, 32 layers, 64 frames, four open documents, 524,288 allocated cel pixels; history 32 entries/32 MiB conservative estimate. No tile-patch history, blend groups, linked cels, tilemaps, lasso/indexed editing or `.aseprite` I/O yet.

Breaking changes require a changelog entry and migration/unsupported-feature notes. No silent format reinterpretation or flattening of unsupported data.
