# SCOPE_v0.1.md

## Saturday Morning Engine (2D/2.5D) - v0.1 Scope

Status: LOCKED unless explicitly reopened.

---

## 1. v0.1 Goal

Deliver a playable vertical slice that proves engine identity.

Must demonstrate:

- Layered sprite scene (parallax + foreground occlusion)
- Deterministic fixed-timestep movement
- Collision underlay (grid-based, optional vector refinement)
- Texture atlas batching
- Debug overlay with measurable stats
- Hot reload for atlas + scene + collision
- Tier toggles (Mobile-safe baseline + PC polish tier)

Success Criteria:
The engine produces a distinct "Saturday morning" feel within 30 seconds of running.

---

## 2. Target Platforms

Primary:

- Windows PC

Design Constraint:

- Must run within mobile-safe performance budgets (even if mobile build is not shipped in v0.1)

Deferred:

- iOS / Android runtime builds
- Console targets
- Cloud deployment (architecture must not block it)

---

## 3. Runtime Features - In Scope

### 3.1 Platform Layer

- Window creation
- Event loop
- Keyboard + mouse input
- Basic gamepad support
- Minimal audio playback
- File I/O abstraction

### 3.2 Simulation

- Fixed timestep (60 Hz default)
- Deterministic input sampling
- Explicit update loop
- Pause and single-step debug control

### 3.3 Rendering

- Sprite renderer with batching
- Texture atlas support
- Orthographic camera
- Scene Layers:
  - Ordered layers
  - Parallax factor
  - Sort modes (None, Y-sort)
  - Foreground occlusion layer
- Debug draw primitives (lines, rects)
- Debug text rendering (bitmap font acceptable)

### 3.4 Scene System

- Scene loader (JSON or compact engine format)
- Layer definitions
- Sprite instance definitions
- Entity IDs

### 3.5 Collision Underlay

- Grid-based collision map
- AABB collision response
- Debug collision visualization
- Optional vector colliders (if low-risk)

### 3.5.1 Physics Baseline (v0.1)

In scope for v0.1 (M3):

- Deterministic fixed-step kinematic movement (`input -> desired motion -> collision-resolved motion`)
- Axis-separable AABB move-and-slide against collision underlay
- Collision query helpers needed for controller behavior (blocked axis, overlap checks)
- Stable player motion at 60 Hz fixed timestep (no tunneling in normal play speeds)
- Debug visualization for collision cells and character bounds

Out of scope for v0.1:

- General-purpose rigid body simulation
- Stacks/constraints/joints
- Continuous collision detection for arbitrary high-speed bodies
- Advanced materials (mass, restitution, friction coefficients)
- Fully generalized slope/step solver

Ownership split:

- Engine owns deterministic collision/movement primitives.
- Game code owns movement feel tuning (accel/decel, jump arcs, coyote time, abilities).

### 3.6 Hot Reload

Must support:

- Atlas reload
- Scene reload
- Collision reload

Rules:

- Reload only at safe frame boundaries
- No partial corrupted states
- Reload may reset current scene safely

### 3.7 Debug & Profiling

Overlay must display:

- FPS
- Frame time (ms)
- Fixed timestep stats
- Draw calls
- Sprite count
- Atlas binds
- Basic memory usage estimate

### 3.8 Lua Gameplay Scripting (v0.1 Bounded Scope)

In scope for v0.1:

- Embed Lua runtime for gameplay behaviors (entity/controller logic only)
- Script hot reload at safe frame boundaries
- Deterministic scripting contract:
  - Lua update runs only during fixed-step simulation updates
  - No direct wall-clock or platform API access from gameplay scripts
- Explicit Rust->Lua API boundary for:
  - Input queries
  - Read/write gameplay component state
  - Spawn/despawn and event callbacks

Out of scope for v0.1:

- General plugin/mod framework
- Unrestricted scripting access to platform/render internals
- Live editing of engine-core logic in Lua

---

## 4. Tooling - In Scope

### 4.1 Texture Atlas Packer

- Packs sprites into atlas textures
- Outputs metadata with stable IDs
- Emits consistent UV mappings
- Generates versioned output

### 4.2 Scene Authoring

- External JSON workflow acceptable
- Manual editing acceptable
- Minimal internal editor only if trivial

---

## 5. Out of Scope (v0.1)

- General-purpose editor suite
- Multiplayer / networking
- Full plugin/mod scripting ecosystem
- Advanced physics engine
- Full 3D mesh pipeline
- Skeletal animation system
- Multi-render-backend abstraction
- Plugin ecosystem
- Save/load persistence beyond scene reload
- UI framework beyond debug tools

---

## 6. Fidelity Tiers

Tier 0 (Mobile-Safe Baseline):

- No dynamic lighting
- Minimal particles
- No heavy post-processing

Tier 2 (PC Polish):

- Optional stylized lighting pass
- Optional bloom or rim-light
- Increased particle counts

Tier changes must never affect simulation or determinism.

---

## 7. Deliverables Checklist

- Builds and runs on Windows
- Demonstrates layered sprite scene
- Character moves and collides deterministically
- Debug overlay functional
- Hot reload stable
- Tier toggles functional
- One command builds and runs sample

---

## 8. Milestones

M1 - Core loop + sprite rendering + debug overlay  
M2 - Scene layers + parallax + occlusion  
M3 - Collision underlay + character controller  
M4 - Atlas packer + stable asset IDs  
M5 - Hot reload + tier toggles + polish pass + Lua gameplay bridge

### M2: Scene Layers + Parallax + Occlusion

Goal: Prove the sprite-scene-first authoring model with layered illustration.

Acceptance Criteria:

- Scene loads from a JSON file
- 3+ layers (background, mid, foreground) with independent parallax factors
- Foreground layer acts as occlusion mask (draws in front)
- Camera movement demonstrates parallax effect
- Layers support Y-sort mode (sprites sort by Y position)
- Scene can be reloaded at runtime (hot reload foundation)

Key Deliverables:

1. Scene JSON schema - define layer + sprite instance format
2. Scene loader - parse JSON, build layer hierarchy
3. Layer renderer - ordered draw with parallax transform
4. Sample scene - JSON with 3 layers and 5+ sprites
5. Y-sort - optional per-layer sorting by Y position
6. Hot reload foundation - file watcher + reload trigger

Pre-requisite:

The scene JSON schema must be defined before coding starts in `docs/planning/asset_formats_v0.1.md`.

Risk:

- Soft dependency on M4 (atlas packer) - can use individual PNG files as placeholders
- Hot reload here is simple; full hot reload is M5

---

## 9. M2 Execution Plan

### Phase 0 - Spec Lock

- Finalize scene JSON schema in `docs/planning/asset_formats_v0.1.md`
- Confirm optional/required fields and default values
- Add one canonical sample JSON file that matches schema

### Phase 1 - Loader

- Implement parser and validation (JSON -> in-memory scene)
- Surface clear validation errors (file path + field name)
- Load 3-layer scene with placeholder PNG references

### Phase 2 - Rendering Behavior

- Render layers in deterministic order
- Apply per-layer parallax against camera movement
- Implement optional per-layer Y-sort for sprite instances
- Verify foreground occlusion layer draws after gameplay sprite layer

### Phase 3 - Hot Reload Foundation

- Add file watch trigger for scene file changes
- Reload at safe frame boundary only
- On parse failure, keep previous valid scene loaded and report error

### Definition of Done

- All M2 acceptance criteria pass in a sample scene
- Scene loader + renderer paths are covered by at least smoke tests
- Team can iterate on scene JSON without engine restart

### Failure Modes to Watch

- Layer ordering regressions after Y-sort enabled
- Camera/parallax mismatch from incorrect transform space
- Hot reload applying partial invalid state

### Validation Steps

1. Move camera through sample scene and verify depth illusion visually
2. Toggle Y-sort on/off per layer and confirm expected ordering changes
3. Edit scene JSON while running and confirm safe reload behavior
4. Force invalid JSON and confirm rollback to last valid scene

---

### M3: Collision Underlay + Character Controller

Goal: Prove deterministic gameplay simulation with collision truth separate from visuals.

Acceptance Criteria:

- Collision grid loads from file
- Character AABB collides with grid cells
- Character moves with keyboard input with deterministic response at fixed timestep
- Collision debug visualization draws solid cells and character bounds
- No obvious tunneling or jitter at 60 Hz fixed timestep
- Collision state remains independent of render layering/parallax

Key Deliverables:

1. Collision grid data format - define schema in `docs/planning/asset_formats_v0.1.md`
2. Collision loader - parse grid into runtime collision map
3. Collision query + resolve - AABB move-and-slide against grid cells
4. Character controller - input -> desired velocity -> collision-resolved movement
5. Debug draw - collision cells + player AABB overlay
6. Sample collision file - author one grid matching current sample scene scale

Pre-requisite:

Collision file format must be finalized in `docs/planning/asset_formats_v0.1.md` before implementation starts.

Risk:

- Collision tuning can consume time (corner jitter, edge sticking, penetration correction)
- Determinism regressions if movement uses variable delta time paths
- Optional vector collider overlay should be deferred if it threatens M3 timeline

---

## 10. M3 Execution Plan

### Phase 0 - Collision Spec Lock

- Finalize collision grid schema and coordinate conventions
- Decide cell size and origin convention relative to world units
- Add canonical sample file and loader validation rules

### Phase 1 - Collision Runtime Core

- Implement collision grid storage and solid-cell query
- Implement AABB sweep/resolve with axis-separable move-and-slide
- Add deterministic movement update at fixed timestep only

### Phase 2 - Character Controller

- Add input-driven character motor (walk only for M3)
- Resolve motion against collision grid each fixed step
- Clamp/normalize movement so diagonal speed is stable

### Phase 3 - Debug + Validation Harness

- Draw collision grid and character bounds in debug overlay
- Add repeatable scripted movement test path for deterministic regression checks
- Validate that visual scene layers do not alter collision behavior

### Definition of Done

- All M3 acceptance criteria pass on sample scene/collision file
- Character controller movement is deterministic under repeated input sequence
- Collision debug views are sufficient to diagnose stuck/penetration issues quickly

### Failure Modes to Watch

- Corner snagging and oscillation near tile boundaries
- Sub-step/timestep mismatch causing jitter
- Collision map/world transform mismatch (off-by-one cell errors)

### Validation Steps

1. Run a fixed scripted input sequence twice and compare final position/state
2. Walk into walls/corners at multiple angles and verify stable slide behavior
3. Toggle scene visual layers/parallax and confirm collision result is unchanged
4. Enable debug collision overlay and verify grid alignment against expected world space

---

### M4: Atlas Packer + Stable Asset IDs

Goal: Prove a stable asset pipeline that supports batching and safe iteration without brittle path references.

Acceptance Criteria:

- Atlas packer runs as a standalone CLI tool
- Packs multiple source sprites into one or more atlas textures
- Emits metadata with stable sprite IDs and UV mappings
- Runtime loads atlas metadata and resolves sprite references by stable ID
- Scene rendering batches sprites by atlas where possible
- Atlas metadata/texture reload path exists and does not corrupt running state
- Debug overlay reports atlas count/binds and draw call count

Key Deliverables:

1. Atlas metadata schema - finalize in `docs/planning/asset_formats_v0.1.md`
2. Atlas packer CLI - input folder/manifest -> atlas image + metadata JSON
3. Runtime atlas loader - parse metadata, upload textures, register stable IDs
4. Sprite ID resolver - scene/runtime references stable IDs instead of raw file paths
5. Batch renderer integration - group draws by atlas/texture bindings
6. Sample atlas output - include at least one generated atlas + metadata artifact

Pre-requisites:

- M1 renderer foundation is complete
- M2 scene loader/render path is stable enough to switch sprite reference mode
- M4 atlas metadata schema is locked before implementation starts

Risk:

- Rect packing and padding mistakes can cause texture bleeding artifacts
- Asset ID migration (path -> stable ID) can break existing sample scene data
- Hot reload and GPU resource replacement can introduce invalid handles if not frame-boundary safe

---

## 11. M4 Execution Plan

### Phase 0 - Format Lock + ID Policy

- Finalize atlas metadata schema and validation rules in `docs/planning/asset_formats_v0.1.md`
- Choose and document stable ID policy (UUID string format for v0.1 metadata)
- Define migration rule for scene sprite references (`asset` path -> `sprite_id`)

### Phase 1 - Atlas Build Tool

- Implement packer CLI entrypoint in devtools/tools crate
- Support deterministic packing output for unchanged inputs
- Write atlas PNG and metadata JSON atomically to avoid partial files

### Phase 2 - Runtime Integration

- Add atlas metadata loader and runtime registry (`sprite_id` -> atlas rect data)
- Update scene loader/runtime to resolve sprite instances through stable IDs
- Integrate batch submission path to minimize atlas binds and draw calls

### Phase 3 - Reload + Diagnostics

- Add safe-frame-boundary atlas reload trigger
- On reload parse/build failure, keep previous valid atlas assets live
- Expose bind/draw counters in debug overlay for quick batching verification

### Definition of Done

- All M4 acceptance criteria pass on sample content
- Existing sample scene renders correctly using stable sprite IDs
- Atlas build + runtime load path is covered by smoke tests

### Failure Modes to Watch

- UV mismatches from off-by-one atlas rect calculations
- Non-deterministic packing output causing noisy diffs and cache churn
- Missing ID references causing runtime sprite dropouts

### Validation Steps

1. Build atlas twice with unchanged inputs and compare output metadata deterministically
2. Render sample scene and verify visual parity before/after ID migration
3. Hot reload atlas metadata/texture while running and confirm no crash or corrupted sprites
4. Confirm debug overlay shows reduced atlas binds when many sprites share an atlas

---

## 12. Lua Integration Plan (Milestone-Aligned)

Objective:
Enable gameplay authoring in Lua while keeping engine determinism and performance guarantees.

### M3 (Foundation Hooks)

- Keep gameplay state deterministic in Rust simulation loop
- Define data model boundary that Lua will control (controller intent, ability state, simple events)
- Add deterministic test harness patterns that can validate scripted behavior later

### M4 (Runtime Boundary + Data Ownership)

- Integrate Lua runtime (`mlua`) into engine/game framework layer
- Implement minimal Rust->Lua API:
  - Input read
  - Query collision flags/grounded state
  - Write desired movement/action intents
- Enforce ownership rule:
  - Rust owns authoritative simulation state and collision resolution
  - Lua owns behavior decisions and gameplay rules

### M5 (Production Lua Path)

- Move character controller behavior logic from hardcoded Rust flow to Lua script
- Add safe hot reload for Lua scripts at frame boundaries
- Add script error handling/fallback policy (no partial corrupted state)
- Ship sample gameplay script as canonical authoring example

### Lua Definition of Done (v0.1)

- A sample playable character is behavior-authored in Lua
- Script reload does not crash or corrupt running simulation
- Same scripted input sequence yields same simulation result across repeated runs
- Rust-only fallback remains available if script load fails

