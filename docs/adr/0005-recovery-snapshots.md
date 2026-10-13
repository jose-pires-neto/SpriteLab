# ADR 0005: Runtime recovery snapshots

Accepted for SpriteLab 0.4.0.

Recovery is a Rust runtime file effect, shared by desktop, CLI and MCP. `recovery.checkpoint` atomically writes a versioned document snapshot without changing revision, undo or saved baseline. Each runtime owns a unique namespace; independent sessions never overwrite each other's checkpoints. Repeated identical checkpoints avoid writes. The desktop checkpoints all dirty documents every 15 seconds, including edits submitted by attached agents. The interval can lose the last 15 seconds on abrupt termination; this is not a write-ahead log or a power-loss durability guarantee.

`recovery.list` discovers previous-session snapshots, including damaged entries with explicit errors. `recovery.restore` validates and installs a new unsaved document; it never overwrites a current document or source file. `recovery.delete` requires an explicit entry and only deletes that checkpoint. Recovery operations do not enter pixel undo and cannot join an edit batch. Snapshot paths are restricted by the runtime workspace policy, and symlink recovery directories are rejected.

Snapshots live in `.spritelab-recovery`, with a maximum of 128 entries and 16 MiB per file. No silent pruning: storage exhaustion is reported in the desktop recovery panel. Recovery preserves content, not history or viewport. Closed documents retain their disk checkpoints. Headless runtimes expose the same commands but do not start an automatic timer.
