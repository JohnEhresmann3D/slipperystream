# ARCHITECTURE_SKETCH.md

## Saturday Morning Engine (2D/2.5D) - v0.1 Architecture

Architectural stance:

- Opinionated
- Deterministic
- Minimal magic
- Clear module boundaries
- Sprite-scene-first

---

## 1. High-Level Module Structure

Game
->
Game Framework (Scene + Entities + Systems)
->
Engine Core (Time + Input + Assets + Config + Profiler)
->
Rendering Collision
-> ->
Platform Layer (Window + Input + Audio + Filesystem)

Rules:

- Game never talks directly to Platform
- Rendering never owns gameplay truth
- Devtools observe everything, mutate only through explicit APIs

---

## 2. Main Loop (Deterministic)

while running:
platform.poll_events()
input.sample()
time.accumulate(real_dt)

while time.should_step():
game.update_fixed(fixed_dt)

render_context = time.build_render_context()
game.build_render_queue(render_context)
renderer.present(render_context)

Simulation state mutates only inside update_fixed().

---

## 3. Platform Layer

Responsibilities:

- Window creation
- OS event handling
- Raw input device events
- Basic audio device setup
- File I/O primitives

No gameplay logic allowed.

---

## 4. Engine Core

Responsibilities:

- Fixed timestep scheduler
- Input sampling and buffering
- Asset handle management
- Stable asset ID resolution
- Memory tracking
- Profiler zones
- Hot reload watchers
- Config + tier flags

Exports:

- Engine lifecycle API
- Time API
- Asset API (handle-based)
- Debug overlay API

---

## 5. Game Framework

Responsibilities:

- Scene management
- Entity storage (ECS-lite)
- Systems execution (explicit order)
- Gameplay truth state

Core Components:

- Transform2D
- SpriteRenderer
- Collider
- Controller

No rendering logic beyond submitting draw commands.

---

## 6. Scene Representation

Scene:

- Ordered list of Layers

Layer:

- Name
- Parallax factor
- Sort mode (None | Y-sort)
- Role (Background | Mid | Foreground | Occluder)
- Sprite instances

Sprite Instance:

- Asset handle
- Transform
- Pivot
- Tint
- Optional explicit sort key

---

## 7. Collision Underlay

CollisionGrid:

- Width / height
- Cell flags

Optional:

- Vector colliders (segments or polygons)

Exports:

- MoveAndCollide()
- QueryAABB()
- DebugDrawCollision()

Collision defines gameplay truth.
Rendering does not override collision.

---

## 8. Rendering Pipeline

Game builds a RenderQueue:

- Iterate layers in order
- Apply parallax transform
- Apply sorting rules
- Submit sprites to batch system

Batch Key:

- Atlas ID
- Blend mode
- Shader variant
- Sampler state

Renderer outputs:

- Draw calls
- Atlas binds
- Sprite count

---

## 9. Fidelity Tiering

Renderer reads tier flags:

Tier 0:

- Disable stylized lighting
- Minimal effects

Tier 2:

- Enable optional polish passes

Tier flags never change simulation timing or collision.

---

## 10. Hot Reload Architecture

Watchers monitor:

- Atlas source directory
- Scene files
- Collision files

Reload process:

- Queue reload request
- Apply at safe frame boundary
- Reset scene if necessary
- Never allow partial asset state

---

## 11. Debug & Profiling

Overlay must show:

- FPS
- Frame time
- Simulation steps per frame
- Draw calls
- Sprite count
- Atlas binds
- Memory estimate

Profiler zones required around:

- Input sampling
- Fixed update
- Render queue build
- Present

---

## 12. Suggested Directory Layout

/engine
/platform
/core
/render
/collision
/framework
/devtools

/tools
/atlas_packer

/game
/vertical_slice

/assets
/raw
/built

/docs/planning
engine_identity.md
scope.md
architecture_sketch.md
implementation_decisions.md


