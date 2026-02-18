//! Animation registry for managing loaded animation definition files.
//!
//! Wraps the core `AnimationFile`/`AnimationClip` types from `sme_core::animation`
//! and provides a registry that can hold multiple animation files, resolve clips
//! by name, and cross-validate sprite_ids against the atlas registry.

use std::collections::HashMap;
use std::path::Path;

use sme_core::animation::{load_animation_file, AnimationClip};

use crate::atlas::MultiAtlasRegistry;

/// Registry holding animation clips from multiple animation definition files.
///
/// Clips are organized by `animation_id` (from the JSON file) and clip name.
/// The `resolve_clip` method supports both targeted lookup (with a source id)
/// and global search (first match across all files).
pub struct AnimationRegistry {
    /// animation_id -> clip_name -> clip
    clips: HashMap<String, HashMap<String, AnimationClip>>,
}

impl AnimationRegistry {
    pub fn new() -> Self {
        Self {
            clips: HashMap::new(),
        }
    }

    /// Load an animation file and register its clips under its `animation_id`.
    pub fn load_file(&mut self, path: &Path) -> Result<(), String> {
        let file = load_animation_file(path)?;
        self.clips.insert(file.animation_id, file.animations);
        Ok(())
    }

    /// Remove all clips from a previously loaded animation file.
    pub fn remove_file(&mut self, animation_id: &str) {
        self.clips.remove(animation_id);
    }

    /// Clear all loaded animation data.
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.clips.clear();
    }

    /// Resolve a clip by name. If `source` is given, only search that animation file.
    /// If `source` is None, search all loaded files (first match wins).
    pub fn resolve_clip(&self, source: Option<&str>, name: &str) -> Option<&AnimationClip> {
        if let Some(source_id) = source {
            return self.clips.get(source_id).and_then(|clips| clips.get(name));
        }
        for file_clips in self.clips.values() {
            if let Some(clip) = file_clips.get(name) {
                return Some(clip);
            }
        }
        None
    }

    /// Validate that all frame sprite_ids in all clips exist in the multi-atlas registry.
    #[allow(dead_code)]
    pub fn validate_sprites(&self, multi_atlas: &MultiAtlasRegistry) -> Result<(), String> {
        for (anim_id, file_clips) in &self.clips {
            for (clip_name, clip) in file_clips {
                for frame in &clip.frames {
                    if multi_atlas.resolve(&frame.sprite_id).is_none() {
                        return Err(format!(
                            "Animation '{}' clip '{}' references missing sprite_id '{}'",
                            anim_id, clip_name, frame.sprite_id
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atlas::{AtlasRegistry, AtlasSpriteEntry};
    use std::collections::HashMap;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_file_path(name_hint: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "sme_animreg_test_{}_{}_{}.json",
            name_hint,
            std::process::id(),
            nanos
        ))
    }

    fn write_valid_animation_file(path: &Path) {
        let json = r#"
        {
          "version": "0.1",
          "animation_id": "hero",
          "animations": {
            "idle": {
              "frames": [
                { "sprite_id": "sprite-a", "duration_ms": 100 },
                { "sprite_id": "sprite-b", "duration_ms": 100 }
              ],
              "looping": true
            },
            "jump": {
              "frames": [
                { "sprite_id": "sprite-c", "duration_ms": 120 }
              ],
              "looping": false
            }
          }
        }
        "#;
        fs::write(path, json).expect("write temp anim file");
    }

    fn make_multi_atlas(sprite_ids: &[&str]) -> MultiAtlasRegistry {
        let mut entries = HashMap::new();
        for &id in sprite_ids {
            entries.insert(
                id.to_string(),
                AtlasSpriteEntry {
                    texture_path: "test.png".to_string(),
                    size_px: (32, 32),
                    uv: [0.0, 0.0, 1.0, 1.0],
                    pivot: (0.5, 0.5),
                },
            );
        }
        let reg = AtlasRegistry {
            atlas_id: "test".to_string(),
            sprite_entries: entries,
        };
        let mut multi = MultiAtlasRegistry::new();
        multi.add_atlas("test.json", reg).unwrap();
        multi
    }

    #[test]
    fn load_valid_animation_file() {
        let path = temp_file_path("valid_reg");
        write_valid_animation_file(&path);

        let mut registry = AnimationRegistry::new();
        registry.load_file(&path).expect("should load");

        assert!(registry.resolve_clip(Some("hero"), "idle").is_some());
        assert!(registry.resolve_clip(Some("hero"), "jump").is_some());
        assert!(registry.resolve_clip(Some("hero"), "nonexistent").is_none());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn resolve_clip_without_source() {
        let path = temp_file_path("no_source");
        write_valid_animation_file(&path);

        let mut registry = AnimationRegistry::new();
        registry.load_file(&path).expect("should load");

        // Without source, should still find by name
        assert!(registry.resolve_clip(None, "idle").is_some());
        assert!(registry.resolve_clip(None, "nonexistent").is_none());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn validate_sprites_passes_when_all_exist() {
        let path = temp_file_path("validate_pass");
        write_valid_animation_file(&path);

        let mut registry = AnimationRegistry::new();
        registry.load_file(&path).expect("should load");

        let multi = make_multi_atlas(&["sprite-a", "sprite-b", "sprite-c"]);
        registry.validate_sprites(&multi).expect("should pass");

        let _ = fs::remove_file(path);
    }

    #[test]
    fn validate_sprites_fails_when_missing() {
        let path = temp_file_path("validate_fail");
        write_valid_animation_file(&path);

        let mut registry = AnimationRegistry::new();
        registry.load_file(&path).expect("should load");

        // Only provide sprite-a, missing sprite-b and sprite-c
        let multi = make_multi_atlas(&["sprite-a"]);
        let err = registry
            .validate_sprites(&multi)
            .expect_err("should fail with missing sprites");
        assert!(err.contains("missing sprite_id"));

        let _ = fs::remove_file(path);
    }
}
