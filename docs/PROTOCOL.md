# Command protocol v1

The JSONL CLI and Rust runtime accept the same envelope:

```json
{"request_id":"job-17","document_id":"hero","expected_revision":5,"command":"canvas.stroke","args":{"layer_id":1,"frame_id":2,"points":[[1,2],[8,2]],"color":[255,64,32,255]}}
```

`command` is mandatory. `args` defaults to `{}`. Other envelope fields are optional structurally. `document_id` is mandatory for document-targeted commands; `expected_revision` must be omitted for create/open/import. IDs are case-sensitive ASCII letters/digits/hyphens/underscores, length 1–64. Object IDs are unsigned integers bounded to the prototype's supported range. RGBA channels are bytes; coordinates are signed integer canvas coordinates. Frames are addressed by ID, not timeline index. Out-of-bounds painting rejects the whole command; there is no silent clipping. Unknown fields reject.

Responses include `protocol_version: 1`, the request ID, `ok`, the current revision where applicable, and exactly one of `result` or `error`. Errors have stable code and diagnostic message. The message is for humans; clients branch on code. Errors leave document edits uncommitted. Malformed JSON has no recoverable request ID. The CLI continues after a command error and reports it as JSON; process status reflects transport startup/I/O, not whether every request succeeded.

Codes include `invalid_request`, `invalid_arguments`, `unknown_command`, `not_found`, `already_exists`, `revision_conflict`, `layer_locked`, `history_empty`, `resource_limit`, `path_denied`, `overwrite_required` and `io_error`. `request_id` correlates responses only. Replaying a successful edit is a new edit; durable deduplication is not implemented.

`command.list` or CLI `--catalog` provides schemas from Rust types. Structural schemas describe accepted argument shapes; semantic constraints (IDs existing, tag ordering, layer limits, locked state, memory budget, revision and workspace permissions) are enforced by the engine/runtime. The generated catalog in `commands.json` is a convenience snapshot, not another implementation.

## Batch

```json
{"document_id":"hero","expected_revision":5,"command":"command.batch","args":{"commands":[{"command":"frame.duration","args":{"frame_id":2,"duration_ms":160}},{"command":"layer.update","args":{"layer_id":1,"name":"Body"}}]}}
```

1–256 typed editing commands target one document. Query, save, export, create/open/close, history and nested batch commands are not valid batch members. The caller supplies explicit existing IDs; newly allocated IDs are returned after commit, so a batch cannot use an ID alias invented by a previous child. To create then edit a new layer, use separate calls and read its returned ID. One changed batch adds one revision and undo entry. Every intermediate state must validate. No external filesystem rollback is implied.

## Files and previews

Projects are plain `.spritelab` JSON `format_version:1`, `kind:"sprite"`. They contain no transport IDs, revision or undo log. Creation/open/import assigns a new session identity and revision zero. Save/export requires workspace-relative paths and `overwrite:true` to replace files. `document.close` discards the session without autosave.

`document.render` returns canonical flat RGBA byte data. `document.preview` returns a base64 PNG and dimensions. MCP adapts that PNG to an image content block and returns metadata/response in structured content. `export.spritesheet` writes an untrimmed row-major PNG atlas with a border and inter-frame padding. It returns `spritelab.atlas/1` metadata: image, size, frame IDs, pixel rectangles, duration_ms and forward tags. This is a documented neutral format; it is not a ready-made Godot `.tres` or Unity importer.

Headless processes own isolated sessions. Desktop GUI and explicit attached CLI/MCP clients serialize requests through the same runtime and share document revisions. The adapter's 64-request queue and 30-second engine timeout are transport limits. On a timeout the adapter closes the session and explicitly reports uncertain external write outcome; it does not claim rollback. There are no progress/cancellation/job messages in v1.

`command.preview {edit, frame_id}` validates/applies a single typed edit to a detached document and returns canonical flat RGBA bytes. It checks expected_revision but does not publish a revision, consume IDs or add history. Invalid drafts return ordinary engine errors. GUI pointer-up submits the same edit directly; cancel submits nothing.

## Version 0.3 additions

Layer duplication/reordering and frame removal/reordering are reversible typed edits. Frame tag repair retains surviving prior members: boundaries become their minimum/maximum positions after reordering; this can include intervening frames. Tags with no surviving member are removed. IDs stay stable and undo restores the original range.

Rectangles are integer, canvas-bounded regions. `canvas.transform` supports flip_x, flip_y, rotate90, move, copy and clear. Overlapping moves snapshot the source before clearing; out-of-bounds destinations fail atomically. Rotation is exact clockwise 90°, not arbitrary rotation. Resize uses nearest-neighbor; canvas_size crops/pads with explicit offsets. Shapes and brushes replace RGBA samples; they do not alpha-blend a new brush dab into the active cel. Mirror reflects sample centers; even-size square brushes use a floor-half anchor. Pixel perfect removes orthogonal single-pixel staircase corners.

`editor.configure` changes shared view state without asset history; `editor.inspect` reports active context, current revision, last saved path and dirty status by comparing live content with the saved baseline. `workspace.inspect` lists up to 1024 project/PNG/GIF filenames. `document.snapshot` atomically returns editable state and a canonical composited frame. `document.onion` renders a separate red/blue adjacent-frame overlay; export pixels exclude it.

GIF export uses per-frame quantization to <=256 colors, thresholds alpha at 128, rounds durations to centiseconds with minimum 10 ms, loops indefinitely and clears each frame to avoid trails. It preserves neither arbitrary RGBA alpha nor sub-centisecond timing. These format limitations are returned as metadata. GIF is a file effect, not an asset mutation.

`document.list` returns session documents with revision, path (null until saved/opened) and dirty status. `editor.inspect` includes the same document list alongside the selected context. Color alpha and `alpha` are one view value; when both are configured explicitly, `alpha` takes precedence.
