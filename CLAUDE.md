# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Slipstream Governance (read first)

Read `MANIFEST.md` at session start. It lists which Slipstream personas,
constitutions, and skills are active. The base constitution at
`slipstream/core/constitution/BASE_CONSTITUTION.md` is **supreme** — every
persona, skill, and instruction here is subordinate to it. Active personas
(producer, game_designer, gameplay_engineer) live in `.claude/agents/`; adopt
one by name. Log decisions and human rejections to `STATE.md`.

## Project Overview

**Saturday Morning Engine** — an opinionated 2D/2.5D game engine targeting a "Saturday morning cartoon" aesthetic (Cuphead-adjacent staging, Scott Pilgrim-adjacent readability). This is a **style engine**, not a general-purpose engine.

**Current status:** Implemented Rust engine with native sandbox/Lua and native + WASM
examples. Web-first, lightweight, not web-only; preserve desktop publishing options.
Source and `STATE.md` take precedence over historical planning claims. Current shared
runner, event queues, input/timing and authored-data APIs are documented in
`docs/ENGINE_FOUNDATION.md`. Telemetry with encryption is planned only, not implemented.

## Architecture Summary

The engine uses a **sprite-scene-first** world model: art is authored as layered illustrations, while collision and gameplay run on a separate simplified representation (the "collision underlay").

**Language:** Rust for engine/core and current browser gameplay; native sandbox Lua 5.4
via mlua (not LuaJIT). Native Lua does not currently build for the WASM browser target.

### Module Hierarchy (top-down dependency)

```
Game
  → Game Framework (Scene + Entities + Systems)
    → Engine Core (Time + Input + Assets + Config + Profiler)
      → Rendering    Collision
        → Platform Layer (Window + Input + Audio + Filesystem)
```

**Rules:** Game never talks directly to Platform. Rendering never owns gameplay truth. Devtools observe everything, mutate only through explicit APIs.

### Key Design Pillars

- **Deterministic simulation:** Fixed timestep (60 Hz), inputs sampled deterministically, simulation state only mutates inside `update_fixed()`
- **Sprite-scene-first:** Scenes are ordered layers with parallax, depth sorting, and foreground occlusion — not tilemaps
- **Collision underlay:** Gameplay truth lives in a separate grid+vector collision representation, independent of visuals
- **Fidelity tiers:** Tier 0 (mobile-safe baseline) through Tier 2 (PC polish). Tiers add optional fidelity without changing core architecture or simulation
- **Hot reload:** Atlases, scenes, and collision data reload at safe frame boundaries

### Toolchain

- **Build:** Cargo workspace
- **Rendering:** wgpu 24; WebGL2 on web, DX12/Vulkan selected on native
- **Windowing/Input:** winit 0.30; shared thin runner in `sme_platform::app`
- **Lua runtime:** mlua with vendored Lua 5.4, native sandbox only
- **Debug UI:** egui 0.31
- **Audio:** no shared engine audio runtime yet; standalone games own their audio backends
- **Verification:** workspace tests/build/clippy/fmt and scripts in `scripts/`.
  GitHub Actions CI remains future work; local scripts are not an installed CI service.

See `docs/planning/implementation_decisions.md` for full decision status (accepted vs proposed).

### Planned Directory Layout

```
/crates
  /platform    — window, input, audio, filesystem
  /core        — time, assets, config, profiler
  /render      — sprite batching, camera, layers, debug draw
  /collision   — grid, AABB, vector colliders
  /framework   — scene, entities, systems
  /devtools    — debug overlay, profiler UI
/tools
  /atlas_packer
/game          — vertical_slice
/assets        — raw/, built/
/docs/planning — design docs
```

## Canonical Documents (Read Before Making Decisions)

Priority order when conflicts arise: `scope.md` wins over `implementation_decisions.md`.

| Document | Purpose |
|----------|---------|
| `docs/planning/scope.md` | **v0.1 scope lock** — what's in and out |
| `docs/planning/engine_identity.md` | Engine identity, aesthetic rules, non-goals, world model |
| `docs/planning/architecture_sketch.md` | Module structure, main loop, scene/collision/rendering design |
| `docs/planning/implementation_decisions.md` | Toolchain/backend choices (check approval status) |
| `docs/planning/producer_review.md` | Milestone definitions with acceptance criteria, dependency map, risks |
| `.claude/implementation_decisions.md` | Decision ledger template |

## v0.1 Milestones

```
M1: Core loop + sprite rendering + debug overlay (blocks everything)
M2: Scene layers + parallax + occlusion        ─┐
M3: Collision underlay + character controller    ├─ parallel after M1
M4: Atlas packer + stable asset IDs             ─┘
M5: Hot reload + tier toggles + polish (requires M1-M4)
```

M2 and M3 can proceed in parallel after M1. M4 can also overlap. M5 is the integration milestone.

## Hard Non-Goals (v0.1)

- No general-purpose editor suite
- No networked multiplayer
- No general 3D engine / PBR / AAA lighting
- No multi-backend rendering abstraction (pick one backend, ship it)
- No plugin marketplace or mod framework

## Skills and Agents System

This repo has a Claude Code skills/agents framework under `.claude/skills/` and `.claude/agents/`. Skills are organized by domain (rendering, audio, assets, etc.) with numbered subskills. Agents have defined domain ownership and handoff rules.

**Key pattern:** The orchestrator agent (`orchestrator-technical-producer`) owns roadmap/scope/risk. Domain-specific work is handed off to specialist agents (engine-architect, rendering-engineer, gameplay-scripting-engineer, etc.).

**Decision discipline:** Major decisions must be recorded in `implementation_decisions.md` with Decision / Rationale / Alternatives / Revisit condition.

## Build Commands

Run from repository root:

```bash
cargo build              # Debug build
cargo build --release    # Release build
cargo run                # Run the engine/game
cargo test               # Run all tests
cargo test <test_name>   # Run a single test
cargo clippy             # Lint
cargo fmt                # Format
```

## Current Boundaries / Next Work

1. Portable byte parsers exist; a shared browser asset fetch/cache and packaged authored
   scene demo are next. Do not mistake filesystem hot reload for browser asset loading.
2. Event queues are local, typed and bounded. Future encrypted telemetry must remain
   unimplemented until receiver, privacy and key-management requirements are approved.
3. GPU failure/recovery UX, CI, export manifests and packaging budgets remain future work.
4. Standalone games live in separate repositories. Do not change controller feel or
   promote disposable example gameplay into production merely by extracting it.
5. Historical plans below `docs/planning/` contain proposed features that may not exist.
   Verify actual source and current state before asserting implementation.
