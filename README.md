# SpriteLab

Pixel art for humans. Creative tools for agents.

SpriteLab is the first specialized editor in the proposed AssetForge open-source ecosystem. The Command Engine owns editing; desktop UI, CLI, Python and MCP are clients of that engine. This repository contains an executable Rust foundation, a **Tauri + React desktop prototype**, research and architectural decisions.

## What runs today

- Original Rust command engine with 49 discoverable operations and generated JSON Schemas.
- RGBA sprite documents, ordered layers, sparse frame/layer cels, palette metadata and forward animation tags.
- Pixel strokes/lines, four-connected fill and rectangular buffer patches.
- Atomic edit batches, monotonic session revisions, expected-revision guards and undo/redo.
- Versioned editable `.spritelab` JSON projects; PNG import, canonical PNG preview/export and row-major spritesheets with metadata.
- JSONL CLI; a Python batch example; local stdio MCP adapter using the official SDK.
- Desktop canvas with pencil, eraser, line, fill, eyedropper, zoom, layers, frame playback, rectangular selections/transforms, shapes, brush alpha/size/symmetry, onion/grid, GIF, save/import/export and undo/redo. Detached stroke previews are rendered by Rust.
- Tests comparing real CLI, Python and MCP paths. No external AI service is required.

## Desktop (macOS arm64)

The delivered `outputs/SpriteLab-0.4.0-macOS-arm64.zip` contains the desktop app. Double-click `outputs/Abrir SpriteLab.command` to extract and open a verified local copy; no development server or npm installation is needed. Projects live in `~/Documents/ChatGPT/SpriteLab/assets`; the app installs an original example project there on first launch. Add existing `.spritelab` or `.png` files to that folder to open/import them from the sidebar. Each desktop process owns a runtime; CLI/MCP can explicitly attach to its private local Unix session.

B/E/L/F/I select pencil/eraser/line/fill/eyedropper, Escape discards a pending stroke, Space toggles playback, Command-Z undoes, Command-Shift-Z redoes, Command-S saves. A duplicate/new frame is selected automatically. Save preserves layers/frames; PNG exports the current frame and spritesheet exports all frames in one row. This is a local debug prototype, locally ad-hoc signed; not a Developer ID signed/notarized public release. See [Portuguese usage guide](docs/COMO-USAR.md).

Build the desktop with system Rust/Node and the platform prerequisites for Tauri 2:

```sh
cd apps/desktop
npm ci --ignore-scripts
npm run tauri -- build --debug --bundles app
```

In this delivered workspace, `./scripts/build-desktop.sh` uses the isolated Rust toolchain; `./scripts/package-desktop.sh` produces the locally signed ZIP. For headless-only builds use `cargo build -p spritelab-cli --locked`; desktop compilation on Linux also needs Tauri system libraries.

## Build and run

With Rust installed:

```sh
cargo build --locked
cargo test --workspace --locked
./target/debug/spritelab --workspace ./demo --catalog
```

In this delivered workspace, an isolated Rust toolchain is available under `work/toolchain`. It did not change the global shell PATH. Use `./scripts/local-cargo.sh build --locked` or `./scripts/local-cargo.sh test --workspace --locked` as a convenience. The toolchain is not part of this repository.

A single CLI process owns a session and accepts newline-delimited JSON on stdin. Keep it running while submitting related operations:

```sh
./target/debug/spritelab --workspace ./demo <<'JSONL'
{"request_id":"1","document_id":"example","command":"document.create","args":{"width":32,"height":32}}
{"request_id":"2","document_id":"example","expected_revision":0,"command":"canvas.stroke","args":{"layer_id":1,"frame_id":2,"points":[[8,8],[23,23]],"color":[242,104,68,255]}}
{"request_id":"3","document_id":"example","command":"export.png","args":{"path":"example.png","frame_id":2,"overwrite":true}}
JSONL
```

`demo/` already exists. File paths must be relative to the selected workspace. Parent directories must exist. Creation starts with layer `1`, frame `2`, revision `0`; inspect returned IDs after later edits. Request IDs correlate responses; they are **not** durable idempotency keys.

Generate four-frame example assets without third-party Python packages:

```sh
python3 examples/generate_mushrooms.py --engine ./target/debug/spritelab --workspace ./demo
python3 examples/generate_mushrooms.py --engine ./target/debug/spritelab --workspace ./demo --count 200
```

The bulk example intentionally creates copies of a small deterministic test sprite. It demonstrates throughput and editable animation files, not AI-generated character diversity. It explicitly overwrites its own generated filenames.

## MCP

Node.js 22.19+ is supported for the adapter:

```sh
cd adapters/mcp
npm ci --ignore-scripts
npm test
node server.mjs --engine /absolute/path/to/spritelab/target/debug/spritelab --workspace /absolute/path/to/project
```

Tools and their argument schemas are discovered from Rust at startup. For example, tool `spritelab_canvas_stroke` receives `{document_id, expected_revision?, request_id?, args}`. `spritelab_document_preview` returns a native MCP PNG image. `spritelab://documents/{document_id}` resources expose authoritative document snapshots. The adapter has no raster algorithms, history or persistence implementation.

Example MCP registration (replace absolute paths):

```json
{
  "mcpServers": {
    "spritelab": {
      "command": "/absolute/path/to/node",
      "args": ["/absolute/path/to/spritelab/adapters/mcp/server.mjs", "--engine", "/absolute/path/to/spritelab/target/debug/spritelab", "--workspace", "/absolute/path/to/project"]
    }
  }
}
```

Version 0.3 registers `spritelab_live` and `spritelab_headless` in the local Codex configuration. The live adapter attaches to the newest active desktop session in the configured workspace; headless keeps an isolated engine. See [session guide](docs/MCP-SESSAO.md).

## Architecture and evidence

- [Functional coverage and remaining gaps](docs/COBERTURA.md), [live MCP guide](docs/MCP-SESSAO.md).
- [Architecture](docs/ARCHITECTURE.md), [roadmap](docs/ROADMAP.md), [protocol](docs/PROTOCOL.md).
- [Portuguese research/decision report](docs/research/RELATORIO.md).
- [Pinned Aseprite source investigation](docs/research/aseprite-source-map.md).
- [REA evidence index](docs/research/evidence/index.json) and [generated command catalog](docs/commands.json).
- [Validation results](docs/VALIDATION.md), [third-party notices](THIRD_PARTY_NOTICES.md).

## Prototype boundaries

Mask/lasso selections, dedicated pan tool, advanced onion controls, indexed-color editing, linked-cel edit semantics, blend modes/groups/tilemap/slices, Aseprite importer, GPU rendering, 3D editor, plugins or asset database has been implemented yet. The palette is metadata and does not constrain RGBA drawing. Tags are forward ranges only.

Dimensions are limited to 512×512, 32 layers, 64 frames, 256 palette entries, 64 tags and 524,288 allocated cel pixels per document. A session supports four open documents; `document.close` releases one without auto-saving. Edits are serialized by the current process. History retains up to 32 entries and a conservative 32 MiB pixel estimate per document. This is copy-on-write snapshot history, not production tile-patch history.

Input lines/files are limited to 16 MiB; atlases to 4,194,304 pixels. Long-running jobs, cancellation, durable idempotency and multi-process locking are future work. File replacement uses a same-directory temporary file with file sync; filesystem policy is application-level and does not promise confinement against concurrent hostile symlink changes. No power-loss durability claim is made for directory metadata.

MIT applies to original SpriteLab code. Aseprite is a research reference, not a dependency, and no Aseprite implementation or artwork is included. Nothing has been published to GitHub or packaged as a public release.

## Version 0.4.0

Desktop recovery checkpoints all dirty documents every 15 seconds. Open a checkpoint from the Recuperação panel to restore it into a new unsaved document. Originals are never replaced. The same recovery commands are available through MCP and CLI; headless automation requests checkpoints explicitly. See [release notes](CHANGELOG.md) and [recovery contract](docs/adr/0005-recovery-snapshots.md). The last 15 seconds, undo history and viewport are not covered.
