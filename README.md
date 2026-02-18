# SlipperyStreamEngine

A 2D/2.5D engine prototype focused on a "Saturday morning" visual style:
layered sprite scenes, parallax, deterministic gameplay simulation, and fast iteration.

## License

This project is licensed under Apache License 2.0.

- See `LICENSE` for full terms.
- See `NOTICE` for attribution notice.

## Current Status

This repository is in active v0.1 milestone development and is ready for prototype gameplay work.

- M1: Core loop + sprite rendering + debug overlay
- M2: Scene layers + parallax + occlusion
- M3: Collision underlay + character controller
- M4: Atlas packer + stable asset IDs
- M5: Hot reload + tier toggles + polish

Planning docs live in `docs/planning/`.

## Requirements

- Windows (primary dev target)
- Rust toolchain (stable)
- Cargo

Detailed requirements and setup options:
- `docs/setup/requirements.md`

## Quick Start

1. Clone repo.
2. From repo root, build:

```powershell
cargo build
```

3. Run the sample game executable:

```powershell
cargo run -p sme_game
```

Automated dependency setup (Windows PowerShell):

```powershell
.\scripts\install_requirements.ps1 -CheckOnly
.\scripts\install_requirements.ps1
```

## Prototype Readiness

You can prototype platformer gameplay now with:
- Layered scene authoring from JSON (`assets/scenes/m4_scene.json`)
- Collision underlay from JSON (`assets/collision/m3_collision.json`)
- Deterministic movement + jump controller
- Collision debug visualization
- Determinism replay tests (`assets/tests/m3_replay_input.json`)
- Atlas metadata + `sprite_id` scene references (`assets/generated/m4_sample_atlas.json`)

Not ready yet (planned next milestones):
- Lua-authored gameplay path (M4 foundation, M5 production path)

## Controls (Current Demo)

- `A D` or left/right arrows: move character
- `Space`, `W`, or up arrow: jump
- `R`: reload scene and collision data from disk
- `R`: reload scene, collision, and atlas metadata from disk
- `F3`: toggle debug overlay
- `F4`: toggle collision grid debug draw
- `Esc`: quit

## Build A Prototype Level

1. Edit scene visuals:
   - `assets/scenes/m4_scene.json`
2. Edit collision layout:
   - `assets/collision/m3_collision.json`
3. Run:

```powershell
cargo run -p sme_game
```

4. Validate quickly:
   - Character movement and jumping feel correct
   - Collision matches intended geometry (`F4` debug on)
   - No obvious jitter or tunneling

## Atlas Packer (M4)

Build an atlas and metadata JSON from a folder of PNG sprites:

```powershell
cargo run -p sme_atlas_packer -- assets/textures assets/generated/m4_sample_atlas.png assets/generated/m4_sample_atlas.json 128
```

Then reference sprites in scene JSON using `sprite_id` values from the generated metadata file.

## Project Layout

- `crates/sme_platform`: windowing/platform integration
- `crates/sme_core`: timing/input/core runtime primitives
- `crates/sme_render`: rendering systems and GPU abstractions
- `crates/sme_devtools`: debug overlay and developer tooling
- `crates/sme_game`: runnable sample game/app entry point
- `assets/`: runtime assets used by demo
- `docs/planning/`: scope, milestone and architecture planning

## How To Work In This Repo

1. Read scope first:
   - `docs/planning/scope.md`
2. Check architecture and design intent:
   - `docs/planning/architecture_sketch.md`
   - `docs/planning/engine_identity.md`
3. Confirm implementation decisions:
   - `docs/planning/implementation_decisions.md`
4. For M2 scene work, use:
   - `docs/planning/asset_formats_v0.1.md`

## Starter Guide: Building a New Scene (M2 Path)

1. Define a scene JSON using the schema in `docs/planning/asset_formats_v0.1.md`.
2. Include at least 3 layers:
   - `background`
   - `mid`
   - `foreground` (`occlusion: true`)
3. Set per-layer `parallax` values.
4. Use `sort_mode: "y"` on layers that need y-based depth ordering.
5. Place placeholder PNG assets under `assets/` while atlas integration is in progress.
6. Run `cargo run -p sme_game` and validate:
   - camera movement shows parallax
   - foreground occludes correctly
   - overlay toggles with `F3`

## M3 Collision Files

- Scene file: `assets/scenes/m4_scene.json`
- Collision file: `assets/collision/m3_collision.json`
- Replay input sample: `assets/tests/m3_replay_input.json`

Determinism verification (M3):

```powershell
cargo test -p sme_game replay_run_is_deterministic
```

Full `sme_game` test run:

```powershell
cargo test -p sme_game
```

## Roadmap Notes

- Scene JSON supports both `asset` paths and `sprite_id` references.
- Atlas packing now ships via `sme_atlas_packer`.
- Full hot reload coverage across scene/collision/atlas is finalized in M5.

## Contributing (Starter)

1. Keep changes aligned with `docs/planning/scope.md`.
2. Add or update planning docs when behavior contracts change.
3. Prefer small, milestone-scoped pull requests.
