# ADR 0002: Single writer, atomic drafts and monotonic revisions

Status: accepted for the prototype; patch storage and shared local host are planned replacements behind the same contract.

Decision: one runtime owns each document session. Typed editing commands run on a copy-on-write draft; a changed batch commits once. All intermediate domain states validate. Failure publishes neither state nor tentative IDs/history. Undo/redo increment revisions, and expected-revision guards prevent stale edits. IDs do not rewind during the session.

Tradeoff: conservative copy-on-write snapshots are simple and testable for bounded documents, but dense cel copying is unsuitable for professional high-volume editing. Replace internals with region/tile/structural patches after measuring representative workflows. I/O and lifecycle commands cannot join edit batches. Cross-process sessions are currently isolated, not automatically collaborative.
