# Saturday Morning Engine (SlipperyStream)

A purpose-built 2D/2.5D game engine targeting a "Saturday morning cartoon" aesthetic — Cuphead-adjacent staging, Scott Pilgrim-adjacent readability. Rust engine core with Lua gameplay scripting for fast iteration.

This is a **style engine**, not a general-purpose engine. Art is authored as layered sprite illustrations; collision and gameplay run on a separate simplified representation (the "collision underlay").

## License

Apache License 2.0. See `LICENSE` for full terms and `NOTICE` for attribution.

---

## Current Status: M5 Complete (v0.1 Feature-Complete)

All five milestones are implemented and passing 99 tests across 6 crates. Post-v0.1 work has added multi-atlas support and sprite sheet animation.

| Milestone | Status | Summary |
|-----------|--------|---------|
| M1 | Done | Core loop, sprite rendering, debug overlay |
| M2 | Done | Scene layers, parallax, occlusion, Y-sort |
| M3 | Done | Collision underlay, character controller, determinism tests |
| M4 | Done | Atlas packer CLI, stable sprite IDs, batched rendering |
| M5 | Done | Lua scripting, fidelity tiers, hot reload, pause/step |

---

## Feature Breakdown

### Rendering

- **Sprite batch renderer** with draw call merging — consecutive sprites sharing the same atlas texture collapse into a single `draw_indexed` call, minimizing GPU bind-group switches.
- **Multi-atlas support** — scenes declare which atlases they need via the `atlases` field. Multiple atlases are loaded into a flat O(1) sprite index. Individual atlases can be hot-reloaded without rebuilding the entire registry. Legacy single-atlas scenes work unchanged via automatic fallback.
- **Texture atlas system** with content-addressed stable IDs (UUID v5). Sprites are referenced by deterministic hash-based IDs, not brittle file paths. Atlas metadata survives repacking without breaking scene references.
- **Sprite sheet animation** — frame-based animation clips defined in JSON, with per-frame durations and looping control. Animation timing uses integer microseconds for deterministic advancement under fixed timestep. Animations are ticked in the simulation loop and freeze/advance correctly with pause/single-step.
- **Ordered scene layers** with per-layer parallax factors. Foreground layers support occlusion masking. Layers optionally Y-sort their sprites for depth ordering.
- **Orthographic camera** with position/zoom controls and per-layer parallax offset computation.
- **Fidelity tier system** — Tier 0 (mobile-safe baseline) and Tier 2 (PC polish) are runtime-switchable. Tier 2 adds a warm sprite color tint and enhanced clear color. Tiers never affect simulation or determinism.

### Simulation

- **Fixed 60 Hz timestep** with accumulator pattern. Spiral-of-death cap at 250ms prevents feedback loops. Interpolation alpha available for visual smoothing.
- **Deterministic simulation** — same inputs always produce same outputs. Validated by input replay regression tests.
- **Grid-based collision underlay** — O(1) cell lookup, axis-separable move-and-slide resolution (X then Y to prevent diagonal tunneling). Collision truth is independent of visual scene layers.
- **Character controller** — intent-driven design (acceleration, friction, gravity, jump). Grounded state is collision-contact-driven, not position-heuristic. Configurable physics parameters (max speed, accel, friction, gravity, jump speed).
- **Pause and single-step** — simulation can be paused and advanced one fixed step at a time via debug overlay.

### Lua Scripting

- **mlua integration** (Lua 5.4 vendored) with intent-based Rust-to-Lua API boundary. Lua provides desired motion/actions, Rust resolves physics and collision.
- **Engine API surface** exposed to Lua:
  - `engine.input.is_held(key)` / `engine.input.is_just_pressed(key)` — input queries
  - `engine.actor.grounded` / `engine.actor.velocity_x` / `engine.actor.velocity_y` — read-only actor state
  - `engine.actor.current_animation` / `engine.actor.animation_finished` — read-only animation state
  - `engine.actor.set_intent(move_x, jump_pressed)` — write movement intent
  - `engine.actor.play_animation(name)` / `engine.actor.stop_animation()` — control sprite animation from scripts
- **Script lifecycle**: `on_init()` called on load/reload, `on_update(dt)` called each fixed step.
- **Rust fallback controller** — if Lua script is missing or errors, the engine seamlessly falls back to an identical Rust-native controller. No gameplay interruption.
- **Script hot reload** via file modification time polling. Errors are logged without crashing; previous valid script stays active.

### Hot Reload

All asset types support hot reload with validate-before-swap safety:

| Asset | Trigger | On Error |
|-------|---------|----------|
| Scene JSON | File watcher + R key | Keeps previous valid scene |
| Collision JSON | File watcher + R key | Keeps previous valid collision |
| Atlas metadata | File watcher + R key | Per-atlas reload, validates sprite refs before swap |
| Animation JSON | File watcher + R key | Reloads clips, resets affected animation states |
| Lua scripts | File watcher + R key | Falls back to Rust controller |

Reload only happens at frame boundaries — never mid-simulation-step. See `docs/planning/hot_reload_guide.md` for details.

### Debug Overlay (F3)

- FPS, frame time, fixed-step count
- Draw calls, atlas binds, sprite count
- Loaded atlas count and active animation count
- Estimated GPU memory usage
- Current fidelity tier with cycle button
- Lua runtime status (loaded / error / fallback)
- Simulation pause/resume and single-step controls
- Collision grid debug visualization (F4)

### Asset Pipeline

- **Atlas packer CLI** (`sme_atlas_packer`) — packs a folder of PNGs into an atlas texture + metadata JSON with stable sprite IDs.
- **Transactional writes** — atlas outputs are written to temp files first, then atomically promoted to prevent partial/corrupt assets.
- **ID registry** — persistent mapping of sprite paths to stable UUIDs, stored alongside atlas output. IDs survive repacking.

---

## Quick Start

### Requirements

- Windows (primary dev target)
- Rust toolchain (stable)
- Cargo

### Build and Run

```powershell
git clone https://github.com/JohnEhresmann3D/slipperystream.git
cd slipperystream
cargo run
```

The engine launches with the sample scene, collision grid, and Lua controller.

### Run Tests

```powershell
cargo test --workspace    # All 99 tests
cargo clippy --workspace  # Lint check
cargo fmt --check         # Format check
```

### Controls

| Key | Action |
|-----|--------|
| A/D or Left/Right | Move character |
| Space, W, or Up | Jump |
| R | Force reload all assets (scene, collision, atlas, Lua) |
| F3 | Toggle debug overlay |
| F4 | Toggle collision grid debug draw |
| F5 | Cycle fidelity tier (Tier 0 / Tier 2) |
| Esc | Quit |

---

## How to Build a Game with This Engine

### Step 1: Understand the Architecture

The engine separates **visuals** from **gameplay truth**:

```
Visual Layer (what players see)        Gameplay Layer (what the engine simulates)
+--------------------------+           +---------------------------+
| Scene JSON               |           | Collision JSON            |
| - Ordered sprite layers  |           | - Grid of solid cells     |
| - Parallax factors       |           | - AABB move-and-slide     |
| - Y-sort, occlusion      |           | - Character controller    |
+--------------------------+           +---------------------------+
         |                                        |
         v                                        v
   Sprite Renderer                     Fixed 60Hz Simulation
   (batched draw calls)                (deterministic physics)
```

You author art as layered illustrations (scene JSON) and separately define where solid ground/walls are (collision JSON). The two are independent — you can change visuals without affecting gameplay, and vice versa.

### Step 2: Create Your Scene

Create a scene JSON file (see `assets/scenes/m4_scene.json` as a reference):

```json
{
  "version": "0.2",
  "scene_id": "my_level",
  "atlases": [
    "assets/generated/characters_atlas.json",
    "assets/generated/environment_atlas.json"
  ],
  "animations": [
    "assets/animations/hero_animations.json"
  ],
  "camera": { "start_x": 0, "start_y": 100, "zoom": 1.0 },
  "layers": [
    {
      "id": "background",
      "parallax": 0.5,
      "sort_mode": "none",
      "visible": true,
      "occlusion": false,
      "sprites": [
        {
          "id": "bg_sky",
          "sprite_id": "sky_sprite_uuid_here",
          "x": 0, "y": 200,
          "scale_x": 1.0, "scale_y": 1.0,
          "rotation_deg": 0, "z": 0
        }
      ]
    },
    {
      "id": "gameplay",
      "parallax": 1.0,
      "sort_mode": "y",
      "visible": true,
      "occlusion": false,
      "sprites": []
    },
    {
      "id": "foreground",
      "parallax": 1.2,
      "sort_mode": "none",
      "visible": true,
      "occlusion": true,
      "sprites": []
    }
  ]
}
```

Key concepts:
- **parallax < 1.0** = background (moves slower than camera)
- **parallax = 1.0** = gameplay layer (moves with camera)
- **parallax > 1.0** = foreground (moves faster than camera)
- **occlusion: true** = layer draws in front of everything (foreground mask)
- **sort_mode: "y"** = sprites auto-sort by Y position (for depth in side-view or top-down)

Sprites can reference assets by `sprite_id` (atlas-stable UUID) or `asset` (raw file path). Sprites with `animation` and `animation_source` fields will play frame-based animations from the declared animation files.

The `atlases` field declares which atlas metadata files the scene uses (v0.2). If omitted (v0.1), the engine falls back to the legacy single atlas path.

### Step 3: Define Collision

Create a collision JSON file (see `assets/collision/m3_collision.json`):

```json
{
  "version": "0.1",
  "collision_id": "my_level_collision",
  "cell_size": 32,
  "origin": { "x": -320, "y": -192 },
  "width": 20,
  "height": 12,
  "solids": [
    { "x": 0, "y": 0 },
    { "x": 1, "y": 0 },
    { "x": 2, "y": 0 }
  ]
}
```

Each entry in `solids` marks a grid cell as impassable. The character controller's AABB will slide against these cells. Use F4 in-game to visualize the collision grid overlaid on your scene.

### Step 4: Write Gameplay in Lua

Create or edit `assets/scripts/controller.lua`:

```lua
function on_init()
    -- Called once on script load/reload.
    -- Use for one-time setup (e.g., initializing state variables).
end

function on_update(dt)
    -- Called every fixed simulation step (60 Hz).
    -- Read input, decide what the character should do.

    local move_x = 0
    if engine.input.is_held("left") or engine.input.is_held("a") then
        move_x = move_x - 1
    end
    if engine.input.is_held("right") or engine.input.is_held("d") then
        move_x = move_x + 1
    end

    local jump = engine.input.is_just_pressed("space")
        or engine.input.is_just_pressed("w")
        or engine.input.is_just_pressed("up")

    -- Tell the engine what you WANT to do.
    -- The engine handles physics, collision, and actual movement.
    engine.actor.set_intent(move_x, jump)
end
```

The key principle: **Lua provides intents, Rust resolves physics.** Your script says "I want to move right and jump." The engine's character controller handles acceleration, gravity, friction, and collision response. You never directly set position or velocity from Lua.

You can also control sprite animations from Lua:

```lua
function on_update(dt)
    local move_x = 0
    -- ... input handling ...
    engine.actor.set_intent(move_x, jump)

    -- Switch animation based on state
    if move_x ~= 0 then
        engine.actor.play_animation("run")
    else
        engine.actor.play_animation("idle")
    end

    -- Check if a one-shot animation finished
    if engine.actor.animation_finished then
        engine.actor.play_animation("idle")
    end
end
```

Available input keys: `"left"`, `"right"`, `"up"`, `"down"`, `"space"`, `"w"`, `"a"`, `"s"`, `"d"`

Available actor state (read-only from Lua):
- `engine.actor.grounded` — is the character standing on solid ground?
- `engine.actor.velocity_x` — current horizontal velocity
- `engine.actor.velocity_y` — current vertical velocity
- `engine.actor.current_animation` — name of active animation clip, or nil
- `engine.actor.animation_finished` — true if a non-looping animation has completed

### Step 5: Pack Your Atlas

Once you have sprite PNGs, pack them into an atlas:

```powershell
cargo run -p sme_atlas_packer -- assets/textures output_atlas.png output_atlas.json 256
```

Arguments: `<input_folder> <output_texture> <output_metadata> <atlas_size>`

The packer generates:
- Atlas PNG texture (packed sprites)
- Metadata JSON with stable sprite IDs and UV rectangles
- ID registry JSON (maps file paths to persistent UUIDs)

Then reference sprites in your scene JSON using `sprite_id` values from the metadata.

### Step 6: Iterate

Run the engine and edit files while it's running:

1. **Edit scene JSON** — save the file, engine auto-reloads at next frame boundary
2. **Edit collision JSON** — same auto-reload behavior
3. **Edit Lua scripts** — same auto-reload, with error logging if your script has bugs
4. **Press R** to force-reload everything immediately
5. **Press F3** to check performance stats and Lua status
6. **Press F4** to visualize collision grid alignment

The engine never crashes from asset errors. Bad files are logged and the previous valid state is kept.

---

## Project Layout

```
crates/
  sme_platform/    Thin winit wrapper (window creation, event loop)
  sme_core/        Engine primitives (time, input, fidelity tiers, animation types)
  sme_render/      Sprite pipeline, camera, texture loading (wgpu)
  sme_devtools/    Debug overlay (egui), developer controls
  sme_game/        Game binary — main loop, scene/collision/atlas/Lua integration
  sme_atlas_packer/ Standalone CLI tool for atlas generation

assets/
  scenes/          Scene JSON files
  collision/       Collision grid JSON files
  animations/      Animation definition JSON files
  scripts/         Lua gameplay scripts
  textures/        Source sprite PNGs
  generated/       Atlas packer output (PNG + metadata JSON)
  tests/           Replay input files for determinism tests

docs/planning/     Architecture, scope, decisions, asset format specs
```

### Crate Dependency Graph

```
sme_game (binary)
  -> sme_devtools -> sme_core
  -> sme_render   -> sme_platform
  -> sme_core (leaf crate, no platform dependencies)

sme_atlas_packer (standalone binary, no engine dependencies)
```

---

## Roadmap

### Completed (v0.1)

- [x] Fixed 60Hz timestep with deterministic simulation
- [x] Sprite batch rendering with draw call merging
- [x] Ordered scene layers with parallax and Y-sort
- [x] Grid-based collision underlay with AABB move-and-slide
- [x] Character controller (walk, jump, gravity, friction)
- [x] Atlas packer with stable content-addressed sprite IDs
- [x] Debug overlay (FPS, draw calls, memory, controls)
- [x] Hot reload for scenes, collision, atlases, and Lua scripts
- [x] Lua gameplay scripting with intent-based API
- [x] Fidelity tier system (Tier 0 / Tier 2)
- [x] Simulation pause and single-step
- [x] Input replay determinism tests
- [x] Multi-atlas support with per-scene atlas declarations
- [x] Sprite sheet animation with deterministic frame timing
- [x] Lua animation control API (play/stop/query)
- [x] 99 unit tests across all crates

### Next Up (Post-v0.1)

- [ ] **Audio system** — kira integration for music/SFX with bus routing
- [ ] **Lightweight collision editor** — in-engine egui tool for painting collision grids
- [ ] **Entity system** — multiple actors with Lua-authored behaviors
- [ ] **Expanded Lua API** — spawn/despawn entities, event callbacks, world queries
- [ ] **Scene transitions** — fade/wipe between scenes with state preservation
- [ ] **Gamepad support** — controller input mapping alongside keyboard

### Future Vision

- [ ] 2.5D layer depth effects (parallax z-offset, depth-of-field hints)
- [ ] Tier 2 post-processing (bloom, vignette, color grading)
- [ ] Particle system with tier-scaled density
- [ ] Save/load state serialization
- [ ] Mobile builds (iOS/Android) targeting Tier 0 budgets
- [ ] Cloud gaming hooks (headless mode, dynamic resolution)

---

## Technical Stack

| Component | Choice | Version |
|-----------|--------|---------|
| Language | Rust | Stable toolchain |
| Rendering | wgpu | 24 |
| Windowing | winit | 0.30 |
| Debug UI | egui | 0.31 |
| Scripting | mlua (Lua 5.4) | 0.10 |
| Math | glam | 0.29 |
| Images | image | 0.25 |
| Serialization | serde + serde_json | 1 |
| Asset IDs | uuid (v4/v5) + sha2 | 1 / 0.10 |

---

## Contributing

1. Read `docs/planning/scope.md` for what's in and out of scope.
2. Check `docs/planning/engine_identity.md` for design philosophy.
3. Review `docs/planning/implementation_decisions.md` for accepted technical choices.
4. Keep changes aligned with milestone scope — prefer small, focused PRs.
5. All code must pass `cargo test`, `cargo clippy`, and `cargo fmt --check`.
