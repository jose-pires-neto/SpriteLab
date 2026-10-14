# Security

0.5.0 is an experimental local preview, not a hardened multi-user service. Use trusted projects and agents. Do not expose its Unix socket or stdio adapter over a public network. Any agent connected to the live MCP can edit/save/export within the selected workspace; approval decisions belong to its host.

Workspace path checks and a private Unix socket reduce accidental access. They do not promise confinement against concurrent hostile filesystem/symlink changes. Recovery snapshots can contain unsaved asset data. Documents/history, messages, connections and exports have bounded limits; see README and docs/ARCHITECTURE.md.

For security reports, use the repository's private vulnerability reporting when enabled. Otherwise contact the maintainer through their GitHub profile and arrange a private channel before sending sensitive details. Do not post credentials, tokens or exploitable private data in public issues. Normal reproducible bugs belong in Issues.

There is no formal response-time SLA. Only the current preview is maintained; fixes are released with explicit compatibility notes. Dependencies remain locked and are reviewed during releases. A checksum detects download changes; ad-hoc signing is not Developer ID authentication or notarization.
