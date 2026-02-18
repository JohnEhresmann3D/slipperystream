# Saturday Morning Engine — Code Review Report

**Branch:** `m4-atlas-packer-stable-asset-ids` | **Codebase:** 3,847 lines / 20 files / 7 crates | **Tests:** 19/19 passing
**Date:** 2026-02-17

---

## 11.1 Architecture & Module Boundaries — CLEAN

The dependency graph is **correct and well-layered**:

```
sme_game → sme_render → sme_platform (winit only)
         → sme_core (leaf, no deps)
         → sme_devtools → sme_core
sme_atlas_packer (standalone)
```

- No upward dependencies, no circular references
- `pub`/`pub(crate)` discipline is proper across all crates
- No rendering code owns gameplay truth
- Devtools observes but never mutates game state
- Zero unsafe blocks anywhere

**Verdict: No issues.**

---

## 11.2 Safety & Correctness — 1 Warning

All `unwrap()`/`expect()` calls are on initialization paths (GPU init, window creation, event loop) where panicking is appropriate. Error propagation via `Result<T, String>` is consistent in loaders.

| Severity | Finding | Location |
|----------|---------|----------|
| **Warning** | Texture size validation uses unchecked multiply `(width as usize) * (height as usize) * 4` — could overflow on extreme dimensions | `sme_render/src/texture.rs:81-85` |
| Note | egui_winit::State::new() 3 trailing `None` params undocumented | `sme_devtools/src/debug_overlay.rs:25-32` |

**Positives:** Checked arithmetic in atlas validation (`checked_add`), transactional file writes with rollback in atlas packer, graceful GPU surface loss handling, proven deterministic simulation.

---

## 11.3 Performance & Allocation Patterns — 3 Critical, 3 Warning

This is the area with the most actionable findings.

| Severity | Finding | Location |
|----------|---------|----------|
| **Critical** | `DrawCall.texture_key: String` — allocates a new String per draw call per frame via `.to_string()`. Should be `&'static str`, `Cow`, or a `TextureId(u32)` newtype | `main.rs:36-41, 925` |
| **Critical** | `layer.sprites.clone()` — full Vec clone every frame when Y-sort is active, just for sorting | `main.rs:424` |
| **Critical** | `build_mesh()` creates fresh `Vec::new()` x3 every frame with no capacity hints from previous frame | `main.rs:396, 413-416` |
| **Warning** | `FALLBACK_TEXTURE_BYTES.to_vec()` copies embedded bytes on every failed texture load instead of caching | `main.rs:940` |
| **Warning** | Vertex attribute offsets are hardcoded magic numbers (0, 8, 16) — should derive from struct layout | `sme_render/src/vertex.rs:10-35` |
| **Warning** | Scene validated twice — once in `load_scene_from_path()`, again in main via `validate_scene_sprite_references()` | `scene.rs:95-101`, `main.rs:136` |

**Recommendations (priority order):**
1. Replace `DrawCall.texture_key: String` with `TextureId(u32)` — eliminates per-sprite allocation + string comparison in render loop
2. Sort sprites by reference (index sort) instead of cloning the Vec
3. Carry `Vec` capacity across frames or use a reusable scratch buffer

---

## 11.4 API Surface & Ergonomics — 6 Notes

| Severity | Finding | Location |
|----------|---------|----------|
| Warning | `DrawCall` uses `String` key instead of typed ID — easy to misuse | `main.rs:36-41` |
| Note | `Camera2D.viewport` and `GpuContext.size` use raw `(u32, u32)` tuples — easy to swap width/height | `camera.rs:11`, `gpu_context.rs:10` |
| Note | `InputState` missing `is_mouse_just_released()` in public API | `sme_core/src/input.rs` |
| Note | `SceneWatcher` is used for scene, collision, AND atlas files — should be a generic `FileWatcher` in sme_core | `scene.rs:65-93` |
| Note | `PlatformConfig.title: String` allocates for a typically static window title | `window.rs:5-7` |
| Note | Inconsistent error types — some modules use `Result<T, String>`, main uses `panic!()` | Multiple files |

---

## 11.5 Test Coverage & Quality — 6 Critical Gaps

The 19 existing tests are **high quality** (determinism proofs, field-level assertions, tolerance-based float comparisons). The problem is **coverage breadth** — only sme_game modules have tests.

| Severity | Finding | Location |
|----------|---------|----------|
| **Critical** | `sme_core::TimeState` — NO TESTS for spiral-of-death capping, accumulator logic, interpolation. This is the foundation of deterministic simulation | `sme_core/src/time.rs:24-85` |
| **Critical** | `sme_core::InputState` — NO TESTS for key state transitions, frame lifecycle cleanup, mouse tracking | `sme_core/src/input.rs:40-108` |
| **Critical** | `sme_render` — NO TESTS for Camera2D transform math, Texture validation, SpritePipeline creation | Entire crate |
| **Critical** | `sme_devtools::DebugOverlay` — NO TESTS | Entire crate |
| **Critical** | `sme_atlas_packer` — NO TESTS for packing algorithm, ID registry persistence, transactional output promotion | `sme_atlas_packer/src/main.rs` |
| **Critical** | `sme_platform::create_window()` — NO TESTS | `window.rs:21-30` |
| Warning | Test isolation — all tests use raw filesystem temp files, no tempfile crate. `temp_file_path()` helper duplicated 4 times | `atlas.rs`, `collision.rs`, `scene.rs`, `replay.rs` |
| Warning | Missing edge cases: zero-sized AABB, empty sprites array, extreme float values, very large repeat counts in replay | Multiple |

**Coverage map:**

| Crate | Test Count | Status |
|-------|-----------|--------|
| sme_core | 0 | **UNTESTED** |
| sme_platform | 0 | **UNTESTED** |
| sme_render | 0 | **UNTESTED** |
| sme_devtools | 0 | **UNTESTED** |
| sme_atlas_packer | 0 | **UNTESTED** |
| sme_game | 19 | Good coverage |

---

## 11.6 Style & Consistency — Clean

| Severity | Finding | Location |
|----------|---------|----------|
| Note | Zero `///` doc comments on any public API — `cargo doc` would be sparse | All crates |
| Note | Import grouping lacks blank-line separators between std/external/crate groups | Most files |
| Note | No module-level orientation comments (per commit 90eabf5 intent) | All files |

**Positives:** `cargo fmt --check` passes clean, naming conventions 100% consistent (snake_case, PascalCase, SCREAMING_SNAKE), zero TODO/FIXME debris, zero commented-out code, zero dead code warnings, zero unused imports.

---

## Summary Scorecard

| Domain | Grade | Critical | Warning | Note |
|--------|-------|----------|---------|------|
| 11.1 Architecture | **A** | 0 | 0 | 0 |
| 11.2 Safety | **A-** | 0 | 1 | 1 |
| 11.3 Performance | **C** | 3 | 3 | 0 |
| 11.4 API Ergonomics | **B** | 0 | 1 | 5 |
| 11.5 Test Coverage | **D** | 6 | 2 | 0 |
| 11.6 Style | **A-** | 0 | 0 | 3 |

---

## Top 5 Priorities

1. **Add tests for `sme_core` (TimeState + InputState)** — These are the determinism foundation with zero test coverage
2. **Eliminate per-frame String allocation in DrawCall** — Replace with `TextureId(u32)` newtype
3. **Stop cloning sprite Vec for Y-sort** — Sort by index reference instead
4. **Add tests for `sme_atlas_packer`** — Packing algorithm + transactional writes have no safety net
5. **Add `Vec` capacity reuse in `build_mesh()`** — Carry capacity across frames

The architecture and safety story is strong. The hot-path allocation patterns and test coverage gaps are the two areas that need attention before M5.
