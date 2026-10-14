# SpriteLab 0.5.0 preview — quick start

This is an original MIT pixel-art editor. The Rust Command Engine owns editing; GUI, CLI, Python and MCP share it. The preview is incomplete; see COMPATIBILITY.md in the source repository.

## Portable macOS arm64 download

1. Download and extract the preview ZIP and verify its SHA-256 against SHA256SUMS.
2. Double-click `Open SpriteLab.command`, or open `SpriteLab.app`. The build is ad-hoc signed and not notarized. If macOS blocks it, use the operating system's deliberate approval flow only if you trust the download; the installer does not disable Gatekeeper.
3. The default workspace is `~/Documents/ChatGPT/SpriteLab/assets`. New projects, imported PNGs and exports stay within that workspace. Opening another project preserves existing tabs.
4. Optional user-local installation: with Node 22.19+ installed, run `sh scripts/install-macos.sh` from the extracted folder. It installs a versioned folder in `~/Applications` and refuses to replace existing installations.

For a custom workspace, run the executable explicitly:

```sh
./SpriteLab.app/Contents/MacOS/spritelab-desktop --workspace /absolute/path/to/assets
```

## Codex MCP

Node 22.19+ and the Codex CLI must be available. Open SpriteLab first, using the same workspace you register:

```sh
node scripts/configure-mcp.mjs --workspace "$HOME/Documents/ChatGPT/SpriteLab/assets"
```

Run the script from your final installed location; do not move/delete that folder afterwards. It registers `spritelab_live` and `spritelab_headless` with absolute paths discovered on your machine. `--print` emits JSON for other MCP clients without editing Codex configuration. A trusted project's `.codex/config.toml` may override global entries; update/remove stale SpriteLab overrides in that project.

Open a fresh Codex chat. If tools are absent, reload Codex with SpriteLab already open. Live attachment chooses the newest running session in that workspace; confirm `editor.inspect`, document name and revision before editing when multiple windows are open. Headless creates an independent engine and never edits the desktop's documents. Relaunching the editor requires a new MCP connection.

Suggested first request:

> Use spritelab_live. Inspect the selected document and command catalog, then show a preview. Create a separate test document; use expected_revision for edits, save the editable project and export PNG/GIF. Do not change SpriteLab source code or overwrite my current document.

## Source checkout

Install Rust 1.88+, Node 22.19+, Python 3 and [Tauri platform prerequisites](https://v2.tauri.app/start/prerequisites/).

```sh
cargo build -p spritelab-cli --locked
npm ci --ignore-scripts --prefix adapters/mcp
npm ci --ignore-scripts --prefix apps/desktop
./scripts/build-desktop.sh
node scripts/configure-mcp.mjs --workspace "$HOME/Documents/ChatGPT/SpriteLab/assets"
```

No account, model API key or internet service is required for editing. MCP agents require their own host. Report reproducible issues with version, adapter, OS, steps and a minimal safe sample. Never include credentials in issue logs.
