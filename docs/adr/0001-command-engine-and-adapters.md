# ADR 0001: Command Engine owns durable editing

Status: accepted for the initial implementation.

Human tools, scripts and agents must produce the same operations. Keeping business rules in an MCP server would make headless/desktop use depend on a protocol designed for external clients.

Decision: original Rust domain code owns mutation, validation, transactions, rendering and history. A transport-independent envelope/catalog connects it to application runtime, CLI, MCP and future Tauri UI. File operations use shared runtime ports. The MCP adapter contains no painting algorithms or mutation implementation.

Consequences: remove the MCP adapter and the CLI/core remain fully functional. Add a new adapter without rewriting drawing rules. Parity is tested through actual consumer paths. The registry has stable IDs/versioning; viewport state is distinct from persisted asset state.
