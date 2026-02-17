# IMPLEMENTATION_DECISIONS.md

Status: ACTIVE. Update this file when technical build decisions change.

## Canonical Rule
- This file is the canonical source for implementation choices (toolchain, backend, platform layer, CI/build flow).
- If this file conflicts with `docs/planning/scope.md`, `docs/planning/scope.md` wins.

## Decision Set (Fill Before Scaffolding)
1. Language/toolchain: `Rust (engine/platform/perf) + Lua (gameplay scripting, hot reload)` (accepted)
2. Build system: `Cargo (workspace)` (accepted)
3. Primary Windows render backend: `wgpu` (proposed)
4. Platform layer library (window/input): `winit` (proposed)
5. Asset ID strategy (GUID vs path-based): `GUID-based stable IDs` (proposed)
6. Scene/ECS model: `ECS-lite runtime + explicit ordered scene layers for authoring` (proposed)
7. Collision baseline for v0.1: `Grid collision map with optional vector refinement` (proposed)
8. Hot reload reset policy: `Apply only at safe frame boundary; atlas in place when valid; scene/collision reload may reset current scene` (proposed)
9. Debug UI stack: `egui overlay + debug draw primitives` (proposed)
10. CI baseline: `GitHub Actions (Windows) - cargo build, cargo test, cargo clippy, cargo fmt --check` (proposed)
11. Lua runtime: `mlua (LuaJIT backend)` (accepted - phased rollout M4/M5)
12. Audio library: `kira` (proposed)

## Rationale (Short)
- Rust enforces memory safety, fearless concurrency, and zero-cost abstractions — ideal for engine internals.
- Lua via mlua gives fast gameplay iteration with hot reload; LuaJIT backend for performance.
- wgpu provides a safe, cross-platform GPU abstraction (Vulkan/Metal/DX12) with a clear path to mobile.
- winit is the standard Rust windowing library with cross-platform support.
- GUID IDs protect asset references across renames/moves.
- ECS-lite + ordered layers matches deterministic runtime + authored scene identity.
- egui is a pure-Rust immediate-mode UI, simpler integration than FFI to Dear ImGui.
- kira provides expressive audio mixing with a Rust-native API.
- Collision/hot-reload choices align with locked v0.1 scope and safety rules.
- Lua integration is bounded: Rust remains simulation authority; Lua drives gameplay behavior decisions.

## Approval State
- Current state: PROPOSED defaults pending approval.
- When confirmed, replace `(proposed)` with `(accepted)` per line.

## Change Log
- 2026-02-15: File created to prevent workflow-link drift.
- 2026-02-15: Added proposed defaults and rationale for collaborative review.
- 2026-02-16: Pivoted from C++20/CMake/D3D11/SDL3 to Rust/Cargo/wgpu/winit. Language+build accepted; other decisions proposed for review.
- 2026-02-17: Marked Lua runtime decision accepted with phased M4/M5 rollout and deterministic boundary constraints.
