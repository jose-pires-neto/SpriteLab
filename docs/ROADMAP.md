# Roadmap and release gates

Build an independent pixel-art editor first, with command parity as an architectural gate. Milestones are capability gates, not delivery-date promises.

| Milestone | Result | Gate | Current state |
|---|---|---|---|
| 0. Command-first foundation | Headless Rust raster domain, CLI/Python/MCP proof, schemas, project and PNG/atlas operations | Deterministic same-input results; failed batch rollback; undo round trip; stale revision conflict; real adapter parity | Implemented at prototype level |
| 1. Human-usable SpriteLab | Tauri/React shell, canvas, zoom/pan/grid, pencil/eraser/fill/line, layers/palette, save/import/export, history | An artist creates and reopens a real sprite; GUI submits the same engine edits; cancellation leaves no persisted pixels | Desktop, grid and scroll navigation implemented; dedicated pan, accessibility and broader QA pending |
| 2. Animation studio | Timeline, duration, frame duplication/reorder/removal, onion skin, playback, tags and export panel | Four-frame animated sprite composed manually and by script; displayed frame/export pixels agree; edit/export metadata parity | Frame reorder/removal, onion skin, tags and GIF implemented; professional timeline, playback modes and broader manual QA pending |
| 3. Shared local session and agents | Desktop-owned session host; CLI/MCP attach; stroke sessions; event subscription; job progress/cancel; resource limits | Human and agent edit the same document; explicit revisions, ownership and reconnection; interrupted work respects declared commit points | Unix attachment implemented and tested against the live GUI; events, jobs and reconnection pending |
| 4. Production raster engine | Tile/region history, sparse cel bounds, image store, deliberate linked cels, selections, symmetry, brushes, profiles/indexed color, GIF | Memory/latency budgets on representative documents; corruption/recovery tests; platform builds and manual QA | Rectangular selection, shapes, transforms, symmetry, square brush and GIF implemented; memory architecture, advanced selections/brushes/profiles pending |
| 5. Interoperability and ecosystem | Loss-aware `.aseprite` importer, Godot/Unity extensions, catalog and recipe pipeline, public SDK | Golden format fixtures; unsupported-feature reports; real engine import; plugin/isolation and license audit | Planned |
| 6. Another domain | A specialized 3D asset tool uses the common host/contracts and its own model/rendering/patches | New domain added without rewriting raster operations or transport adapters | Future |

The original proposal put CLI/SDK/headless work late and MCP in phase 3. Bring the CLI, a thin MCP adapter and parity tests into milestone 0; this makes the shared-engine claim testable before substantial UI code exists. Live session attachment is implemented. Event subscriptions, job progress/cancellation, reconnect semantics and durable idempotency remain later gates.

Before the first public release: human-usable editor, reliable undo/reopen, offline operation, dependency notices, contributor docs, Windows/macOS/Linux build and runtime checks, distributable original branding/assets, project schema compatibility policy and a reviewed publication workflow. No release gate is satisfied merely because a headless test passed.

## Immediate implementation backlog

The current feature matrix is maintained in [COBERTURA.md](COBERTURA.md). Every new feature must be accessible through generated command schemas and actual MCP parity tests.

1. Recovery checkpoints delivered in 0.4.0; broaden crash interruption tests and harden the newly implemented session document selector and all-document close guard. Measure rendering and memory on representative projects.
2. Improve masks, clipboard, irregular selections and transforms; introduce region patches before increasing current resource limits.
3. Expand the timeline and animation controls with tag selection, loop directions and linked cels.
4. Add layer groups, blend modes and merge semantics with canonical compositor tests.
5. Implement loss-aware format interoperability from documented specifications and golden fixtures; report unsupported features.
6. Add events, host reconnection and Windows named pipes while keeping headless sessions explicit.

Do not begin a 3D editor, generic plugin ABI or cloud asset service while the first manual sprite workflow remains incomplete. Preserve extension seams without prematurely implementing every future subsystem.
