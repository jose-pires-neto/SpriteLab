# ADR 0004: Versioned open project; file effects are not document history

Status: prototype format accepted for review; not a permanent compatibility promise.

Decision: `.spritelab` v1 is human-readable JSON with kind/version and full editable document payload. Imports validate before installation. Save/PNG export stages a same-directory file and publishes it atomically at the filename level. Explicit overwrite is required. Atlas metadata is returned separately; no multi-file atomicity claim.

Consequences: external tools can read current projects without the editor. Revision/history/viewport remain session state. Future manifest/blob containers require explicit migrations. Add crash recovery, directory durability, multi-artifact publication and platform QA before release. The workspace path policy is not a hostile-process sandbox.
