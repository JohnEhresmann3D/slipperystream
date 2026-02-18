use crate::collision::{Aabb, CollisionGrid, CollisionMoveResult};

#[derive(Debug, Clone, Copy)]
pub struct ControllerInput {
    pub move_x: f32,
    pub jump_pressed: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ControllerConfig {
    pub max_speed: f32,
    pub accel_ground: f32,
    pub accel_air: f32,
    pub friction_ground: f32,
    pub gravity: f32,
    pub max_fall_speed: f32,
    pub jump_speed: f32,
}

impl Default for ControllerConfig {
    fn default() -> Self {
        Self {
            max_speed: 180.0,
            accel_ground: 1600.0,
            accel_air: 900.0,
            friction_ground: 2000.0,
            gravity: -1800.0,
            max_fall_speed: -900.0,
            jump_speed: 620.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CharacterController {
    pub aabb: Aabb,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub grounded: bool,
    pub contacts: ContactState,
    pub config: ControllerConfig,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ContactState {
    pub left: bool,
    pub right: bool,
    pub down: bool,
    pub up: bool,
}

impl CharacterController {
    pub fn new(aabb: Aabb) -> Self {
        Self {
            aabb,
            velocity_x: 0.0,
            velocity_y: 0.0,
            grounded: false,
            contacts: ContactState::default(),
            config: ControllerConfig::default(),
        }
    }

    pub fn step(&mut self, input: ControllerInput, dt: f32, collision_grid: &CollisionGrid) {
        // Horizontal control: accelerate toward intent, friction when grounded and idle.
        let accel = if self.grounded {
            self.config.accel_ground
        } else {
            self.config.accel_air
        };

        if input.move_x != 0.0 {
            let target = input.move_x * self.config.max_speed;
            self.velocity_x = move_towards(self.velocity_x, target, accel * dt);
        } else if self.grounded {
            self.velocity_x = move_towards(self.velocity_x, 0.0, self.config.friction_ground * dt);
        }

        // Jump is edge-triggered and only legal from grounded state.
        if input.jump_pressed && self.grounded {
            self.velocity_y = self.config.jump_speed;
            self.grounded = false;
        }

        // Gravity is always applied in fixed-step simulation.
        self.velocity_y =
            (self.velocity_y + self.config.gravity * dt).max(self.config.max_fall_speed);

        let dx = self.velocity_x * dt;
        let dy = self.velocity_y * dt;
        let result = collision_grid.move_and_collide_detailed(self.aabb, dx, dy);
        self.apply_collision_result(result);
    }

    fn apply_collision_result(&mut self, result: CollisionMoveResult) {
        self.aabb = result.aabb;
        self.contacts = ContactState {
            left: result.blocked_left,
            right: result.blocked_right,
            down: result.blocked_down,
            up: result.blocked_up,
        };

        if (result.blocked_left && self.velocity_x < 0.0)
            || (result.blocked_right && self.velocity_x > 0.0)
        {
            self.velocity_x = 0.0;
        }

        if result.blocked_up && self.velocity_y > 0.0 {
            self.velocity_y = 0.0;
        }
        // Grounded is driven from collision contact, not from y-position heuristics.
        if result.blocked_down && self.velocity_y < 0.0 {
            self.velocity_y = 0.0;
            self.grounded = true;
        } else if result.collided_y {
            self.velocity_y = 0.0;
            self.grounded = false;
        } else {
            self.grounded = false;
        }
    }

    #[allow(dead_code)]
    pub fn is_grounded(&self) -> bool {
        self.grounded
    }

    #[allow(dead_code)]
    pub fn is_blocked_left(&self) -> bool {
        self.contacts.left
    }

    #[allow(dead_code)]
    pub fn is_blocked_right(&self) -> bool {
        self.contacts.right
    }

    #[allow(dead_code)]
    pub fn is_blocked_up(&self) -> bool {
        self.contacts.up
    }

    #[allow(dead_code)]
    pub fn is_blocked_down(&self) -> bool {
        self.contacts.down
    }
}

fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    if (target - current).abs() <= max_delta {
        target
    } else if target > current {
        current + max_delta
    } else {
        current - max_delta
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collision::{CollisionFile, GridCell, GridOrigin};

    fn sample_grid() -> CollisionGrid {
        CollisionGrid::from_file(CollisionFile {
            version: "0.1".to_string(),
            collision_id: "test".to_string(),
            cell_size: 32,
            origin: GridOrigin { x: -320, y: -192 },
            width: 20,
            height: 12,
            solids: vec![
                GridCell { x: 0, y: 0 },
                GridCell { x: 1, y: 0 },
                GridCell { x: 2, y: 0 },
                GridCell { x: 3, y: 0 },
                GridCell { x: 4, y: 0 },
                GridCell { x: 5, y: 0 },
                GridCell { x: 6, y: 0 },
                GridCell { x: 7, y: 0 },
                GridCell { x: 8, y: 0 },
                GridCell { x: 9, y: 0 },
                GridCell { x: 10, y: 0 },
                GridCell { x: 11, y: 0 },
                GridCell { x: 12, y: 0 },
                GridCell { x: 13, y: 0 },
                GridCell { x: 14, y: 0 },
                GridCell { x: 15, y: 0 },
                GridCell { x: 16, y: 0 },
                GridCell { x: 17, y: 0 },
                GridCell { x: 18, y: 0 },
                GridCell { x: 19, y: 0 },
                GridCell { x: 6, y: 1 },
                GridCell { x: 6, y: 2 },
                GridCell { x: 10, y: 1 },
                GridCell { x: 10, y: 2 },
            ],
        })
    }

    #[test]
    fn deterministic_sequence_reaches_same_final_state() {
        let grid = sample_grid();
        let start = Aabb {
            center_x: grid.origin.x as f32 + 64.0,
            center_y: grid.origin.y as f32 + 96.0,
            half_w: 10.0,
            half_h: 14.0,
        };

        let mut inputs = Vec::new();
        for _ in 0..60 {
            inputs.push(ControllerInput {
                move_x: 1.0,
                jump_pressed: false,
            });
        }
        inputs.push(ControllerInput {
            move_x: 1.0,
            jump_pressed: true,
        });
        for _ in 0..120 {
            inputs.push(ControllerInput {
                move_x: 1.0,
                jump_pressed: false,
            });
        }
        for _ in 0..60 {
            inputs.push(ControllerInput {
                move_x: -1.0,
                jump_pressed: false,
            });
        }

        let dt = 1.0 / 60.0;
        let mut run_a = CharacterController::new(start);
        let mut run_b = CharacterController::new(start);

        for input in &inputs {
            run_a.step(*input, dt, &grid);
        }
        for input in &inputs {
            run_b.step(*input, dt, &grid);
        }

        assert!((run_a.aabb.center_x - run_b.aabb.center_x).abs() < 0.0001);
        assert!((run_a.aabb.center_y - run_b.aabb.center_y).abs() < 0.0001);
        assert!((run_a.velocity_x - run_b.velocity_x).abs() < 0.0001);
        assert!((run_a.velocity_y - run_b.velocity_y).abs() < 0.0001);
        assert_eq!(run_a.grounded, run_b.grounded);
    }

    #[test]
    fn jump_only_activates_when_grounded() {
        let grid = sample_grid();
        let start = Aabb {
            center_x: grid.origin.x as f32 + 64.0,
            center_y: grid.origin.y as f32 + 96.0,
            half_w: 10.0,
            half_h: 14.0,
        };

        let mut controller = CharacterController::new(start);
        controller.grounded = false;
        controller.step(
            ControllerInput {
                move_x: 0.0,
                jump_pressed: true,
            },
            1.0 / 60.0,
            &grid,
        );
        assert!(controller.velocity_y <= 0.0);
    }

    #[test]
    fn contact_state_reports_wall_block() {
        let grid = sample_grid();
        let start = Aabb {
            center_x: grid.origin.x as f32 + (6.0 * 32.0) - 12.0,
            center_y: grid.origin.y as f32 + (1.0 * 32.0) + 20.0,
            half_w: 10.0,
            half_h: 14.0,
        };

        let mut controller = CharacterController::new(start);
        controller.grounded = true;
        let mut hit_right_wall = false;
        for _ in 0..120 {
            controller.step(
                ControllerInput {
                    move_x: 1.0,
                    jump_pressed: false,
                },
                1.0 / 60.0,
                &grid,
            );
            if controller.is_blocked_right() {
                hit_right_wall = true;
                break;
            }
        }

        assert!(
            hit_right_wall,
            "controller should eventually hit right wall"
        );
    }
}
