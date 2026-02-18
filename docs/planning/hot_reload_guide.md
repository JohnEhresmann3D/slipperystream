# Hot Reload Guide

## Overview

Hot reload allows you to modify assets and scripts while the engine is running and see changes immediately without restarting. This eliminates the edit-compile-launch cycle for scene layout, collision tuning, atlas changes, and gameplay scripting. Iteration time drops from minutes to seconds.

Reload is designed to be safe: new data is always validated before it replaces old data, and errors never crash the engine.

## Supported Asset Types

| Asset Type       | File Location                  | Trigger              | On Error                                      |
|------------------|--------------------------------|----------------------|-----------------------------------------------|
| Scenes (JSON)    | `assets/scenes/*.json`         | File watcher + R key | Keeps previous valid scene                    |
| Collision (JSON) | `assets/collision/*.json`      | File watcher + R key | Keeps previous valid collision                |
| Atlas Metadata   | `assets/generated/*.json`      | File watcher + R key | Validates sprite references; keeps previous atlas |
| Lua Scripts      | `assets/scripts/*.lua`         | File watcher + R key | Falls back to Rust controller, logs error     |

### Scenes (JSON)

Scene files define layer ordering, parallax factors, and sprite placement. When a scene file changes on disk, the engine deserializes the new JSON and validates layer structure before swapping it in.

### Collision (JSON)

Collision files define the collision underlay (grid cells, AABB regions, vector colliders). On reload, the engine validates grid dimensions and collider definitions before replacing the active collision data.

### Atlas Metadata (JSON)

Atlas metadata maps sprite IDs to regions within texture atlases. On reload, the engine validates that all sprite references resolve to valid atlas entries before performing the swap. Invalid or missing sprite references cause the reload to be rejected.

### Lua Scripts

Lua gameplay scripts are polled for modification time changes. On successful reload, the engine calls `on_init()` on the new script to reinitialize state. On error (parse failure, runtime error), the engine falls back to the built-in Rust controller and logs the error. Lua errors never crash the engine.

## How to Trigger Reload

### Automatic

Save the file in your editor. The engine polls file modification times each frame and detects changes automatically. No additional action is required.

### Manual

Press **R** to force reload ALL asset types (scene, collision, atlas metadata, and Lua scripts) regardless of whether file modification times have changed. This is useful when file watcher polling misses a change or when you want to guarantee a clean reload.

## Safety Rules

1. **Frame-boundary reload only.** Reload never happens mid-simulation-step. All swaps occur between frames, after the current fixed-timestep update completes and before the next one begins.

2. **Validate-before-swap.** New data is fully deserialized and validated before it replaces the current data. If validation fails, the swap does not happen.

3. **Previous data survives errors.** On any reload failure (malformed JSON, missing references, Lua parse errors), the previously loaded valid data remains active. The engine continues running with the last known good state.

4. **Lua errors are contained.** Lua script errors are caught and logged. They never propagate as panics or crashes. The engine falls back to the Rust controller when a Lua script fails to load.

## Debug Overlay

Press **F3** to toggle the debug overlay. The overlay displays:

- **Reload status** -- Indicates whether the most recent reload succeeded or failed, and which asset type was involved.
- **Lua status** -- Shows one of: `loaded`, `error`, or `fallback` (Rust controller active).
- **Console log** -- Displays timestamped reload success/failure messages with error details when applicable.

## Workflow Tips

- **Keep the engine running while editing scene and collision JSON.** Save in your editor and the engine picks up changes within one frame. No restart needed.
- **Edit Lua scripts for instant gameplay iteration.** Tweak movement speeds, ability parameters, or state machine logic and see results immediately.
- **Press F4 to toggle collision debug view.** Use this while editing collision grid JSON to see collider outlines rendered over the scene.
- **Press F5 to cycle fidelity tiers.** Step through Tier 0 (mobile-safe baseline), Tier 1, and Tier 2 (PC polish) to verify that your changes look correct at each fidelity level.
