use rand::Rng;

const MAX_FISH_COUNT: usize = 200;
const AUTO_SCALE_CELLS_PER_FISH: usize = 220;
const AUTO_SCALE_MIN_FISH: usize = 4;
const AUTO_SCALE_MAX_FISH: usize = 24;

const MIN_H_SPEED: f32 = 4.0;
const MAX_H_SPEED: f32 = 12.0;
const MIN_V_SPEED: f32 = 1.2;
const MAX_V_SPEED: f32 = 3.8;

const MIN_INITIAL_TURN_COOLDOWN: f32 = 0.4;
const MAX_INITIAL_TURN_COOLDOWN: f32 = 1.2;
const MIN_TURN_COOLDOWN: f32 = 0.8;
const MAX_TURN_COOLDOWN: f32 = 1.6;

const BASE_H_TURN_RATE: f32 = 0.03;
const EDGE_H_TURN_RATE: f32 = 1.4;
const BASE_V_TURN_RATE: f32 = 0.04;
const EDGE_V_TURN_RATE: f32 = 1.4;

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

/// Terminal color palette for small aquarium fish.
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

/// A single swimming fish inside the aquarium.
#[derive(Debug, Clone)]
pub struct Fish {
    x: f32,
    y: f32,
    direction: Direction,
    speed: f32,
    v_direction: VerticalDirection,
    v_speed: f32,
    h_turn_cooldown: f32,
    v_turn_cooldown: f32,
    color: FishColor,
    sprite: FishSprite,
}

impl Fish {
    /// Construct a fish from explicit horizontal state with zero vertical speed by default.
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

    /// Construct a fish with explicit 2D motion and turn cooldown parameters.
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
        Self {
            x,
            y,
            direction,
            speed: speed.abs(),
            v_direction,
            v_speed: v_speed.abs(),
            h_turn_cooldown: h_turn_cooldown.max(0.0),
            v_turn_cooldown: v_turn_cooldown.max(0.0),
            color,
            sprite: resolve_fish_sprite(sprite),
        }
    }

    /// Compute frame-rate-independent edge-weighted turn probability for a step `dt`.
    pub fn edge_weighted_turn_prob(progress: f32, dt: f32, base_rate: f32, edge_rate: f32) -> f32 {
        edge_weighted_turn_prob(progress, dt, base_rate, edge_rate)
    }

    /// Spawn a fish directly inside the visible terminal bounds.
    pub fn spawn_initial<R: Rng + ?Sized>(
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
        let sprite = SMALL_FISH_SPRITES[rng.gen_range(0..SMALL_FISH_SPRITES.len())];
        let sprite_width = sprite.right.len();
        let color = FishColor::ALL[rng.gen_range(0..FishColor::ALL.len())];
        let speed = rng.gen_range(MIN_H_SPEED..=MAX_H_SPEED);
        let v_speed = rng.gen_range(MIN_V_SPEED..=MAX_V_SPEED);
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

        let max_y = screen_height.saturating_sub(1) as f32;
        let y = if max_y <= 0.0 {
            0.0
        } else {
            rng.gen_range(0.0..=max_y)
        };

        Self {
            x,
            y,
            direction,
            speed,
            v_direction,
            v_speed,
            h_turn_cooldown,
            v_turn_cooldown,
            color,
            sprite,
        }
    }

    /// Current horizontal position in terminal cell coordinates.
    pub fn x(&self) -> f32 {
        self.x
    }

    /// Current vertical position in terminal row coordinates (`0.0..=height - 1`).
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

    /// Assigned display color for this fish.
    pub fn color(&self) -> FishColor {
        self.color
    }

    /// Single-line ASCII sprite string matching the fish's current horizontal direction.
    pub fn sprite(&self) -> &'static str {
        match self.direction {
            Direction::Right => self.sprite.right,
            Direction::Left => self.sprite.left,
        }
    }

    /// Width of the fish's ASCII sprite in terminal columns.
    pub fn width(&self) -> usize {
        self.sprite().len()
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
                self.speed = rng.gen_range(MIN_H_SPEED..=MAX_H_SPEED);
            } else if self.direction == Direction::Left && self.x <= 0.0 {
                self.x = 0.0;
                self.direction = Direction::Right;
                self.h_turn_cooldown = rng.gen_range(MIN_TURN_COOLDOWN..=MAX_TURN_COOLDOWN);
                self.speed = rng.gen_range(MIN_H_SPEED..=MAX_H_SPEED);
            } else if self.h_turn_cooldown <= 0.0 && max_x > 0.0 {
                let px = match self.direction {
                    Direction::Right => (self.x / max_x).clamp(0.0, 1.0),
                    Direction::Left => (1.0 - self.x / max_x).clamp(0.0, 1.0),
                };
                let p_turn =
                    edge_weighted_turn_prob(px, dt_eff, BASE_H_TURN_RATE, EDGE_H_TURN_RATE);
                if rng.gen::<f32>() < p_turn {
                    self.direction = self.direction.flip();
                    self.h_turn_cooldown = rng.gen_range(MIN_TURN_COOLDOWN..=MAX_TURN_COOLDOWN);
                    self.speed = rng.gen_range(MIN_H_SPEED..=MAX_H_SPEED);
                }
            }
        } else {
            self.x = self.x.clamp(0.0, max_x);
        }

        let max_y = screen_height.saturating_sub(1) as f32;
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
                self.v_speed = rng.gen_range(MIN_V_SPEED..=MAX_V_SPEED);
            } else if self.v_direction == VerticalDirection::Up && self.y <= 0.0 {
                self.y = 0.0;
                self.v_direction = VerticalDirection::Down;
                self.v_turn_cooldown = rng.gen_range(MIN_TURN_COOLDOWN..=MAX_TURN_COOLDOWN);
                self.v_speed = rng.gen_range(MIN_V_SPEED..=MAX_V_SPEED);
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
                    self.v_speed = rng.gen_range(MIN_V_SPEED..=MAX_V_SPEED);
                }
            }
        } else {
            self.y = self.y.clamp(0.0, max_y);
        }
    }

    fn clamp_bounds(&mut self, screen_width: u16, screen_height: u16) {
        let max_x = (screen_width as f32 - self.width() as f32).max(0.0);
        let max_y = screen_height.saturating_sub(1) as f32;
        self.x = self.x.clamp(0.0, max_x);
        self.y = self.y.clamp(0.0, max_y);
    }
}

/// Aquarium world state managing dimensions and active swimming fish.
#[derive(Debug, Clone)]
pub struct Aquarium {
    width: u16,
    height: u16,
    fish: Vec<Fish>,
    target_fish_count: usize,
    speed_scale: f32,
}

impl Aquarium {
    /// Create a new aquarium with the given terminal dimensions, optional fixed fish count, and speed scale.
    pub fn new<R: Rng + ?Sized>(
        width: u16,
        height: u16,
        fish_count: Option<usize>,
        speed_scale: f32,
        rng: &mut R,
    ) -> Self {
        let target_fish_count = match fish_count {
            None => Self::auto_scaled_count(width, height),
            Some(n) => n.min(MAX_FISH_COUNT),
        };

        let mut fish = Vec::new();
        if width > 0 && height > 0 {
            fish.reserve(target_fish_count);
            for _ in 0..target_fish_count {
                fish.push(Fish::spawn_initial(width, height, rng));
            }
        }

        Self {
            width,
            height,
            fish,
            target_fish_count,
            speed_scale,
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
        }
    }

    fn auto_scaled_count(width: u16, height: u16) -> usize {
        ((width as usize * height as usize) / AUTO_SCALE_CELLS_PER_FISH)
            .clamp(AUTO_SCALE_MIN_FISH, AUTO_SCALE_MAX_FISH)
    }

    /// Current aquarium width in terminal columns.
    pub fn width(&self) -> u16 {
        self.width
    }

    /// Current aquarium height in terminal rows.
    pub fn height(&self) -> u16 {
        self.height
    }

    /// Slice of all active fish in the aquarium.
    pub fn fish(&self) -> &[Fish] {
        &self.fish
    }

    /// Number of currently active fish in the aquarium.
    pub fn fish_count(&self) -> usize {
        self.fish.len()
    }

    /// Advance the simulation by `dt` seconds while keeping all fish permanently inside bounds.
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
    }

    /// Handle terminal resize by clamping existing fish into the new bounds without adding or removing fish.
    pub fn resize<R: Rng + ?Sized>(&mut self, width: u16, height: u16, rng: &mut R) {
        self.width = width;
        self.height = height;

        if width == 0 || height == 0 {
            return;
        }

        if self.fish.is_empty() && self.target_fish_count > 0 {
            self.fish.reserve(self.target_fish_count);
            for _ in 0..self.target_fish_count {
                self.fish.push(Fish::spawn_initial(width, height, rng));
            }
            return;
        }

        for fish in &mut self.fish {
            fish.clamp_bounds(width, height);
        }
    }

    /// Manually add a fish inside the terminal bounds (up to 200).
    pub fn add_fish<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        if self.width > 0 && self.height > 0 {
            if self.fish.len() < MAX_FISH_COUNT {
                self.fish
                    .push(Fish::spawn_initial(self.width, self.height, rng));
                self.target_fish_count = self.fish.len();
            }
        } else if self.fish.is_empty() && self.target_fish_count < MAX_FISH_COUNT {
            self.target_fish_count += 1;
        }
    }

    /// Manually remove a fish (keeping at least 1 when non-empty).
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
        // (80 * 24) / 220 = 1920 / 220 = 8
        assert_eq!(aq.fish_count(), 8);
        assert_eq!(aq.width(), 80);
        assert_eq!(aq.height(), 24);

        let large_aq = Aquarium::new(80, 24, Some(100), 1.0, &mut rng);
        assert_eq!(large_aq.fish_count(), 100);

        for fish in aq.fish().iter().chain(large_aq.fish().iter()) {
            let max_x = (80 - fish.width()) as f32;
            assert!(
                fish.x() >= 0.0 && fish.x() <= max_x,
                "fish x={} must be in 0.0..={max_x}",
                fish.x()
            );
            assert!(
                fish.y() >= 0.0 && fish.y() <= 23.0,
                "fish y={} must be in 0.0..=23.0",
                fish.y()
            );
            assert!(fish.width() >= 3);
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
        assert_eq!(aq.fish_count(), 8);

        let initial_identities: Vec<(FishColor, FishSprite)> =
            aq.fish().iter().map(|f| (f.color(), f.sprite)).collect();

        for _ in 0..600 {
            aq.update(0.05, &mut rng);
            assert_eq!(aq.fish_count(), 8);
            for (i, fish) in aq.fish().iter().enumerate() {
                let max_x = (aq.width() as usize - fish.width()) as f32;
                assert!(fish.x() >= 0.0 && fish.x() <= max_x);
                assert!(fish.y() >= 0.0 && fish.y() <= (aq.height() - 1) as f32);
                assert_eq!((fish.color(), fish.sprite), initial_identities[i]);
            }
        }

        // Resize larger: population and identities must not change
        aq.resize(160, 40, &mut rng);
        assert_eq!(aq.width(), 160);
        assert_eq!(aq.height(), 40);
        assert_eq!(aq.fish_count(), 8);

        for _ in 0..100 {
            aq.update(0.05, &mut rng);
        }

        // Resize smaller: population and identities must not change, positions clamped in bounds
        aq.resize(30, 8, &mut rng);
        assert_eq!(aq.width(), 30);
        assert_eq!(aq.height(), 8);
        assert_eq!(aq.fish_count(), 8);

        for (i, fish) in aq.fish().iter().enumerate() {
            let max_x = (30 - fish.width()) as f32;
            assert!(
                fish.x() >= 0.0 && fish.x() <= max_x,
                "fish x={} out of shrunk bounds 0..={max_x}",
                fish.x()
            );
            assert!(
                fish.y() >= 0.0 && fish.y() <= 7.0,
                "fish y={} out of shrunk bounds 0..=7",
                fish.y()
            );
            assert_eq!((fish.color(), fish.sprite), initial_identities[i]);
        }
    }

    #[test]
    fn horizontal_wall_bounces_clamp_and_flip_direction_and_sprite() {
        let mut rng = StdRng::seed_from_u64(99);
        // Screen width 40, sprite "><>" has width 3 => max_x = 37.0
        let right_fish = Fish::from_parts(36.0, 4.0, Direction::Right, 10.0, FishColor::Green, "><>");
        let left_fish = Fish::from_parts(1.0, 6.0, Direction::Left, 10.0, FishColor::Magenta, "<'))><");
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
                assert!(
                    fish.y() >= 0.0 && fish.y() <= 23.0,
                    "fish y={} out of 0.0..=23.0 bounds",
                    fish.y()
                );
                min_y[i] = min_y[i].min(fish.y());
                max_y[i] = max_y[i].max(fish.y());
            }
        }

        for i in 0..count {
            let span = max_y[i] - min_y[i];
            assert!(
                span >= 8.0,
                "fish #{i} should roam across a wide vertical span (>= 8 rows), got span={span} ({}..={})",
                min_y[i],
                max_y[i]
            );
        }
    }

    #[test]
    fn edge_weighted_turn_probability_is_monotonic_and_triggers_near_edges() {
        assert_eq!(
            edge_weighted_turn_prob(0.5, 0.0, BASE_H_TURN_RATE, EDGE_H_TURN_RATE),
            0.0
        );
        assert_eq!(
            edge_weighted_turn_prob(0.5, -0.5, BASE_H_TURN_RATE, EDGE_H_TURN_RATE),
            0.0
        );
        assert_eq!(
            edge_weighted_turn_prob(0.5, f32::NAN, BASE_H_TURN_RATE, EDGE_H_TURN_RATE),
            0.0
        );
        assert_eq!(
            edge_weighted_turn_prob(0.5, f32::INFINITY, BASE_H_TURN_RATE, EDGE_H_TURN_RATE),
            0.0
        );

        let p_low = edge_weighted_turn_prob(0.1, 0.1, BASE_H_TURN_RATE, EDGE_H_TURN_RATE);
        let p_mid = edge_weighted_turn_prob(0.5, 0.1, BASE_H_TURN_RATE, EDGE_H_TURN_RATE);
        let p_high = edge_weighted_turn_prob(0.9, 0.1, BASE_H_TURN_RATE, EDGE_H_TURN_RATE);
        let p_edge = edge_weighted_turn_prob(1.0, 0.1, BASE_H_TURN_RATE, EDGE_H_TURN_RATE);

        assert!(p_low > 0.0 && p_low < 0.01, "center turn prob should be very low: {p_low}");
        assert!(p_low < p_mid, "expected p(0.1) < p(0.5): {p_low} vs {p_mid}");
        assert!(p_mid < p_high, "expected p(0.5) < p(0.9): {p_mid} vs {p_high}");
        assert!(p_high < p_edge, "expected p(0.9) < p(1.0): {p_high} vs {p_edge}");
        assert!(
            p_high > p_low * 15.0,
            "near-edge turn prob ({p_high}) should be much larger than near-center ({p_low})"
        );

        // Statistical horizontal check over 1000 single-step trials (fish stops well short of wall max_x=97)
        let mut rng = StdRng::seed_from_u64(314159);
        let mut center_h_turns = 0usize;
        let mut edge_h_turns = 0usize;

        for _ in 0..1000 {
            let center_fish = Fish::with_motion(
                20.0,
                10.0,
                Direction::Right,
                5.0,
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
                5.0,
                VerticalDirection::Down,
                0.0,
                0.0,
                1.0,
                FishColor::Cyan,
                "><>",
            );
            let mut aq = Aquarium::from_fish(100, 24, vec![center_fish, edge_fish], 1.0);
            aq.update(0.1, &mut rng);

            // Neither fish reached max_x = 97.0 (x is 20.5 and 93.5)
            assert!(aq.fish()[0].x() < 97.0);
            assert!(aq.fish()[1].x() < 97.0);

            if aq.fish()[0].direction() == Direction::Left {
                center_h_turns += 1;
            }
            if aq.fish()[1].direction() == Direction::Left {
                edge_h_turns += 1;
            }
        }

        assert!(
            edge_h_turns > 60,
            "expected many pre-wall turns near horizontal edge, got {edge_h_turns}"
        );
        assert!(
            edge_h_turns > center_h_turns * 5,
            "expected near-edge horizontal turns ({edge_h_turns}) >> center turns ({center_h_turns})"
        );

        // Statistical vertical check over 1000 single-step trials (fish stops short of max_y=23)
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
    fn add_and_remove_fish_respect_bounds_and_limits() {
        let mut rng = StdRng::seed_from_u64(555);
        let mut aq = Aquarium::new(80, 24, Some(2), 1.0, &mut rng);
        assert_eq!(aq.fish_count(), 2);

        aq.add_fish(&mut rng);
        assert_eq!(aq.fish_count(), 3);
        for fish in aq.fish() {
            let max_x = (80 - fish.width()) as f32;
            assert!(fish.x() >= 0.0 && fish.x() <= max_x);
            assert!(fish.y() >= 0.0 && fish.y() <= 23.0);
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
        assert_eq!(roundtrip.fish_count(), 8);
        let snapshot: Vec<(f32, f32, FishColor, &'static str)> = roundtrip
            .fish()
            .iter()
            .map(|f| (f.x(), f.y(), f.color(), f.sprite()))
            .collect();

        roundtrip.resize(0, 0, &mut rng);
        assert_eq!(roundtrip.fish_count(), 8);
        roundtrip.update(0.5, &mut rng);
        roundtrip.resize(80, 24, &mut rng);
        assert_eq!(roundtrip.fish_count(), 8);

        let after: Vec<(f32, f32, FishColor, &'static str)> = roundtrip
            .fish()
            .iter()
            .map(|f| (f.x(), f.y(), f.color(), f.sprite()))
            .collect();
        assert_eq!(after, snapshot);
    }
}
