//! Atlas metadata loading and sprite ID resolution.
//!
//! The atlas packer (tools/atlas_packer) outputs a JSON metadata file that maps
//! stable `sprite_id` strings to texture regions (UV rects, pixel sizes, pivots).
//! Sprite IDs are content-addressed hashes, so renaming or re-packing the atlas
//! sheet does not break scene references.
//!
//! `AtlasRegistry::resolve(sprite_id)` is the primary lookup used at render time.
//! It returns an `AtlasSpriteEntry` containing the texture path, UV rect, and
//! pixel dimensions needed to build a sprite quad.

use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct AtlasFile {
    pub version: String,
    pub atlas_id: String,
    pub texture: AtlasTexture,
    pub sprites: Vec<AtlasSprite>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AtlasTexture {
    pub path: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AtlasSprite {
    pub sprite_id: String,
    #[allow(dead_code)]
    pub name: Option<String>,
    #[allow(dead_code)]
    pub source_path: String,
    pub rect_px: AtlasRectPx,
    pub uv: AtlasUvRect,
    #[serde(default)]
    pub pivot: AtlasPivot,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct AtlasRectPx {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct AtlasUvRect {
    pub u0: f32,
    pub v0: f32,
    pub u1: f32,
    pub v1: f32,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct AtlasPivot {
    pub x: f32,
    pub y: f32,
}

impl Default for AtlasPivot {
    fn default() -> Self {
        Self { x: 0.5, y: 0.5 }
    }
}

#[derive(Debug, Clone)]
pub struct AtlasSpriteEntry {
    pub texture_path: String,
    pub size_px: (u32, u32),
    pub uv: [f32; 4],
    pub pivot: (f32, f32),
}

#[derive(Debug, Clone)]
pub struct AtlasRegistry {
    pub atlas_id: String,
    pub sprite_entries: HashMap<String, AtlasSpriteEntry>,
}

impl AtlasRegistry {
    pub fn resolve(&self, sprite_id: &str) -> Option<&AtlasSpriteEntry> {
        self.sprite_entries.get(sprite_id)
    }
}

pub fn load_atlas_from_path(path: &Path) -> Result<AtlasRegistry, String> {
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read atlas metadata {}: {e}", path.display()))?;
    let atlas: AtlasFile = serde_json::from_str(&raw)
        .map_err(|e| format!("Failed to parse atlas metadata {}: {e}", path.display()))?;
    validate_atlas(&atlas)?;

    let mut sprite_entries = HashMap::new();
    for sprite in &atlas.sprites {
        sprite_entries.insert(
            sprite.sprite_id.clone(),
            AtlasSpriteEntry {
                texture_path: atlas.texture.path.clone(),
                size_px: (sprite.rect_px.w, sprite.rect_px.h),
                uv: [sprite.uv.u0, sprite.uv.v0, sprite.uv.u1, sprite.uv.v1],
                pivot: (sprite.pivot.x, sprite.pivot.y),
            },
        );
    }

    Ok(AtlasRegistry {
        atlas_id: atlas.atlas_id,
        sprite_entries,
    })
}

fn validate_atlas(atlas: &AtlasFile) -> Result<(), String> {
    if atlas.version != "0.1" {
        return Err(format!(
            "Atlas validation failed: unsupported version '{}'",
            atlas.version
        ));
    }
    if atlas.texture.width == 0 || atlas.texture.height == 0 {
        return Err("Atlas validation failed: texture width/height must be > 0".to_string());
    }

    let mut ids = std::collections::HashSet::new();
    for sprite in &atlas.sprites {
        if !ids.insert(sprite.sprite_id.clone()) {
            return Err(format!(
                "Atlas validation failed: duplicate sprite_id '{}'",
                sprite.sprite_id
            ));
        }
        if sprite.rect_px.w == 0 || sprite.rect_px.h == 0 {
            return Err(format!(
                "Atlas validation failed: sprite '{}' has zero-sized rect",
                sprite.sprite_id
            ));
        }
        let right = sprite
            .rect_px
            .x
            .checked_add(sprite.rect_px.w)
            .ok_or_else(|| {
                format!(
                    "Atlas validation failed: sprite '{}' rect overflows u32 range",
                    sprite.sprite_id
                )
            })?;
        let bottom = sprite
            .rect_px
            .y
            .checked_add(sprite.rect_px.h)
            .ok_or_else(|| {
                format!(
                    "Atlas validation failed: sprite '{}' rect overflows u32 range",
                    sprite.sprite_id
                )
            })?;
        if right > atlas.texture.width || bottom > atlas.texture.height {
            return Err(format!(
                "Atlas validation failed: sprite '{}' rect exceeds atlas bounds",
                sprite.sprite_id
            ));
        }
        if !(0.0..=1.0).contains(&sprite.uv.u0)
            || !(0.0..=1.0).contains(&sprite.uv.v0)
            || !(0.0..=1.0).contains(&sprite.uv.u1)
            || !(0.0..=1.0).contains(&sprite.uv.v1)
        {
            return Err(format!(
                "Atlas validation failed: sprite '{}' has UV outside [0, 1]",
                sprite.sprite_id
            ));
        }
        if sprite.uv.u0 >= sprite.uv.u1 || sprite.uv.v0 >= sprite.uv.v1 {
            return Err(format!(
                "Atlas validation failed: sprite '{}' has invalid UV range",
                sprite.sprite_id
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_file_path(name_hint: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "sme_atlas_test_{}_{}_{}.json",
            name_hint,
            std::process::id(),
            nanos
        ))
    }

    #[test]
    fn load_atlas_from_path_parses_valid_file() {
        let path = temp_file_path("valid");
        let json = r#"
        {
          "version": "0.1",
          "atlas_id": "test",
          "texture": { "path": "assets/generated/test.png", "width": 64, "height": 64 },
          "sprites": [
            {
              "sprite_id": "id-1",
              "source_path": "assets/textures/a.png",
              "rect_px": { "x": 0, "y": 0, "w": 32, "h": 32 },
              "uv": { "u0": 0.0, "v0": 0.0, "u1": 0.5, "v1": 0.5 }
            }
          ]
        }
        "#;
        fs::write(&path, json).expect("failed to write temp atlas file");

        let atlas = load_atlas_from_path(&path).expect("atlas should load");
        assert_eq!(atlas.atlas_id, "test");
        assert!(atlas.resolve("id-1").is_some());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn load_atlas_from_path_rejects_overflowing_rect() {
        let path = temp_file_path("overflow_rect");
        let json = r#"
        {
          "version": "0.1",
          "atlas_id": "test",
          "texture": { "path": "assets/generated/test.png", "width": 64, "height": 64 },
          "sprites": [
            {
              "sprite_id": "id-overflow",
              "source_path": "assets/textures/a.png",
              "rect_px": { "x": 4294967295, "y": 0, "w": 8, "h": 8 },
              "uv": { "u0": 0.0, "v0": 0.0, "u1": 0.5, "v1": 0.5 }
            }
          ]
        }
        "#;
        fs::write(&path, json).expect("failed to write temp atlas file");

        let err = load_atlas_from_path(&path).expect_err("overflow rect should fail");
        assert!(err.contains("rect overflows u32 range"));

        let _ = fs::remove_file(path);
    }
}
