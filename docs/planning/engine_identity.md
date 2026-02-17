# ENGINE_IDENTITY.md
## Saturday Morning Engine (2D / 2.5D) - v0.1

### One-sentence identity
This engine is **sprite-scene-first**: art is authored as layered illustrations; **collision + gameplay run on a separate simplified representation**.

---

## 1) Purpose
Build an opinionated, shippable custom engine that produces a distinct "Saturday morning" feel:
- Cuphead-adjacent staging (hand-authored scenes, layered illustration)
- Scott Pilgrim-adjacent readability (sprite clarity, belt-scroller lane support)
- Stylized lighting and effects that enhance identity (not realism)
- Deterministic simulation and mobile-safe performance budgets

This is not a general-purpose engine. It is a style engine.

---

## 2) Target Platforms
### Primary
- PC (Windows first)

### Secondary
- Mobile (iOS/Android) as a first-class performance target (feature tiers scale down cleanly)

### Future-facing
- Cloud gaming viability via:
  - clean headless mode support (optional, not required for v0.1)
  - dynamic resolution hooks
  - input latency tolerance features designed in (not built prematurely)

---

## 3) Non-goals (Hard Boundaries)
- No "Unity-but-smaller" general editor suite in v0.1
- No general 3D engine ambitions (no open-world, no PBR pipeline, no AAA lighting stack)
- No premature plugin marketplace / mod framework
- No networked multiplayer foundation in v0.1 (unless the first shipped game requires it)
- No premature multi-backend rendering abstraction (start with one backend; design seams, don't overbuild)

---

## 4) Core Aesthetic Rules
### Pixel + sprite readability is sacred
- Sprites must remain crisp and intentional.
- No default post-processing that smears edges or destroys line clarity.

### Stylization over realism
- Lighting is a *style tool* (rim, toon ramps, selective highlights), not physically accurate.

### Constraints create identity
- Budgets are enforced.
- Optional fidelity is added via tiers, not via changing fundamentals.

---

## 5) World Representation (Canonical Model)
### Visual World (Authoring Reality): Sprite Scene Layers
A **Scene** is composed of ordered **Layers**:
- Each Layer contains sprite groups and optional set pieces
- Each Layer may have:
  - parallax factor
  - depth behavior (sorting rules)
  - occlusion role (foreground masks / silhouettes)
  - optional light interaction flags

This supports:
- hand-authored "illustration" staging (Cuphead feel)
- foreground occluders and dramatic framing
- dense set dressing without tilemap aesthetic leakage

### Gameplay World (Truth Reality): Collision Underlay
Gameplay runs on a simplified representation, separate from visuals:
- **Collision Layer** is either:
  - Grid collision map (tile-grid, not necessarily visual tiles), OR
  - Vector colliders (segments/polygons), OR
  - Hybrid (grid for broad phase + vector for authored precision)

**Default for v0.1:** Hybrid collision underlay (grid + optional vector refinement)

---

## 6) 2.5D Support Policy (Opinionated)
2.5D exists to enhance the vibe, not to become a 3D engine.

Allowed:
- Depth-sorted sprites (Y-sort and/or explicit depth bands)
- Billboard sprites in a 3D-ish camera
- Simple meshes for props/set dressing (optional module)
- "Lane depth" for belt-scroller movement (Scott Pilgrim style)

Not allowed (v0.1):
- General 3D world building workflow
- Fully dynamic 3D lighting pipeline
- Complex skeletal mesh workflows as a primary path

---

## 7) Simulation & Update Model
### Determinism is a design pillar
- Fixed timestep simulation (e.g., 60 Hz)
- Render can interpolate, simulation does not drift
- Inputs are sampled and applied deterministically

### Engine stance
- "If it's not profiled, it's a guess."
- "If it can't be removed cleanly, the architecture is wrong."

---

## 8) Core Systems (v0.1 Scope)
### Must-have runtime
- Platform: window, input, audio (basic), filesystem
- Timing: fixed timestep + frame pacing
- Rendering:
  - sprite batching (atlas-based)
  - tile collision debug visualization (even if visuals are not tiles)
  - text rendering (debug first; production later)
  - particles (simple 2D system)
- Scene:
  - scene layers with parallax & depth sorting
- Collision:
  - broad-phase grid
  - optional vector colliders
- Debug:
  - overlay: FPS, frame time, draw calls, sprite count, atlas binds, memory estimates
  - debug draw: colliders, bounds, camera frustums

### Tooling must-haves (v0.1)
- Texture atlas packer (with stable IDs)
- Hot reload:
  - atlases
  - scene files
  - collision data
- Build-friendly asset packaging (chunking by scene/level)

---

## 9) Performance & Memory Budgets (Enforced)
Budgets are targets; the engine reports violations.

### Frame targets
- Mobile: 60 fps baseline target (tiered effects)
- PC: 60+ fps with optional fidelity (no changes to fundamentals)

### Render budgets (initial targets; tune later)
- Texture atlas binds per frame: low and predictable
- Sprite batching is default; unbatched sprites are a warning
- Overdraw awareness:
  - foreground masks and large translucent stacks should be measurable and reported

### Memory posture
- Explicit texture budgets per scene
- Streaming-ready asset packaging (even if not fully implemented v0.1)

---

## 10) Quality Tiers (PC vs Mobile)
The engine supports tier toggles without branching codepaths everywhere.

### Tier 0 (Mobile Safe)
- No dynamic lights (or extremely limited)
- Minimal post (optional subtle bloom OFF by default)
- Conservative particle counts
- Strict overdraw discipline

### Tier 1 (Standard)
- Limited stylized lights (key + rim)
- Cheap stylized shading options
- More particles, still budgeted

### Tier 2 (PC "Beautiful")
- Stylized lighting enhancements
- Better particles and screen-space polish (still constrained)
- Optional higher-res assets (without requiring them)

Rule:
- Tiers add optional fidelity. They do not change core architecture.

---

## 11) Data Formats (v0.1)
### Scene file (authoring)
- JSON or a compact custom format (engine-owned)
- Contains:
  - layers
  - sprite instances (asset refs, transforms)
  - parallax & sorting rules
  - optional markers (spawn points, triggers)

### Collision file (authoring)
- Grid collision map (compressed)
- Optional vector overlays (polygons/segments)

### Asset references
- Stable asset IDs (not file-path fragile)
- Atlas entries generate stable handles

---

## 12) Shipping Doctrine (Rules of Development)
- An engine that never ships a game is a tech demo.
- Iteration speed beats theoretical elegance.
- Tooling quality determines team velocity.
- Compile time matters.
- Hot reload earns real engineering effort.
- Experimental features must have kill switches.
- Every major system must be removable cleanly.

---

## 13) Decision Ledger (Frozen for v0.1)
### Locked
- World representation: Sprite Scene Layers + Collision Underlay
- Fixed timestep deterministic simulation
- Tier-based fidelity (mobile-safe baseline)
- 2.5D is constrained and vibe-driven (no general 3D scope)

### Deferred (explicitly not v0.1)
- Networking/multiplayer
- General-purpose editor
- Full multi-backend RHI abstraction
- Complex 3D asset workflows

---

## 14) Open Questions (Answer before v0.2)
1) Collision authoring tool preference:
   - external (Tiled/LDtk/custom) vs minimal internal layer editor?
2) Canonical "lane" model for beat-'em-up mode:
   - depth bands vs continuous pseudo-Z?
3) Rendering backend choice for v0.1:
   - pick one and ship (don't abstract early)
4) Asset pipeline:
   - raw files direct vs build step only?
5) Hot reload boundaries:
   - which assets can reload without restarting the scene?

---

## 15) Success Criteria (v0.1)
A playable vertical slice exists that demonstrates:
- a layered sprite scene with parallax and occlusion
- deterministic movement + collision
- atlas batching + debug overlay
- hot reload of scene + atlas
- tier toggles that make the same content run on mobile budgets
- the game "feels" like Saturday morning immediately


