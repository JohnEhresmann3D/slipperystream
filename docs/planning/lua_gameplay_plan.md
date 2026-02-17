# Lua Gameplay Integration Plan (v0.1)

Status: Planned  
Owner: Gameplay/Engine boundary workstream  
Primary references: `docs/planning/scope.md`, `docs/planning/implementation_decisions.md`

---

## 1. Goal

Allow gameplay behavior authoring in Lua while preserving deterministic simulation, safety, and performance.

---

## 2. Architecture Contract

- Rust remains authoritative for:
  - fixed-step simulation loop
  - collision detection/resolution
  - platform/render/audio/runtime services
- Lua owns:
  - gameplay behavior decisions
  - ability logic/state machines
  - high-level actor intent generation

Boundary rule:

- Lua provides intents (desired motion/actions), not direct world mutation bypassing physics/collision.

---

## 3. Milestone Breakdown

### M4 Slice: Runtime Embed + Minimal API

Deliverables:

1. `mlua` runtime embedded in game framework
2. Script loader for startup scripts
3. Minimal API surface:
   - `input.is_down(action)`
   - `actor.get_state(entity_id)`
   - `actor.set_intent(entity_id, intent)`
   - `world.emit(event_name, payload)`
4. Script error channel to debug overlay/logging

Definition of done:

- Engine boots with Lua loaded
- Lua script can read input and set movement intent for one actor
- Script errors are surfaced without crashing process

### M5 Slice: Hot Reload + Production Gameplay Path

Deliverables:

1. Safe Lua script reload at frame boundaries
2. Fallback strategy:
   - keep last valid script behavior if reload fails
3. Sample gameplay behaviors in Lua:
   - movement state machine
   - jump gating/timing rules
4. Determinism replay check for scripted behavior

Definition of done:

- A playable sample character is authored in Lua
- Reloading script updates behavior live without desync/crash
- Same input playback yields equivalent simulation outputs across runs

---

## 4. API Guardrails

- No direct Lua access to renderer/platform internals.
- No wall-clock or random calls without deterministic wrappers.
- All script callbacks run from fixed-step update context only.
- Script-side allocations and per-frame calls are profile-visible.

---

## 5. Risks and Mitigation

1. Nondeterminism from scripting:
   - Mitigation: deterministic API wrappers and replay checks.
2. Reload safety complexity:
   - Mitigation: transactional reload (parse/validate/apply or rollback).
3. API creep:
   - Mitigation: small v0.1 API and explicit ownership boundaries.
4. Performance drift:
   - Mitigation: script budget metrics in debug overlay.

---

## 6. Initial Task List

1. Add `mlua` dependency and runtime bootstrap.
2. Define first API table (`engine.input`, `engine.actor`, `engine.world`).
3. Implement one scripted controller prototype behind feature flag.
4. Add script load/reload diagnostics to debug overlay.
5. Add deterministic playback test using scripted controller.
