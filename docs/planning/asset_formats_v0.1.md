# Asset Formats v0.1

Status: Draft for M2 kickoff.  
Rule: If format conflicts with locked scope, `scope.md` wins.

---

## 1. Scene JSON Schema (M2 Required)

Purpose: Author layered sprite scenes with parallax, optional Y-sort, and occlusion behavior.

### 1.1 Top-Level Shape

```json
{
  "version": "0.1",
  "scene_id": "sample_forest",
  "camera": {
    "start_x": 0.0,
    "start_y": 0.0,
    "zoom": 1.0
  },
  "layers": []
}
```

### 1.2 Field Definitions

- `version` (string, required): Schema version. Must be `0.1` for this milestone.
- `scene_id` (string, required): Stable scene identifier.
- `camera` (object, optional):
  - `start_x` (number, optional, default `0.0`)
  - `start_y` (number, optional, default `0.0`)
  - `zoom` (number, optional, default `1.0`)
- `layers` (array, required): Ordered from back to front.

### 1.3 Layer Shape

```json
{
  "id": "foreground",
  "parallax": 1.2,
  "sort_mode": "y",
  "occlusion": true,
  "visible": true,
  "sprites": []
}
```

- `id` (string, required): Unique within scene.
- `parallax` (number, required): Camera multiplier. Typical range `0.0` to `2.0`.
- `sort_mode` (string, optional, default `none`): `none` or `y`.
- `occlusion` (bool, optional, default `false`): If true, layer is intended to draw in front for masking/occlusion.
- `visible` (bool, optional, default `true`): Debug/authoring visibility.
- `sprites` (array, required): Sprite instances in this layer.

### 1.4 Sprite Instance Shape

```json
{
  "id": "tree_01",
  "asset": "assets/sprites/tree.png",
  "x": 320.0,
  "y": 140.0,
  "z": 0.0,
  "rotation_deg": 0.0,
  "scale_x": 1.0,
  "scale_y": 1.0,
  "pivot_x": 0.5,
  "pivot_y": 1.0,
  "tint": [1.0, 1.0, 1.0, 1.0]
}
```

- `id` (string, required): Unique sprite instance ID within the scene.
- `asset` (string, required): Asset reference path (M2 placeholder path; migrated to stable GUID lookup in M4).
- `x`, `y` (number, required): World position.
- `z` (number, optional, default `0.0`): Optional tie-breaker for sort or manual ordering.
- `rotation_deg` (number, optional, default `0.0`)
- `scale_x`, `scale_y` (number, optional, default `1.0`)
- `pivot_x`, `pivot_y` (number, optional, default `0.5`)
- `tint` (array[4], optional, default `[1, 1, 1, 1]`): RGBA multiplier in `0.0..1.0`.

### 1.5 Validation Rules

- At least 3 layers are required for M2 sample scenes.
- Layer IDs must be unique.
- Sprite IDs must be unique per scene.
- `sort_mode` must be `none` or `y`.
- If `layers` is empty, load fails.
- Unknown fields are ignored in v0.1, but warn in debug logs.

### 1.6 Canonical M2 Example

```json
{
  "version": "0.1",
  "scene_id": "m2_parallax_demo",
  "camera": { "start_x": 0.0, "start_y": 0.0, "zoom": 1.0 },
  "layers": [
    {
      "id": "background",
      "parallax": 0.4,
      "sort_mode": "none",
      "occlusion": false,
      "sprites": [
        { "id": "bg_sky", "asset": "assets/sprites/sky.png", "x": 0.0, "y": 0.0 }
      ]
    },
    {
      "id": "mid",
      "parallax": 0.8,
      "sort_mode": "y",
      "occlusion": false,
      "sprites": [
        { "id": "tree_a", "asset": "assets/sprites/tree.png", "x": 320.0, "y": 160.0 },
        { "id": "tree_b", "asset": "assets/sprites/tree.png", "x": 460.0, "y": 220.0 }
      ]
    },
    {
      "id": "foreground",
      "parallax": 1.2,
      "sort_mode": "none",
      "occlusion": true,
      "sprites": [
        { "id": "fg_branch", "asset": "assets/sprites/branch.png", "x": 380.0, "y": 120.0 }
      ]
    }
  ]
}
```

---

## 2. Collision Data Format (M3 Required)

Purpose: Define gameplay-truth collision independent from scene rendering layers.

### 2.1 Top-Level Shape

```json
{
  "version": "0.1",
  "collision_id": "m3_demo_collision",
  "cell_size": 32,
  "origin": { "x": 0, "y": 0 },
  "width": 20,
  "height": 12,
  "solids": []
}
```

### 2.2 Field Definitions

- `version` (string, required): Schema version. Must be `0.1`.
- `collision_id` (string, required): Stable identifier for the collision data.
- `cell_size` (integer, required): Cell dimension in world units/pixels.
- `origin` (object, optional, default `{ "x": 0, "y": 0 }`):
  - `x` (integer, required)
  - `y` (integer, required)
- `width` (integer, required): Number of columns.
- `height` (integer, required): Number of rows.
- `solids` (array, required): Solid cell coordinates in grid space.

### 2.3 Solid Cell Shape

```json
{ "x": 4, "y": 7 }
```

- `x` (integer, required): Column index in `0..width-1`.
- `y` (integer, required): Row index in `0..height-1`.

### 2.4 Coordinate Convention

- Grid origin `(0, 0)` is the bottom-left logical cell.
- World position of a cell is:
  - `world_x = origin.x + x * cell_size`
  - `world_y = origin.y + y * cell_size`
- Cells outside bounds are treated as non-solid unless explicitly configured otherwise.

### 2.5 Validation Rules

- `width > 0`, `height > 0`, and `cell_size > 0`.
- Every solid entry must be inside declared bounds.
- Duplicate solid entries are invalid and should fail load.
- Unknown fields are ignored in v0.1, but warn in debug logs.

### 2.6 Canonical M3 Example

```json
{
  "version": "0.1",
  "collision_id": "m3_demo_collision",
  "cell_size": 32,
  "origin": { "x": -320, "y": -192 },
  "width": 20,
  "height": 12,
  "solids": [
    { "x": 0, "y": 0 }, { "x": 1, "y": 0 }, { "x": 2, "y": 0 }, { "x": 3, "y": 0 },
    { "x": 4, "y": 0 }, { "x": 5, "y": 0 }, { "x": 6, "y": 0 }, { "x": 7, "y": 0 },
    { "x": 8, "y": 0 }, { "x": 9, "y": 0 }, { "x": 10, "y": 0 }, { "x": 11, "y": 0 },
    { "x": 12, "y": 0 }, { "x": 13, "y": 0 }, { "x": 14, "y": 0 }, { "x": 15, "y": 0 },
    { "x": 16, "y": 0 }, { "x": 17, "y": 0 }, { "x": 18, "y": 0 }, { "x": 19, "y": 0 },
    { "x": 6, "y": 1 }, { "x": 6, "y": 2 }, { "x": 6, "y": 3 },
    { "x": 13, "y": 1 }, { "x": 13, "y": 2 }
  ]
}
```

---

## 3. Atlas Metadata Format (M4 Required)

Purpose: Define deterministic atlas output and stable sprite ID resolution for runtime rendering.

### 3.1 Top-Level Shape

```json
{
  "version": "0.1",
  "atlas_id": "atlas_demo_main",
  "texture": {
    "path": "assets/generated/atlas_demo_main.png",
    "width": 1024,
    "height": 1024
  },
  "sprites": []
}
```

### 3.2 Field Definitions

- `version` (string, required): Schema version. Must be `0.1`.
- `atlas_id` (string, required): Stable ID for this atlas artifact.
- `texture` (object, required):
  - `path` (string, required): Runtime-loadable texture path.
  - `width` (integer, required): Atlas texture width in pixels.
  - `height` (integer, required): Atlas texture height in pixels.
- `sprites` (array, required): Sprite entries packed into this atlas.

### 3.3 Sprite Entry Shape

```json
{
  "sprite_id": "7f3e9f32-08cc-4ac8-b38d-45deed7b6a10",
  "name": "smiley_large",
  "source_path": "assets/sprites/smiley_large.png",
  "rect_px": { "x": 0, "y": 0, "w": 128, "h": 128 },
  "uv": { "u0": 0.0, "v0": 0.0, "u1": 0.125, "v1": 0.125 },
  "pivot": { "x": 0.5, "y": 0.5 }
}
```

- `sprite_id` (string, required): Stable identifier (UUID string) used by scene/runtime references.
- `name` (string, optional): Human-readable label for debugging.
- `source_path` (string, required): Original source texture path used for packing.
- `rect_px` (object, required): Packed rectangle in atlas pixel space.
  - `x`, `y` (integer, required): Top-left pixel coordinate.
  - `w`, `h` (integer, required): Sprite dimensions in pixels.
- `uv` (object, required): Normalized UV rectangle in atlas texture space.
  - `u0`, `v0`, `u1`, `v1` (number, required) in `0.0..1.0`.
- `pivot` (object, optional, default `{ "x": 0.5, "y": 0.5 }`): Normalized anchor point.

### 3.4 Scene Reference Rule (M4 Migration)

- Scene sprite instances should transition from:
  - `asset: "assets/sprites/tree.png"` (M2 placeholder path)
- To:
  - `sprite_id: "<uuid>"` (M4 stable runtime reference)
- Runtime lookup contract:
  - `sprite_id` must resolve through loaded atlas metadata.
  - Missing `sprite_id` is a load error (fail scene load in strict mode).

### 3.5 Validation Rules

- `texture.width > 0` and `texture.height > 0`.
- Each `sprite_id` must be unique within metadata.
- `rect_px` must be fully inside atlas bounds.
- `rect_px.w > 0` and `rect_px.h > 0`.
- UV values must map to `rect_px` within float tolerance.
- `u0 < u1` and `v0 < v1`.
- Unknown fields are ignored in v0.1, but warn in debug logs.

### 3.6 Canonical M4 Example

```json
{
  "version": "0.1",
  "atlas_id": "m4_sample_atlas",
  "texture": {
    "path": "assets/generated/m4_sample_atlas.png",
    "width": 512,
    "height": 512
  },
  "sprites": [
    {
      "sprite_id": "7f3e9f32-08cc-4ac8-b38d-45deed7b6a10",
      "name": "smiley_large",
      "source_path": "assets/sprites/smiley_large.png",
      "rect_px": { "x": 0, "y": 0, "w": 128, "h": 128 },
      "uv": { "u0": 0.0, "v0": 0.0, "u1": 0.25, "v1": 0.25 },
      "pivot": { "x": 0.5, "y": 0.5 }
    },
    {
      "sprite_id": "08df2a4c-8dc0-4a5c-84ba-ef1e4ec4f82b",
      "name": "smiley_small",
      "source_path": "assets/sprites/smiley_small.png",
      "rect_px": { "x": 128, "y": 0, "w": 64, "h": 64 },
      "uv": { "u0": 0.25, "v0": 0.0, "u1": 0.375, "v1": 0.125 },
      "pivot": { "x": 0.5, "y": 0.5 }
    }
  ]
}
```
