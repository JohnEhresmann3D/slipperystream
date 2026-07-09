# GRIM DELIVERY

*Paperboy, but you're the Grim Reaper and the paper route is your job description.*

48-hour-jam-scope prototype built on the Saturday Morning Engine crates
(`sme_platform`, `sme_core`, `sme_render`). Design source:
`docs/planning/grim_delivery_gdd.md`; decisions logged in `STATE.md`.

## Run

```bash
cargo run -p grim_delivery
```

## Controls

| Key | Action |
|---|---|
| ◄ ► / A D | Change lane |
| SPACE | Throw death notice (left side) / advance screens |
| R | Restart (final screen) |
| ESC | Quit |

## How to play

Houses scroll past on the left. **Porch light OFF = scheduled for collection —
hit it.** Porch light ON = innocent household — hitting it is a misfiling
(score penalty, angry resident, complaint in your permanent record). Meet the
soul quota before the route ends. Missing quota never fails the level; it just
looks bad at your quarterly mortality review. Dodge hearses and dogs — a
collision costs you control for a moment, not the run.

## Implementation notes

- Pure Rust, fixed-timestep (60 Hz), deterministic: all layout randomness is
  consumed at level start via a seeded LCG (`level.rs`); the sim never touches
  a wall clock or runtime RNG. Covered by unit tests (`cargo test -p grim_delivery`).
- All art is vertex-colored quads on a single 1×1 white texture — the whole
  scene renders in one draw call through the engine's `SpritePipeline`.
- HUD is egui, using the same prepare/upload/paint pattern as `sme_devtools`.
- Deliberately not using the engine's Lua bridge: its v0.1 contract is
  move/jump intents for one platformer actor and cannot express spawning,
  projectiles, or HUD. See STATE.md decision log (2026-07-08).

## Not built (per GDD scope guardrails / project state)

- Audio (no audio backend integrated in the workspace yet)
- Chasing angry NPC, charge throw, trick-shot scoring (nice-to-haves, cut first)
