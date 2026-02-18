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
use std::collections::{HashMap, HashSet};
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
    #[allow(dead_code)]
    pub atlas_id: String,
    pub sprite_entries: HashMap<String, AtlasSpriteEntry>,
}

impl AtlasRegistry {
    #[allow(dead_code)]
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

/// Registry that spans multiple atlases with a flat O(1) sprite lookup.
///
/// Each atlas is stored separately (keyed by its file path) so individual
/// atlases can be hot-reloaded without rebuilding the entire index.
/// The `sprite_index` provides a unified view across all loaded atlases.
#[derive(Debug, Clone)]
pub struct MultiAtlasRegistry {
    registries: HashMap<String, AtlasRegistry>,
    sprite_index: HashMap<String, AtlasSpriteEntry>,
}

impl MultiAtlasRegistry {
    pub fn new() -> Self {
        Self {
            registries: HashMap::new(),
            sprite_index: HashMap::new(),
        }
    }

    /// Add an atlas keyed by its file path. Rejects duplicate sprite_ids across atlases.
    pub fn add_atlas(&mut self, key: &str, registry: AtlasRegistry) -> Result<(), String> {
        for sprite_id in registry.sprite_entries.keys() {
            if self.sprite_index.contains_key(sprite_id) {
                return Err(format!(
                    "Duplicate sprite_id '{}' across atlases (adding '{}')",
                    sprite_id, key
                ));
            }
        }
        for (sprite_id, entry) in &registry.sprite_entries {
            self.sprite_index.insert(sprite_id.clone(), entry.clone());
        }
        self.registries.insert(key.to_string(), registry);
        Ok(())
    }

    /// Remove an atlas and all its sprite_ids from the flat index.
    pub fn remove_atlas(&mut self, key: &str) {
        if let Some(registry) = self.registries.remove(key) {
            for sprite_id in registry.sprite_entries.keys() {
                self.sprite_index.remove(sprite_id);
            }
        }
    }

    /// Resolve a sprite_id across all loaded atlases.
    pub fn resolve(&self, sprite_id: &str) -> Option<&AtlasSpriteEntry> {
        self.sprite_index.get(sprite_id)
    }

    /// Return the set of unique texture paths across all loaded atlases.
    pub fn texture_paths(&self) -> HashSet<String> {
        self.sprite_index
            .values()
            .map(|e| e.texture_path.clone())
            .collect()
    }

    pub fn atlas_count(&self) -> usize {
        self.registries.len()
    }

    /// Check if any atlases are loaded.
    pub fn is_empty(&self) -> bool {
        self.registries.is_empty()
    }
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

    fn make_test_registry(atlas_id: &str, sprites: &[(&str, &str)]) -> AtlasRegistry {
        let mut sprite_entries = HashMap::new();
        for &(id, tex) in sprites {
            sprite_entries.insert(
                id.to_string(),
                AtlasSpriteEntry {
                    texture_path: tex.to_string(),
                    size_px: (32, 32),
                    uv: [0.0, 0.0, 1.0, 1.0],
                    pivot: (0.5, 0.5),
                },
            );
        }
        AtlasRegistry {
            atlas_id: atlas_id.to_string(),
            sprite_entries,
        }
    }

    #[test]
    fn multi_atlas_single_atlas_resolve() {
        let mut multi = MultiAtlasRegistry::new();
        let reg = make_test_registry(
            "chars",
            &[("sprite-a", "chars.png"), ("sprite-b", "chars.png")],
        );
        multi
            .add_atlas("chars.json", reg)
            .expect("add should succeed");

        assert_eq!(multi.atlas_count(), 1);
        assert!(multi.resolve("sprite-a").is_some());
        assert!(multi.resolve("sprite-b").is_some());
        assert!(multi.resolve("nonexistent").is_none());
    }

    #[test]
    fn multi_atlas_cross_atlas_resolve() {
        let mut multi = MultiAtlasRegistry::new();
        let reg1 = make_test_registry("chars", &[("sprite-a", "chars.png")]);
        let reg2 = make_test_registry("env", &[("sprite-b", "env.png")]);
        multi.add_atlas("chars.json", reg1).expect("add chars");
        multi.add_atlas("env.json", reg2).expect("add env");

        assert_eq!(multi.atlas_count(), 2);
        assert!(multi.resolve("sprite-a").is_some());
        assert!(multi.resolve("sprite-b").is_some());

        let paths = multi.texture_paths();
        assert!(paths.contains("chars.png"));
        assert!(paths.contains("env.png"));
    }

    #[test]
    fn multi_atlas_rejects_duplicate_sprite_ids() {
        let mut multi = MultiAtlasRegistry::new();
        let reg1 = make_test_registry("chars", &[("sprite-a", "chars.png")]);
        let reg2 = make_test_registry("env", &[("sprite-a", "env.png")]);
        multi.add_atlas("chars.json", reg1).expect("add chars");

        let err = multi
            .add_atlas("env.json", reg2)
            .expect_err("duplicate should fail");
        assert!(err.contains("Duplicate sprite_id"));
    }

    #[test]
    fn multi_atlas_remove_and_readd() {
        let mut multi = MultiAtlasRegistry::new();
        let reg = make_test_registry("chars", &[("sprite-a", "chars.png")]);
        multi.add_atlas("chars.json", reg).expect("add");

        assert!(multi.resolve("sprite-a").is_some());
        multi.remove_atlas("chars.json");
        assert!(multi.resolve("sprite-a").is_none());
        assert_eq!(multi.atlas_count(), 0);

        // Re-add should work
        let reg2 = make_test_registry("chars_v2", &[("sprite-a", "chars_v2.png")]);
        multi.add_atlas("chars.json", reg2).expect("re-add");
        assert!(multi.resolve("sprite-a").is_some());
    }

    #[test]
    fn multi_atlas_texture_paths_union() {
        let mut multi = MultiAtlasRegistry::new();
        let reg1 = make_test_registry("a", &[("s1", "tex1.png"), ("s2", "tex1.png")]);
        let reg2 = make_test_registry("b", &[("s3", "tex2.png")]);
        multi.add_atlas("a.json", reg1).unwrap();
        multi.add_atlas("b.json", reg2).unwrap();

        let paths = multi.texture_paths();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains("tex1.png"));
        assert!(paths.contains("tex2.png"));
    }
}
