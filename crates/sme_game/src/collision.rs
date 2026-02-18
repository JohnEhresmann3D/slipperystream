//! Collision underlay: a grid-based collision representation separate from visuals.
//!
//! The engine's "sprite-scene-first" model means art is authored as layered
//! illustrations, but gameplay truth lives here in a simplified grid. This
//! keeps collision logic fast and independent of visual complexity.
//!
//! Grid-based collision is chosen over polygon-based for two reasons:
//!  1. **Performance** -- HashSet lookup per cell is O(1), no broad-phase needed
//!  2. **Simplicity** -- 2D platformer levels map naturally to a tile grid
//!
//! The core algorithm is **axis-separable move-and-slide**: resolve X movement
//! first against the grid, then resolve Y using the already-corrected X position.
//! This prevents diagonal tunneling and produces the "slide along walls" behavior
//! players expect from platformers.

use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct CollisionFile {
    pub version: String,
    pub collision_id: String,
    pub cell_size: i32,
    #[serde(default)]
    pub origin: GridOrigin,
    pub width: i32,
    pub height: i32,
    pub solids: Vec<GridCell>,
}

#[derive(Debug, Deserialize, Clone, Copy, Default)]
pub struct GridOrigin {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridCell {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub center_x: f32,
    pub center_y: f32,
    pub half_w: f32,
    pub half_h: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct CollisionMoveResult {
    pub aabb: Aabb,
    pub collided_y: bool,
    pub blocked_left: bool,
    pub blocked_right: bool,
    pub blocked_down: bool,
    pub blocked_up: bool,
}

#[derive(Debug, Clone)]
pub struct CollisionGrid {
    pub version: String,
    pub collision_id: String,
    pub cell_size: i32,
    pub origin: GridOrigin,
    pub width: i32,
    pub height: i32,
    solids: HashSet<GridCell>,
}

impl CollisionGrid {
    pub fn from_file(file: CollisionFile) -> Self {
        let solids = file.solids.into_iter().collect();
        Self {
            version: file.version,
            collision_id: file.collision_id,
            cell_size: file.cell_size,
            origin: file.origin,
            width: file.width,
            height: file.height,
            solids,
        }
    }

    pub fn is_solid(&self, x: i32, y: i32) -> bool {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return false;
        }
        self.solids.contains(&GridCell { x, y })
    }

    pub fn solids_iter(&self) -> impl Iterator<Item = &GridCell> {
        self.solids.iter()
    }

    #[allow(dead_code)]
    pub fn move_and_collide(&self, aabb: Aabb, dx: f32, dy: f32) -> Aabb {
        self.move_and_collide_detailed(aabb, dx, dy).aabb
    }

    pub fn move_and_collide_detailed(&self, aabb: Aabb, dx: f32, dy: f32) -> CollisionMoveResult {
        const EPS: f32 = 0.0001;

        // Axis-separable move-and-slide:
        // resolve X first, then resolve Y using updated X position.
        let resolved_x = self.resolve_axis_x(aabb, dx);
        let x_expected = aabb.center_x + dx;
        let collided_x = (resolved_x - x_expected).abs() > EPS;

        let mut moved = aabb;
        moved.center_x = resolved_x;
        let resolved_y = self.resolve_axis_y(moved, dy);
        let y_expected = aabb.center_y + dy;
        let collided_y = (resolved_y - y_expected).abs() > EPS;
        moved.center_y = resolved_y;

        // Directional block flags are consumed by controller code to zero
        // velocities only when motion was actually blocked on that side.
        let blocked_left = collided_x && dx < 0.0;
        let blocked_right = collided_x && dx > 0.0;
        let blocked_down = collided_y && dy < 0.0;
        let blocked_up = collided_y && dy > 0.0;

        CollisionMoveResult {
            aabb: moved,
            collided_y,
            blocked_left,
            blocked_right,
            blocked_down,
            blocked_up,
        }
    }

    fn resolve_axis_x(&self, aabb: Aabb, dx: f32) -> f32 {
        if dx == 0.0 {
            return aabb.center_x;
        }

        const EPS: f32 = 0.001;
        let mut candidate_x = aabb.center_x + dx;
        let min_y = aabb.center_y - aabb.half_h + EPS;
        let max_y = aabb.center_y + aabb.half_h - EPS;
        let y0 = self.world_to_cell_y(min_y);
        let y1 = self.world_to_cell_y(max_y);

        if dx > 0.0 {
            let max_x = candidate_x + aabb.half_w - EPS;
            let x_cell = self.world_to_cell_x(max_x);
            for y in y0..=y1 {
                if self.is_solid(x_cell, y) {
                    let cell_left = self.cell_left_world(x_cell);
                    candidate_x = candidate_x.min(cell_left - aabb.half_w);
                }
            }
            // Guardrail: never push opposite direction during resolution.
            candidate_x = candidate_x.max(aabb.center_x);
        } else {
            let min_x = candidate_x - aabb.half_w + EPS;
            let x_cell = self.world_to_cell_x(min_x);
            for y in y0..=y1 {
                if self.is_solid(x_cell, y) {
                    let cell_right = self.cell_right_world(x_cell);
                    candidate_x = candidate_x.max(cell_right + aabb.half_w);
                }
            }
            // Guardrail: never push opposite direction during resolution.
            candidate_x = candidate_x.min(aabb.center_x);
        }

        candidate_x
    }

    fn resolve_axis_y(&self, aabb: Aabb, dy: f32) -> f32 {
        if dy == 0.0 {
            return aabb.center_y;
        }

        const EPS: f32 = 0.001;
        let mut candidate_y = aabb.center_y + dy;
        let min_x = aabb.center_x - aabb.half_w + EPS;
        let max_x = aabb.center_x + aabb.half_w - EPS;
        let x0 = self.world_to_cell_x(min_x);
        let x1 = self.world_to_cell_x(max_x);

        if dy > 0.0 {
            let max_y = candidate_y + aabb.half_h - EPS;
            let y_cell = self.world_to_cell_y(max_y);
            for x in x0..=x1 {
                if self.is_solid(x, y_cell) {
                    let cell_bottom = self.cell_bottom_world(y_cell);
                    candidate_y = candidate_y.min(cell_bottom - aabb.half_h);
                }
            }
            // Guardrail: never push opposite direction during resolution.
            candidate_y = candidate_y.max(aabb.center_y);
        } else {
            let min_y = candidate_y - aabb.half_h + EPS;
            let y_cell = self.world_to_cell_y(min_y);
            for x in x0..=x1 {
                if self.is_solid(x, y_cell) {
                    let cell_top = self.cell_top_world(y_cell);
                    candidate_y = candidate_y.max(cell_top + aabb.half_h);
                }
            }
            // Guardrail: never push opposite direction during resolution.
            candidate_y = candidate_y.min(aabb.center_y);
        }

        candidate_y
    }

    fn world_to_cell_x(&self, world_x: f32) -> i32 {
        ((world_x - self.origin.x as f32) / self.cell_size as f32).floor() as i32
    }

    fn world_to_cell_y(&self, world_y: f32) -> i32 {
        ((world_y - self.origin.y as f32) / self.cell_size as f32).floor() as i32
    }

    fn cell_left_world(&self, x: i32) -> f32 {
        self.origin.x as f32 + (x * self.cell_size) as f32
    }

    fn cell_right_world(&self, x: i32) -> f32 {
        self.origin.x as f32 + ((x + 1) * self.cell_size) as f32
    }

    fn cell_bottom_world(&self, y: i32) -> f32 {
        self.origin.y as f32 + (y * self.cell_size) as f32
    }

    fn cell_top_world(&self, y: i32) -> f32 {
        self.origin.y as f32 + ((y + 1) * self.cell_size) as f32
    }
}

pub fn load_collision_from_path(path: &Path) -> Result<CollisionGrid, String> {
    let raw =
        fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
    let file: CollisionFile = serde_json::from_str(&raw)
        .map_err(|e| format!("Failed to parse collision JSON {}: {e}", path.display()))?;
    validate_collision_file(&file)?;
    Ok(CollisionGrid::from_file(file))
}

fn validate_collision_file(file: &CollisionFile) -> Result<(), String> {
    if file.cell_size <= 0 {
        return Err("Collision validation failed: cell_size must be > 0".to_string());
    }
    if file.width <= 0 || file.height <= 0 {
        return Err("Collision validation failed: width and height must be > 0".to_string());
    }

    let mut seen = HashSet::new();
    for cell in &file.solids {
        if cell.x < 0 || cell.x >= file.width || cell.y < 0 || cell.y >= file.height {
            return Err(format!(
                "Collision validation failed: solid cell out of bounds ({}, {})",
                cell.x, cell.y
            ));
        }
        if !seen.insert(*cell) {
            return Err(format!(
                "Collision validation failed: duplicate solid cell ({}, {})",
                cell.x, cell.y
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
            "sme_collision_test_{}_{}_{}.json",
            name_hint,
            std::process::id(),
            nanos
        ))
    }

    #[test]
    fn load_collision_valid_file_parses() {
        let path = temp_file_path("valid");
        fs::write(
            &path,
            r#"{
              "version":"0.1",
              "collision_id":"test",
              "cell_size":32,
              "origin":{"x":0,"y":0},
              "width":4,
              "height":4,
              "solids":[{"x":1,"y":1},{"x":2,"y":1}]
            }"#,
        )
        .expect("write temp file");

        let grid = load_collision_from_path(&path).expect("valid collision should load");
        assert_eq!(grid.cell_size, 32);
        assert!(grid.is_solid(1, 1));
        assert!(!grid.is_solid(0, 0));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn load_collision_rejects_duplicate_cells() {
        let path = temp_file_path("dup");
        fs::write(
            &path,
            r#"{
              "version":"0.1",
              "collision_id":"test",
              "cell_size":32,
              "width":4,
              "height":4,
              "solids":[{"x":1,"y":1},{"x":1,"y":1}]
            }"#,
        )
        .expect("write temp file");

        let err = load_collision_from_path(&path).expect_err("duplicate cells should fail");
        assert!(err.contains("duplicate solid cell"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn move_and_collide_blocks_motion_into_wall() {
        let grid = CollisionGrid::from_file(CollisionFile {
            version: "0.1".to_string(),
            collision_id: "test".to_string(),
            cell_size: 32,
            origin: GridOrigin { x: 0, y: 0 },
            width: 8,
            height: 8,
            solids: vec![GridCell { x: 2, y: 1 }],
        });

        let start = Aabb {
            center_x: 32.0 + 8.0,
            center_y: 32.0 + 8.0,
            half_w: 8.0,
            half_h: 8.0,
        };
        let moved = grid.move_and_collide(start, 40.0, 0.0);
        assert!(
            moved.center_x <= 64.0 - start.half_w + 0.001,
            "AABB should stop at left edge of wall cell"
        );
    }

    #[test]
    fn move_up_against_obstacle_does_not_push_downward() {
        let grid = CollisionGrid::from_file(CollisionFile {
            version: "0.1".to_string(),
            collision_id: "test".to_string(),
            cell_size: 32,
            origin: GridOrigin { x: 0, y: 0 },
            width: 8,
            height: 8,
            solids: vec![
                // floor
                GridCell { x: 0, y: 0 },
                GridCell { x: 1, y: 0 },
                GridCell { x: 2, y: 0 },
                GridCell { x: 3, y: 0 },
                // side obstacle to the right of player
                GridCell { x: 2, y: 1 },
            ],
        });

        let start = Aabb {
            // Standing on floor and touching near side obstacle.
            center_x: 48.0,
            center_y: 40.0,
            half_w: 8.0,
            half_h: 8.0,
        };

        let moved = grid.move_and_collide(start, 0.0, 10.0);
        assert!(
            moved.center_y >= start.center_y - 0.0001,
            "Moving up should never push the character downward"
        );
    }

    #[test]
    fn move_and_collide_detailed_sets_blocked_direction_flags() {
        let grid = CollisionGrid::from_file(CollisionFile {
            version: "0.1".to_string(),
            collision_id: "test".to_string(),
            cell_size: 32,
            origin: GridOrigin { x: 0, y: 0 },
            width: 8,
            height: 8,
            solids: vec![GridCell { x: 2, y: 1 }],
        });

        let start = Aabb {
            center_x: 32.0 + 8.0,
            center_y: 32.0 + 8.0,
            half_w: 8.0,
            half_h: 8.0,
        };

        let moved = grid.move_and_collide_detailed(start, 40.0, 0.0);
        assert!(moved.blocked_right);
        assert!(!moved.blocked_left);
        assert!(!moved.collided_y);
    }
}
