# SpriteLab 0.5.0 — experimental preview

Original open-source pixel-art/animation editor with a Rust Command Engine. Desktop, CLI, Python and MCP use the same commands; MCP is an adapter, not the editor's core.

## Download

- `SpriteLab-0.5.0-preview-macOS-arm64.zip`: portable GUI, CLI, MCP adapter/dependencies, documentation, notices and original editable Slime Menta example.
- `SpriteLab-0.5.0-SHA256SUMS.txt`: verify the ZIP before use.
- `editor-slime-demo.mp4`: actual eight-second desktop recording.

Extract and open `Open SpriteLab.command` or `SpriteLab.app`. For installation/MCP registration, read the included README or [Quick start](https://github.com/jose-pires-neto/SpriteLab/blob/main/docs/QUICKSTART.md). No Rust/npm build setup is required for the downloaded GUI; MCP needs Node 22.19+ and an agent host. Open the GUI before live MCP connection.

## What's included

50 typed operations; RGBA layers/cels, per-layer thumbnails, drawing/rectangular transforms, frames, forward tags/playback, onion skin, revision guards, atomic batches, undo/redo, PNG/GIF/spritesheet export and recovery checkpoints. Source checkout builds do not depend on a developer-specific toolchain; portable configuration discovers paths on the user's machine.

Validation: Rust fmt/Clippy/build, 34 Rust tests, nine real MCP/CLI/Python tests including SIGKILL recovery, path-with-spaces installer/config tests, portable SDK handshake/preview and native desktop capture. CI status is available in repository Actions; local validation is not exhaustive certification.

## Important preview limits

Only macOS arm64 desktop has native runtime validation. No Windows desktop/session support. Download is debug/ad-hoc signed, not Developer ID signed/notarized. macOS may require deliberate user approval. Source/headless use on Linux is covered by CI, not GUI runtime certification.

This is not a feature-complete Aseprite replacement: no lasso/OS clipboard, indexed editing, advanced brushes/blend groups/linked cels, tilemaps/slices, `.aseprite` I/O or reverse/ping-pong yet. Recovery preserves successful checkpoints, not the last 15-second interval, undo or power-loss durability. See compatibility/coverage/security documents for resource and transport limits.

Original MIT code/assets, original engine-created slime example, separately labeled AI-assisted promotional cover. No Aseprite/Photoshop source/art/icons/fonts/palettes are bundled. Feedback and focused contributions welcome.
