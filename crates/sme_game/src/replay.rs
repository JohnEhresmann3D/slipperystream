use crate::controller::ControllerInput;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct ReplaySequence {
    #[serde(default = "default_dt")]
    pub fixed_dt: f32,
    pub frames: Vec<ReplayFrame>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ReplayFrame {
    #[serde(default)]
    pub move_x: f32,
    #[serde(default)]
    pub jump_pressed: bool,
    #[serde(default = "default_repeat")]
    pub repeat: u32,
}

impl ReplaySequence {
    pub fn expanded_inputs(&self) -> Vec<ControllerInput> {
        let mut out = Vec::new();
        for frame in &self.frames {
            for _ in 0..frame.repeat.max(1) {
                out.push(ControllerInput {
                    move_x: frame.move_x.clamp(-1.0, 1.0),
                    jump_pressed: frame.jump_pressed,
                });
            }
        }
        out
    }
}

pub fn load_replay_from_path(path: &Path) -> Result<ReplaySequence, String> {
    let raw =
        fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
    let replay: ReplaySequence = serde_json::from_str(&raw)
        .map_err(|e| format!("Failed to parse replay JSON {}: {e}", path.display()))?;
    validate_replay(&replay)?;
    Ok(replay)
}

fn validate_replay(replay: &ReplaySequence) -> Result<(), String> {
    if replay.fixed_dt <= 0.0 {
        return Err("Replay validation failed: fixed_dt must be > 0".to_string());
    }
    if replay.frames.is_empty() {
        return Err("Replay validation failed: frames list is empty".to_string());
    }
    Ok(())
}

const fn default_dt() -> f32 {
    1.0 / 60.0
}

const fn default_repeat() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collision::{Aabb, CollisionFile, CollisionGrid, GridCell, GridOrigin};
    use crate::controller::CharacterController;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_file_path(name_hint: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "sme_replay_test_{}_{}_{}.json",
            name_hint,
            std::process::id(),
            nanos
        ))
    }

    fn sample_grid() -> CollisionGrid {
        CollisionGrid::from_file(CollisionFile {
            version: "0.1".to_string(),
            collision_id: "test".to_string(),
            cell_size: 32,
            origin: GridOrigin { x: -320, y: -192 },
            width: 20,
            height: 12,
            solids: (0..20).map(|x| GridCell { x, y: 0 }).collect(),
        })
    }

    #[test]
    fn replay_file_parses_and_expands() {
        let path = temp_file_path("parse");
        fs::write(
            &path,
            r#"{
              "fixed_dt": 0.016666667,
              "frames": [
                { "move_x": 1.0, "repeat": 3 },
                { "jump_pressed": true, "repeat": 1 }
              ]
            }"#,
        )
        .expect("write replay file");

        let replay = load_replay_from_path(&path).expect("replay should load");
        let expanded = replay.expanded_inputs();
        assert_eq!(expanded.len(), 4);
        assert!(expanded[3].jump_pressed);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn replay_run_is_deterministic() {
        let path = temp_file_path("deterministic");
        fs::write(
            &path,
            r#"{
              "fixed_dt": 0.016666667,
              "frames": [
                { "move_x": 1.0, "repeat": 60 },
                { "move_x": 1.0, "jump_pressed": true, "repeat": 1 },
                { "move_x": 1.0, "repeat": 120 },
                { "move_x": -1.0, "repeat": 45 }
              ]
            }"#,
        )
        .expect("write replay file");

        let replay = load_replay_from_path(&path).expect("replay should load");
        let inputs = replay.expanded_inputs();
        let grid = sample_grid();
        let start = Aabb {
            center_x: grid.origin.x as f32 + 64.0,
            center_y: grid.origin.y as f32 + 96.0,
            half_w: 10.0,
            half_h: 14.0,
        };

        let mut run_a = CharacterController::new(start);
        let mut run_b = CharacterController::new(start);
        for input in &inputs {
            run_a.step(*input, replay.fixed_dt, &grid);
        }
        for input in &inputs {
            run_b.step(*input, replay.fixed_dt, &grid);
        }

        assert!((run_a.aabb.center_x - run_b.aabb.center_x).abs() < 0.0001);
        assert!((run_a.aabb.center_y - run_b.aabb.center_y).abs() < 0.0001);
        assert!((run_a.velocity_x - run_b.velocity_x).abs() < 0.0001);
        assert!((run_a.velocity_y - run_b.velocity_y).abs() < 0.0001);
        assert_eq!(run_a.grounded, run_b.grounded);

        let _ = fs::remove_file(path);
    }
}
