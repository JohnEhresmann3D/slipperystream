# Producer Review - Saturday Morning Engine v0.1
**Date:** 2026-02-15
**Status:** Planning Phase Assessment
**Reviewer:** Slipstream Producer

---

## Executive Summary

The planning foundation is **solid and well-thought-out**, with strong identity, clear constraints, and realistic scope. However, **milestones lack execution-level detail** needed to start building. The team needs:

1. **Acceptance criteria** for each milestone (measurable deliverables)
2. **Technical decision approval** (10 proposed decisions waiting for sign-off)
3. **Dependency mapping** between milestones
4. **Task breakdowns** for M1 to enable immediate work

**Recommendation:** Refine milestone definitions and approve implementation decisions before scaffolding begins.

---

## What's Solid and Ready to Go

### 1. Identity & Vision (engine_identity.md)
**Status: LOCKED and STRONG**

- Clear one-sentence identity: "sprite-scene-first with separate collision underlay"
- Distinct aesthetic pillars: Saturday morning cartoon vibe, Cuphead/Scott Pilgrim references
- Hard non-goals prevent scope creep (no Unity-but-smaller, no premature networking, no general 3D)
- Performance budgets and tier system well-defined
- Shipping doctrine emphasizes iteration speed and hot reload

**No changes needed.** This is production-ready guidance.

### 2. Scope Constraints (scope.md)
**Status: LOCKED and CLEAR**

- v0.1 goal is focused: playable vertical slice proving engine identity
- Platform scope is realistic (Windows primary, mobile-safe budgets)
- Out-of-scope list prevents feature creep
- Success criteria is visceral: "Saturday morning feel within 30 seconds"

**Strength:** The scope is ambitious but achievable.
**Risk:** Milestones are too high-level to execute against (see gaps below).

### 3. Architectural Foundation (architecture_sketch.md)
**Status: SOLID STRUCTURE**

- Clean module boundaries (Platform -> Core -> Framework -> Game)
- Deterministic main loop with fixed timestep
- Scene representation (layers, parallax, sorting) matches identity
- Collision underlay separation preserves gameplay truth
- Hot reload architecture is thought through

**Strength:** This is executable architecture. No hand-waving.
**Minor gap:** No mention of asset file formats (addressed partially in scope.md as "JSON or compact format").

### 4. Implementation Decisions (implementation_decisions.md)
**Status: PROPOSED (NOT APPROVED)**

- 10 technical decisions cover toolchain, rendering, platform layer, asset strategy, and CI
- Rationale is brief but sensible (wgpu, winit, Rust+Lua, Cargo, GUID assets, ECS-lite)
- Approval state is clearly marked as PROPOSED

**Strength:** Decisions are reasonable defaults.
**Critical Gap:** These decisions need explicit approval before scaffolding begins. Without approval, engineers will be blocked or will make assumptions.

---

## What Needs Refinement

### 1. Milestones Lack Execution-Level Detail

#### Current Milestone Definitions (scope.md, line 183-187)
```
M1 - Core loop + sprite rendering + debug overlay
M2 - Scene layers + parallax + occlusion
M3 - Collision underlay + character controller
M4 - Atlas packer + stable asset IDs
M5 - Hot reload + tier toggles + polish pass
```

**Problem:**
- No acceptance criteria (how do we know M1 is done?)
- No dependency graph (can M2 start before M4?)
- No task breakdowns (what are the actual engineering tasks?)
- No time estimates or risk assessment

**Impact:**
Without these, M1 cannot start cleanly. Engineers will either:
- Spin in uncertainty about what "done" means
- Make isolated assumptions that create integration debt later
- Build the wrong thing and discover it late

#### Recommended Fix
Define each milestone with:
1. **Acceptance Criteria** (measurable, testable outcomes)
2. **Deliverables** (code, tools, assets)
3. **Dependencies** (what must exist first)
4. **Key Tasks** (3-7 concrete engineering tasks)
5. **Risk Notes** (what could go wrong)

See "Refined Milestone Definitions" section below for proposed detail.

---

### 2. Technical Decisions Are Not Approved

**Current State:**
All 10 implementation decisions are marked `(proposed)`, not `(accepted)`.

**Problem:**
Scaffolding and M1 work depends on these decisions:
- Can't set up Cargo workspace without confirming Rust toolchain
- Can't integrate winit without approving windowing library
- Can't design asset pipeline without approving GUID strategy
- Can't start renderer without confirming wgpu

**Impact:**
M1 is blocked until these are approved.

**Recommended Action:**
Either:
- **Approve all 10 proposed decisions** (they are sensible defaults), OR
- **Explicitly defer decisions that can wait** (e.g., CI can be configured after M1), OR
- **Revise decisions that need more thought** (e.g., if winit feels risky, consider alternatives)

**Producer Call:**
I recommend **approving decisions 1-9 immediately** to unblock M1 scaffolding. Decision 10 (CI) can be deferred to M4 or M5 without blocking work.

---

### 3. Asset Pipeline is Underspecified

**Current State:**
- engine_identity.md mentions "stable asset IDs" and "JSON or compact format"
- scope.md defers scene authoring to "external JSON workflow acceptable"
- implementation_decisions.md proposes "GUID-based stable IDs"

**Gap:**
No concrete definition of:
1. What does a scene JSON file look like? (Schema needed for M2)
2. What does a collision file format look like? (Needed for M3)
3. What does atlas metadata output look like? (Needed for M4)
4. Who authors GUIDs? (Build tool? Hand-written? Auto-generated?)

**Impact:**
M2 and M4 will stall when engineers need to deserialize assets and have no format spec.

**Recommended Action:**
Add a new doc: `docs/planning/asset_formats_v0.1.md` that defines:
- Scene JSON schema (layers, sprite instances, transforms)
- Collision data format (grid + optional vectors)
- Atlas metadata schema (GUID -> UV mappings)
- GUID generation strategy (tooling-owned vs hand-authored)

This can happen **during M1** (doesn't block scaffolding), but **must exist before M2 starts**.

---

### 4. Dependency Ordering is Implicit, Not Explicit

**Current State:**
Milestones are listed in sequence, but dependencies are not mapped.

**Example Ambiguities:**
- Can M2 (scene layers) start without M4 (atlas packer)? Or do we need placeholder assets?
- Does M3 (collision) depend on M2 (scene system)? Or can they proceed in parallel?
- Can M5 (hot reload) start before M4 (stable IDs)?

**Impact:**
Parallel work is risky without a dependency map. Teams may build incompatible pieces.

**Recommended Action:**
Add a dependency diagram to scope.md or producer_review.md showing:
- Which milestones are sequential (must finish before next starts)
- Which milestones can overlap (parallel work safe)
- Which milestones have shared dependencies (e.g., both M2 and M3 need asset loading)

See "Milestone Dependency Map" section below.

---

### 5. Risk Assessment is Missing

**Current State:**
No explicit risk tracking in any doc.

**Potential Risks (Not Yet Documented):**
1. **Hot reload complexity:** Stateful reloads are hard. Risk of corrupted state or crashes.
2. **wgpu learning curve:** If team is unfamiliar, budget ramp-up time.
3. **Collision underlay authoring:** No tool exists yet. Manual JSON editing is painful.
4. **Determinism validation:** How do we test that simulation is deterministic?
5. **Tier system integration:** Tier flags touch multiple systems. Risk of last-minute integration pain.

**Impact:**
Unknown risks become surprises mid-milestone.

**Recommended Action:**
Add a "Known Risks & Mitigation" section to scope.md or create `docs/planning/risks_v0.1.md`.

---

## Refined Milestone Definitions

Below are **proposed expanded milestone definitions** with acceptance criteria, deliverables, dependencies, tasks, and risk notes.

---

### M1: Core Loop + Sprite Rendering + Debug Overlay

**Goal:**
Prove the engine can run a deterministic fixed-timestep loop and draw sprites.

**Acceptance Criteria:**
- [ ] Engine runs at 60 Hz fixed timestep with correct frame pacing
- [ ] Input sampling works (keyboard/mouse via winit)
- [ ] Simple sprite draws to screen (hardcoded quad, no batching yet)
- [ ] Debug overlay shows: FPS, frame time, fixed-step count
- [ ] Window resizes correctly
- [ ] Exit on ESC key or window close

**Deliverables:**
- Cargo workspace configured (builds on Windows)
- Platform layer: winit window + input + event loop
- Engine core: fixed timestep scheduler
- Minimal wgpu renderer (clear screen + draw single sprite)
- Debug overlay (text rendering via bitmap font or egui)
- Sample executable runs and shows stats

**Dependencies:**
- Requires approval of implementation decisions 1-4 (Rust+Lua, Cargo, wgpu, winit)

**Key Tasks:**
1. Scaffold Cargo workspace (`/crates/engine`, `/crates/platform`, `/crates/core`, `/crates/render`, `/crates/game`)
2. Integrate winit (window + event loop + input polling)
3. Initialize wgpu device, surface, render pipeline
4. Implement fixed timestep accumulator (`crates/core/src/time.rs`)
5. Render single hardcoded sprite (texture + quad)
6. Add debug overlay with FPS/frame time display
7. Write sample `main.rs` that runs the loop

**Risk Notes:**
- wgpu setup requires understanding of surface/device/queue model; budget ramp-up time
- Fixed timestep accumulator needs careful testing (spiral of death case)
- Debug text rendering may need fallback if egui integration is slow

**Estimated Effort:** 1-2 weeks (depending on team wgpu familiarity)

---

### M2: Scene Layers + Parallax + Occlusion

**Goal:**
Prove the sprite-scene-first authoring model with layered illustration.

**Acceptance Criteria:**
- [ ] Scene loads from JSON file
- [ ] Scene contains 3+ layers (background, mid, foreground)
- [ ] Each layer has independent parallax factor
- [ ] Foreground layer acts as occlusion mask (draws in front)
- [ ] Camera movement demonstrates parallax effect
- [ ] Layers support Y-sort mode (sprites sort by Y position)
- [ ] Scene can be reloaded at runtime (hot reload foundation)

**Deliverables:**
- Scene JSON schema definition (`docs/planning/asset_formats_v0.1.md`)
- Scene loader (parses JSON, builds layer hierarchy)
- Layer renderer (ordered draw, parallax transform applied)
- Sample scene JSON with 3 layers and 5+ sprites
- Camera control (WASD movement for testing parallax)

**Dependencies:**
- M1 complete (rendering and debug overlay working)
- Asset format spec defined (scene JSON schema)
- Texture assets (either placeholder or from M4's atlas packer)

**Key Tasks:**
1. Define scene JSON schema (layers, sprite instances, parallax factors)
2. Implement scene loader (JSON -> scene graph)
3. Implement layer rendering with parallax offset
4. Add Y-sort support (optional per layer)
5. Add camera transform system
6. Write sample scene JSON file
7. Test hot reload (file watcher + reload trigger)

**Risk Notes:**
- Dependency on M4 (atlas packer) is soft: can use individual texture files as placeholder
- JSON schema needs sign-off before coding starts
- Hot reload foundation here is simple; full hot reload is M5

**Estimated Effort:** 1-2 weeks

---

### M3: Collision Underlay + Character Controller

**Goal:**
Prove deterministic gameplay simulation with collision truth separate from visuals.

**Acceptance Criteria:**
- [ ] Collision grid loads from file
- [ ] Character AABB collides with grid cells
- [ ] Character moves with keyboard input (deterministic response)
- [ ] Collision debug visualization draws grid cells
- [ ] No tunneling or jitter at 60 Hz fixed timestep
- [ ] Optional: vector colliders (segments/polygons) overlay grid
- [ ] Collision state is independent of rendering state

**Deliverables:**
- Collision grid data format spec (`docs/planning/asset_formats_v0.1.md`)
- Collision grid loader
- AABB collision resolver (MoveAndCollide function)
- Debug draw for collision grid and character bounds
- Character controller component (input -> movement -> collision response)
- Sample collision grid file

**Dependencies:**
- M1 complete (fixed timestep working)
- M2 helpful but not required (collision can be tested with simple sprite)

**Key Tasks:**
1. Define collision grid file format (binary or JSON)
2. Implement grid collision storage and query
3. Implement AABB sweep/response (move and slide)
4. Add character controller (movement + collision)
5. Add collision debug draw (grid cells, character bounds)
6. Author sample collision grid matching M2 scene
7. Test determinism (same inputs = same outputs)

**Risk Notes:**
- Collision response tuning (slope handling, corner cases) can be time-consuming
- Determinism validation requires explicit test harness
- Optional vector colliders add complexity; defer if risky

**Estimated Effort:** 1-2 weeks

---

### M4: Atlas Packer + Stable Asset IDs

**Goal:**
Prove stable asset pipeline with GUID-based references.

**Acceptance Criteria:**
- [ ] Atlas packer tool runs standalone (CLI)
- [ ] Packs input sprites into atlas texture
- [ ] Outputs metadata with GUID -> UV mappings
- [ ] Engine loads atlas and resolves sprite by GUID
- [ ] Atlas can be updated and reloaded without breaking references
- [ ] Sprite batching works (multiple sprites from same atlas in one draw call)
- [ ] Debug overlay shows atlas bind count

**Deliverables:**
- Atlas packer tool (`/tools/atlas_packer`)
- Atlas metadata format spec (`docs/planning/asset_formats_v0.1.md`)
- Atlas loader in engine
- Sprite batch renderer (replaces M1's single-sprite renderer)
- Sample atlas texture and metadata
- GUID generation strategy documented

**Dependencies:**
- M1 complete (rendering foundation exists)
- M2 helpful (scene system can consume atlas references)

**Key Tasks:**
1. Design atlas metadata format (GUID, UV rects, atlas texture ID)
2. Write atlas packer tool (input: sprite directory, output: texture + metadata)
3. Implement GUID generation (hash-based or UUID)
4. Implement sprite batch renderer (sort by atlas, emit draw calls)
5. Integrate atlas loader into engine
6. Update M2 scene files to reference sprites by GUID
7. Measure batching efficiency (draw calls, atlas binds)

**Risk Notes:**
- Atlas packing algorithm (rect packing) can be complex; use existing library if possible
- GUID strategy needs clear documentation (who owns GUID assignment?)
- Batching efficiency depends on sprite sorting; needs profiling

**Estimated Effort:** 1-2 weeks

---

### M5: Hot Reload + Tier Toggles + Polish Pass

**Goal:**
Prove iteration velocity (hot reload) and fidelity scaling (tier system).

**Acceptance Criteria:**
- [ ] Atlas hot reload works (in-place update without scene reset)
- [ ] Scene hot reload works (may reset scene safely)
- [ ] Collision hot reload works (may reset scene safely)
- [ ] File watcher monitors asset directories
- [ ] Tier 0 and Tier 2 toggles functional (runtime switch via debug UI)
- [ ] Debug overlay shows current tier
- [ ] Sample scene demonstrates tier difference (e.g., particles on/off)
- [ ] No crashes or corrupted state during reload

**Deliverables:**
- Hot reload system (file watcher + reload orchestration)
- Tier toggle system (config flags, renderer branches)
- Sample tier-aware effects (particles, optional lighting)
- Polish pass on debug UI and controls
- Documentation: "How to use hot reload" guide

**Dependencies:**
- M1, M2, M3, M4 complete (all systems must be reload-safe)

**Key Tasks:**
1. Implement file watcher (monitor asset directories)
2. Implement atlas hot reload (reload texture + metadata, rebind sprites)
3. Implement scene hot reload (reload JSON, rebuild scene graph)
4. Implement collision hot reload (reload grid, reset physics state)
5. Add tier flag system (config + runtime toggle)
6. Add tier-aware rendering (particles, optional effects)
7. Test reload stability (no crashes, no leaks)

**Risk Notes:**
- Hot reload is stateful and error-prone; needs careful testing
- Tier system touches multiple systems (rendering, particles, etc.); integration risk
- File watchers can be flaky on Windows (polling fallback may be needed)

**Estimated Effort:** 1-2 weeks

---

## Milestone Dependency Map

```
M1: Core Loop + Sprite Rendering + Debug Overlay
  |
  ├─> M2: Scene Layers + Parallax + Occlusion (depends on M1 rendering)
  |     |
  |     └─> (soft dependency on M4 for atlas references, can use placeholders)
  |
  ├─> M3: Collision Underlay + Character Controller (depends on M1 fixed timestep)
  |     |
  |     └─> (soft dependency on M2 for scene context, but can test standalone)
  |
  └─> M4: Atlas Packer + Stable Asset IDs (depends on M1 rendering)
        |
        └─> (M2 and M3 benefit from M4 but not blocked)

M5: Hot Reload + Tier Toggles + Polish Pass
  (depends on ALL: M1, M2, M3, M4 complete)
```

**Recommended Execution Order:**
1. M1 (foundational, blocks everything)
2. M2 + M3 in parallel (independent work streams)
3. M4 (once M1 is solid, integrate into M2/M3)
4. M5 (integration and polish after all systems exist)

**Parallel Work Opportunities:**
- M2 and M3 can proceed simultaneously after M1
- M4 can start before M2/M3 finish (atlas packer is standalone tool)
- Asset format specs (scene JSON, collision format, atlas metadata) can be written during M1

---

## Critical Path to First Commit

**To unblock M1 scaffolding, the team needs:**

1. **Approve implementation decisions 1-9** (Rust+Lua, Cargo, wgpu, winit, GUID assets, ECS-lite, grid collision, hot reload policy, egui)
   - Decision 10 (CI) can be deferred to M4 or M5
   - **Action:** Mark decisions 1-9 as `(accepted)` in implementation_decisions.md

2. **Define asset file format specs** (can happen during M1, but needed before M2)
   - Create `docs/planning/asset_formats_v0.1.md`
   - Document scene JSON schema, collision format, atlas metadata schema
   - **Action:** Assign to technical lead or producer to draft during M1

3. **Scaffold Cargo workspace**
   - Crates: `platform`, `core`, `render`, `collision`, `framework`, `devtools`, `tools`, `game`
   - **Action:** Technical lead creates initial `Cargo.toml` workspace and crate structure

4. **Set up winit and wgpu integration**
   - Add winit + wgpu as dependencies
   - Write minimal window + event loop + wgpu device/surface init
   - **Action:** Assign to rendering engineer

**Estimated Time to First Runnable Build:** 3-5 days after decision approval

---

## Risks and Concerns

### High-Priority Risks
1. **Hot reload complexity (M5)**
   - **Risk:** Stateful reloads can cause crashes or corrupted state
   - **Mitigation:** Design reload boundaries carefully; test incrementally; allow scene reset as fallback
   - **Fallback:** If in-place atlas reload proves too hard, accept scene reset for all reloads in v0.1

2. **wgpu learning curve (M1)**
   - **Risk:** Team may be unfamiliar with wgpu; surface/device/pipeline setup has learning curve
   - **Mitigation:** Budget extra time for M1; reference wgpu examples and learn-wgpu tutorial
   - **Fallback:** If wgpu proves too complex, consider raw OpenGL via glow crate (but this limits cross-platform story)

3. **Collision authoring workflow (M3)**
   - **Risk:** No collision editor exists; manual JSON editing is painful
   - **Mitigation:** Use Tiled or LDtk as external tool; write simple converter
   - **Fallback:** Accept placeholder grid for v0.1; build editor in v0.2

### Medium-Priority Risks
4. **Asset format churn (M2/M3/M4)**
   - **Risk:** JSON schemas may need multiple iterations as systems integrate
   - **Mitigation:** Version asset files; write migration scripts early
   - **Fallback:** Accept manual asset regeneration for v0.1

5. **Tier system integration (M5)**
   - **Risk:** Tier flags touch multiple systems (rendering, particles, etc.); late integration may reveal conflicts
   - **Mitigation:** Design tier flag plumbing early (even if features are stubbed)
   - **Fallback:** Ship Tier 0 only for v0.1; add Tier 2 in v0.2

### Low-Priority Risks
6. **Determinism validation (M3)**
   - **Risk:** Hard to verify that simulation is truly deterministic without test harness
   - **Mitigation:** Write simple input replay test; log state hashes
   - **Fallback:** Accept manual testing for v0.1

---

## Recommended Next Steps (Priority Order)

### Immediate (Before M1 Starts)
1. **Approve implementation decisions 1-9** in implementation_decisions.md
   - Change status from `(proposed)` to `(accepted)`
   - Defer decision 10 (CI) to M4/M5
   - **Owner:** Project lead / producer

2. **Expand milestone definitions** in scope.md
   - Copy refined milestone definitions from this review
   - Add acceptance criteria, deliverables, dependencies, tasks
   - **Owner:** Producer (this review provides draft)

3. **Create asset format spec doc** (`docs/planning/asset_formats_v0.1.md`)
   - Define scene JSON schema
   - Define collision grid format
   - Define atlas metadata format
   - Document GUID generation strategy
   - **Owner:** Technical lead

### Week 1 (M1 Start)
4. **Scaffold Cargo workspace**
   - Create crate layout (`crates/platform`, `crates/core`, `crates/render`, `crates/game`, etc.)
   - Write root `Cargo.toml` workspace manifest and per-crate manifests
   - **Owner:** Technical lead

5. **Integrate winit and wgpu**
   - Add winit + wgpu as dependencies
   - Write minimal window + event loop + wgpu device/surface init
   - **Owner:** Rendering engineer

6. **Implement fixed timestep scheduler**
   - Write engine core timing system (accumulator, fixed step, render context)
   - **Owner:** Engine engineer

### Week 2-3 (M1 Completion)
7. **Complete M1 acceptance criteria**
   - Sprite rendering, debug overlay, input sampling
   - Test and validate FPS stability
   - **Owner:** Team

### Week 3-5 (M2 + M3 Parallel)
8. **Execute M2 and M3**
   - M2: Scene layers, parallax, occlusion
   - M3: Collision grid, character controller
   - **Owner:** Separate engineers or small teams

### Week 5-7 (M4 + M5)
9. **Execute M4 and M5**
   - M4: Atlas packer, stable IDs, batching
   - M5: Hot reload, tier toggles, polish
   - **Owner:** Team

### End of v0.1 (Week 8)
10. **Validate success criteria**
    - Does the engine produce "Saturday morning feel" within 30 seconds?
    - Are all acceptance criteria met?
    - **Owner:** Producer + team

---

## Conclusion

The planning foundation is **excellent**. The identity is clear, the scope is realistic, and the architecture is executable. The primary gap is **execution-level milestone detail**.

**Next action:** Approve implementation decisions and expand milestone definitions. After that, the team can start building immediately.

**Timeline estimate for v0.1:** 8-10 weeks (assuming single engineer or small team).

**Confidence:** High. The scope is well-bounded, the architecture is clean, and the risks are known.

---

## Appendix: Suggested Asset Format Spec Outline

**File:** `docs/planning/asset_formats_v0.1.md`

**Contents:**
1. Scene JSON Schema
   - Layer definition (name, parallax, sort mode, role)
   - Sprite instance (asset GUID, transform, tint, sort key)
   - Example JSON snippet

2. Collision Grid Format
   - Grid dimensions (width, height, cell size)
   - Cell flags (solid, platform, trigger, etc.)
   - Optional vector colliders (segments, polygons)
   - Binary vs JSON tradeoffs
   - Example format snippet

3. Atlas Metadata Format
   - Atlas texture ID
   - Sprite GUID -> UV rect mapping
   - Pivot point, padding, metadata
   - Example JSON snippet

4. GUID Generation Strategy
   - Who generates GUIDs? (build tool, artist, engine?)
   - Collision handling (what if two assets get same GUID?)
   - Migration path (path-based -> GUID migration)

**Action:** Draft this during M1, finalize before M2 starts.

---

**End of Producer Review**
