# Contributing

Implement original code and assets. Aseprite investigation material is reference evidence, not permission to copy its implementation or artwork.

Every new durable operation must add a typed engine command, generated schema/catalog entry, semantic validation, explicit effect/undo behavior, documentation and relevant tests. UI and transport adapters delegate; they do not mutate owned raster buffers or invent editing rules.

Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`, `cargo build --locked`, then `npm ci --ignore-scripts && npm test` under `adapters/mcp`. Python 3 is needed by the adapter parity suite. Update `docs/commands.json` from `spritelab --workspace demo --catalog` when command schemas change. Keep lockfiles committed.

Tests should prove behavior: undo round trips, failure rollback, revision conflicts, renderer/export agreement and real adapter parity. Avoid tests that merely reproduce implementation steps. New formats need golden fixtures and explicit unsupported-feature reporting. Performance claims require measurements and a stated document/workload.

The provided CI configuration is a proposed workflow; it has not run on GitHub. Public repository creation, governance and release distribution remain separate actions.
