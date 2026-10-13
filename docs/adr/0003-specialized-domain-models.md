# ADR 0003: Specialized asset domains share infrastructure

Status: accepted architecture; generic domain host/plugins/catalog are not implemented.

Decision: raster documents keep layer/frame/cel/image semantics. Geometry domains get mesh/material/scene/animation models and kernels. Share command envelopes, discovery and capabilities, scheduling/history interfaces, jobs, I/O facilities and asset/dependency metadata.

Consequence: never place vertices or skeletons into the raster cel model. Register versioned domain operations behind common contracts. Extract engine services based on a second domain/live-session need, preserving SpriteLab standalone use. A plugin ABI, WebAssembly isolation or subprocess boundary requires a separate decision; Rust data structures alone are not a stable binary plugin ABI.
