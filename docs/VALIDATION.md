# Validation record

Date: 2026-10-13. Host: macOS arm64. Rust 1.99.0, Node 26.5.0, official MCP SDK 1.32.1, Python 3. Prototype desktop 0.2.0.

| Verification | Outcome | Scope |
|---|---|---|
| Rust workspace build | Passed | Three domain/runtime crates, CLI and Tauri desktop binary |
| `cargo fmt --all --check` | Passed | Rust source formatting |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed | Static Rust checks |
| Rust tests | 19 passed | 15 core behavior tests + 4 runtime/file tests |
| Real MCP adapter tests | 4 passed | Official SDK client initializes stdio server and executes actual Rust engine calls |
| Same operation stream via CLI/Python/MCP | Passed | 19 requests including detached preview; exact structured responses, document state, RGBA bytes and exported project/PNG/atlas bytes |
| Revision conflict through concurrent MCP calls | Passed | One commit and one rejection from the same expected revision |
| MCP image/resources/catalog | Passed | Native PNG image, document resource, argument schemas discovered from Rust |
| Engine child startup/failure handling | Passed | Pending request rejects and session closes; no hanging Promise |
| Bulk Python generation | Passed | 200 documents, 800 frames, two layers/eight colors/idle tag, project+PNG+atlas+metadata for each; one CLI session |
| Demonstration artifact inspection | Completed | Viewed original 32×32 pixel sprite atlas generated through engine; demonstration only |
| REA bundle/plist/version capture | Completed | Pinned official trial artifact identity and version; original Evidence records saved |
| REA help capture | Partial | Process returned 0 with help output; REA reported `cleanup_incomplete` and retained partial observation |

Core tests cover stroke interpolation/one-step undo, batch commit/rollback (including tentative IDs), monotonic revisions, stale edits/undo, frame buffer detachment, no ID reuse after undo, fill boundaries, layer locks/unknown arguments, no-op redo preservation, compositing alpha/visibility, invalid documents/tags, allocated-pixel limits, saveable bounded state and session closure. Runtime tests cover editable-project/PNG round trips, atlas rectangles/durations, explicit overwrite, workspace traversal/symlink rejection, preview/export equality and malformed project rejection.

The same-input adapter test includes an intentional failed out-of-bounds edit and confirms the later document is unchanged. The atlas/project/PNG bytes are compared across direct CLI, Python CLI replay and actual MCP execution. This is prototype parity for the tested commands/sequence, not a claim that every future operation is already verified.

Bulk outputs used for verification remain under the chat's `work/bulk-verification`, not in the source repository. The small user-facing example is under `demo/`. No throughput/latency benchmark was recorded. No real Codex global MCP connection, Windows/Linux build/runtime, Godot importer, GIF, Aseprite importer, production memory/latency workload or public GitHub CI run was validated. The proposed CI file is configuration only.

The temporary Aseprite disk image was unmounted after research. REA/Rust dependencies were isolated to this chat workspace; no global PATH or MCP configuration was changed. The source contains no Aseprite implementation/assets.

## Desktop validation

Tauri 2.12.3, React 19, Vite 7; macOS arm64 local debug bundle. TypeScript/Vite production assets and native `.app` bundle built successfully. Native UI was opened and visually inspected. The original four-frame mushroom loaded; a dragged pencil gesture committed one revision, undo restored the saved state, redo restored the edit, and the GUI saved `validacao-interface.spritelab` and exported `validacao-interface.png`. Reopening that saved project through the actual CLI and exporting frame 2 produced identical PNG bytes: [parity evidence](desktop-parity.json).

The detached-preview behavioral test verifies preview/commit RGBA equality, unchanged live pixels/revision/history and unconsumed draft IDs. Basic native layer creation was also exercised. This is a bounded smoke test, not exhaustive interactive QA of every tool, playback timing, all keyboard shortcuts or application shutdown paths. CLI/Python/MCP parity tests remain passing; At the time of the 0.2 check, MCP still had a separate session. This limitation was resolved by the 0.3 shared-session implementation below.

## Version 0.3 — shared session and expanded editing

Project migrated to `/Users/josepires/Documents/ChatGPT/SpriteLab`. 45 generated commands. 20 core tests and 10 runtime/session/file tests, plus 5 real MCP tests. The parity fixture now includes 39 requests, expanded brush/shapes/transforms/layer/frame/resize/view/onion/GIF operations, with exact CLI/Python/MCP responses and project/PNG/atlas/GIF bytes.

Native desktop opened, and an actual official-SDK MCP client attached to its socket. It opened the example under a separate document ID, created a layer, drew a rectangle and stroke, selected its document/context and exported editable project, PNG and four-frame GIF. The native screenshot/AX tree showed the new `Desenhado pelo MCP` layer and cyan/yellow test marks. Records: `live-desktop-mcp.json`. The snapshot was rendered by the same engine instance as the GUI.

Private socket mode 0600, two-client stale revision rejection, idle client startup, bounded-message recovery, saved-state/undo tracking and GIF decoded frame timing/transparency are covered by behavior tests. Headless and attached modes remain explicit. The local Codex registry recognizes both servers; actual tool availability after client reload is not asserted for this chat.

This does not certify complete Aseprite parity, all possible gestures/keyboard shortcuts, arbitrary project sizes, production latency, crash recovery, Windows/Linux runtime or an Apple-signed public build. See COBERTURA.md.

## 0.3.1 package verification

The saved baseline/path and color/shape-fill view settings are shared by the runtime. All 30 Rust behavior tests and 5 actual MCP tests passed; the attached SDK test discovers the live host via `--attach-latest`, changes color/fill settings, saves, and verifies that another CLI client sees the same view and clean saved state. Formatting, workspace Clippy with warnings denied, locked build, clean npm installation and TypeScript/Vite build passed. Dependency license texts include GIF quantization/encoding dependencies.

The 0.3.1 package is built separately from the running 0.3 app. The user's current live drawing session was preserved. The earlier native MCP smoke test covers the shared host; 0.3.1 baseline/view corrections were also verified in an isolated native copy of the final bundle, as described below. This is not exhaustive native interaction testing.

Inactive documents are listed with path/dirty status; the GUI offers a session document selector and queries all unsaved documents before closing. Alpha and color alpha are synchronized by the runtime (explicit `alpha` wins if both fields are supplied). These behaviors have dedicated runtime assertions.

The isolated 0.3.1 native smoke test verified: MCP-created rectangle appears on the canonical canvas; MCP color `#0C2238`, alpha 143, filled shapes, zoom 24, grid and onion appear in native controls; saved path/clean status is recognized; a GUI ellipse gesture increments revision and its exact cel pixel `[12,34,56,143]` is read through the actual MCP SDK; GUI undo restores the clean saved state. The session selector switches to the MCP-created inactive 2×2 document. Closing with the saved sprite active displays a dialog identifying the two other unsaved documents; cancel leaves the window open. Evidence: `native-0.3.1-mcp.json`, `native-0.3.1-gui-mcp.json`; native AX/screenshot observations in the chat.

Native QA exposed a missing Tauri `core:window:allow-destroy` capability. It was added for the main local window only. This demonstrates why headless tests do not substitute for native closure checks.

The corrected final bundle was launched as a second isolated QA copy. Clicking the native close button presented the unsaved-document guard; confirming for its empty QA canvas terminated that QA process. App inventory confirmed both QA processes stopped while the preexisting user SpriteLab session remained running.

The generated app was packaged in a temporary local directory to avoid Finder metadata added to app directories by the workspace file provider. No quarantine attribute was present on this locally built code. The ad-hoc signed ZIP was extracted again into a fresh temporary directory, where `codesign --verify --deep --strict` passed. The launcher repeats this extraction and verification before opening; assets stay in the normal workspace. This is not a Developer ID signature or notarized public release.

Final ZIP native check: the isolated QA identity was re-signed after changing its bundle ID, to keep the test signature valid. The app initialized its workspace. Actual MCP closed the sole empty document; the GUI cleared the canvas/context and disabled asset controls. The native New dialog then created another 32×32 document and restored editing. Closing this empty QA document completed normally. Evidence: `native-0.3.1-empty-mcp.json`.

## Version 0.4.0

31 Rust tests and 6 real SDK MCP tests passed, including recovery across independent runtimes. Formatting, locked workspace Clippy/build and desktop TypeScript/Vite build passed. Native GUI restoration and exact document equality were confirmed through attached MCP; the approved slime file was preserved. See [release record](RELEASE-0.4.0.md) and [native recovery evidence](native-0.4.0-recovery.json). Performance budgets and Windows/Linux remain unverified.

## Version 0.4.1

31 Rust tests, 6 official SDK MCP tests, locked build/Clippy, formatting and TypeScript/Vite build passed. Native menus opened the separate file dialog; opening the slime preserved the unsaved editor tab. Rectangle and selection tools exposed only their relevant controls; selection actions were disabled without a region. Attached MCP confirmed two preserved documents and exact original slime content at revision 0. Evidence: native-0.4.1-workspace.json. This is scoped interaction QA, not an exhaustive UI certification.

## 0.4.2 — timeline de cels

Validado em macOS arm64: fmt, Clippy workspace/all-targets/locked com warnings negados, 33 testes Rust (22 core + 11 runtime), build locked; npm ci ignore-scripts e 7 testes SDK MCP, TypeScript/Vite e bundle Tauri. Scanner de design: 0 sinais no código; inspeção nativa confirmou layout da grade.

Documento de QA criado via MCP numa nova janela: dois quadros/duas camadas. Cópia do cel original e colagem no segundo quadro pela GUI resultaram em exatamente uma revisão; MCP confirmou buffers iguais. Command-Z pela GUI e inspeção MCP confirmaram recuperação integral da estrutura/conteúdo inicial. Evidência: `native-0.4.2-timeline.json`. O documento não foi salvo sobre assets do usuário.

Transferência e rejeição de destino ocupado, bloqueios, IDs inválidos, rollback em batch, no-op, undo/redo e cópia independente foram testados no core; descoberta e execução através do SDK MCP também passaram. Drag com modificador, resize por pointer e todos os tamanhos de janela não receberam cobertura interativa exaustiva. Cels vinculados e seleção múltipla continuam pendentes. Bundle local ad-hoc signed/debug, sem notarização ou publicação.

## 0.4.3 — camadas sem duplicação

Passaram fmt, Clippy locked/all-targets com warnings negados, build locked, 34 testes Rust (22 core + 12 runtime), npm ci ignore-scripts, 8 testes MCP SDK e build TypeScript/Vite/Tauri. Miniatura isolada foi testada com alpha, camada oculta/opacidade zero, cel ausente, IDs inválidos e ausência de mutação/revisão. O teste MCP confirmou a imagem PNG isolada pelo SDK real.

Na janela nativa 0.4.3, Slime-Menta foi aberto pela GUI: três camadas com miniaturas próprias apareceram somente na lateral; timeline com 32 quadros sem grade de camadas ou barra de cels. Selecionar Gel na lateral e o quadro 11 na timeline funcionou. Miniaturas conferidas visualmente após o carregamento. Não foi feita edição persistente no projeto do slime. Bundle ZIP verificado com codesign. A inspeção não cobre todas as janelas/tamanhos ou todos os recursos do editor.

## 0.5.0 preview release gate

Local macOS arm64: fmt, Clippy locked/all-targets/warnings denied, locked build and 34 Rust tests passed. Nine real MCP tests passed, including a host killed with SIGKILL after checkpoint and restoration in another process: editable state/undo preserved and original saved project not replaced. Config registration was tested with relocated paths containing spaces, explicit failure cases and a mock Codex CLI (no real global writes during unit tests).

Portable ZIP was extracted/installed to a different location; signature verified and `--print` resolved engine/MCP paths from the installed directory. Actual installed SDK discovery/preview must use the matching workspace; no assumption of shared headless state. TypeScript/Vite/Tauri build passed. Catalog compared byte-for-byte to the binary. YAML syntax checked. Tracked source credential-pattern scan found no matching tokens/private keys; this is a targeted scan, not a security audit.

Native 0.5.0 opened an original slime copy, with layer thumbnails and compact frame timeline. Screenshot and actual ScreenCaptureKit app-window recording are in docs/media; 235 frames, approximately 8.23 seconds, 1280×860. Video decoded through FFmpeg without errors and a representative frame was inspected. GIF is derived from the real recording; the separately labeled promotional cover is not evidence of engine output.

`benchmark-0.5.0.json` records 128×128, 16 frames/two layers (524,288 cel pixels), 40 sequential JSONL round trips in a debug build. Median stroke edit about 2.9 ms; median PNG preview about 4.6 ms on this machine. These are workload-specific, include serialization, exclude GUI/MCP, and do not measure peak memory or certify production performance.

GitHub runner validation is distinct from these local checks and must be observed after publication. Windows GUI/runtime, Linux GUI, notarization, crash within filesystem publication and power-loss scenarios remain unvalidated.
