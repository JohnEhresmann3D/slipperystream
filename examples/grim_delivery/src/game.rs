//! Grim Delivery gameplay simulation.
//!
//! Runs only inside fixed timestep steps (60 Hz), mirrors the engine's
//! determinism contract: no wall clock, no runtime randomness (all layout
//! randomness is consumed at level start in `level.rs`), state only mutates
//! in `update_fixed()`.
//!
//! The simulation knows nothing about wgpu or egui. It emits:
//!  - `build_quads()` — a back-to-front list of colored quads for the renderer
//!  - `banners` / phase state — read by the HUD layer

use crate::level::{self, world, HouseSpawn, LevelDef, ObstacleKind, HOUSE_PALETTE};

const LANE_CHANGE_SPEED: f32 = 650.0;
const NOTICE_SPEED_X: f32 = -430.0;
const THROW_COOLDOWN: f32 = 0.35;
const WOBBLE_TIME: f32 = 0.9;
const IFRAME_TIME: f32 = 1.6;
const WOBBLE_SPEED_FACTOR: f32 = 0.45;
const DOG_SPEED: f32 = 95.0;
const CAMERA_LOOKAHEAD: f32 = 180.0;

const SCORE_CORRECT: i64 = 100;
const SCORE_COMBO_STEP: i64 = 25;
const SCORE_WRONG: i64 = -50;

/// A single colored rectangle in world space. The whole game is drawn with these.
#[derive(Debug, Clone, Copy)]
pub struct Quad {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub color: [f32; 4],
}

/// Input snapshot for one fixed step, pre-digested from engine input state.
#[derive(Debug, Clone, Copy, Default)]
pub struct GameInput {
    pub lane_left_pressed: bool,
    pub lane_right_pressed: bool,
    pub throw_pressed: bool,
    /// Space on menu-ish screens.
    pub advance_pressed: bool,
    /// R on the final screen.
    pub restart_pressed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Level card: route name, quota, controls. Space to start.
    Intro,
    Riding,
    /// End-of-route clipboard summary. Space to continue.
    Summary,
    /// After level 3: totals + performance rank. R restarts.
    Final,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BannerKind {
    Good,
    Bad,
    Neutral,
}

#[derive(Debug, Clone)]
pub struct Banner {
    pub text: String,
    pub kind: BannerKind,
    pub ttl: f32,
}

struct House {
    spawn: HouseSpawn,
    delivered: bool,
    wrong_hit: bool,
    missed: bool,
}

struct Notice {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
}

struct Obstacle {
    kind: ObstacleKind,
    x: f32,
    y: f32,
    dir: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FxKind {
    /// White wisp rising from a correct delivery.
    Soul,
    /// Resident storming onto the lawn after a misfile.
    OopsNpc,
    /// Notice landing on grass, accomplishing nothing.
    Splat,
}

struct Fx {
    kind: FxKind,
    x: f32,
    y: f32,
    age: f32,
    ttl: f32,
}

struct Bike {
    x: f32,
    y: f32,
    lane: usize,
    wobble_timer: f32,
    iframe_timer: f32,
    /// Advances with distance for wobble jiggle; deterministic, not wall clock.
    anim_t: f32,
}

/// Per-level tallies, kept after the level ends for the summary screen.
#[derive(Debug, Clone, Copy, Default)]
pub struct LevelStats {
    pub souls: u32,
    pub quota: u32,
    pub wrong: u32,
    pub missed: u32,
    pub score: i64,
    pub best_combo: u32,
}

pub struct Game {
    levels: Vec<LevelDef>,
    pub level_index: usize,
    pub phase: Phase,

    bike: Bike,
    houses: Vec<House>,
    notices: Vec<Notice>,
    obstacles: Vec<Obstacle>,
    fx: Vec<Fx>,
    pub banners: Vec<Banner>,

    throw_cooldown: f32,
    pub combo: u32,
    pub stats: LevelStats,
    pub completed_levels: Vec<LevelStats>,
    pub total_score: i64,
}

impl Game {
    pub fn new() -> Self {
        let levels = level::levels();
        let mut game = Self {
            levels,
            level_index: 0,
            phase: Phase::Intro,
            bike: Bike {
                x: world::LANES[1],
                y: 0.0,
                lane: 1,
                wobble_timer: 0.0,
                iframe_timer: 0.0,
                anim_t: 0.0,
            },
            houses: Vec::new(),
            notices: Vec::new(),
            obstacles: Vec::new(),
            fx: Vec::new(),
            banners: Vec::new(),
            throw_cooldown: 0.0,
            combo: 0,
            stats: LevelStats::default(),
            completed_levels: Vec::new(),
            total_score: 0,
        };
        game.load_level(0);
        game
    }

    pub fn level(&self) -> &LevelDef {
        &self.levels[self.level_index]
    }

    pub fn camera_y(&self) -> f32 {
        self.bike.y + CAMERA_LOOKAHEAD
    }

    fn load_level(&mut self, index: usize) {
        self.level_index = index;
        let def = &self.levels[index];
        self.houses = level::generate_houses(def)
            .into_iter()
            .map(|spawn| House {
                spawn,
                delivered: false,
                wrong_hit: false,
                missed: false,
            })
            .collect();
        self.obstacles = def
            .obstacles
            .iter()
            .map(|o| Obstacle {
                kind: o.kind,
                x: o.x,
                y: o.y,
                dir: o.dir,
            })
            .collect();
        self.notices.clear();
        self.fx.clear();
        self.banners.clear();
        self.bike = Bike {
            x: world::LANES[1],
            y: 0.0,
            lane: 1,
            wobble_timer: 0.0,
            iframe_timer: 0.0,
            anim_t: 0.0,
        };
        self.throw_cooldown = 0.0;
        self.combo = 0;
        self.stats = LevelStats {
            quota: def.target_count as u32,
            ..LevelStats::default()
        };
        self.phase = Phase::Intro;
    }

    pub fn update_fixed(&mut self, dt: f32, input: GameInput) {
        for banner in &mut self.banners {
            banner.ttl -= dt;
        }
        self.banners.retain(|b| b.ttl > 0.0);

        match self.phase {
            Phase::Intro => {
                if input.advance_pressed {
                    self.phase = Phase::Riding;
                }
            }
            Phase::Riding => self.step_riding(dt, input),
            Phase::Summary => {
                if input.advance_pressed {
                    if self.level_index + 1 < self.levels.len() {
                        let next = self.level_index + 1;
                        self.load_level(next);
                    } else {
                        self.phase = Phase::Final;
                    }
                }
            }
            Phase::Final => {
                if input.restart_pressed {
                    self.completed_levels.clear();
                    self.total_score = 0;
                    self.load_level(0);
                }
            }
        }
    }

    fn step_riding(&mut self, dt: f32, input: GameInput) {
        let scroll_speed = self.level().scroll_speed;
        let length = self.level().length;

        self.throw_cooldown = (self.throw_cooldown - dt).max(0.0);
        self.bike.wobble_timer = (self.bike.wobble_timer - dt).max(0.0);
        self.bike.iframe_timer = (self.bike.iframe_timer - dt).max(0.0);

        let in_control = self.bike.wobble_timer <= 0.0;

        // --- Lane movement -----------------------------------------------------
        if in_control {
            if input.lane_left_pressed && self.bike.lane > 0 {
                self.bike.lane -= 1;
            }
            if input.lane_right_pressed && self.bike.lane < world::LANES.len() - 1 {
                self.bike.lane += 1;
            }
        }
        let target_x = world::LANES[self.bike.lane];
        let max_dx = LANE_CHANGE_SPEED * dt;
        let diff = target_x - self.bike.x;
        if in_control {
            self.bike.x += diff.clamp(-max_dx, max_dx);
        }

        let speed = if in_control {
            scroll_speed
        } else {
            scroll_speed * WOBBLE_SPEED_FACTOR
        };
        self.bike.y += speed * dt;
        self.bike.anim_t += dt;

        // --- Throwing ----------------------------------------------------------
        if in_control && input.throw_pressed && self.throw_cooldown <= 0.0 {
            self.notices.push(Notice {
                x: self.bike.x - 22.0,
                y: self.bike.y + 8.0,
                vx: NOTICE_SPEED_X,
                vy: speed * 0.25,
            });
            self.throw_cooldown = THROW_COOLDOWN;
        }

        // --- Notice flight + landing --------------------------------------------
        let mut landed_events: Vec<(usize, f32, f32)> = Vec::new(); // (house idx, x, y)
        let mut splats: Vec<(f32, f32)> = Vec::new();
        self.notices.retain_mut(|notice| {
            notice.x += notice.vx * dt;
            notice.y += notice.vy * dt;
            if notice.x <= world::PORCH_ZONE_RIGHT {
                if let Some(idx) = self
                    .houses
                    .iter()
                    .position(|h| (notice.y - h.spawn.y).abs() <= world::PORCH_HALF_H)
                {
                    landed_events.push((idx, notice.x, notice.y));
                    return false;
                }
            }
            if notice.x < world::NOTICE_DEAD_X {
                splats.push((notice.x, notice.y));
                return false;
            }
            true
        });
        for (idx, x, y) in landed_events {
            self.resolve_delivery(idx, x, y);
        }
        for (x, y) in splats {
            self.fx.push(Fx {
                kind: FxKind::Splat,
                x,
                y,
                age: 0.0,
                ttl: 0.8,
            });
        }

        // --- Obstacles -----------------------------------------------------------
        for obstacle in &mut self.obstacles {
            if obstacle.kind == ObstacleKind::Dog {
                obstacle.x += obstacle.dir * DOG_SPEED * dt;
                if obstacle.x > 120.0 {
                    obstacle.x = 120.0;
                    obstacle.dir = -1.0;
                } else if obstacle.x < -120.0 {
                    obstacle.x = -120.0;
                    obstacle.dir = 1.0;
                }
            }
        }
        if self.bike.iframe_timer <= 0.0 {
            let (bike_hw, bike_hh) = (14.0, 22.0);
            let bike_x = self.bike.x;
            let bike_y = self.bike.y;
            let hit = self.obstacles.iter().find(|o| {
                let (ohw, ohh) = match o.kind {
                    ObstacleKind::ParkedCar => (30.0, 52.0),
                    ObstacleKind::Dog => (16.0, 11.0),
                };
                (bike_x - o.x).abs() < bike_hw + ohw && (bike_y - o.y).abs() < bike_hh + ohh
            });
            if let Some(obstacle) = hit {
                let what = match obstacle.kind {
                    ObstacleKind::ParkedCar => "a parked hearse",
                    ObstacleKind::Dog => "an unleashed dog",
                };
                self.bike.wobble_timer = WOBBLE_TIME;
                self.bike.iframe_timer = IFRAME_TIME;
                self.push_banner(
                    format!("ROUTE DISRUPTION: collided with {what}. Steady the bike!"),
                    BannerKind::Neutral,
                );
            }
        }

        // --- Missed targets (scrolled past undelivered) ---------------------------
        let passed_y = self.bike.y - 140.0;
        for house in &mut self.houses {
            if house.spawn.is_target
                && !house.delivered
                && !house.missed
                && house.spawn.y + world::PORCH_HALF_H < passed_y
            {
                house.missed = true;
                self.stats.missed += 1;
                self.combo = 0;
                self.banners.push(Banner {
                    text: format!(
                        "STOP MISSED — {} (№{}) lives to see Tuesday.",
                        house.spawn.name, house.spawn.number
                    ),
                    kind: BannerKind::Bad,
                    ttl: 2.5,
                });
            }
        }

        // --- FX aging -------------------------------------------------------------
        for fx in &mut self.fx {
            fx.age += dt;
        }
        self.fx.retain(|f| f.age < f.ttl);

        // --- End of route -----------------------------------------------------------
        if self.bike.y >= length {
            self.total_score += self.stats.score;
            self.completed_levels.push(self.stats);
            self.phase = Phase::Summary;
        }
    }

    fn resolve_delivery(&mut self, house_idx: usize, x: f32, y: f32) {
        let (name, number) = {
            let spawn = &self.houses[house_idx].spawn;
            (spawn.name, spawn.number)
        };
        let house = &mut self.houses[house_idx];
        if house.delivered || house.wrong_hit {
            // Already handled; the notice flutters into the hedge.
            self.fx.push(Fx {
                kind: FxKind::Splat,
                x,
                y,
                age: 0.0,
                ttl: 0.8,
            });
            return;
        }

        if house.spawn.is_target {
            house.delivered = true;
            self.combo += 1;
            self.stats.best_combo = self.stats.best_combo.max(self.combo);
            let bonus = SCORE_COMBO_STEP * (self.combo.saturating_sub(1)) as i64;
            let points = SCORE_CORRECT + bonus;
            self.stats.souls += 1;
            self.stats.score += points;
            self.fx.push(Fx {
                kind: FxKind::Soul,
                x: world::HOUSE_CENTER_X + 40.0,
                y: house.spawn.y,
                age: 0.0,
                ttl: 1.4,
            });
            let combo_note = if self.combo > 1 {
                format!("  (streak x{})", self.combo)
            } else {
                String::new()
            };
            self.push_banner(
                format!("SOUL COLLECTED — {name} (+{points}){combo_note}"),
                BannerKind::Good,
            );
        } else {
            house.wrong_hit = true;
            self.combo = 0;
            self.stats.wrong += 1;
            self.stats.score += SCORE_WRONG;
            self.fx.push(Fx {
                kind: FxKind::OopsNpc,
                x: world::HOUSE_CENTER_X + 60.0,
                y: house.spawn.y - 20.0,
                age: 0.0,
                ttl: 1.8,
            });
            self.push_banner(
                format!(
                    "WRONG ADDRESS! {name} of №{number} was NOT on the list ({SCORE_WRONG}). Filing error logged."
                ),
                BannerKind::Bad,
            );
        }
    }

    fn push_banner(&mut self, text: String, kind: BannerKind) {
        self.banners.push(Banner {
            text,
            kind,
            ttl: 2.5,
        });
        if self.banners.len() > 3 {
            let excess = self.banners.len() - 3;
            self.banners.drain(0..excess);
        }
    }

    /// Overall rank for the final screen, from total souls vs total quota.
    pub fn final_rank(&self) -> &'static str {
        let quota: u32 = self.completed_levels.iter().map(|s| s.quota).sum();
        let souls: u32 = self.completed_levels.iter().map(|s| s.souls).sum();
        let ratio = if quota == 0 {
            0.0
        } else {
            souls as f32 / quota as f32
        };
        if ratio >= 1.0 {
            "PERFORMANCE RANK: EMPLOYEE OF THE EPOCH"
        } else if ratio >= 0.7 {
            "PERFORMANCE RANK: SATISFACTORY REAPING"
        } else if ratio >= 0.4 {
            "PERFORMANCE RANK: NEEDS IMPROVEMENT (ETERNALLY)"
        } else {
            "PERFORMANCE RANK: REASSIGNED TO LIMBO (ADMINISTRATIVE)"
        }
    }

    // ======================================================================
    // Rendering: back-to-front quad list
    // ======================================================================

    pub fn build_quads(&self, out: &mut Vec<Quad>) {
        let cam_y = self.camera_y();
        let view_top = cam_y + 450.0;
        let view_bottom = cam_y - 450.0;

        // Street + sidewalk strips (drawn as camera-height slabs; the world is
        // an infinite straight road so a single tall quad per strip suffices).
        let strip_h = 1000.0;
        out.push(Quad {
            // asphalt
            x: (world::STREET_LEFT + world::STREET_RIGHT) * 0.5,
            y: cam_y,
            w: world::STREET_RIGHT - world::STREET_LEFT,
            h: strip_h,
            color: [0.35, 0.36, 0.40, 1.0],
        });
        out.push(Quad {
            // left sidewalk
            x: (world::SIDEWALK_LEFT + world::STREET_LEFT) * 0.5,
            y: cam_y,
            w: world::STREET_LEFT - world::SIDEWALK_LEFT,
            h: strip_h,
            color: [0.72, 0.70, 0.66, 1.0],
        });
        out.push(Quad {
            // right curb
            x: world::STREET_RIGHT + 10.0,
            y: cam_y,
            w: 20.0,
            h: strip_h,
            color: [0.72, 0.70, 0.66, 1.0],
        });

        // Lane divider dashes.
        let dash_spacing = 90.0;
        let first_dash = (view_bottom / dash_spacing).floor() * dash_spacing;
        let mut dash_y = first_dash;
        while dash_y < view_top {
            for lane_edge in [-47.5, 47.5] {
                out.push(Quad {
                    x: lane_edge,
                    y: dash_y,
                    w: 6.0,
                    h: 34.0,
                    color: [0.92, 0.92, 0.85, 1.0],
                });
            }
            dash_y += dash_spacing;
        }

        // Houses (with driveway, body, roof band, door, windows, porch light).
        for house in &self.houses {
            let hy = house.spawn.y;
            if hy < view_bottom - 120.0 || hy > view_top + 120.0 {
                continue;
            }
            self.push_house_quads(out, house, hy);
        }

        // Finish line checker band at route end.
        let finish_y = self.level().length + 60.0;
        if finish_y > view_bottom && finish_y < view_top {
            let mut x = world::STREET_LEFT;
            let mut dark = false;
            while x < world::STREET_RIGHT {
                out.push(Quad {
                    x: x + 12.5,
                    y: finish_y,
                    w: 25.0,
                    h: 24.0,
                    color: if dark {
                        [0.15, 0.15, 0.18, 1.0]
                    } else {
                        [0.95, 0.95, 0.95, 1.0]
                    },
                });
                dark = !dark;
                x += 25.0;
            }
        }

        // Obstacles.
        for obstacle in &self.obstacles {
            if obstacle.y < view_bottom - 80.0 || obstacle.y > view_top + 80.0 {
                continue;
            }
            match obstacle.kind {
                ObstacleKind::ParkedCar => {
                    // A hearse: long black body, gray windows, somber little wreath.
                    out.push(Quad {
                        x: obstacle.x,
                        y: obstacle.y,
                        w: 56.0,
                        h: 100.0,
                        color: [0.08, 0.08, 0.10, 1.0],
                    });
                    out.push(Quad {
                        x: obstacle.x,
                        y: obstacle.y + 18.0,
                        w: 40.0,
                        h: 30.0,
                        color: [0.55, 0.60, 0.65, 1.0],
                    });
                    out.push(Quad {
                        x: obstacle.x,
                        y: obstacle.y - 34.0,
                        w: 14.0,
                        h: 14.0,
                        color: [0.20, 0.45, 0.22, 1.0],
                    });
                }
                ObstacleKind::Dog => {
                    out.push(Quad {
                        x: obstacle.x,
                        y: obstacle.y,
                        w: 30.0,
                        h: 18.0,
                        color: [0.60, 0.42, 0.24, 1.0],
                    });
                    out.push(Quad {
                        // head leads in the run direction
                        x: obstacle.x + obstacle.dir * 17.0,
                        y: obstacle.y + 4.0,
                        w: 12.0,
                        h: 12.0,
                        color: [0.66, 0.48, 0.28, 1.0],
                    });
                }
            }
        }

        // Notices in flight: fluttering cream paper.
        for notice in &self.notices {
            out.push(Quad {
                x: notice.x,
                y: notice.y,
                w: 14.0,
                h: 10.0,
                color: [0.96, 0.94, 0.82, 1.0],
            });
            out.push(Quad {
                // black wax seal
                x: notice.x + 2.0,
                y: notice.y,
                w: 4.0,
                h: 4.0,
                color: [0.1, 0.1, 0.1, 1.0],
            });
        }

        // The bike + Death.
        self.push_bike_quads(out);

        // FX on top.
        for fx in &self.fx {
            let t = (fx.age / fx.ttl).clamp(0.0, 1.0);
            match fx.kind {
                FxKind::Soul => {
                    let alpha = 1.0 - t;
                    let rise = t * 70.0;
                    out.push(Quad {
                        x: fx.x,
                        y: fx.y + rise,
                        w: 18.0 + t * 10.0,
                        h: 26.0 + t * 14.0,
                        color: [0.92, 0.98, 1.0, alpha * 0.85],
                    });
                    out.push(Quad {
                        x: fx.x,
                        y: fx.y + rise + 6.0,
                        w: 8.0,
                        h: 8.0,
                        color: [0.6, 0.85, 1.0, alpha],
                    });
                }
                FxKind::OopsNpc => {
                    // Resident in pajamas storming toward the street, shaking fists.
                    let stomp = ((fx.age * 18.0).sin() * 3.0).abs();
                    let march = t * 50.0;
                    out.push(Quad {
                        x: fx.x + march,
                        y: fx.y + stomp,
                        w: 16.0,
                        h: 28.0,
                        color: [0.85, 0.45, 0.65, 1.0],
                    });
                    out.push(Quad {
                        x: fx.x + march,
                        y: fx.y + stomp + 18.0,
                        w: 12.0,
                        h: 12.0,
                        color: [0.95, 0.80, 0.68, 1.0],
                    });
                }
                FxKind::Splat => {
                    let alpha = 1.0 - t;
                    out.push(Quad {
                        x: fx.x,
                        y: fx.y,
                        w: 16.0,
                        h: 6.0,
                        color: [0.85, 0.83, 0.72, alpha],
                    });
                }
            }
        }
    }

    fn push_house_quads(&self, out: &mut Vec<Quad>, house: &House, hy: f32) {
        let hx = world::HOUSE_CENTER_X;
        let body = HOUSE_PALETTE[house.spawn.color_index];

        // Driveway/walkway from sidewalk to porch.
        out.push(Quad {
            x: (world::PORCH_ZONE_LEFT + world::SIDEWALK_LEFT) * 0.5,
            y: hy,
            w: world::SIDEWALK_LEFT - world::PORCH_ZONE_LEFT,
            h: 26.0,
            color: [0.62, 0.60, 0.58, 1.0],
        });

        // Lawn pad behind the house strip (slightly darker green than clear color).
        out.push(Quad {
            x: hx,
            y: hy,
            w: world::HOUSE_HALF_W * 2.0 + 60.0,
            h: world::HOUSE_HALF_H * 2.0 + 50.0,
            color: [0.36, 0.62, 0.30, 1.0],
        });

        // House body.
        out.push(Quad {
            x: hx,
            y: hy,
            w: world::HOUSE_HALF_W * 2.0,
            h: world::HOUSE_HALF_H * 2.0,
            color: body,
        });
        // Roof band (top-down: darker rectangle overlapping the street-facing half).
        out.push(Quad {
            x: hx - 20.0,
            y: hy,
            w: world::HOUSE_HALF_W * 2.0 - 40.0,
            h: world::HOUSE_HALF_H * 2.0 - 24.0,
            color: [body[0] * 0.55, body[1] * 0.50, body[2] * 0.50, 1.0],
        });

        // Door facing the street.
        out.push(Quad {
            x: hx + world::HOUSE_HALF_W - 8.0,
            y: hy - 14.0,
            w: 14.0,
            h: 26.0,
            color: [0.42, 0.26, 0.16, 1.0],
        });

        // Windows.
        for wy in [hy + 30.0, hy - 42.0] {
            out.push(Quad {
                x: hx + world::HOUSE_HALF_W - 10.0,
                y: wy,
                w: 12.0,
                h: 16.0,
                color: [0.80, 0.88, 0.95, 1.0],
            });
        }

        // THE TELL — porch light beside the door, facing the street.
        // ON (warm glow) = innocent household. OFF (dead bulb) = marked for death.
        let light_x = hx + world::HOUSE_HALF_W + 8.0;
        let light_y = hy + 8.0;
        if house.spawn.is_target {
            out.push(Quad {
                x: light_x,
                y: light_y,
                w: 9.0,
                h: 9.0,
                color: [0.16, 0.16, 0.20, 1.0],
            });
        } else {
            out.push(Quad {
                // halo
                x: light_x,
                y: light_y,
                w: 22.0,
                h: 22.0,
                color: [1.0, 0.85, 0.35, 0.35],
            });
            out.push(Quad {
                x: light_x,
                y: light_y,
                w: 10.0,
                h: 10.0,
                color: [1.0, 0.92, 0.55, 1.0],
            });
        }

        // Delivery outcome markers.
        if house.delivered {
            // Notice pinned to the door.
            out.push(Quad {
                x: hx + world::HOUSE_HALF_W - 8.0,
                y: hy - 12.0,
                w: 10.0,
                h: 8.0,
                color: [0.96, 0.94, 0.82, 1.0],
            });
        }
        if house.wrong_hit {
            // Angry red porch stripe: a complaint has been filed.
            out.push(Quad {
                x: hx + world::HOUSE_HALF_W + 14.0,
                y: hy - 14.0,
                w: 6.0,
                h: 30.0,
                color: [0.90, 0.20, 0.15, 1.0],
            });
        }
    }

    fn push_bike_quads(&self, out: &mut Vec<Quad>) {
        let wobble = if self.bike.wobble_timer > 0.0 {
            (self.bike.anim_t * 40.0).sin() * 5.0
        } else {
            0.0
        };
        // Blink while invulnerable (post-collision), skip every other flicker window.
        if self.bike.iframe_timer > 0.0 && ((self.bike.anim_t * 12.0) as i32) % 2 == 0 {
            return;
        }
        let bx = self.bike.x + wobble;
        let by = self.bike.y;

        // Shadow.
        out.push(Quad {
            x: bx + 3.0,
            y: by - 4.0,
            w: 30.0,
            h: 46.0,
            color: [0.0, 0.0, 0.0, 0.22],
        });
        // Wheels.
        for wy in [by - 18.0, by + 18.0] {
            out.push(Quad {
                x: bx,
                y: wy,
                w: 10.0,
                h: 16.0,
                color: [0.10, 0.10, 0.12, 1.0],
            });
        }
        // Frame.
        out.push(Quad {
            x: bx,
            y: by,
            w: 8.0,
            h: 34.0,
            color: [0.75, 0.15, 0.15, 1.0],
        });
        // Death's robe.
        out.push(Quad {
            x: bx,
            y: by + 2.0,
            w: 22.0,
            h: 26.0,
            color: [0.12, 0.10, 0.16, 1.0],
        });
        // Scythe slung across the back (angled illusion via two offset segments).
        out.push(Quad {
            x: bx - 14.0,
            y: by + 10.0,
            w: 26.0,
            h: 4.0,
            color: [0.45, 0.35, 0.25, 1.0],
        });
        out.push(Quad {
            x: bx - 26.0,
            y: by + 14.0,
            w: 8.0,
            h: 12.0,
            color: [0.80, 0.84, 0.88, 1.0],
        });
        // Skull.
        out.push(Quad {
            x: bx,
            y: by + 18.0,
            w: 14.0,
            h: 13.0,
            color: [0.94, 0.94, 0.90, 1.0],
        });
        // Eye sockets, forever unimpressed.
        for ex in [bx - 3.0, bx + 3.0] {
            out.push(Quad {
                x: ex,
                y: by + 19.0,
                w: 3.0,
                h: 4.0,
                color: [0.05, 0.05, 0.08, 1.0],
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn advance(game: &mut Game, steps: usize, input: GameInput) {
        let dt = 1.0 / 60.0;
        for _ in 0..steps {
            game.update_fixed(dt, input);
        }
    }

    fn start_riding(game: &mut Game) {
        game.update_fixed(
            1.0 / 60.0,
            GameInput {
                advance_pressed: true,
                ..Default::default()
            },
        );
        assert_eq!(game.phase, Phase::Riding);
    }

    #[test]
    fn correct_delivery_scores_and_counts_soul() {
        let mut game = Game::new();
        start_riding(&mut game);
        let target_idx = game
            .houses
            .iter()
            .position(|h| h.spawn.is_target)
            .expect("level 1 must contain a target");
        game.resolve_delivery(target_idx, -200.0, 0.0);
        assert_eq!(game.stats.souls, 1);
        assert_eq!(game.stats.score, 100);
        assert_eq!(game.combo, 1);
    }

    #[test]
    fn wrong_delivery_penalizes_and_resets_combo() {
        let mut game = Game::new();
        start_riding(&mut game);
        let target_idx = game.houses.iter().position(|h| h.spawn.is_target).unwrap();
        let wrong_idx = game.houses.iter().position(|h| !h.spawn.is_target).unwrap();
        game.resolve_delivery(target_idx, -200.0, 0.0);
        game.resolve_delivery(wrong_idx, -200.0, 0.0);
        assert_eq!(game.stats.wrong, 1);
        assert_eq!(game.stats.score, 100 - 50);
        assert_eq!(game.combo, 0);
    }

    #[test]
    fn combo_streak_adds_bonus() {
        let mut game = Game::new();
        start_riding(&mut game);
        let targets: Vec<usize> = game
            .houses
            .iter()
            .enumerate()
            .filter(|(_, h)| h.spawn.is_target)
            .map(|(i, _)| i)
            .collect();
        assert!(targets.len() >= 2);
        game.resolve_delivery(targets[0], -200.0, 0.0);
        game.resolve_delivery(targets[1], -200.0, 0.0);
        // 100 + (100 + 25 combo bonus)
        assert_eq!(game.stats.score, 225);
        assert_eq!(game.stats.best_combo, 2);
    }

    #[test]
    fn duplicate_delivery_has_no_effect() {
        let mut game = Game::new();
        start_riding(&mut game);
        let target_idx = game.houses.iter().position(|h| h.spawn.is_target).unwrap();
        game.resolve_delivery(target_idx, -200.0, 0.0);
        game.resolve_delivery(target_idx, -200.0, 0.0);
        assert_eq!(game.stats.souls, 1);
        assert_eq!(game.stats.score, 100);
    }

    #[test]
    fn simulation_is_deterministic() {
        let script = |game: &mut Game| {
            start_riding(game);
            // Ride, weave, and throw on a fixed schedule for ~20 seconds.
            for step in 0..1200usize {
                let input = GameInput {
                    lane_left_pressed: step % 240 == 30,
                    lane_right_pressed: step % 240 == 150,
                    throw_pressed: step % 45 == 0,
                    ..Default::default()
                };
                game.update_fixed(1.0 / 60.0, input);
            }
        };

        let mut a = Game::new();
        let mut b = Game::new();
        script(&mut a);
        script(&mut b);

        assert_eq!(a.bike.y, b.bike.y);
        assert_eq!(a.bike.x, b.bike.x);
        assert_eq!(a.stats.souls, b.stats.souls);
        assert_eq!(a.stats.wrong, b.stats.wrong);
        assert_eq!(a.stats.score, b.stats.score);
        assert_eq!(a.notices.len(), b.notices.len());
    }

    #[test]
    fn level_completes_into_summary_with_no_hard_fail() {
        let mut game = Game::new();
        start_riding(&mut game);
        // Ride the whole route without throwing anything: quota missed,
        // but the level must still complete (design decision: no hard fail).
        let steps = (game.level().length / game.level().scroll_speed * 60.0) as usize + 120;
        advance(&mut game, steps, GameInput::default());
        assert_eq!(game.phase, Phase::Summary);
        assert_eq!(game.stats.souls, 0);
        assert_eq!(game.completed_levels.len(), 1);
    }

    #[test]
    fn full_run_reaches_final_screen() {
        let mut game = Game::new();
        for _ in 0..3 {
            // Intro -> Riding
            game.update_fixed(
                1.0 / 60.0,
                GameInput {
                    advance_pressed: true,
                    ..Default::default()
                },
            );
            let steps = (game.level().length / game.level().scroll_speed * 60.0) as usize + 120;
            advance(&mut game, steps, GameInput::default());
            assert_eq!(game.phase, Phase::Summary);
            // Summary -> next Intro (or Final after last level)
            game.update_fixed(
                1.0 / 60.0,
                GameInput {
                    advance_pressed: true,
                    ..Default::default()
                },
            );
        }
        assert_eq!(game.phase, Phase::Final);
        assert_eq!(game.completed_levels.len(), 3);
    }
}
