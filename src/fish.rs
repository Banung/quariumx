use rand::Rng;

pub use crate::creatures::{
    AnimatedSprite, CreatureKind, DirectionalSprite, CRAB_SPECIES, JELLYFISH_SPECIES,
    LARGE_FISH_SPECIES, MEDIUM_FISH_SPECIES,
};
pub use crate::scenery::{Coral, CoralSprite, Scenery, Seaweed, CORAL_SPECIES};

const MAX_FISH_COUNT: usize = 200;
const AUTO_SCALE_CELLS_PER_FISH: usize = 220;
const AUTO_SCALE_MIN_SMALL_FISH: usize = 4;
const AUTO_SCALE_MAX_SMALL_FISH: usize = 18;

const MIN_H_SPEED: f32 = 4.0;
const MAX_H_SPEED: f32 = 12.0;
const MIN_V_SPEED: f32 = 1.2;
const MAX_V_SPEED: f32 = 3.8;

const MEDIUM_MIN_H_SPEED: f32 = 3.0;
const MEDIUM_MAX_H_SPEED: f32 = 8.0;
const MEDIUM_MIN_V_SPEED: f32 = 0.8;
const MEDIUM_MAX_V_SPEED: f32 = 2.4;

const LARGE_MIN_H_SPEED: f32 = 2.2;
const LARGE_MAX_H_SPEED: f32 = 5.5;
const LARGE_MIN_V_SPEED: f32 = 0.4;
const LARGE_MAX_V_SPEED: f32 = 1.4;

const JELLY_MIN_H_SPEED: f32 = 0.5;
const JELLY_MAX_H_SPEED: f32 = 1.6;
const JELLY_MIN_V_SPEED: f32 = 0.8;
const JELLY_MAX_V_SPEED: f32 = 2.2;
const JELLY_ANIM_INTERVAL: f32 = 0.45;

const CRAB_MIN_H_SPEED: f32 = 2.5;
const CRAB_MAX_H_SPEED: f32 = 6.0;
const CRAB_ANIM_INTERVAL: f32 = 0.25;
const CRAB_FLOOR_ROWS: u16 = 2;

const MIN_INITIAL_TURN_COOLDOWN: f32 = 0.4;
const MAX_INITIAL_TURN_COOLDOWN: f32 = 1.2;
const MIN_TURN_COOLDOWN: f32 = 0.8;
const MAX_TURN_COOLDOWN: f32 = 1.6;

const H_TURN_START: f32 = 0.68;
const H_EDGE_TURN_STRENGTH: f32 = 1.5;
const BASE_V_TURN_RATE: f32 = 0.04;
const EDGE_V_TURN_RATE: f32 = 1.4;

const LARGE_FISH_COLORS: &[FishColor] = &[
    FishColor::Cyan,
    FishColor::BrightCyan,
    FishColor::Blue,
    FishColor::White,
];

const JELLYFISH_COLORS: &[FishColor] = &[
    FishColor::BrightMagenta,
    FishColor::Magenta,
    FishColor::BrightCyan,
    FishColor::Cyan,
    FishColor::White,
];

const CRAB_COLORS: &[FishColor] = &[
    FishColor::Red,
    FishColor::BrightRed,
    FishColor::Yellow,
    FishColor::BrightYellow,
];

/// Horizontal swimming direction of a fish.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
}

impl Direction {
    fn flip(self) -> Self {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

/// Vertical swimming direction of a fish.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalDirection {
    Up,
    Down,
}

impl VerticalDirection {
    fn flip(self) -> Self {
        match self {
            VerticalDirection::Up => VerticalDirection::Down,
            VerticalDirection::Down => VerticalDirection::Up,
        }
    }
}

/// Terminal color palette for aquarium creatures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FishColor {
    Cyan,
    Yellow,
    Magenta,
    Green,
    Blue,
    Red,
    White,
    BrightCyan,
    BrightYellow,
    BrightMagenta,
    BrightGreen,
    BrightRed,
}

impl FishColor {
    pub const ALL: &'static [FishColor] = &[
        FishColor::Cyan,
        FishColor::Yellow,
        FishColor::Magenta,
        FishColor::Green,
        FishColor::Blue,
        FishColor::Red,
        FishColor::White,
        FishColor::BrightCyan,
        FishColor::BrightYellow,
        FishColor::BrightMagenta,
        FishColor::BrightGreen,
        FishColor::BrightRed,
    ];
}

/// Directional pair of single-line ASCII sprites for a fish type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FishSprite {
    pub right: &'static str,
    pub left: &'static str,
}

/// Catalog of small single-line ASCII fish sprites.
pub const SMALL_FISH_SPRITES: &[FishSprite] = &[
    FishSprite {
        right: "><>",
        left: "<><",
    },
    FishSprite {
        right: "><'>",
        left: "<'><",
    },
    FishSprite {
        right: ">o>",
        left: "<o<",
    },
    FishSprite {
        right: "><))>",
        left: "<((><",
    },
    FishSprite {
        right: ">=:>",
        left: "<:=<",
    },
    FishSprite {
        right: ">))'>",
        left: "<'((<",
    },
    FishSprite {
        right: ">=>",
        left: "<=<",
    },
    FishSprite {
        right: "><(('>",
        left: "<'))><",
    },
];

fn resolve_fish_sprite(sprite: &'static str) -> FishSprite {
    SMALL_FISH_SPRITES
        .iter()
        .copied()
        .find(|entry| entry.right == sprite || entry.left == sprite)
        .unwrap_or(FishSprite {
            right: sprite,
            left: sprite,
        })
}

fn h_speed_range(kind: CreatureKind) -> std::ops::RangeInclusive<f32> {
    match kind {
        CreatureKind::SmallFish => MIN_H_SPEED..=MAX_H_SPEED,
        CreatureKind::MediumFish => MEDIUM_MIN_H_SPEED..=MEDIUM_MAX_H_SPEED,
        CreatureKind::LargeFish => LARGE_MIN_H_SPEED..=LARGE_MAX_H_SPEED,
        CreatureKind::Jellyfish => JELLY_MIN_H_SPEED..=JELLY_MAX_H_SPEED,
        CreatureKind::Crab => CRAB_MIN_H_SPEED..=CRAB_MAX_H_SPEED,
    }
}

fn v_speed_range(kind: CreatureKind) -> Option<std::ops::RangeInclusive<f32>> {
    match kind {
        CreatureKind::SmallFish => Some(MIN_V_SPEED..=MAX_V_SPEED),
        CreatureKind::MediumFish => Some(MEDIUM_MIN_V_SPEED..=MEDIUM_MAX_V_SPEED),
        CreatureKind::LargeFish => Some(LARGE_MIN_V_SPEED..=LARGE_MAX_V_SPEED),
        CreatureKind::Jellyfish => Some(JELLY_MIN_V_SPEED..=JELLY_MAX_V_SPEED),
        CreatureKind::Crab => None,
    }
}

/// Compute frame-rate-independent edge-weighted turn probability for a step `dt`.
///
/// `progress` is the normalized proximity (`0.0..=1.0`) toward the edge the fish is
/// currently approaching (`0.0` at the far edge, `1.0` at the target edge).
pub fn edge_weighted_turn_prob(progress: f32, dt: f32, base_rate: f32, edge_rate: f32) -> f32 {
    if !dt.is_finite() || dt <= 0.0 || progress.is_nan() {
        return 0.0;
    }
    let p = progress.clamp(0.0, 1.0);
    let rate = (base_rate + edge_rate * p.powi(4)).max(0.0);
    (1.0 - (-rate * dt).exp()).clamp(0.0, 1.0)
}

/// Compute width-, speed-, and frame-rate-invariant horizontal edge-zone turn probability.
///
/// `progress` is the normalized proximity (`0.0..=1.0`) toward the horizontal edge the fish
/// is currently approaching (`0.0` at the far wall, `1.0` at the target wall).
/// Returns `0.0` when `progress <= H_TURN_START` (`0.68`) so fish swim freely across the
/// middle without spontaneous center turning.
pub fn horizontal_edge_turn_prob(progress: f32, step_distance: f32, max_x: f32) -> f32 {
    if !step_distance.is_finite()
        || step_distance <= 0.0
        || !max_x.is_finite()
        || max_x <= 0.0
        || progress.is_nan()
    {
        return 0.0;
    }
    let p = progress.clamp(0.0, 1.0);
    if p <= H_TURN_START {
        return 0.0;
    }
    let edge_span = 1.0 - H_TURN_START;
    let u = (p - H_TURN_START) / edge_span;
    let du = step_distance / (max_x * edge_span);
    (1.0 - (-(H_EDGE_TURN_STRENGTH * u * u) * du).exp()).clamp(0.0, 1.0)
}

#[derive(Debug, Clone)]
enum CreatureArt {
    Single {
        right: [&'static str; 1],
        left: [&'static str; 1],
    },
    Directional(DirectionalSprite),
    Animated {
        sprite: AnimatedSprite,
        frame_idx: usize,
        timer: f32,
        interval: f32,
    },
}

/// A single swimming or scuttling marine creature inside the aquarium.
#[derive(Debug, Clone)]
pub struct Fish {
    kind: CreatureKind,
    x: f32,
    y: f32,
    direction: Direction,
    speed: f32,
    v_direction: VerticalDirection,
    v_speed: f32,
    h_turn_cooldown: f32,
    v_turn_cooldown: f32,
    color: FishColor,
    art: CreatureArt,
}

impl Fish {
    /// Construct a small fish from explicit horizontal state with zero vertical speed by default.
    pub fn from_parts(
        x: f32,
        y: f32,
        direction: Direction,
        speed: f32,
        color: FishColor,
        sprite: &'static str,
    ) -> Self {
        Self::with_motion(
            x,
            y,
            direction,
            speed,
            VerticalDirection::Down,
            0.0,
            0.0,
            0.0,
            color,
            sprite,
        )
    }

    /// Alias constructor for deterministic testing.
    pub fn new_for_test(
        x: f32,
        y: f32,
        direction: Direction,
        color: FishColor,
        sprite: &'static str,
    ) -> Self {
        Self::from_parts(x, y, direction, 8.0, color, sprite)
    }

    /// Construct a small fish with explicit 2D motion and turn cooldown parameters.
    #[allow(clippy::too_many_arguments)]
    pub fn with_motion(
        x: f32,
        y: f32,
        direction: Direction,
        speed: f32,
        v_direction: VerticalDirection,
        v_speed: f32,
        h_turn_cooldown: f32,
        v_turn_cooldown: f32,
        color: FishColor,
        sprite: &'static str,
    ) -> Self {
        let resolved = resolve_fish_sprite(sprite);
        Self {
            kind: CreatureKind::SmallFish,
            x,
            y,
            direction,
            speed: speed.abs(),
            v_direction,
            v_speed: v_speed.abs(),
            h_turn_cooldown: h_turn_cooldown.max(0.0),
            v_turn_cooldown: v_turn_cooldown.max(0.0),
            color,
            art: CreatureArt::Single {
                right: [resolved.right],
                left: [resolved.left],
            },
        }
    }

    /// Compute frame-rate-independent edge-weighted turn probability for a step `dt`.
    pub fn edge_weighted_turn_prob(progress: f32, dt: f32, base_rate: f32, edge_rate: f32) -> f32 {
        edge_weighted_turn_prob(progress, dt, base_rate, edge_rate)
    }

    /// Compute width-, speed-, and frame-rate-invariant horizontal edge-zone turn probability.
    pub fn horizontal_edge_turn_prob(progress: f32, step_distance: f32, max_x: f32) -> f32 {
        horizontal_edge_turn_prob(progress, step_distance, max_x)
    }

    /// Spawn a small fish directly inside the visible terminal bounds.
    pub fn spawn_initial<R: Rng + ?Sized>(
        screen_width: u16,
        screen_height: u16,
        rng: &mut R,
    ) -> Self {
        Self::spawn_kind(CreatureKind::SmallFish, screen_width, screen_height, rng)
    }

    /// Spawn a specific marine creature kind directly inside the visible terminal bounds.
    pub fn spawn_kind<R: Rng + ?Sized>(
        kind: CreatureKind,
        screen_width: u16,
        screen_height: u16,
        rng: &mut R,
    ) -> Self {
        let direction = if rng.gen_bool(0.5) {
            Direction::Right
        } else {
            Direction::Left
        };
        let v_direction = if rng.gen_bool(0.5) {
            VerticalDirection::Down
        } else {
            VerticalDirection::Up
        };

        let (art, sprite_width, sprite_height, color) = match kind {
            CreatureKind::SmallFish => {
                let s = SMALL_FISH_SPRITES[rng.gen_range(0..SMALL_FISH_SPRITES.len())];
                let c = FishColor::ALL[rng.gen_range(0..FishColor::ALL.len())];
                (
                    CreatureArt::Single {
                        right: [s.right],
                        left: [s.left],
                    },
                    s.right.len(),
                    1usize,
                    c,
                )
            }
            CreatureKind::MediumFish => {
                let s = MEDIUM_FISH_SPECIES[rng.gen_range(0..MEDIUM_FISH_SPECIES.len())];
                let c = FishColor::ALL[rng.gen_range(0..FishColor::ALL.len())];
                (
                    CreatureArt::Directional(s),
                    s.right.first().map_or(0, |l| l.len()),
                    s.right.len(),
                    c,
                )
            }
            CreatureKind::LargeFish => {
                let s = LARGE_FISH_SPECIES[rng.gen_range(0..LARGE_FISH_SPECIES.len())];
                let c = LARGE_FISH_COLORS[rng.gen_range(0..LARGE_FISH_COLORS.len())];
                (
                    CreatureArt::Directional(s),
                    s.right.first().map_or(0, |l| l.len()),
                    s.right.len(),
                    c,
                )
            }
            CreatureKind::Jellyfish => {
                let s = JELLYFISH_SPECIES[rng.gen_range(0..JELLYFISH_SPECIES.len())];
                let frame_idx = rng.gen_range(0..s.frames.len());
                let c = JELLYFISH_COLORS[rng.gen_range(0..JELLYFISH_COLORS.len())];
                let frame0 = s.frames[0];
                (
                    CreatureArt::Animated {
                        sprite: s,
                        frame_idx,
                        timer: 0.0,
                        interval: JELLY_ANIM_INTERVAL,
                    },
                    frame0.first().map_or(0, |l| l.len()),
                    frame0.len(),
                    c,
                )
            }
            CreatureKind::Crab => {
                let s = CRAB_SPECIES[rng.gen_range(0..CRAB_SPECIES.len())];
                let frame_idx = rng.gen_range(0..s.frames.len());
                let c = CRAB_COLORS[rng.gen_range(0..CRAB_COLORS.len())];
                let frame0 = s.frames[0];
                (
                    CreatureArt::Animated {
                        sprite: s,
                        frame_idx,
                        timer: 0.0,
                        interval: CRAB_ANIM_INTERVAL,
                    },
                    frame0.first().map_or(0, |l| l.len()),
                    frame0.len(),
                    c,
                )
            }
        };

        let speed = rng.gen_range(h_speed_range(kind));
        let v_speed = v_speed_range(kind).map_or(0.0, |range| rng.gen_range(range));
        let h_turn_cooldown =
            rng.gen_range(MIN_INITIAL_TURN_COOLDOWN..=MAX_INITIAL_TURN_COOLDOWN);
        let v_turn_cooldown =
            rng.gen_range(MIN_INITIAL_TURN_COOLDOWN..=MAX_INITIAL_TURN_COOLDOWN);

        let max_x = (screen_width as f32 - sprite_width as f32).max(0.0);
        let x = if max_x <= 0.0 {
            0.0
        } else {
            rng.gen_range(0.0..=max_x)
        };

        let max_y = Self::max_y_for_kind(kind, sprite_height, screen_height);
        let y = if kind == CreatureKind::Crab {
            max_y
        } else if max_y <= 0.0 {
            0.0
        } else {
            rng.gen_range(0.0..=max_y)
        };

        Self {
            kind,
            x,
            y,
            direction,
            speed,
            v_direction,
            v_speed,
            h_turn_cooldown,
            v_turn_cooldown,
            color,
            art,
        }
    }

    fn max_y_for_kind(kind: CreatureKind, sprite_height: usize, screen_height: u16) -> f32 {
        let h = sprite_height as u16;
        match kind {
            CreatureKind::SmallFish | CreatureKind::Crab => {
                (screen_height as f32 - h as f32).max(0.0)
            }
            CreatureKind::MediumFish | CreatureKind::LargeFish | CreatureKind::Jellyfish => {
                if screen_height >= h + CRAB_FLOOR_ROWS + 2 {
                    (screen_height - h - CRAB_FLOOR_ROWS) as f32
                } else {
                    (screen_height as f32 - h as f32).max(0.0)
                }
            }
        }
    }

    /// Species category of this marine creature.
    pub fn kind(&self) -> CreatureKind {
        self.kind
    }

    /// Current horizontal position in terminal cell coordinates.
    pub fn x(&self) -> f32 {
        self.x
    }

    /// Current vertical position in terminal row coordinates (`0.0..=height - self.height()`).
    pub fn y(&self) -> f32 {
        self.y
    }

    /// Current horizontal swimming direction.
    pub fn direction(&self) -> Direction {
        self.direction
    }

    /// Current vertical swimming direction.
    pub fn vertical_direction(&self) -> VerticalDirection {
        self.v_direction
    }

    /// Assigned display color for this creature.
    pub fn color(&self) -> FishColor {
        self.color
    }

    /// Multi-line ASCII sprite slice for the creature's current direction or animation frame.
    pub fn sprite_lines(&self) -> &[&'static str] {
        match &self.art {
            CreatureArt::Single { right, left } => match self.direction {
                Direction::Right => right.as_slice(),
                Direction::Left => left.as_slice(),
            },
            CreatureArt::Directional(sprite) => match self.direction {
                Direction::Right => sprite.right,
                Direction::Left => sprite.left,
            },
            CreatureArt::Animated {
                sprite, frame_idx, ..
            } => sprite.frames[*frame_idx % sprite.frames.len()],
        }
    }

    /// Single-line (first row) ASCII sprite string matching the creature's current state.
    pub fn sprite(&self) -> &'static str {
        self.sprite_lines().first().copied().unwrap_or("")
    }

    /// Height of the creature's ASCII sprite in terminal rows.
    pub fn height(&self) -> usize {
        self.sprite_lines().len()
    }

    /// Width of the creature's ASCII sprite in terminal columns.
    pub fn width(&self) -> usize {
        self.sprite_lines().first().map_or(0, |line| line.len())
    }

    fn advance<R: Rng + ?Sized>(
        &mut self,
        dt: f32,
        speed_scale: f32,
        screen_width: u16,
        screen_height: u16,
        rng: &mut R,
    ) {
        let dt_eff = dt * speed_scale;
        if !dt_eff.is_finite() || dt_eff <= 0.0 {
            return;
        }

        if let CreatureArt::Animated {
            sprite,
            frame_idx,
            timer,
            interval,
        } = &mut self.art
        {
            if *interval > 0.0 && !sprite.frames.is_empty() {
                *timer += dt_eff;
                if *timer >= *interval {
                    let steps = (*timer / *interval) as usize;
                    *timer %= *interval;
                    *frame_idx = (*frame_idx + steps) % sprite.frames.len();
                }
            }
        }

        let max_x = (screen_width as f32 - self.width() as f32).max(0.0);
        self.h_turn_cooldown = (self.h_turn_cooldown - dt_eff).max(0.0);

        if self.speed > 0.0 {
            let vx = match self.direction {
                Direction::Right => self.speed,
                Direction::Left => -self.speed,
            };
            self.x += vx * dt_eff;

            if self.direction == Direction::Right && self.x >= max_x {
                self.x = max_x;
                self.direction = Direction::Left;
                self.h_turn_cooldown = rng.gen_range(MIN_TURN_COOLDOWN..=MAX_TURN_COOLDOWN);
                self.speed = rng.gen_range(h_speed_range(self.kind));
            } else if self.direction == Direction::Left && self.x <= 0.0 {
                self.x = 0.0;
                self.direction = Direction::Right;
                self.h_turn_cooldown = rng.gen_range(MIN_TURN_COOLDOWN..=MAX_TURN_COOLDOWN);
                self.speed = rng.gen_range(h_speed_range(self.kind));
            } else if self.h_turn_cooldown <= 0.0 && max_x > 0.0 {
                let px = match self.direction {
                    Direction::Right => (self.x / max_x).clamp(0.0, 1.0),
                    Direction::Left => (1.0 - self.x / max_x).clamp(0.0, 1.0),
                };
                let step_distance = self.speed * dt_eff;
                let p_turn = horizontal_edge_turn_prob(px, step_distance, max_x);
                if rng.gen::<f32>() < p_turn {
                    self.direction = self.direction.flip();
                    self.h_turn_cooldown = rng.gen_range(MIN_TURN_COOLDOWN..=MAX_TURN_COOLDOWN);
                    self.speed = rng.gen_range(h_speed_range(self.kind));
                }
            }
        } else {
            self.x = self.x.clamp(0.0, max_x);
        }

        let max_y = Self::max_y_for_kind(self.kind, self.height(), screen_height);
        if self.kind == CreatureKind::Crab {
            self.v_speed = 0.0;
            self.y = max_y;
            return;
        }

        self.v_turn_cooldown = (self.v_turn_cooldown - dt_eff).max(0.0);

        if self.v_speed > 0.0 {
            let vy = match self.v_direction {
                VerticalDirection::Down => self.v_speed,
                VerticalDirection::Up => -self.v_speed,
            };
            self.y += vy * dt_eff;

            if self.v_direction == VerticalDirection::Down && self.y >= max_y {
                self.y = max_y;
                self.v_direction = VerticalDirection::Up;
                self.v_turn_cooldown = rng.gen_range(MIN_TURN_COOLDOWN..=MAX_TURN_COOLDOWN);
                if let Some(range) = v_speed_range(self.kind) {
                    self.v_speed = rng.gen_range(range);
                }
            } else if self.v_direction == VerticalDirection::Up && self.y <= 0.0 {
                self.y = 0.0;
                self.v_direction = VerticalDirection::Down;
                self.v_turn_cooldown = rng.gen_range(MIN_TURN_COOLDOWN..=MAX_TURN_COOLDOWN);
                if let Some(range) = v_speed_range(self.kind) {
                    self.v_speed = rng.gen_range(range);
                }
            } else if self.v_turn_cooldown <= 0.0 && max_y > 0.0 {
                let py = match self.v_direction {
                    VerticalDirection::Down => (self.y / max_y).clamp(0.0, 1.0),
                    VerticalDirection::Up => (1.0 - self.y / max_y).clamp(0.0, 1.0),
                };
                let p_vturn =
                    edge_weighted_turn_prob(py, dt_eff, BASE_V_TURN_RATE, EDGE_V_TURN_RATE);
                if rng.gen::<f32>() < p_vturn {
                    self.v_direction = self.v_direction.flip();
                    self.v_turn_cooldown = rng.gen_range(MIN_TURN_COOLDOWN..=MAX_TURN_COOLDOWN);
                    if let Some(range) = v_speed_range(self.kind) {
                        self.v_speed = rng.gen_range(range);
                    }
                }
            }
        } else {
            self.y = self.y.clamp(0.0, max_y);
        }
    }

    fn clamp_bounds(&mut self, screen_width: u16, screen_height: u16) {
        let max_x = (screen_width as f32 - self.width() as f32).max(0.0);
        let max_y = Self::max_y_for_kind(self.kind, self.height(), screen_height);
        self.x = self.x.clamp(0.0, max_x);
        if self.kind == CreatureKind::Crab {
            self.y = max_y;
        } else {
            self.y = self.y.clamp(0.0, max_y);
        }
    }
}

/// Aquarium world state managing dimensions and active marine creatures.
#[derive(Debug, Clone)]
pub struct Aquarium {
    width: u16,
    height: u16,
    fish: Vec<Fish>,
    target_fish_count: usize,
    speed_scale: f32,
    scenery: Scenery,
}

impl Aquarium {
    /// Create a new aquarium with the given terminal dimensions, optional fixed creature count, and speed scale.
    pub fn new<R: Rng + ?Sized>(
        width: u16,
        height: u16,
        fish_count: Option<usize>,
        speed_scale: f32,
        rng: &mut R,
    ) -> Self {
        let plan = match fish_count {
            None => Self::auto_species_plan(width, height),
            Some(n) => Self::explicit_species_plan(width, height, n.min(MAX_FISH_COUNT)),
        };
        let target_fish_count = match fish_count {
            None => plan.len(),
            Some(n) => n.min(MAX_FISH_COUNT),
        };

        let mut fish = Vec::new();
        if width > 0 && height > 0 {
            fish.reserve(plan.len());
            for kind in plan {
                fish.push(Fish::spawn_kind(kind, width, height, rng));
            }
        }

        let scenery = Scenery::generate(width, height, rng);

        Self {
            width,
            height,
            fish,
            target_fish_count,
            speed_scale,
            scenery,
        }
    }

    /// Construct an aquarium directly from an explicit list of fish (useful for deterministic renderer tests).
    pub fn from_fish(width: u16, height: u16, fish: Vec<Fish>, speed_scale: f32) -> Self {
        let target_fish_count = fish.len();
        Self {
            width,
            height,
            fish,
            target_fish_count,
            speed_scale,
            scenery: Scenery::empty(),
        }
    }

    /// Replace the aquarium's scenery (useful for deterministic renderer tests).
    pub fn with_scenery(mut self, scenery: Scenery) -> Self {
        self.scenery = scenery;
        self
    }

    fn auto_species_plan(width: u16, height: u16) -> Vec<CreatureKind> {
        let area = width as usize * height as usize;
        let small = (area / AUTO_SCALE_CELLS_PER_FISH)
            .clamp(AUTO_SCALE_MIN_SMALL_FISH, AUTO_SCALE_MAX_SMALL_FISH);
        let medium = if width >= 30 && height >= 10 {
            (area / 900).clamp(1, 4)
        } else {
            0
        };
        let large = if width >= 50 && height >= 14 {
            (area / 2200).clamp(1, 2)
        } else {
            0
        };
        let jelly = if width >= 24 && height >= 12 {
            (area / 1400).clamp(1, 3)
        } else {
            0
        };
        let crabs = if width >= 24 && height >= 8 {
            (width as usize / 45).clamp(1, 3)
        } else {
            0
        };

        let mut plan = Vec::with_capacity(small + medium + large + jelly + crabs);
        plan.extend(std::iter::repeat_n(CreatureKind::SmallFish, small));
        plan.extend(std::iter::repeat_n(CreatureKind::MediumFish, medium));
        plan.extend(std::iter::repeat_n(CreatureKind::LargeFish, large));
        plan.extend(std::iter::repeat_n(CreatureKind::Jellyfish, jelly));
        plan.extend(std::iter::repeat_n(CreatureKind::Crab, crabs));
        plan
    }

    fn explicit_species_plan(width: u16, height: u16, n: usize) -> Vec<CreatureKind> {
        if n == 0 {
            return Vec::new();
        }
        if n <= 2 {
            return vec![CreatureKind::SmallFish; n];
        }

        let can_medium = width >= 30 && height >= 10;
        let can_crab = width >= 24 && height >= 8;
        let can_jelly = width >= 24 && height >= 12;
        let can_large = width >= 50 && height >= 14;

        let mut remaining = n;
        let medium = if can_medium && n >= 3 {
            (n / 6).max(1).min(remaining / 2)
        } else {
            0
        };
        remaining -= medium;

        let crabs = if can_crab && n >= 4 && remaining > 1 {
            (n / 10).max(1).min((remaining - 1).min(6))
        } else {
            0
        };
        remaining -= crabs;

        let jelly = if can_jelly && n >= 5 && remaining > 1 {
            (n / 8).max(1).min((remaining - 1).min(8))
        } else {
            0
        };
        remaining -= jelly;

        let large = if can_large && n >= 6 && remaining > 1 {
            (n / 12).max(1).min((remaining - 1).min(4))
        } else {
            0
        };
        remaining -= large;

        let small = remaining;

        let mut plan = Vec::with_capacity(n);
        plan.extend(std::iter::repeat_n(CreatureKind::SmallFish, small));
        plan.extend(std::iter::repeat_n(CreatureKind::MediumFish, medium));
        plan.extend(std::iter::repeat_n(CreatureKind::LargeFish, large));
        plan.extend(std::iter::repeat_n(CreatureKind::Jellyfish, jelly));
        plan.extend(std::iter::repeat_n(CreatureKind::Crab, crabs));
        plan
    }

    fn choose_fitting_kind<R: Rng + ?Sized>(width: u16, height: u16, rng: &mut R) -> CreatureKind {
        let mut choices: [(CreatureKind, u32); 5] = [
            (CreatureKind::SmallFish, 55),
            (CreatureKind::MediumFish, 0),
            (CreatureKind::Jellyfish, 0),
            (CreatureKind::Crab, 0),
            (CreatureKind::LargeFish, 0),
        ];
        if width >= 30 && height >= 10 {
            choices[1].1 = 20;
        }
        if width >= 24 && height >= 12 {
            choices[2].1 = 10;
        }
        if width >= 24 && height >= 8 {
            choices[3].1 = 10;
        }
        if width >= 50 && height >= 14 {
            choices[4].1 = 5;
        }

        let total_weight: u32 = choices.iter().map(|(_, w)| *w).sum();
        let mut roll = rng.gen_range(0..total_weight);
        for (kind, weight) in choices {
            if roll < weight {
                return kind;
            }
            roll -= weight;
        }
        CreatureKind::SmallFish
    }

    /// Current aquarium width in terminal columns.
    pub fn width(&self) -> u16 {
        self.width
    }

    /// Current aquarium height in terminal rows.
    pub fn height(&self) -> u16 {
        self.height
    }

    /// Slice of all active creatures in the aquarium.
    pub fn fish(&self) -> &[Fish] {
        &self.fish
    }

    /// Number of currently active creatures in the aquarium.
    pub fn fish_count(&self) -> usize {
        self.fish.len()
    }

    /// Current seabed scenery (coral reefs and swaying seaweed).
    pub fn scenery(&self) -> &Scenery {
        &self.scenery
    }

    /// Advance the simulation by `dt` seconds while keeping all creatures permanently inside bounds.
    pub fn update<R: Rng + ?Sized>(&mut self, dt: f32, rng: &mut R) {
        if self.width == 0 || self.height == 0 || !dt.is_finite() || dt <= 0.0 {
            return;
        }
        let scale = if self.speed_scale.is_finite() && self.speed_scale > 0.0 {
            self.speed_scale
        } else {
            0.0
        };

        for fish in &mut self.fish {
            fish.advance(dt, scale, self.width, self.height, rng);
        }
        self.scenery.update(dt * scale);
    }

    /// Handle terminal resize by clamping existing creatures into the new bounds without adding or removing any.
    pub fn resize<R: Rng + ?Sized>(&mut self, width: u16, height: u16, rng: &mut R) {
        self.width = width;
        self.height = height;

        if width == 0 || height == 0 {
            return;
        }

        self.scenery.resize(width, height, rng);

        if self.fish.is_empty() && self.target_fish_count > 0 {
            let plan = Self::explicit_species_plan(width, height, self.target_fish_count);
            self.fish.reserve(plan.len());
            for kind in plan {
                self.fish.push(Fish::spawn_kind(kind, width, height, rng));
            }
            return;
        }

        for fish in &mut self.fish {
            fish.clamp_bounds(width, height);
        }
    }

    /// Manually add a random fitting creature inside the terminal bounds (up to 200).
    pub fn add_fish<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        if self.width > 0 && self.height > 0 {
            if self.fish.len() < MAX_FISH_COUNT {
                let kind = Self::choose_fitting_kind(self.width, self.height, rng);
                self.fish
                    .push(Fish::spawn_kind(kind, self.width, self.height, rng));
                self.target_fish_count = self.fish.len();
            }
        } else if self.fish.is_empty() && self.target_fish_count < MAX_FISH_COUNT {
            self.target_fish_count += 1;
        }
    }

    /// Manually remove a creature (keeping at least 1 when non-empty).
    pub fn remove_fish(&mut self) {
        if self.fish.len() > 1 {
            self.fish.pop();
            self.target_fish_count = self.fish.len();
        } else if !self.fish.is_empty() {
            self.target_fish_count = 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn small_fish_sprites_are_ascii_single_line_and_symmetric() {
        assert!(
            SMALL_FISH_SPRITES.len() >= 6,
            "expected at least 6 distinct small fish sprites"
        );
        for (i, sprite) in SMALL_FISH_SPRITES.iter().enumerate() {
            assert!(!sprite.right.is_empty());
            assert_eq!(
                sprite.right.len(),
                sprite.left.len(),
                "sprite #{i} right/left lengths must match"
            );
            assert!(
                sprite
                    .right
                    .bytes()
                    .all(|b| (0x20..=0x7E).contains(&b)),
                "sprite #{i} right must be printable single-line ASCII"
            );
            assert!(
                sprite
                    .left
                    .bytes()
                    .all(|b| (0x20..=0x7E).contains(&b)),
                "sprite #{i} left must be printable single-line ASCII"
            );
            for (j, other) in SMALL_FISH_SPRITES.iter().enumerate() {
                if i != j {
                    assert_ne!(sprite, other, "sprites #{i} and #{j} must be distinct");
                }
            }
        }
    }

    #[test]
    fn initial_spawning_places_all_fish_fully_inside_bounds() {
        let mut rng = StdRng::seed_from_u64(42);
        let aq = Aquarium::new(80, 24, None, 1.0, &mut rng);
        // 80x24 auto-scale: 8 small + 2 medium + 1 large + 1 jellyfish + 1 crab = 13
        assert_eq!(aq.fish_count(), 13);
        assert_eq!(aq.width(), 80);
        assert_eq!(aq.height(), 24);

        let large_aq = Aquarium::new(80, 24, Some(100), 1.0, &mut rng);
        assert_eq!(large_aq.fish_count(), 100);

        for fish in aq.fish().iter().chain(large_aq.fish().iter()) {
            let max_x = (80 - fish.width()) as f32;
            let max_y = (24 - fish.height()) as f32;
            assert!(
                fish.x() >= 0.0 && fish.x() <= max_x,
                "fish x={} must be in 0.0..={max_x}",
                fish.x()
            );
            assert!(
                fish.y() >= 0.0 && fish.y() <= max_y,
                "fish y={} must be in 0.0..={max_y}",
                fish.y()
            );
            assert!(fish.width() >= 3);
            assert!(fish.height() >= 1);
        }
    }

    #[test]
    fn horizontal_movement_respects_direction_and_speed_scale() {
        let mut rng = StdRng::seed_from_u64(7);
        let right_fish = Fish::with_motion(
            10.0,
            5.0,
            Direction::Right,
            6.0,
            VerticalDirection::Down,
            0.0,
            2.0,
            2.0,
            FishColor::Cyan,
            "><>",
        );
        let left_fish = Fish::with_motion(
            20.0,
            8.0,
            Direction::Left,
            4.0,
            VerticalDirection::Down,
            0.0,
            2.0,
            2.0,
            FishColor::Yellow,
            "<><",
        );
        let mut aq = Aquarium::from_fish(80, 24, vec![right_fish, left_fish], 2.0);

        aq.update(0.5, &mut rng);

        assert!((aq.fish()[0].x() - 16.0).abs() < 1e-5);
        assert!((aq.fish()[1].x() - 16.0).abs() < 1e-5);
    }

    #[test]
    fn fixed_population_and_identities_preserved_across_updates_and_resizes() {
        let mut rng = StdRng::seed_from_u64(1337);
        let mut aq = Aquarium::new(80, 24, None, 1.0, &mut rng);
        let expected_count = aq.fish_count();
        assert_eq!(expected_count, 13);

        let initial_identities: Vec<(CreatureKind, FishColor, usize, usize)> = aq
            .fish()
            .iter()
            .map(|f| (f.kind(), f.color(), f.width(), f.height()))
            .collect();

        for _ in 0..600 {
            aq.update(0.05, &mut rng);
            assert_eq!(aq.fish_count(), expected_count);
            for (i, fish) in aq.fish().iter().enumerate() {
                let max_x = (aq.width() as usize - fish.width()) as f32;
                let max_y = (aq.height() as usize - fish.height()) as f32;
                assert!(fish.x() >= 0.0 && fish.x() <= max_x);
                assert!(fish.y() >= 0.0 && fish.y() <= max_y);
                assert_eq!(
                    (fish.kind(), fish.color(), fish.width(), fish.height()),
                    initial_identities[i]
                );
            }
        }

        // Resize larger: population and identities must not change
        aq.resize(160, 40, &mut rng);
        assert_eq!(aq.width(), 160);
        assert_eq!(aq.height(), 40);
        assert_eq!(aq.fish_count(), expected_count);

        for _ in 0..100 {
            aq.update(0.05, &mut rng);
        }

        // Resize smaller: population and identities must not change, positions clamped in bounds
        aq.resize(30, 8, &mut rng);
        assert_eq!(aq.width(), 30);
        assert_eq!(aq.height(), 8);
        assert_eq!(aq.fish_count(), expected_count);

        for (i, fish) in aq.fish().iter().enumerate() {
            let max_x = (30 - fish.width()) as f32;
            let max_y = (8 - fish.height()) as f32;
            assert!(
                fish.x() >= 0.0 && fish.x() <= max_x,
                "fish x={} out of shrunk bounds 0..={max_x}",
                fish.x()
            );
            assert!(
                fish.y() >= 0.0 && fish.y() <= max_y,
                "fish y={} out of shrunk bounds 0..={max_y}",
                fish.y()
            );
            assert_eq!(
                (fish.kind(), fish.color(), fish.width(), fish.height()),
                initial_identities[i]
            );
        }
    }

    #[test]
    fn horizontal_wall_bounces_clamp_and_flip_direction_and_sprite() {
        let mut rng = StdRng::seed_from_u64(99);
        // Screen width 40, sprite "><>" has width 3 => max_x = 37.0
        let right_fish =
            Fish::from_parts(36.0, 4.0, Direction::Right, 10.0, FishColor::Green, "><>");
        let left_fish =
            Fish::from_parts(1.0, 6.0, Direction::Left, 10.0, FishColor::Magenta, "<'))><");
        let mut aq = Aquarium::from_fish(40, 12, vec![right_fish, left_fish], 1.0);

        aq.update(0.2, &mut rng);

        let bounced_right = &aq.fish()[0];
        assert!((bounced_right.x() - 37.0).abs() < 1e-5);
        assert_eq!(bounced_right.direction(), Direction::Left);
        assert_eq!(bounced_right.sprite(), "<><");

        let bounced_left = &aq.fish()[1];
        assert!((bounced_left.x() - 0.0).abs() < 1e-5);
        assert_eq!(bounced_left.direction(), Direction::Right);
        assert_eq!(bounced_left.sprite(), "><(('>");

        // Now force bounced_right all the way to the left edge and verify it flips back to "><>"
        let mut single = Aquarium::from_fish(
            10,
            5,
            vec![Fish::from_parts(
                0.5,
                2.0,
                Direction::Left,
                10.0,
                FishColor::Green,
                bounced_right.sprite(),
            )],
            1.0,
        );
        single.update(0.2, &mut rng);
        assert!((single.fish()[0].x() - 0.0).abs() < 1e-5);
        assert_eq!(single.fish()[0].direction(), Direction::Right);
        assert_eq!(single.fish()[0].sprite(), "><>");
    }

    #[test]
    fn vertical_wall_bounces_clamp_and_flip_vertical_direction() {
        let mut rng = StdRng::seed_from_u64(2025);
        // Height 10 => max_y = 9.0
        let down_fish = Fish::with_motion(
            10.0,
            8.5,
            Direction::Right,
            0.0,
            VerticalDirection::Down,
            5.0,
            1.0,
            1.0,
            FishColor::Cyan,
            "><>",
        );
        let up_fish = Fish::with_motion(
            15.0,
            0.5,
            Direction::Left,
            0.0,
            VerticalDirection::Up,
            5.0,
            1.0,
            1.0,
            FishColor::Yellow,
            "<><",
        );
        let mut aq = Aquarium::from_fish(40, 10, vec![down_fish, up_fish], 1.0);

        aq.update(0.2, &mut rng);

        assert!((aq.fish()[0].y() - 9.0).abs() < 1e-5);
        assert_eq!(aq.fish()[0].vertical_direction(), VerticalDirection::Up);

        assert!((aq.fish()[1].y() - 0.0).abs() < 1e-5);
        assert_eq!(aq.fish()[1].vertical_direction(), VerticalDirection::Down);
    }

    #[test]
    fn fish_roam_vertically_across_full_terminal_height() {
        let mut rng = StdRng::seed_from_u64(12345);
        let mut aq = Aquarium::new(80, 24, None, 1.0, &mut rng);
        let count = aq.fish_count();
        let mut min_y: Vec<f32> = aq.fish().iter().map(Fish::y).collect();
        let mut max_y: Vec<f32> = aq.fish().iter().map(Fish::y).collect();

        for _ in 0..600 {
            aq.update(0.05, &mut rng);
            for (i, fish) in aq.fish().iter().enumerate() {
                let bound_y = (24 - fish.height()) as f32;
                assert!(
                    fish.y() >= 0.0 && fish.y() <= bound_y,
                    "fish y={} out of 0.0..={bound_y} bounds",
                    fish.y()
                );
                min_y[i] = min_y[i].min(fish.y());
                max_y[i] = max_y[i].max(fish.y());
            }
        }

        for i in 0..count {
            let kind = aq.fish()[i].kind();
            let span = max_y[i] - min_y[i];
            match kind {
                CreatureKind::SmallFish => {
                    assert!(
                        span >= 8.0,
                        "small fish #{i} should roam across a wide vertical span (>= 8 rows), got span={span} ({}..={})",
                        min_y[i],
                        max_y[i]
                    );
                }
                CreatureKind::MediumFish | CreatureKind::Jellyfish | CreatureKind::LargeFish => {
                    assert!(
                        span >= 4.0,
                        "{kind:?} #{i} should roam vertically (>= 4 rows), got span={span}"
                    );
                }
                CreatureKind::Crab => {
                    assert!(
                        span.abs() < 1e-5,
                        "crab #{i} must stay anchored on the floor, got vertical span={span}"
                    );
                }
            }
        }
    }

    #[test]
    fn edge_weighted_turn_probability_is_monotonic_and_triggers_near_edges() {
        assert_eq!(
            edge_weighted_turn_prob(0.5, 0.0, BASE_V_TURN_RATE, EDGE_V_TURN_RATE),
            0.0
        );
        assert_eq!(
            edge_weighted_turn_prob(0.5, -0.5, BASE_V_TURN_RATE, EDGE_V_TURN_RATE),
            0.0
        );
        assert_eq!(
            edge_weighted_turn_prob(0.5, f32::NAN, BASE_V_TURN_RATE, EDGE_V_TURN_RATE),
            0.0
        );
        assert_eq!(
            edge_weighted_turn_prob(0.5, f32::INFINITY, BASE_V_TURN_RATE, EDGE_V_TURN_RATE),
            0.0
        );

        let p_low = edge_weighted_turn_prob(0.1, 0.1, BASE_V_TURN_RATE, EDGE_V_TURN_RATE);
        let p_mid = edge_weighted_turn_prob(0.5, 0.1, BASE_V_TURN_RATE, EDGE_V_TURN_RATE);
        let p_high = edge_weighted_turn_prob(0.9, 0.1, BASE_V_TURN_RATE, EDGE_V_TURN_RATE);
        let p_edge = edge_weighted_turn_prob(1.0, 0.1, BASE_V_TURN_RATE, EDGE_V_TURN_RATE);

        assert!(
            p_low > 0.0 && p_low < 0.01,
            "center turn prob should be very low: {p_low}"
        );
        assert!(p_low < p_mid, "expected p(0.1) < p(0.5): {p_low} vs {p_mid}");
        assert!(
            p_mid < p_high,
            "expected p(0.5) < p(0.9): {p_mid} vs {p_high}"
        );
        assert!(
            p_high < p_edge,
            "expected p(0.9) < p(1.0): {p_high} vs {p_edge}"
        );
        assert!(
            p_high > p_low * 15.0,
            "near-edge turn prob ({p_high}) should be much larger than near-center ({p_low})"
        );

        // Statistical vertical check over 1000 single-step trials (fish stops short of max_y=23)
        let mut rng = StdRng::seed_from_u64(314159);
        let mut center_v_turns = 0usize;
        let mut edge_v_turns = 0usize;

        for _ in 0..1000 {
            let center_fish = Fish::with_motion(
                20.0,
                5.0,
                Direction::Right,
                0.0,
                VerticalDirection::Down,
                2.0,
                1.0,
                0.0,
                FishColor::Yellow,
                "><>",
            );
            let edge_fish = Fish::with_motion(
                20.0,
                21.8,
                Direction::Right,
                0.0,
                VerticalDirection::Down,
                2.0,
                1.0,
                0.0,
                FishColor::Yellow,
                "><>",
            );
            let mut aq = Aquarium::from_fish(100, 24, vec![center_fish, edge_fish], 1.0);
            aq.update(0.1, &mut rng);

            assert!(aq.fish()[0].y() < 23.0);
            assert!(aq.fish()[1].y() < 23.0);

            if aq.fish()[0].vertical_direction() == VerticalDirection::Up {
                center_v_turns += 1;
            }
            if aq.fish()[1].vertical_direction() == VerticalDirection::Up {
                edge_v_turns += 1;
            }
        }

        assert!(
            edge_v_turns > 60,
            "expected many pre-wall turns near vertical edge, got {edge_v_turns}"
        );
        assert!(
            edge_v_turns > center_v_turns * 5,
            "expected near-edge vertical turns ({edge_v_turns}) >> center turns ({center_v_turns})"
        );
    }

    #[test]
    fn horizontal_edge_turn_probability_is_zero_in_center_and_monotonic_in_edge_zone() {
        // Zero throughout the far side and center (progress <= 0.68)
        assert_eq!(horizontal_edge_turn_prob(0.0, 1.0, 100.0), 0.0);
        assert_eq!(horizontal_edge_turn_prob(0.25, 1.0, 100.0), 0.0);
        assert_eq!(horizontal_edge_turn_prob(0.5, 1.0, 100.0), 0.0);
        assert_eq!(horizontal_edge_turn_prob(0.68, 1.0, 100.0), 0.0);
        assert_eq!(Fish::horizontal_edge_turn_prob(0.68, 1.0, 100.0), 0.0);

        // Invalid or non-positive inputs return 0.0
        assert_eq!(horizontal_edge_turn_prob(0.9, 0.0, 100.0), 0.0);
        assert_eq!(horizontal_edge_turn_prob(0.9, -1.0, 100.0), 0.0);
        assert_eq!(horizontal_edge_turn_prob(0.9, f32::NAN, 100.0), 0.0);
        assert_eq!(horizontal_edge_turn_prob(0.9, f32::INFINITY, 100.0), 0.0);
        assert_eq!(horizontal_edge_turn_prob(0.9, 1.0, 0.0), 0.0);
        assert_eq!(horizontal_edge_turn_prob(0.9, 1.0, -50.0), 0.0);
        assert_eq!(horizontal_edge_turn_prob(0.9, 1.0, f32::NAN), 0.0);
        assert_eq!(horizontal_edge_turn_prob(0.9, 1.0, f32::INFINITY), 0.0);
        assert_eq!(horizontal_edge_turn_prob(f32::NAN, 1.0, 100.0), 0.0);

        // Strictly monotonically increasing in the outer edge zone (0.68..=1.0)
        let p_70 = horizontal_edge_turn_prob(0.70, 1.0, 100.0);
        let p_85 = horizontal_edge_turn_prob(0.85, 1.0, 100.0);
        let p_95 = horizontal_edge_turn_prob(0.95, 1.0, 100.0);
        let p_100 = horizontal_edge_turn_prob(1.0, 1.0, 100.0);

        assert!(p_70 > 0.0, "expected p(0.70) > 0.0, got {p_70}");
        assert!(p_70 < p_85, "expected p(0.70) < p(0.85): {p_70} vs {p_85}");
        assert!(p_85 < p_95, "expected p(0.85) < p(0.95): {p_85} vs {p_95}");
        assert!(p_95 < p_100, "expected p(0.95) < p(1.0): {p_95} vs {p_100}");

        let mut rng = StdRng::seed_from_u64(314159);
        let mut center_h_turns = 0usize;
        let mut edge_h_turns = 0usize;

        for _ in 0..1000 {
            let center_fish = Fish::with_motion(
                48.5,
                10.0,
                Direction::Right,
                8.0,
                VerticalDirection::Down,
                0.0,
                0.0,
                1.0,
                FishColor::Cyan,
                "><>",
            );
            let edge_fish = Fish::with_motion(
                93.0,
                10.0,
                Direction::Right,
                8.0,
                VerticalDirection::Down,
                0.0,
                0.0,
                1.0,
                FishColor::Cyan,
                "><>",
            );
            let mut aq = Aquarium::from_fish(100, 24, vec![center_fish, edge_fish], 1.0);
            aq.update(0.2, &mut rng);

            assert!(aq.fish()[0].x() < 97.0);
            assert!(aq.fish()[1].x() < 97.0);

            if aq.fish()[0].direction() == Direction::Left {
                center_h_turns += 1;
            }
            if aq.fish()[1].direction() == Direction::Left {
                edge_h_turns += 1;
            }
        }

        assert_eq!(
            center_h_turns, 0,
            "fish in the center of the tank must never turn spontaneously, got {center_h_turns}"
        );
        assert!(
            edge_h_turns > 0,
            "expected pre-wall turns near horizontal edge, got {edge_h_turns}"
        );
    }

    #[test]
    fn wide_terminal_fish_spread_across_full_width_without_center_clustering() {
        let mut rng = StdRng::seed_from_u64(202604);
        let mut aq = Aquarium::new(160, 24, Some(20), 1.0, &mut rng);

        let mut left_samples = 0usize;
        let mut middle_samples = 0usize;
        let mut right_samples = 0usize;
        let mut total_samples = 0usize;

        for _ in 0..1000 {
            aq.update(0.05, &mut rng);
            for fish in aq.fish() {
                let max_x = (160 - fish.width()) as f32;
                let x = fish.x();
                if x < max_x / 3.0 {
                    left_samples += 1;
                } else if x > 2.0 * max_x / 3.0 {
                    right_samples += 1;
                } else {
                    middle_samples += 1;
                }
                total_samples += 1;
            }
        }

        let left_frac = left_samples as f32 / total_samples as f32;
        let middle_frac = middle_samples as f32 / total_samples as f32;
        let right_frac = right_samples as f32 / total_samples as f32;

        assert!(
            left_frac >= 0.20,
            "expected left outer third >= 20%, got {:.1}%",
            left_frac * 100.0
        );
        assert!(
            right_frac >= 0.20,
            "expected right outer third >= 20%, got {:.1}%",
            right_frac * 100.0
        );
        assert!(
            middle_frac <= 0.55,
            "expected middle third <= 55% (no center clustering), got {:.1}%",
            middle_frac * 100.0
        );
    }

    #[test]
    fn add_and_remove_fish_respect_bounds_and_limits() {
        let mut rng = StdRng::seed_from_u64(555);
        let mut aq = Aquarium::new(80, 24, Some(2), 1.0, &mut rng);
        assert_eq!(aq.fish_count(), 2);

        aq.add_fish(&mut rng);
        assert_eq!(aq.fish_count(), 3);
        for fish in aq.fish() {
            let max_x = (80 - fish.width()) as f32;
            let max_y = (24 - fish.height()) as f32;
            assert!(fish.x() >= 0.0 && fish.x() <= max_x);
            assert!(fish.y() >= 0.0 && fish.y() <= max_y);
        }

        // Remove down to 1; further removals must not drop below 1
        aq.remove_fish();
        assert_eq!(aq.fish_count(), 2);
        aq.remove_fish();
        assert_eq!(aq.fish_count(), 1);
        aq.remove_fish();
        assert_eq!(aq.fish_count(), 1);

        // Cap at 200
        let mut maxed = Aquarium::new(80, 24, Some(200), 1.0, &mut rng);
        assert_eq!(maxed.fish_count(), 200);
        maxed.add_fish(&mut rng);
        assert_eq!(maxed.fish_count(), 200);

        // Resizing after add/remove keeps the exact count
        aq.resize(160, 50, &mut rng);
        assert_eq!(aq.fish_count(), 1);
    }

    #[test]
    fn handles_degenerate_dimensions_bad_dt_and_zero_resize_roundtrip_safely() {
        let mut rng = StdRng::seed_from_u64(777);

        // 0x0 with explicit fish count populates once resized to non-zero
        let mut aq = Aquarium::new(0, 0, Some(6), 1.0, &mut rng);
        assert_eq!(aq.fish_count(), 0);
        aq.update(0.1, &mut rng);
        aq.remove_fish();
        assert_eq!(aq.fish_count(), 0);

        aq.resize(1, 1, &mut rng);
        assert_eq!(aq.fish_count(), 6);
        for fish in aq.fish() {
            assert_eq!(fish.x(), 0.0);
            assert_eq!(fish.y(), 0.0);
        }

        // Narrower-than-sprite dimensions (2x3)
        aq.resize(2, 3, &mut rng);
        assert_eq!(aq.fish_count(), 6);
        aq.update(0.25, &mut rng);
        for fish in aq.fish() {
            assert_eq!(fish.x(), 0.0);
            assert!(fish.y() >= 0.0 && fish.y() <= 2.0);
        }

        // Stepping with non-positive / NaN / infinite dt should never panic
        aq.update(0.0, &mut rng);
        aq.update(-1.0, &mut rng);
        aq.update(f32::NAN, &mut rng);
        aq.update(f32::INFINITY, &mut rng);
        aq.update(f32::NEG_INFINITY, &mut rng);

        // Resizing 80x24 -> 0x0 -> 80x24 preserves existing fish
        let mut roundtrip = Aquarium::new(80, 24, None, 1.0, &mut rng);
        assert_eq!(roundtrip.fish_count(), 13);
        let snapshot: Vec<(f32, f32, FishColor, &'static str)> = roundtrip
            .fish()
            .iter()
            .map(|f| (f.x(), f.y(), f.color(), f.sprite()))
            .collect();

        roundtrip.resize(0, 0, &mut rng);
        assert_eq!(roundtrip.fish_count(), 13);
        roundtrip.update(0.5, &mut rng);
        roundtrip.resize(80, 24, &mut rng);
        assert_eq!(roundtrip.fish_count(), 13);

        let after: Vec<(f32, f32, FishColor, &'static str)> = roundtrip
            .fish()
            .iter()
            .map(|f| (f.x(), f.y(), f.color(), f.sprite()))
            .collect();
        assert_eq!(after, snapshot);
    }

    #[test]
    fn ecosystem_spawns_all_five_creature_kinds_on_standard_and_large_terminals() {
        let mut rng = StdRng::seed_from_u64(2026);
        for (w, h) in [(80, 24), (160, 40)] {
            let aq = Aquarium::new(w, h, None, 1.0, &mut rng);
            for kind in [
                CreatureKind::SmallFish,
                CreatureKind::MediumFish,
                CreatureKind::LargeFish,
                CreatureKind::Jellyfish,
                CreatureKind::Crab,
            ] {
                assert!(
                    aq.fish().iter().any(|f| f.kind() == kind),
                    "expected {kind:?} to be spawned in {w}x{h} aquarium"
                );
            }
        }
    }

    #[test]
    fn explicit_fish_count_spawns_exact_requested_population() {
        let mut rng = StdRng::seed_from_u64(888);
        for n in [1, 5, 12, 200] {
            let aq = Aquarium::new(80, 24, Some(n), 1.0, &mut rng);
            assert_eq!(
                aq.fish_count(),
                n,
                "expected exact count {n}, got {}",
                aq.fish_count()
            );
            for fish in aq.fish() {
                let max_x = (80 - fish.width()) as f32;
                let max_y = (24 - fish.height()) as f32;
                assert!(fish.x() >= 0.0 && fish.x() <= max_x);
                assert!(fish.y() >= 0.0 && fish.y() <= max_y);
            }
        }
    }

    #[test]
    fn crabs_stay_anchored_on_seabed_across_updates_and_resizes() {
        let mut rng = StdRng::seed_from_u64(404);
        let crab = Fish::spawn_kind(CreatureKind::Crab, 80, 24, &mut rng);
        assert_eq!(crab.kind(), CreatureKind::Crab);
        assert_eq!(crab.height(), 2);
        assert_eq!(crab.y(), 22.0);

        let mut aq = Aquarium::from_fish(80, 24, vec![crab], 1.0);
        let start_x = aq.fish()[0].x();
        let mut moved_horizontally = false;

        for _ in 0..200 {
            aq.update(0.05, &mut rng);
            let c = &aq.fish()[0];
            assert_eq!(c.y(), 22.0, "crab must remain anchored at y = 24 - 2 = 22");
            assert!(c.x() >= 0.0 && c.x() <= (80 - c.width()) as f32);
            if (c.x() - start_x).abs() > 0.5 {
                moved_horizontally = true;
            }
        }
        assert!(moved_horizontally, "crab should scuttle horizontally along the seabed");

        // Resize taller -> crab snaps to new floor (40 - 2 = 38)
        aq.resize(80, 40, &mut rng);
        assert_eq!(aq.fish()[0].y(), 38.0);

        // Resize shorter -> crab snaps to new floor (12 - 2 = 10)
        aq.resize(80, 12, &mut rng);
        assert_eq!(aq.fish()[0].y(), 10.0);
    }

    #[test]
    fn jellyfish_and_crabs_advance_animation_frames_over_time() {
        let mut rng = StdRng::seed_from_u64(909);
        let jelly = Fish::spawn_kind(CreatureKind::Jellyfish, 80, 24, &mut rng);
        let crab = Fish::spawn_kind(CreatureKind::Crab, 80, 24, &mut rng);

        let jelly_frame_0: Vec<&'static str> = jelly.sprite_lines().to_vec();
        let crab_frame_0: Vec<&'static str> = crab.sprite_lines().to_vec();

        let mut aq = Aquarium::from_fish(80, 24, vec![jelly, crab], 1.0);

        // Advance by 0.25s: crab interval is 0.25s so crab frame flips; jellyfish interval is 0.45s
        aq.update(0.25, &mut rng);
        assert_ne!(
            aq.fish()[1].sprite_lines(),
            crab_frame_0.as_slice(),
            "crab animation frame should advance after 0.25s"
        );

        // Advance by another 0.20s (total 0.45s): jellyfish frame flips
        aq.update(0.20, &mut rng);
        assert_ne!(
            aq.fish()[0].sprite_lines(),
            jelly_frame_0.as_slice(),
            "jellyfish animation frame should advance after 0.45s"
        );
    }

    #[test]
    fn large_fish_flip_multiline_sprites_on_wall_bounce_and_stay_in_bounds() {
        let mut rng = StdRng::seed_from_u64(1111);
        let mut large = Fish::spawn_kind(CreatureKind::LargeFish, 80, 24, &mut rng);
        let w = large.width();
        let h = large.height();
        assert_eq!(h, 5);
        assert!(w >= 20);

        // Position right-facing large fish near right wall
        large.direction = Direction::Right;
        large.x = (80 - w) as f32 - 0.5;
        large.y = 5.0;
        large.speed = 5.0;
        let right_lines: Vec<&'static str> = large.sprite_lines().to_vec();

        let mut aq = Aquarium::from_fish(80, 24, vec![large], 1.0);
        aq.update(0.2, &mut rng);

        let bounced = &aq.fish()[0];
        assert_eq!(bounced.direction(), Direction::Left);
        assert!((bounced.x() - (80 - w) as f32).abs() < 1e-5);
        assert_ne!(
            bounced.sprite_lines(),
            right_lines.as_slice(),
            "large fish must switch to left-facing multi-line sprite after bouncing off right wall"
        );

        for _ in 0..300 {
            aq.update(0.05, &mut rng);
            let f = &aq.fish()[0];
            assert!(f.x() >= 0.0 && f.x() <= (80 - f.width()) as f32);
            assert!(f.y() >= 0.0 && f.y() <= (24 - f.height()) as f32);
        }
    }

    #[test]
    fn tiny_terminals_spawn_and_update_safely_without_out_of_bounds() {
        let mut rng = StdRng::seed_from_u64(3030);
        for (w, h) in [(10, 5), (2, 3), (1, 1), (0, 0)] {
            let mut aq = Aquarium::new(w, h, None, 1.0, &mut rng);
            aq.update(0.1, &mut rng);
            for fish in aq.fish() {
                let max_x = (w as f32 - fish.width() as f32).max(0.0);
                let max_y = (h as f32 - fish.height() as f32).max(0.0);
                assert!(fish.x() >= 0.0 && fish.x() <= max_x);
                assert!(fish.y() >= 0.0 && fish.y() <= max_y);
            }
        }
    }

    #[test]
    fn aquarium_scenery_lifecycle_generates_animates_and_resizes() {
        let mut rng = StdRng::seed_from_u64(2027);

        let mut aq = Aquarium::new(80, 24, None, 1.0, &mut rng);
        assert!(!aq.scenery().corals().is_empty());
        assert!(!aq.scenery().seaweed().is_empty());

        let fish_count = aq.fish_count();
        let initial_cells: Vec<Vec<(i32, i32, char)>> = aq
            .scenery()
            .seaweed()
            .iter()
            .map(|s| s.cells(80, 24))
            .collect();

        let mut animated = false;
        for _ in 0..10 {
            aq.update(0.2, &mut rng);
            let next_cells: Vec<Vec<(i32, i32, char)>> = aq
                .scenery()
                .seaweed()
                .iter()
                .map(|s| s.cells(80, 24))
                .collect();
            if next_cells != initial_cells {
                animated = true;
            }
        }
        assert!(animated, "Aquarium::update should animate seaweed stalks");
        assert_eq!(aq.fish_count(), fish_count);

        let coral_count = aq.scenery().corals().len();
        let seaweed_count = aq.scenery().seaweed().len();
        aq.resize(120, 30, &mut rng);
        assert_eq!(aq.scenery().corals().len(), coral_count);
        assert_eq!(aq.scenery().seaweed().len(), seaweed_count);

        // Aquarium::from_fish starts with empty scenery
        let det = Aquarium::from_fish(80, 24, Vec::new(), 1.0);
        assert!(det.scenery().corals().is_empty());
        assert!(det.scenery().seaweed().is_empty());

        // Aquarium::new(0, 0, ...) starts empty and populates scenery when resized to 80x24
        let mut zero_aq = Aquarium::new(0, 0, None, 1.0, &mut rng);
        assert!(zero_aq.scenery().corals().is_empty());
        assert!(zero_aq.scenery().seaweed().is_empty());
        zero_aq.resize(80, 24, &mut rng);
        assert!(!zero_aq.scenery().corals().is_empty());
        assert!(!zero_aq.scenery().seaweed().is_empty());
    }
}
