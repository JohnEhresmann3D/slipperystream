//! Frame-based sprite animation types and deterministic tick logic.
//!
//! Animation clips are sequences of sprite frames with per-frame durations.
//! All timing uses integer microseconds (`u64`) to guarantee deterministic
//! advancement under the engine's fixed-timestep model -- no floating-point
//! drift across platforms.
//!
//! The JSON format stores `duration_ms` for human readability; on load this
//! is converted to `duration_us` for internal use.

use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// A single frame in an animation clip.
#[derive(Debug, Clone)]
pub struct AnimationFrame {
    pub sprite_id: String,
    pub duration_us: u64,
}

/// A named sequence of frames that can loop or play once.
#[derive(Debug, Clone)]
pub struct AnimationClip {
    pub frames: Vec<AnimationFrame>,
    pub looping: bool,
}

impl AnimationClip {
    /// Total duration of one full cycle in microseconds.
    pub fn total_duration_us(&self) -> u64 {
        self.frames.iter().map(|f| f.duration_us).sum()
    }
}

/// Top-level animation definition file (deserialized from JSON).
#[derive(Debug, Clone)]
pub struct AnimationFile {
    pub version: String,
    pub animation_id: String,
    pub animations: HashMap<String, AnimationClip>,
}

/// Runtime state for one active animation instance.
#[derive(Debug, Clone)]
pub struct AnimationState {
    pub source_id: String,
    pub clip_name: String,
    pub frame_index: usize,
    pub elapsed_us: u64,
    pub finished: bool,
}

impl AnimationState {
    pub fn new(source_id: &str, clip_name: &str) -> Self {
        Self {
            source_id: source_id.to_string(),
            clip_name: clip_name.to_string(),
            frame_index: 0,
            elapsed_us: 0,
            finished: false,
        }
    }

    /// Advance the animation by `dt_us` microseconds. Returns the current frame's
    /// `sprite_id`. Uses integer arithmetic only for determinism.
    pub fn tick<'a>(&mut self, dt_us: u64, clip: &'a AnimationClip) -> &'a str {
        if clip.frames.is_empty() || self.finished {
            return if let Some(frame) = clip.frames.get(self.frame_index) {
                &frame.sprite_id
            } else if let Some(frame) = clip.frames.last() {
                &frame.sprite_id
            } else {
                ""
            };
        }

        self.elapsed_us += dt_us;

        loop {
            let current_frame = &clip.frames[self.frame_index];
            if self.elapsed_us < current_frame.duration_us {
                break;
            }

            self.elapsed_us -= current_frame.duration_us;
            self.frame_index += 1;

            if self.frame_index >= clip.frames.len() {
                if clip.looping {
                    self.frame_index = 0;
                } else {
                    self.frame_index = clip.frames.len() - 1;
                    self.elapsed_us = 0;
                    self.finished = true;
                    break;
                }
            }
        }

        &clip.frames[self.frame_index].sprite_id
    }
}

// --- JSON deserialization types (private) ---

#[derive(Debug, Deserialize)]
struct AnimationFileJson {
    version: String,
    animation_id: String,
    animations: HashMap<String, AnimationClipJson>,
}

#[derive(Debug, Deserialize)]
struct AnimationClipJson {
    frames: Vec<AnimationFrameJson>,
    #[serde(default)]
    looping: bool,
}

#[derive(Debug, Deserialize)]
struct AnimationFrameJson {
    sprite_id: String,
    duration_ms: u64,
}

/// Load an animation definition file from disk.
pub fn load_animation_file(path: &Path) -> Result<AnimationFile, String> {
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read animation file {}: {e}", path.display()))?;
    let json: AnimationFileJson = serde_json::from_str(&raw)
        .map_err(|e| format!("Failed to parse animation file {}: {e}", path.display()))?;
    validate_animation_json(&json)?;

    let mut animations = HashMap::new();
    for (name, clip_json) in json.animations {
        let frames = clip_json
            .frames
            .into_iter()
            .map(|f| AnimationFrame {
                sprite_id: f.sprite_id,
                duration_us: f.duration_ms * 1000,
            })
            .collect();
        animations.insert(
            name,
            AnimationClip {
                frames,
                looping: clip_json.looping,
            },
        );
    }

    Ok(AnimationFile {
        version: json.version,
        animation_id: json.animation_id,
        animations,
    })
}

fn validate_animation_json(json: &AnimationFileJson) -> Result<(), String> {
    if json.version != "0.1" {
        return Err(format!(
            "Animation validation failed: unsupported version '{}'",
            json.version
        ));
    }
    if json.animation_id.is_empty() {
        return Err("Animation validation failed: animation_id is empty".to_string());
    }
    for (name, clip) in &json.animations {
        if clip.frames.is_empty() {
            return Err(format!(
                "Animation validation failed: clip '{}' has no frames",
                name
            ));
        }
        for (i, frame) in clip.frames.iter().enumerate() {
            if frame.sprite_id.is_empty() {
                return Err(format!(
                    "Animation validation failed: clip '{}' frame {} has empty sprite_id",
                    name, i
                ));
            }
            if frame.duration_ms == 0 {
                return Err(format!(
                    "Animation validation failed: clip '{}' frame {} has zero duration",
                    name, i
                ));
            }
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
            "sme_anim_test_{}_{}_{}.json",
            name_hint,
            std::process::id(),
            nanos
        ))
    }

    fn make_clip(durations_ms: &[u64], looping: bool) -> AnimationClip {
        AnimationClip {
            frames: durations_ms
                .iter()
                .enumerate()
                .map(|(i, &d)| AnimationFrame {
                    sprite_id: format!("sprite_{}", i),
                    duration_us: d * 1000,
                })
                .collect(),
            looping,
        }
    }

    #[test]
    fn tick_advances_through_frames() {
        let clip = make_clip(&[100, 100, 100], true);
        let mut state = AnimationState::new("test", "walk");

        // At t=0, should be on frame 0
        let id = state.tick(0, &clip);
        assert_eq!(id, "sprite_0");

        // Advance 50ms — still on frame 0
        let id = state.tick(50_000, &clip);
        assert_eq!(id, "sprite_0");

        // Advance another 60ms (total 110ms) — should be on frame 1
        let id = state.tick(60_000, &clip);
        assert_eq!(id, "sprite_1");
    }

    #[test]
    fn looping_wraps_around() {
        let clip = make_clip(&[100, 100], true);
        let mut state = AnimationState::new("test", "idle");

        // Advance past both frames (250ms total)
        let id = state.tick(250_000, &clip);
        assert_eq!(id, "sprite_0");
        assert!(!state.finished);
    }

    #[test]
    fn non_looping_stops_on_last_frame() {
        let clip = make_clip(&[100, 100], false);
        let mut state = AnimationState::new("test", "jump");

        // Advance past total duration
        let id = state.tick(300_000, &clip);
        assert_eq!(id, "sprite_1");
        assert!(state.finished);

        // Further ticks stay on last frame
        let id = state.tick(100_000, &clip);
        assert_eq!(id, "sprite_1");
        assert!(state.finished);
    }

    #[test]
    fn variable_frame_durations() {
        let clip = make_clip(&[50, 200, 100], true);
        let mut state = AnimationState::new("test", "attack");

        // 50ms => end of frame 0, should be on frame 1
        let id = state.tick(50_000, &clip);
        assert_eq!(id, "sprite_1");

        // 150ms more (total 200ms) => still on frame 1 (200ms duration)
        let id = state.tick(150_000, &clip);
        assert_eq!(id, "sprite_1");

        // 50ms more (total 250ms) => frame 1 done, now on frame 2
        let id = state.tick(50_000, &clip);
        assert_eq!(id, "sprite_2");
    }

    #[test]
    fn determinism_identical_results() {
        let clip = make_clip(&[100, 150, 80], true);
        let dt = 16_667u64; // ~60fps fixed step
        let steps = 100;

        let mut state_a = AnimationState::new("test", "run");
        let mut state_b = AnimationState::new("test", "run");

        for _ in 0..steps {
            let id_a = state_a.tick(dt, &clip);
            let id_b = state_b.tick(dt, &clip);
            assert_eq!(id_a, id_b);
        }
        assert_eq!(state_a.frame_index, state_b.frame_index);
        assert_eq!(state_a.elapsed_us, state_b.elapsed_us);
    }

    #[test]
    fn load_animation_file_parses_valid_json() {
        let path = temp_file_path("valid");
        let json = r#"
        {
          "version": "0.1",
          "animation_id": "hero",
          "animations": {
            "idle": {
              "frames": [
                { "sprite_id": "id-aaa", "duration_ms": 100 },
                { "sprite_id": "id-bbb", "duration_ms": 100 }
              ],
              "looping": true
            },
            "jump": {
              "frames": [
                { "sprite_id": "id-ccc", "duration_ms": 120 }
              ],
              "looping": false
            }
          }
        }
        "#;
        fs::write(&path, json).expect("write temp file");

        let file = load_animation_file(&path).expect("should parse");
        assert_eq!(file.animation_id, "hero");
        assert_eq!(file.animations.len(), 2);

        let idle = &file.animations["idle"];
        assert!(idle.looping);
        assert_eq!(idle.frames.len(), 2);
        assert_eq!(idle.frames[0].sprite_id, "id-aaa");
        assert_eq!(idle.frames[0].duration_us, 100_000);

        let jump = &file.animations["jump"];
        assert!(!jump.looping);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn load_animation_file_rejects_bad_version() {
        let path = temp_file_path("bad_version");
        let json = r#"
        {
          "version": "9.9",
          "animation_id": "hero",
          "animations": {
            "idle": {
              "frames": [{ "sprite_id": "a", "duration_ms": 100 }]
            }
          }
        }
        "#;
        fs::write(&path, json).expect("write temp file");
        let err = load_animation_file(&path).expect_err("bad version should fail");
        assert!(err.contains("unsupported version"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn load_animation_file_rejects_zero_duration() {
        let path = temp_file_path("zero_dur");
        let json = r#"
        {
          "version": "0.1",
          "animation_id": "hero",
          "animations": {
            "idle": {
              "frames": [{ "sprite_id": "a", "duration_ms": 0 }]
            }
          }
        }
        "#;
        fs::write(&path, json).expect("write temp file");
        let err = load_animation_file(&path).expect_err("zero duration should fail");
        assert!(err.contains("zero duration"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn total_duration_us() {
        let clip = make_clip(&[100, 200, 300], true);
        assert_eq!(clip.total_duration_us(), 600_000);
    }
}
