//! Level definitions and deterministic layout generation for Grim Delivery.
//!
//! All randomness happens here, at level-start, through a seeded LCG. The
//! fixed-step simulation itself never consumes random numbers, so a level
//! plays out identically on every run — the same determinism contract the
//! engine's own simulation follows.

/// World-space layout constants shared by simulation and rendering.
pub mod world {
    /// Lane center x positions, left to right.
    pub const LANES: [f32; 3] = [-95.0, 0.0, 95.0];
    /// Street asphalt spans this x range.
    pub const STREET_LEFT: f32 = -145.0;
    pub const STREET_RIGHT: f32 = 145.0;
    /// Sidewalk strip between street and lawns (left side, house side).
    pub const SIDEWALK_LEFT: f32 = -185.0;
    /// Delivery strip: a notice landing in this x band checks house porches.
    pub const PORCH_ZONE_RIGHT: f32 = -185.0;
    pub const PORCH_ZONE_LEFT: f32 = -235.0;
    /// House body center x and half-extents.
    pub const HOUSE_CENTER_X: f32 = -310.0;
    pub const HOUSE_HALF_W: f32 = 75.0;
    pub const HOUSE_HALF_H: f32 = 70.0;
    /// A notice past this x with no porch hit is a lawn splat.
    pub const NOTICE_DEAD_X: f32 = -245.0;
    /// Half the y-extent of a house's deliverable porch zone.
    pub const PORCH_HALF_H: f32 = 80.0;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstacleKind {
    /// Static car parked with its nose poking into the right lane.
    ParkedCar,
    /// Dog trotting back and forth across the street.
    Dog,
}

#[derive(Debug, Clone)]
pub struct ObstacleSpawn {
    pub kind: ObstacleKind,
    /// Street position along the route.
    pub y: f32,
    /// ParkedCar: fixed x. Dog: starting x of its ping-pong path.
    pub x: f32,
    /// Dog only: initial horizontal direction (+1 / -1).
    pub dir: f32,
}

#[derive(Debug, Clone)]
pub struct HouseSpawn {
    /// Center y along the route.
    pub y: f32,
    /// True if this address is legitimately marked for death.
    pub is_target: bool,
    /// Resident name, used in HUD flavor text.
    pub name: &'static str,
    /// Street number for corporate-clipboard flavor.
    pub number: u32,
    /// Index into a small palette of pastel house colors.
    pub color_index: usize,
}

#[derive(Debug, Clone)]
pub struct LevelDef {
    pub route_label: &'static str,
    /// Auto-scroll speed in world units/sec.
    pub scroll_speed: f32,
    /// Route length; the level ends when the bike passes this y.
    pub length: f32,
    pub house_spacing: f32,
    pub house_count: usize,
    /// How many of the houses are legitimate targets.
    pub target_count: usize,
    pub obstacles: Vec<ObstacleSpawn>,
    pub layout_seed: u64,
    /// One line from Death's manager on the summary screen.
    pub manager_line_good: &'static str,
    pub manager_line_bad: &'static str,
}

/// Names cycled onto houses. Targets read like they belong on a clipboard;
/// the pool is shared so wrong-address victims sound just as mundane.
const NAMES: [&str; 16] = [
    "H. Pembleton",
    "G. Fusco",
    "M. Okafor",
    "D. Brzezinski",
    "P. Whitlow",
    "R. Calloway",
    "S. Ito",
    "T. Vandermeer",
    "L. Grubbs",
    "A. Duquesne",
    "N. Papadakis",
    "C. Mumford",
    "E. Szabo",
    "B. Ferretti",
    "K. Ollenberger",
    "J. Trask",
];

pub const HOUSE_PALETTE: [[f32; 4]; 5] = [
    [0.93, 0.80, 0.72, 1.0], // peach
    [0.76, 0.86, 0.93, 1.0], // powder blue
    [0.88, 0.91, 0.76, 1.0], // pale lime
    [0.93, 0.86, 0.70, 1.0], // cream
    [0.86, 0.78, 0.90, 1.0], // lilac
];

/// Minimal deterministic LCG (Numerical Recipes constants). Layout-time only.
pub struct Lcg(u64);

impl Lcg {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub fn next_u32(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as u32
    }

    /// Uniform in [0, n).
    pub fn next_below(&mut self, n: u32) -> u32 {
        self.next_u32() % n.max(1)
    }
}

pub fn generate_houses(def: &LevelDef) -> Vec<HouseSpawn> {
    let mut rng = Lcg::new(def.layout_seed);

    // Choose which house slots are targets: shuffle indices, take target_count.
    let mut indices: Vec<usize> = (0..def.house_count).collect();
    for i in (1..indices.len()).rev() {
        let j = rng.next_below((i + 1) as u32) as usize;
        indices.swap(i, j);
    }
    let target_slots: Vec<usize> = indices.into_iter().take(def.target_count).collect();

    let first_y = 350.0; // breathing room before the first stop
    (0..def.house_count)
        .map(|i| {
            let jitter = rng.next_below(40) as f32 - 20.0;
            HouseSpawn {
                y: first_y + i as f32 * def.house_spacing + jitter,
                is_target: target_slots.contains(&i),
                name: NAMES[(rng.next_below(NAMES.len() as u32)) as usize],
                number: 100 + (i as u32) * 2 + rng.next_below(2),
                color_index: rng.next_below(HOUSE_PALETTE.len() as u32) as usize,
            }
        })
        .collect()
}

pub fn levels() -> Vec<LevelDef> {
    vec![
        LevelDef {
            route_label: "ROUTE 12B — MAPLE CIRCLE",
            scroll_speed: 130.0,
            length: 2600.0,
            house_spacing: 300.0,
            house_count: 7,
            target_count: 4,
            obstacles: vec![
                ObstacleSpawn {
                    kind: ObstacleKind::ParkedCar,
                    y: 700.0,
                    x: 112.0,
                    dir: 0.0,
                },
                ObstacleSpawn {
                    kind: ObstacleKind::Dog,
                    y: 1500.0,
                    x: -120.0,
                    dir: 1.0,
                },
            ],
            layout_seed: 0x0DD5_0121,
            manager_line_good: "\"Adequate. The paperwork practically files itself.\" — Mgmt.",
            manager_line_bad:
                "\"Four unresolved souls is four incident reports. On MY desk.\" — Mgmt.",
        },
        LevelDef {
            route_label: "ROUTE 13A — ELM MEADOWS",
            scroll_speed: 155.0,
            length: 3000.0,
            house_spacing: 250.0,
            house_count: 10,
            target_count: 6,
            obstacles: vec![
                ObstacleSpawn {
                    kind: ObstacleKind::ParkedCar,
                    y: 550.0,
                    x: 112.0,
                    dir: 0.0,
                },
                ObstacleSpawn {
                    kind: ObstacleKind::Dog,
                    y: 1200.0,
                    x: 120.0,
                    dir: -1.0,
                },
                ObstacleSpawn {
                    kind: ObstacleKind::ParkedCar,
                    y: 1900.0,
                    x: -112.0,
                    dir: 0.0,
                },
                ObstacleSpawn {
                    kind: ObstacleKind::Dog,
                    y: 2500.0,
                    x: -120.0,
                    dir: 1.0,
                },
            ],
            layout_seed: 0x0DD5_0132,
            manager_line_good:
                "\"Souls balanced. Ledger closed. Do not expect praise again.\" — Mgmt.",
            manager_line_bad: "\"The Elm Meadows HOA has opened a ticket about you.\" — Mgmt.",
        },
        LevelDef {
            route_label: "ROUTE 14C — TERMINAL HEIGHTS",
            scroll_speed: 180.0,
            length: 3400.0,
            house_spacing: 215.0,
            house_count: 13,
            target_count: 8,
            obstacles: vec![
                ObstacleSpawn {
                    kind: ObstacleKind::Dog,
                    y: 500.0,
                    x: -120.0,
                    dir: 1.0,
                },
                ObstacleSpawn {
                    kind: ObstacleKind::ParkedCar,
                    y: 900.0,
                    x: 112.0,
                    dir: 0.0,
                },
                ObstacleSpawn {
                    kind: ObstacleKind::Dog,
                    y: 1400.0,
                    x: 120.0,
                    dir: -1.0,
                },
                ObstacleSpawn {
                    kind: ObstacleKind::ParkedCar,
                    y: 1800.0,
                    x: -112.0,
                    dir: 0.0,
                },
                ObstacleSpawn {
                    kind: ObstacleKind::Dog,
                    y: 2300.0,
                    x: -120.0,
                    dir: 1.0,
                },
                ObstacleSpawn {
                    kind: ObstacleKind::ParkedCar,
                    y: 2800.0,
                    x: 112.0,
                    dir: 0.0,
                },
            ],
            layout_seed: 0x0DD5_0143,
            manager_line_good:
                "\"Route complete. Your bicycle remains a budgetary embarrassment.\" — Mgmt.",
            manager_line_bad:
                "\"Corporate is asking why the living outnumber the invoiced.\" — Mgmt.",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn house_generation_is_deterministic() {
        let defs = levels();
        for def in &defs {
            let a = generate_houses(def);
            let b = generate_houses(def);
            assert_eq!(a.len(), b.len());
            for (ha, hb) in a.iter().zip(b.iter()) {
                assert_eq!(ha.y, hb.y);
                assert_eq!(ha.is_target, hb.is_target);
                assert_eq!(ha.name, hb.name);
            }
        }
    }

    #[test]
    fn target_count_matches_definition() {
        for def in &levels() {
            let houses = generate_houses(def);
            let targets = houses.iter().filter(|h| h.is_target).count();
            assert_eq!(targets, def.target_count, "level {}", def.route_label);
        }
    }

    #[test]
    fn houses_fit_within_route_length() {
        for def in &levels() {
            let houses = generate_houses(def);
            let last = houses.last().unwrap();
            assert!(
                last.y + world::HOUSE_HALF_H < def.length,
                "level {}: last house at {} exceeds length {}",
                def.route_label,
                last.y,
                def.length
            );
        }
    }
}
