use std::f32::consts::TAU;

use rand::Rng;

use crate::fish::FishColor;

/// Multi-line ASCII sprite definition for a seabed coral formation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoralSprite {
    pub name: &'static str,
    pub lines: &'static [&'static str],
}

/// Catalog of recognizable multi-line ASCII coral reef species.
pub const CORAL_SPECIES: &[CoralSprite] = &[
    CoralSprite {
        name: "staghorn",
        lines: &[
            " \\  \\|/  / ",
            "  \\_\\|/_/  ",
            "    \\|/    ",
            "  __/|\\__  ",
        ],
    },
    CoralSprite {
        name: "brain",
        lines: &[
            "  .~~~~~.  ",
            " (~-~-~-~) ",
            "(~_~_~_~_~)",
        ],
    },
    CoralSprite {
        name: "sea_fan",
        lines: &[
            " .\\*|*/. ",
            "\\*\\\\|//*/",
            "  \\\\|//  ",
            "  _/|\\_  ",
        ],
    },
    CoralSprite {
        name: "tube",
        lines: &[
            "  o  O  o  ",
            " (|)(|)(|) ",
            "/|||||||||\\",
        ],
    },
];

const REEF_COLORS: &[FishColor] = &[
    FishColor::Magenta,
    FishColor::BrightMagenta,
    FishColor::Red,
    FishColor::BrightRed,
    FishColor::Yellow,
    FishColor::BrightYellow,
    FishColor::Cyan,
    FishColor::BrightCyan,
];

const KELP_COLORS: &[FishColor] = &[
    FishColor::Green,
    FishColor::Green,
    FishColor::BrightGreen,
    FishColor::BrightGreen,
    FishColor::Cyan,
];

const MIN_SCENERY_WIDTH: u16 = 16;
const MIN_SCENERY_HEIGHT: u16 = 6;

fn clamp_ratio(ratio: f32) -> f32 {
    if ratio.is_finite() {
        ratio.clamp(0.0, 1.0)
    } else {
        0.5
    }
}

/// A seabed coral formation anchored proportionally along the bottom of the tank.
#[derive(Debug, Clone)]
pub struct Coral {
    x_ratio: f32,
    sprite: CoralSprite,
    color: FishColor,
}

impl Coral {
    /// Construct a coral formation at normalized horizontal position `x_ratio` (`0.0..=1.0`).
    pub fn new(x_ratio: f32, sprite: CoralSprite, color: FishColor) -> Self {
        Self {
            x_ratio: clamp_ratio(x_ratio),
            sprite,
            color,
        }
    }

    /// Underlying coral species sprite.
    pub fn sprite(&self) -> CoralSprite {
        self.sprite
    }

    /// Multi-line ASCII rows of this coral sprite.
    pub fn lines(&self) -> &'static [&'static str] {
        self.sprite.lines
    }

    /// Width of the coral formation in terminal columns.
    pub fn width(&self) -> usize {
        self.sprite.lines.first().map_or(0, |line| line.len())
    }

    /// Height of the coral formation in terminal rows.
    pub fn height(&self) -> usize {
        self.sprite.lines.len()
    }

    /// Assigned reef display color.
    pub fn color(&self) -> FishColor {
        self.color
    }

    /// Leftmost column where this coral anchors for `screen_width`, staying within `0..=(screen_width - width)`.
    pub fn anchor_col(&self, screen_width: u16) -> i32 {
        ((screen_width as f32 - self.width() as f32).max(0.0) * self.x_ratio).round() as i32
    }

    /// Top row where this coral starts so its base sits flush on `screen_height - 1`.
    pub fn top_row(&self, screen_height: u16) -> i32 {
        screen_height as i32 - self.height() as i32
    }
}

/// A single swaying kelp/seaweed stalk anchored to the seabed.
#[derive(Debug, Clone)]
pub struct Seaweed {
    x_ratio: f32,
    height: usize,
    color: FishColor,
    sway_phase: f32,
    sway_speed: f32,
    elapsed: f32,
}

impl Seaweed {
    /// Construct a seaweed stalk anchored at `x_ratio` (`0.0..=1.0`) with height `>= 1`.
    pub fn new(
        x_ratio: f32,
        height: usize,
        color: FishColor,
        sway_phase: f32,
        sway_speed: f32,
    ) -> Self {
        Self {
            x_ratio: clamp_ratio(x_ratio),
            height: height.max(1),
            color,
            sway_phase: if sway_phase.is_finite() {
                sway_phase
            } else {
                0.0
            },
            sway_speed: if sway_speed.is_finite() {
                sway_speed.max(0.0)
            } else {
                1.5
            },
            elapsed: 0.0,
        }
    }

    /// Assigned kelp display color.
    pub fn color(&self) -> FishColor {
        self.color
    }

    /// Uncapped stalk height in terminal rows.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Advance the sway animation clock by `dt` seconds.
    pub fn advance(&mut self, dt: f32) {
        if dt.is_finite() && dt > 0.0 {
            self.elapsed += dt;
        }
    }

    /// Compute `(col, row, ch)` cells from the bottom seabed root upward for the given screen size.
    pub fn cells(&self, screen_width: u16, screen_height: u16) -> Vec<(i32, i32, char)> {
        if screen_width == 0 || screen_height == 0 || self.height == 0 {
            return Vec::new();
        }

        let eff_h = self.height.min(((screen_height as usize) / 2).max(1));
        let base_col = ((screen_width.saturating_sub(1) as f32) * self.x_ratio).round() as i32;
        let max_col = screen_width as i32 - 1;
        let mut out = Vec::with_capacity(eff_h);

        for i in 0..eff_h {
            let row = screen_height as i32 - 1 - i as i32;
            let wave = (self.elapsed * self.sway_speed + self.sway_phase + i as f32 * 0.7).sin();

            let offset = if i == 0 {
                0
            } else {
                let frac = i as f32 / eff_h as f32;
                (wave * frac * 1.35).round().clamp(-1.0, 1.0) as i32
            };

            let col = (base_col + offset).clamp(0, max_col);
            let ch = if i == eff_h - 1 && eff_h >= 3 {
                if wave.abs() < 0.35 {
                    '~'
                } else if wave >= 0.0 {
                    ')'
                } else {
                    '('
                }
            } else if (wave >= 0.0) ^ (i % 2 == 1) {
                ')'
            } else {
                '('
            };

            out.push((col, row, ch));
        }

        out
    }
}

/// Collection of seabed coral reefs and swaying kelp stalks.
#[derive(Debug, Clone, Default)]
pub struct Scenery {
    corals: Vec<Coral>,
    seaweed: Vec<Seaweed>,
    initialized: bool,
}

impl Scenery {
    /// Construct empty uninitialized scenery (no corals or seaweed).
    pub fn empty() -> Self {
        Self {
            corals: Vec::new(),
            seaweed: Vec::new(),
            initialized: false,
        }
    }

    /// Construct scenery directly from explicit coral and seaweed lists.
    pub fn from_parts(corals: Vec<Coral>, seaweed: Vec<Seaweed>) -> Self {
        Self {
            corals,
            seaweed,
            initialized: true,
        }
    }

    /// Procedurally generate non-overlapping coral formations and clustered swaying seaweed for `width` x `height`.
    pub fn generate<R: Rng + ?Sized>(width: u16, height: u16, rng: &mut R) -> Self {
        if width < MIN_SCENERY_WIDTH || height < MIN_SCENERY_HEIGHT {
            return Self::empty();
        }

        let max_non_overlapping = (width as usize / 12).max(1);
        let coral_count = (width as usize / 24).clamp(2, 7).min(max_non_overlapping);
        let mut corals = Vec::with_capacity(coral_count);
        let species_offset = rng.gen_range(0..CORAL_SPECIES.len());

        if coral_count == 1 {
            let sprite = CORAL_SPECIES[species_offset];
            let color = REEF_COLORS[rng.gen_range(0..REEF_COLORS.len())];
            let x_ratio = rng.gen_range(0.15..=0.85);
            corals.push(Coral::new(x_ratio, sprite, color));
        } else {
            let band_w = width as usize / coral_count;
            for i in 0..coral_count {
                let sprite = CORAL_SPECIES[(species_offset + i) % CORAL_SPECIES.len()];
                let sprite_w = sprite.lines.first().map_or(0, |l| l.len());
                let color = REEF_COLORS[rng.gen_range(0..REEF_COLORS.len())];

                let band_start = i * band_w;
                let band_end = if i + 1 == coral_count {
                    width as usize
                } else {
                    (i + 1) * band_w
                };
                let gap = usize::from(i + 1 < coral_count);
                let min_col = band_start;
                let max_col_in_band = band_end.saturating_sub(sprite_w + gap).max(min_col);
                let col = if min_col >= max_col_in_band {
                    min_col
                } else {
                    rng.gen_range(min_col..=max_col_in_band)
                };

                let max_anchor = (width as usize).saturating_sub(sprite_w);
                let x_ratio = if max_anchor == 0 {
                    0.0
                } else {
                    col as f32 / max_anchor as f32
                };
                corals.push(Coral::new(x_ratio, sprite, color));
            }
        }

        let stalk_count = (width as usize / 10).clamp(4, 16);
        let max_stalk_h = (height as usize / 3).clamp(3, 10);
        let mut seaweed = Vec::with_capacity(stalk_count);
        let cluster_count = (stalk_count / 2).max(1);
        let mut cluster_idx = 0usize;

        while seaweed.len() < stalk_count {
            let remaining = stalk_count - seaweed.len();
            let cluster_size = if remaining <= 3 {
                remaining
            } else {
                rng.gen_range(2..=3).min(remaining)
            };

            let band_span = 0.92 / cluster_count.max(1) as f32;
            let band_min = 0.04 + (cluster_idx % cluster_count) as f32 * band_span;
            let band_max = (band_min + band_span * 0.85).min(0.96);
            let cluster_x = if band_min >= band_max {
                band_min
            } else {
                rng.gen_range(band_min..=band_max)
            };
            cluster_idx += 1;

            let step_ratio = (1.5 / (width.saturating_sub(1).max(1)) as f32).max(0.012);
            for j in 0..cluster_size {
                let offset =
                    (j as f32 - (cluster_size - 1) as f32 * 0.5) * step_ratio * rng.gen_range(0.8..=1.3);
                let x_ratio = (cluster_x + offset).clamp(0.01, 0.99);
                let stalk_h = rng.gen_range(3..=max_stalk_h);
                let color = KELP_COLORS[rng.gen_range(0..KELP_COLORS.len())];
                let sway_phase = rng.gen_range(0.0..TAU);
                let sway_speed = rng.gen_range(1.2..=2.4);
                seaweed.push(Seaweed::new(x_ratio, stalk_h, color, sway_phase, sway_speed));
            }
        }

        Self {
            corals,
            seaweed,
            initialized: true,
        }
    }

    /// Advance seaweed sway animations by `dt` seconds.
    pub fn update(&mut self, dt: f32) {
        for stalk in &mut self.seaweed {
            stalk.advance(dt);
        }
    }

    /// Populate scenery once when transitioning into valid dimensions; otherwise preserve existing elements.
    pub fn resize<R: Rng + ?Sized>(&mut self, width: u16, height: u16, rng: &mut R) {
        if !self.initialized && width >= MIN_SCENERY_WIDTH && height >= MIN_SCENERY_HEIGHT {
            *self = Self::generate(width, height, rng);
        }
    }

    /// Slice of coral reef formations in the scenery.
    pub fn corals(&self) -> &[Coral] {
        &self.corals
    }

    /// Slice of swaying seaweed stalks in the scenery.
    pub fn seaweed(&self) -> &[Seaweed] {
        &self.seaweed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};
    use std::collections::HashSet;

    #[test]
    fn coral_species_are_distinct_uniform_width_printable_ascii() {
        assert!(
            CORAL_SPECIES.len() >= 4,
            "expected at least 4 coral species, found {}",
            CORAL_SPECIES.len()
        );

        let mut names = HashSet::new();
        for (idx, sprite) in CORAL_SPECIES.iter().enumerate() {
            assert!(!sprite.name.is_empty(), "coral #{idx} name must be non-empty");
            assert!(
                names.insert(sprite.name),
                "duplicate coral species name: {}",
                sprite.name
            );
            assert!(
                !sprite.lines.is_empty(),
                "coral '{}' must have at least one row",
                sprite.name
            );

            let width = sprite.lines[0].len();
            assert!(width > 0, "coral '{}' width must be > 0", sprite.name);

            for (row_idx, line) in sprite.lines.iter().enumerate() {
                assert!(
                    !line.is_empty(),
                    "coral '{}' row {row_idx} must be non-empty",
                    sprite.name
                );
                assert_eq!(
                    line.len(),
                    width,
                    "coral '{}' row {row_idx} width mismatch ({} vs {width})",
                    sprite.name,
                    line.len()
                );
                assert!(
                    line.bytes().all(|b| (0x20..=0x7E).contains(&b)),
                    "coral '{}' row {row_idx} must contain only printable ASCII",
                    sprite.name
                );
            }
        }
    }

    #[test]
    fn generated_corals_do_not_overlap_and_sit_flush_on_seabed() {
        let mut rng = StdRng::seed_from_u64(42);
        for (w, h) in [(80u16, 24u16), (160u16, 40u16)] {
            let scenery = Scenery::generate(w, h, &mut rng);
            assert!(!scenery.corals().is_empty(), "expected corals at {w}x{h}");
            assert!(!scenery.seaweed().is_empty(), "expected seaweed at {w}x{h}");

            let mut spans: Vec<(i32, i32)> = Vec::new();
            for coral in scenery.corals() {
                let col = coral.anchor_col(w);
                let cw = coral.width() as i32;
                let top = coral.top_row(h);
                let bottom = top + coral.height() as i32 - 1;

                assert!(
                    col >= 0 && col + cw <= w as i32,
                    "coral '{}' out of horizontal bounds at {w}x{h}: col={col}, cw={cw}",
                    coral.sprite().name
                );
                assert_eq!(
                    bottom,
                    h as i32 - 1,
                    "coral '{}' must sit flush on bottom row",
                    coral.sprite().name
                );
                spans.push((col, col + cw));
            }

            spans.sort_by_key(|&(start, _)| start);
            for pair in spans.windows(2) {
                assert!(
                    pair[0].1 <= pair[1].0,
                    "corals overlapped horizontally on {w}x{h}: {:?} vs {:?}",
                    pair[0],
                    pair[1]
                );
            }
        }
    }

    #[test]
    fn seaweed_anchors_root_at_bottom_sways_over_time_and_caps_height() {
        let mut stalk = Seaweed::new(0.5, 12, FishColor::Green, 0.0, 2.0);
        let initial = stalk.cells(80, 24);
        assert_eq!(initial.len(), 12);

        // Root at i = 0 is anchored at row = height - 1 and fixed base_col
        let expected_base_col = ((79.0f32) * 0.5).round() as i32;
        assert_eq!(initial[0].0, expected_base_col);
        assert_eq!(initial[0].1, 23);
        assert_eq!(initial[initial.len() - 1].1, 23 - 11);

        let mut changed = false;
        for _ in 0..20 {
            stalk.advance(0.15);
            let next = stalk.cells(80, 24);
            assert_eq!(next[0].0, expected_base_col, "root column must stay fixed");
            assert_eq!(next[0].1, 23, "root row must stay anchored on bottom");
            if next != initial {
                changed = true;
            }
        }
        assert!(changed, "seaweed cells should sway over time as advance(dt) is called");

        // Short terminal caps effective height to (screen_height / 2).max(1)
        let short = stalk.cells(80, 6);
        assert_eq!(short.len(), 3);
        assert_eq!(short[0].1, 5);
        assert_eq!(short[2].1, 3);

        // Very short terminal (height = 1) still renders a single anchored cell safely
        let tiny_h = stalk.cells(10, 1);
        assert_eq!(tiny_h.len(), 1);
        assert_eq!(tiny_h[0].1, 0);
    }

    #[test]
    fn tiny_dimensions_and_resizes_behave_safely() {
        let mut rng = StdRng::seed_from_u64(123);

        for (w, h) in [(0, 0), (1, 1), (12, 4)] {
            let empty_gen = Scenery::generate(w, h, &mut rng);
            assert!(empty_gen.corals().is_empty());
            assert!(empty_gen.seaweed().is_empty());

            let stalk = Seaweed::new(0.5, 5, FishColor::BrightGreen, 0.2, 1.5);
            let cells = stalk.cells(w, h);
            if w == 0 || h == 0 {
                assert!(cells.is_empty());
            } else {
                for (col, row, _) in cells {
                    assert!(col >= 0 && col < w as i32);
                    assert!(row >= 0 && row < h as i32);
                }
            }
        }

        // Resize from 0x0 -> 80x24 populates scenery once
        let mut scenery = Scenery::generate(0, 0, &mut rng);
        assert!(scenery.corals().is_empty());
        assert!(scenery.seaweed().is_empty());

        scenery.resize(80, 24, &mut rng);
        let coral_count = scenery.corals().len();
        let seaweed_count = scenery.seaweed().len();
        assert!(coral_count >= 2);
        assert!(seaweed_count >= 4);

        // Subsequent resizes preserve existing counts and keep anchors in bounds
        for (w, h) in [(120, 30), (40, 12)] {
            scenery.resize(w, h, &mut rng);
            assert_eq!(scenery.corals().len(), coral_count);
            assert_eq!(scenery.seaweed().len(), seaweed_count);

            for coral in scenery.corals() {
                let col = coral.anchor_col(w);
                assert!(col >= 0 && col + coral.width() as i32 <= w as i32);
                assert_eq!(coral.top_row(h) + coral.height() as i32 - 1, h as i32 - 1);
            }
            for stalk in scenery.seaweed() {
                for (col, row, _) in stalk.cells(w, h) {
                    assert!(col >= 0 && col < w as i32);
                    assert!(row >= 0 && row < h as i32);
                }
            }
        }
    }
}
