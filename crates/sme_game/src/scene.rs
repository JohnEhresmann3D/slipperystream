use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Deserialize, Clone)]
pub struct SceneFile {
    pub version: String,
    pub scene_id: String,
    pub camera: Option<SceneCamera>,
    pub layers: Vec<SceneLayer>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SceneCamera {
    #[serde(default)]
    pub start_x: f32,
    #[serde(default)]
    pub start_y: f32,
    #[serde(default = "default_zoom")]
    pub zoom: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SceneLayer {
    pub id: String,
    pub parallax: f32,
    #[serde(default)]
    pub sort_mode: SortMode,
    #[serde(default)]
    pub occlusion: bool,
    #[serde(default = "default_visible")]
    pub visible: bool,
    pub sprites: Vec<SceneSprite>,
}

#[derive(Debug, Deserialize, Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum SortMode {
    #[default]
    None,
    Y,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SceneSprite {
    pub id: String,
    #[serde(default)]
    pub asset: Option<String>,
    #[serde(default)]
    pub sprite_id: Option<String>,
    pub x: f32,
    pub y: f32,
    #[serde(default)]
    pub z: f32,
    #[serde(default)]
    pub rotation_deg: f32,
    #[serde(default = "default_scale")]
    pub scale_x: f32,
    #[serde(default = "default_scale")]
    pub scale_y: f32,
}

pub struct SceneWatcher {
    scene_path: PathBuf,
    last_seen_modified: Option<SystemTime>,
}

impl SceneWatcher {
    pub fn new(scene_path: PathBuf) -> Self {
        let last_seen_modified = modified_time(&scene_path);
        Self {
            scene_path,
            last_seen_modified,
        }
    }

    pub fn should_reload(&mut self) -> bool {
        let current = modified_time(&self.scene_path);
        match (self.last_seen_modified, current) {
            (Some(old), Some(now)) if now > old => {
                self.last_seen_modified = Some(now);
                true
            }
            (None, Some(now)) => {
                self.last_seen_modified = Some(now);
                true
            }
            _ => false,
        }
    }
}

pub fn load_scene_from_path(scene_path: &Path) -> Result<SceneFile, String> {
    let raw = fs::read_to_string(scene_path)
        .map_err(|e| format!("Failed to read scene file {}: {e}", scene_path.display()))?;
    let scene: SceneFile = serde_json::from_str(&raw)
        .map_err(|e| format!("Failed to parse scene JSON {}: {e}", scene_path.display()))?;
    validate_scene(&scene)?;
    Ok(scene)
}

fn validate_scene(scene: &SceneFile) -> Result<(), String> {
    // Validation is intentionally strict on identifiers so loader/runtime paths
    // can assume uniqueness without extra defensive branching.
    if scene.layers.is_empty() {
        return Err("Scene validation failed: layers array is empty".to_string());
    }

    let mut layer_ids = HashSet::new();
    let mut sprite_ids = HashSet::new();

    for layer in &scene.layers {
        if !layer_ids.insert(layer.id.clone()) {
            return Err(format!(
                "Scene validation failed: duplicate layer id '{}'",
                layer.id
            ));
        }
        if layer.sprites.is_empty() {
            log::warn!(
                "Scene layer '{}' has no sprites. This is allowed but often accidental.",
                layer.id
            );
        }
        for sprite in &layer.sprites {
            if !sprite_ids.insert(sprite.id.clone()) {
                return Err(format!(
                    "Scene validation failed: duplicate sprite id '{}'",
                    sprite.id
                ));
            }
            if sprite.asset.is_none() && sprite.sprite_id.is_none() {
                return Err(format!(
                    "Scene validation failed: sprite '{}' must provide either 'asset' or 'sprite_id'",
                    sprite.id
                ));
            }
        }
    }

    Ok(())
}

fn modified_time(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).ok()?.modified().ok()
}

const fn default_zoom() -> f32 {
    1.0
}

const fn default_visible() -> bool {
    true
}

const fn default_scale() -> f32 {
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_file_path(name_hint: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "sme_scene_test_{}_{}_{}.json",
            name_hint,
            std::process::id(),
            nanos
        ))
    }

    fn write_scene_file(path: &Path, body: &str) {
        fs::write(path, body).expect("failed to write temp scene file");
    }

    #[test]
    fn load_scene_from_path_parses_valid_scene() {
        let path = temp_file_path("valid");
        let json = r#"
        {
          "version": "0.1",
          "scene_id": "test_scene",
          "layers": [
            {
              "id": "background",
              "parallax": 0.5,
              "sprites": [
                { "id": "s1", "asset": "assets/textures/test_sprite.png", "x": 0.0, "y": 0.0 }
              ]
            }
          ]
        }
        "#;

        write_scene_file(&path, json);
        let scene = load_scene_from_path(&path).expect("valid scene should load");
        assert_eq!(scene.version, "0.1");
        assert_eq!(scene.scene_id, "test_scene");
        assert_eq!(scene.layers.len(), 1);
        assert!(matches!(scene.layers[0].sort_mode, SortMode::None));
        assert!(scene.layers[0].visible);
        assert_eq!(scene.layers[0].sprites[0].scale_x, 1.0);
        assert_eq!(scene.layers[0].sprites[0].scale_y, 1.0);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn load_scene_from_path_rejects_empty_layers() {
        let path = temp_file_path("empty_layers");
        let json = r#"
        {
          "version": "0.1",
          "scene_id": "test_scene",
          "layers": []
        }
        "#;

        write_scene_file(&path, json);
        let err = load_scene_from_path(&path).expect_err("empty layers should fail");
        assert!(err.contains("layers array is empty"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn load_scene_from_path_rejects_duplicate_layer_ids() {
        let path = temp_file_path("dup_layer");
        let json = r#"
        {
          "version": "0.1",
          "scene_id": "test_scene",
          "layers": [
            {
              "id": "layer_a",
              "parallax": 0.3,
              "sprites": [
                { "id": "s1", "asset": "assets/textures/test_sprite.png", "x": 0.0, "y": 0.0 }
              ]
            },
            {
              "id": "layer_a",
              "parallax": 0.8,
              "sprites": [
                { "id": "s2", "asset": "assets/textures/test_sprite.png", "x": 10.0, "y": 5.0 }
              ]
            }
          ]
        }
        "#;

        write_scene_file(&path, json);
        let err = load_scene_from_path(&path).expect_err("duplicate layer ids should fail");
        assert!(err.contains("duplicate layer id"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn load_scene_from_path_rejects_duplicate_sprite_ids() {
        let path = temp_file_path("dup_sprite");
        let json = r#"
        {
          "version": "0.1",
          "scene_id": "test_scene",
          "layers": [
            {
              "id": "layer_a",
              "parallax": 0.3,
              "sprites": [
                { "id": "same_sprite", "asset": "assets/textures/test_sprite.png", "x": 0.0, "y": 0.0 }
              ]
            },
            {
              "id": "layer_b",
              "parallax": 0.9,
              "sprites": [
                { "id": "same_sprite", "asset": "assets/textures/test_sprite.png", "x": 5.0, "y": 0.0 }
              ]
            }
          ]
        }
        "#;

        write_scene_file(&path, json);
        let err = load_scene_from_path(&path).expect_err("duplicate sprite ids should fail");
        assert!(err.contains("duplicate sprite id"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn scene_camera_zoom_defaults_to_one() {
        let path = temp_file_path("camera_default");
        let json = r#"
        {
          "version": "0.1",
          "scene_id": "test_scene",
          "camera": { "start_x": 10.0, "start_y": 20.0 },
          "layers": [
            {
              "id": "layer_a",
              "parallax": 1.0,
              "sprites": [
                { "id": "s1", "asset": "assets/textures/test_sprite.png", "x": 0.0, "y": 0.0 }
              ]
            }
          ]
        }
        "#;

        write_scene_file(&path, json);
        let scene = load_scene_from_path(&path).expect("scene should parse");
        assert!(scene.camera.is_some());
        assert_eq!(scene.camera.as_ref().expect("camera exists").zoom, 1.0);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn scene_watcher_detects_newly_created_file() {
        let path = temp_file_path("watcher_create");
        let _ = fs::remove_file(&path);

        let mut watcher = SceneWatcher::new(path.clone());
        assert!(!watcher.should_reload(), "missing file should not reload");

        write_scene_file(
            &path,
            r#"{"version":"0.1","scene_id":"watcher","layers":[{"id":"l","parallax":1.0,"sprites":[{"id":"s","asset":"assets/textures/test_sprite.png","x":0.0,"y":0.0}]}]}"#,
        );

        assert!(
            watcher.should_reload(),
            "creating file should trigger reload once"
        );
        assert!(
            !watcher.should_reload(),
            "without changes, second poll should not reload"
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn load_scene_rejects_sprite_without_asset_or_sprite_id() {
        let path = temp_file_path("missing_sprite_ref");
        let json = r#"
        {
          "version": "0.1",
          "scene_id": "test_scene",
          "layers": [
            {
              "id": "layer_a",
              "parallax": 1.0,
              "sprites": [
                { "id": "s1", "x": 0.0, "y": 0.0 }
              ]
            }
          ]
        }
        "#;
        write_scene_file(&path, json);
        let err = load_scene_from_path(&path).expect_err("missing sprite refs should fail");
        assert!(err.contains("must provide either 'asset' or 'sprite_id'"));

        let _ = fs::remove_file(path);
    }
}
