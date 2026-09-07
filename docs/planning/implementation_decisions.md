# IMPLEMENTATION_DECISIONS.md

Status: ACTIVE. Update this file when technical build decisions change.

## Canonical Rule
- This file is the canonical source for implementation choices (toolchain, backend, platform layer, CI/build flow).
- If this file conflicts with `docs/planning/scope.md`, `docs/planning/scope.md` wins, subject to subsequent explicit owner decisions in STATE.md.

## Decision Set
1. Language/toolchain: `Rust + native Lua gameplay scripting` (accepted; Rust gameplay on web)
2. Build system: `Cargo workspace` (accepted)
3. Rendering: `wgpu` (implemented; DX12/Vulkan selected natively, WebGL2 on web)
4. Platform layer: `winit` (implemented)
5. Asset ID strategy: `GUID-based stable IDs` (implemented)
6. Scene model: `explicit ordered scene layers` (implemented; no general ECS required)
7. Collision: `bounded-speed grid-based kinematic AABB resolution` (implemented; arbitrary CCD deferred)
8. Hot reload: `safe boundary, stage/validate candidates, retain last-known-good on failure` (implemented)
9. Debug UI: `egui overlay + debug draw` (implemented)
10. CI baseline: `native build/test/clippy/fmt and web regression` (local scripts implemented; no GitHub Actions workflow currently installed)
11. Lua runtime: `mlua, Lua 5.4 vendored, native` (implemented; not LuaJIT, not browser Lua)
12. Shared engine audio: `kira` (accepted historical planning direction, NOT an implemented engine runtime)

## Rationale

Preserve the tested Rust/wgpu/winit foundation, native scripting and deterministic fixed
update boundary. Keep gameplay independent of platform transport and visual fidelity.
Do not mistake historical implementation claims about CI/audio for inspected source.
Standalone games own their audio/gameplay services until shared requirements are proven.

## Historical Change Log
- 2026-02-15: File created to prevent workflow-link drift.
- 2026-02-15: Added proposed defaults and rationale for collaborative review.
- 2026-02-16: Pivoted from C++20/CMake/D3D11/SDL3 to Rust/Cargo/wgpu/winit. Language+build accepted; other decisions proposed for review.
- 2026-02-17: Marked Lua runtime decision accepted with phased M4/M5 rollout and deterministic boundary constraints.
- 2026-02-17: Accepted remaining proposed decisions (3-10, 12) based on M1-M4 implementation. Lua 5.4 vendored accepted, with a LuaJIT swap option retained. Later source inspection corrected CI/audio implementation claims.

## 2026-09-07 — Foundation and package boundary

- **Decision:** Lightweight, web-first, not web-only. Share typed event queues,
  scene/atlas/animation/collision parsing, input/timing policy and a thin app runner.
  Keep target-specific services optional and game-specific state out of the engine.
- **Rationale:** Reuse tested functionality without copying host code into every game
  or forcing a heavyweight editor/ECS/network stack into browser builds.
- **Alternatives:** Engine/language rewrite, global callback bus and universal plugin
  system add migration/ordering/complexity costs without a demonstrated need.
- **Dependency impact:** Existing pollster and wasm-bindgen-futures versions only;
  target-specific direct dependencies, no new crate/version or online state.
- **Revisit:** A second concrete service implementation or measured target limitation
  justifies a new adapter; browser scripting requires a dedicated compatibility spike.
- **Distribution:** Public engine workspace excludes private game source/assets/docs.
  External game Cargo dependencies pin an exact engine commit.
- **Future telemetry:** Event/metric packaging and encrypted export requested but
  explicitly deferred. No collector, exporter or crypto implementation. Receiver,
  privacy, retention, payload/transport protection and key-management requirements
  must be resolved separately before implementation.
