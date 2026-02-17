# SlipperyStreamEngine

2D/2.5D game engine prototype focused on a "Saturday morning" visual style:
layered sprite scenes, parallax, occlusion, deterministic simulation, and hot-reload-first iteration.

## License

This project is licensed under Apache License 2.0.

- See `LICENSE` for full terms.
- See `NOTICE` for attribution notice.

## Current Status

This repository is in active v0.1 milestone development.

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

## Controls (Current Demo)

- `W A S D` or arrow keys: move camera
- `F3`: toggle debug overlay
- `Esc`: quit

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

## Roadmap Notes

- Scene JSON currently uses direct asset paths for M2 speed.
- Stable asset ID wiring is planned in M4.
- Full hot reload coverage across scene/collision/atlas is finalized in M5.

## Contributing (Starter)

1. Keep changes aligned with `docs/planning/scope.md`.
2. Add or update planning docs when behavior contracts change.
3. Prefer small, milestone-scoped pull requests.
