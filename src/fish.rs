use std::f32::consts::TAU;

use rand::Rng;

const MAX_FISH_COUNT: usize = 200;
const AUTO_SCALE_CELLS_PER_FISH: usize = 220;
const AUTO_SCALE_MIN_FISH: usize = 4;
const AUTO_SCALE_MAX_FISH: usize = 24;
const MIN_SWIM_SPEED: f32 = 4.0;
const MAX_SWIM_SPEED: f32 = 14.0;
const MIN_BOB_AMPLITUDE: f32 = 0.15;
const MAX_BOB_AMPLITUDE: f32 = 0.65;
const MIN_BOB_FREQ: f32 = 1.2;
const MAX_BOB_FREQ: f32 = 3.2;

/// Horizontal swimming direction of a fish.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
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

/// A single swimming fish inside the aquarium.
#[derive(Debug, Clone)]
pub struct Fish {
    x: f32,
    y: f32,
    base_y: f32,
    direction: Direction,
    speed: f32,
    color: FishColor,
    sprite: &'static str,
    bob_amplitude: f32,
    bob_freq: f32,
    bob_phase: f32,
    elapsed: f32,
}

impl Fish {
    /// Construct a fish from explicit state with zero vertical bobbing by default.
    pub fn from_parts(
        x: f32,
        y: f32,
        direction: Direction,
        speed: f32,
        color: FishColor,
        sprite: &'static str,
    ) -> Self {
        Self {
            x,
            y,
            base_y: y,
            direction,
            speed: speed.abs(),
            color,
            sprite,
            bob_amplitude: 0.0,
            bob_freq: 0.0,
            bob_phase: 0.0,
            elapsed: 0.0,
        }
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

    /// Construct a fish with custom bobbing parameters.
    #[allow(clippy::too_many_arguments)]
    pub fn with_bobbing(
        x: f32,
        base_y: f32,
        direction: Direction,
        speed: f32,
        color: FishColor,
        sprite: &'static str,
        bob_amplitude: f32,
        bob_freq: f32,
        bob_phase: f32,
    ) -> Self {
        Self {
            x,
            y: base_y,
            base_y,
            direction,
            speed: speed.abs(),
            color,
            sprite,
            bob_amplitude,
            bob_freq,
            bob_phase,
            elapsed: 0.0,
        }
    }

    /// Spawn a fish distributed across the interior of the screen (`0..screen_width`, `0..screen_height`).
    pub fn spawn_initial<R: Rng + ?Sized>(
        screen_width: u16,
        screen_height: u16,
        rng: &mut R,
    ) -> Self {
        let mut fish = Self::spawn_common(screen_height, rng);
        fish.x = if screen_width <= 1 {
            0.0
        } else {
            rng.gen_range(0.0..(screen_width as f32))
        };
        fish
    }

    /// Spawn a fish just outside the left or right edge, ready to swim into the screen.
    pub fn spawn_edge<R: Rng + ?Sized>(screen_width: u16, screen_height: u16, rng: &mut R) -> Self {
        let mut fish = Self::spawn_common(screen_height, rng);
        fish.x = match fish.direction {
            Direction::Right => -(fish.width() as f32),
            Direction::Left => screen_width as f32,
        };
        fish
    }

    fn spawn_common<R: Rng + ?Sized>(screen_height: u16, rng: &mut R) -> Self {
        let direction = if rng.gen_bool(0.5) {
            Direction::Right
        } else {
            Direction::Left
        };
        let sprite_entry = &SMALL_FISH_SPRITES[rng.gen_range(0..SMALL_FISH_SPRITES.len())];
        let sprite = match direction {
            Direction::Right => sprite_entry.right,
            Direction::Left => sprite_entry.left,
        };
        let color = FishColor::ALL[rng.gen_range(0..FishColor::ALL.len())];
        let speed = rng.gen_range(MIN_SWIM_SPEED..=MAX_SWIM_SPEED);
        let max_y = screen_height.saturating_sub(1) as f32;
        let base_y = if max_y <= 0.0 {
            0.0
        } else {
            rng.gen_range(0.0..=max_y)
        };
        let bob_amplitude = if screen_height <= 1 {
            0.0
        } else {
            rng.gen_range(MIN_BOB_AMPLITUDE..=MAX_BOB_AMPLITUDE)
        };
        let bob_freq = rng.gen_range(MIN_BOB_FREQ..=MAX_BOB_FREQ);
        let bob_phase = rng.gen_range(0.0..TAU);
        let y = (base_y + bob_amplitude * bob_phase.sin()).clamp(0.0, max_y);

        Self {
            x: 0.0,
            y,
            base_y,
            direction,
            speed,
            color,
            sprite,
            bob_amplitude,
            bob_freq,
            bob_phase,
            elapsed: 0.0,
        }
    }

    /// Current horizontal position in terminal cell coordinates.
    pub fn x(&self) -> f32 {
        self.x
    }

    /// Current vertical position in terminal row coordinates (`0.0..height`).
    pub fn y(&self) -> f32 {
        self.y
    }

    /// Current horizontal swimming direction.
    pub fn direction(&self) -> Direction {
        self.direction
    }

    /// Assigned display color for this fish.
    pub fn color(&self) -> FishColor {
        self.color
    }

    /// Single-line ASCII sprite string matching the fish's swimming direction.
    pub fn sprite(&self) -> &'static str {
        self.sprite
    }

    /// Width of the fish's ASCII sprite in terminal columns.
    pub fn width(&self) -> usize {
        self.sprite.len()
    }

    /// Whether the fish has swum completely past the screen edge in its direction of travel.
    pub fn is_offscreen(&self, screen_width: u16) -> bool {
        match self.direction {
            Direction::Left => self.x + (self.width() as f32) <= 0.0,
            Direction::Right => self.x >= (screen_width as f32),
        }
    }

    fn advance(&mut self, dt: f32, speed_scale: f32, screen_height: u16) {
        let vx = match self.direction {
            Direction::Right => self.speed,
            Direction::Left => -self.speed,
        };
        self.x += vx * dt * speed_scale;
        self.elapsed += dt * speed_scale;
        let max_y = screen_height.saturating_sub(1) as f32;
        let bob_offset = self.bob_amplitude * (self.elapsed * self.bob_freq + self.bob_phase).sin();
        self.y = (self.base_y + bob_offset).clamp(0.0, max_y);
    }

    fn clamp_vertical(&mut self, screen_height: u16) {
        let max_y = screen_height.saturating_sub(1) as f32;
        self.base_y = self.base_y.clamp(0.0, max_y);
        let bob_offset = self.bob_amplitude * (self.elapsed * self.bob_freq + self.bob_phase).sin();
        self.y = (self.base_y + bob_offset).clamp(0.0, max_y);
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
    auto_scale: bool,
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
        let (auto_scale, target_fish_count) = match fish_count {
            None => (true, Self::auto_scaled_count(width, height)),
            Some(n) => (false, n.min(MAX_FISH_COUNT)),
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
            auto_scale,
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
            auto_scale: false,
        }
    }

    fn auto_scaled_count(width: u16, height: u16) -> usize {
        if width == 0 || height == 0 {
            0
        } else {
            ((width as usize * height as usize) / AUTO_SCALE_CELLS_PER_FISH)
                .clamp(AUTO_SCALE_MIN_FISH, AUTO_SCALE_MAX_FISH)
        }
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

    /// Advance the simulation by `dt` seconds, respawning any fish that swim past the screen edges.
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
            fish.advance(dt, scale, self.height);
            if fish.is_offscreen(self.width) {
                *fish = Fish::spawn_edge(self.width, self.height, rng);
            }
        }
    }

    /// Handle terminal resize, clamping existing fish rows and adjusting fish population as appropriate.
    pub fn resize<R: Rng + ?Sized>(&mut self, width: u16, height: u16, rng: &mut R) {
        let was_zero = self.width == 0 || self.height == 0;
        self.width = width;
        self.height = height;

        if width == 0 || height == 0 {
            if self.auto_scale {
                self.target_fish_count = 0;
            }
            self.fish.clear();
            return;
        }

        for fish in &mut self.fish {
            fish.clamp_vertical(height);
        }

        if self.auto_scale {
            self.target_fish_count = Self::auto_scaled_count(width, height);
            if self.fish.len() > self.target_fish_count {
                self.fish.truncate(self.target_fish_count);
            } else {
                while self.fish.len() < self.target_fish_count {
                    let spawned = if was_zero {
                        Fish::spawn_initial(width, height, rng)
                    } else {
                        Fish::spawn_edge(width, height, rng)
                    };
                    self.fish.push(spawned);
                }
            }
        } else if was_zero {
            while self.fish.len() < self.target_fish_count {
                self.fish.push(Fish::spawn_initial(width, height, rng));
            }
        }
    }

    /// Manually add a fish entering from a screen edge (up to 200), disabling automatic population scaling.
    pub fn add_fish<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        self.auto_scale = false;
        if self.target_fish_count < MAX_FISH_COUNT {
            self.target_fish_count += 1;
            if self.width > 0 && self.height > 0 && self.fish.len() < MAX_FISH_COUNT {
                self.fish.push(Fish::spawn_edge(self.width, self.height, rng));
            }
        }
    }

    /// Manually remove a fish (keeping at least 1 when non-empty), disabling automatic population scaling.
    pub fn remove_fish(&mut self) {
        self.auto_scale = false;
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
    fn initial_spawning_populates_within_bounds() {
        let mut rng = StdRng::seed_from_u64(42);
        let aq = Aquarium::new(80, 24, None, 1.0, &mut rng);
        // (80 * 24) / 220 = 1920 / 220 = 8
        assert_eq!(aq.fish_count(), 8);
        assert_eq!(aq.width(), 80);
        assert_eq!(aq.height(), 24);

        for fish in aq.fish() {
            assert!(fish.x() >= 0.0 && fish.x() < 80.0);
            assert!(fish.y() >= 0.0 && fish.y() <= 23.0);
            assert!(fish.width() >= 3);
            assert!(!fish.is_offscreen(80));
        }
    }

    #[test]
    fn horizontal_movement_respects_direction_and_speed_scale() {
        let mut rng = StdRng::seed_from_u64(7);
        let right_fish = Fish::from_parts(10.0, 5.0, Direction::Right, 6.0, FishColor::Cyan, "><>");
        let left_fish = Fish::from_parts(20.0, 8.0, Direction::Left, 4.0, FishColor::Yellow, "<><");
        let mut aq = Aquarium::from_fish(80, 24, vec![right_fish, left_fish], 2.0);

        aq.update(0.5, &mut rng);

        assert!((aq.fish()[0].x() - 16.0).abs() < 1e-5);
        assert!((aq.fish()[1].x() - 16.0).abs() < 1e-5);
    }

    #[test]
    fn offscreen_fish_respawn_from_screen_edges() {
        let mut rng = StdRng::seed_from_u64(99);
        // Right-moving fish near right edge (width 40) and left-moving fish near left edge
        let exiting_right =
            Fish::from_parts(39.5, 4.0, Direction::Right, 10.0, FishColor::Green, "><>");
        let exiting_left =
            Fish::from_parts(-2.5, 6.0, Direction::Left, 10.0, FishColor::Magenta, "<><");
        let mut aq = Aquarium::from_fish(40, 12, vec![exiting_right, exiting_left], 1.0);

        aq.update(0.2, &mut rng);
        assert_eq!(aq.fish_count(), 2);

        for fish in aq.fish() {
            match fish.direction() {
                Direction::Right => {
                    assert!(
                        (fish.x() - -(fish.width() as f32)).abs() < 1e-5,
                        "respawned right-moving fish should start at -(width)"
                    );
                }
                Direction::Left => {
                    assert!(
                        (fish.x() - 40.0).abs() < 1e-5,
                        "respawned left-moving fish should start at screen_width"
                    );
                }
            }
            assert!(!fish.is_offscreen(40));
            assert!(fish.y() >= 0.0 && fish.y() <= 11.0);
        }
    }

    #[test]
    fn vertical_bobbing_stays_within_terminal_bounds() {
        let mut rng = StdRng::seed_from_u64(123);
        let top_fish = Fish::with_bobbing(
            5.0,
            0.0,
            Direction::Right,
            5.0,
            FishColor::Cyan,
            "><>",
            0.8,
            2.5,
            0.0,
        );
        let mid_fish = Fish::with_bobbing(
            5.0,
            5.0,
            Direction::Right,
            5.0,
            FishColor::Yellow,
            "><>",
            0.6,
            2.0,
            0.0,
        );
        let bottom_fish = Fish::with_bobbing(
            5.0,
            9.0,
            Direction::Right,
            5.0,
            FishColor::Red,
            "><>",
            0.8,
            2.5,
            0.0,
        );
        let mut aq = Aquarium::from_fish(200, 10, vec![top_fish, mid_fish, bottom_fish], 1.0);

        let mut mid_y_changed = false;
        for _ in 0..40 {
            aq.update(0.1, &mut rng);
            for fish in aq.fish() {
                assert!(
                    fish.y() >= 0.0 && fish.y() <= 9.0,
                    "fish y={} out of 0..10 bounds",
                    fish.y()
                );
            }
            if (aq.fish()[1].y() - 5.0).abs() > 0.05 {
                mid_y_changed = true;
            }
        }
        assert!(mid_y_changed, "middle fish should bob vertically over time");
    }

    #[test]
    fn resize_clamps_positions_and_autoscales_population() {
        let mut rng = StdRng::seed_from_u64(2024);
        let mut aq = Aquarium::new(80, 24, None, 1.0, &mut rng);
        assert_eq!(aq.fish_count(), 8);

        // Grow terminal: (160 * 40) / 220 = 29 -> clamped to 24
        aq.resize(160, 40, &mut rng);
        assert_eq!(aq.width(), 160);
        assert_eq!(aq.height(), 40);
        assert_eq!(aq.fish_count(), 24);

        // Shrink terminal to tiny height: (40 * 3) / 220 = 0 -> clamped to 4
        aq.resize(40, 3, &mut rng);
        assert_eq!(aq.fish_count(), 4);
        for fish in aq.fish() {
            assert!(fish.y() >= 0.0 && fish.y() <= 2.0);
        }
    }

    #[test]
    fn add_and_remove_fish_disable_autoscale_and_enforce_limits() {
        let mut rng = StdRng::seed_from_u64(555);
        let mut aq = Aquarium::new(80, 24, Some(2), 1.0, &mut rng);
        assert_eq!(aq.fish_count(), 2);

        aq.add_fish(&mut rng);
        assert_eq!(aq.fish_count(), 3);

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

        // Resizing after add/remove must not auto-scale the user-adjusted count
        aq.resize(160, 50, &mut rng);
        assert_eq!(aq.fish_count(), 1);
    }

    #[test]
    fn handles_zero_and_one_by_one_dimensions_and_bad_dt_safely() {
        let mut rng = StdRng::seed_from_u64(777);

        // 0x0 with explicit fish count restores once resized to non-zero
        let mut aq = Aquarium::new(0, 0, Some(6), 1.0, &mut rng);
        assert_eq!(aq.fish_count(), 0);
        aq.update(0.1, &mut rng);
        aq.remove_fish();
        assert_eq!(aq.fish_count(), 0);

        aq.resize(1, 1, &mut rng);
        assert_eq!(aq.fish_count(), 6);
        for fish in aq.fish() {
            assert_eq!(fish.y(), 0.0);
        }

        // Stepping on 1x1 and non-positive / NaN dt should never panic
        aq.update(0.0, &mut rng);
        aq.update(-1.0, &mut rng);
        aq.update(f32::NAN, &mut rng);
        aq.update(f32::INFINITY, &mut rng);
        aq.update(0.25, &mut rng);
        for fish in aq.fish() {
            assert_eq!(fish.y(), 0.0);
        }

        // Resizing back to 0 clears safely
        aq.resize(0, 10, &mut rng);
        assert_eq!(aq.fish_count(), 0);
    }
}
