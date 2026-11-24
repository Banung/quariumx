use std::io::{self, Write};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

use crate::fish::{Aquarium, CreatureKind, FishColor};

/// RAII guard that manages terminal raw mode, alternate screen, and cursor visibility.
pub struct TerminalGuard;

impl TerminalGuard {
    /// Enter raw mode, switch to the alternate screen, hide the cursor, and clear the screen.
    ///
    /// If any setup step after enabling raw mode fails, previous changes are rolled back
    /// before returning the error.
    pub fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        let guard = Self;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, Hide, Clear(ClearType::All))?;
        Ok(guard)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore_terminal();
    }
}

fn restore_terminal() {
    let mut stdout = io::stdout();
    let _ = execute!(stdout, ResetColor, Show, LeaveAlternateScreen);
    let _ = disable_raw_mode();
}

/// Install a panic hook that restores terminal state before delegating to the previous hook.
pub fn install_panic_hook() {
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        restore_terminal();
        prev_hook(panic_info);
    }));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Cell {
    ch: char,
    color: Option<FishColor>,
}

impl Cell {
    const EMPTY: Self = Self {
        ch: ' ',
        color: None,
    };
}

/// Flicker-free double-buffered terminal renderer for the aquarium.
#[derive(Debug, Default)]
pub struct Renderer {
    width: u16,
    height: u16,
    prev_grid: Vec<Cell>,
    initialized: bool,
}

impl Renderer {
    /// Create a new renderer with an empty backbuffer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Render the current aquarium state to `out` using cell-diffing against the previous frame.
    pub fn render_to<W: Write>(
        &mut self,
        out: &mut W,
        aquarium: &Aquarium,
        color: bool,
    ) -> io::Result<()> {
        let width = aquarium.width();
        let height = aquarium.height();

        if width == 0 || height == 0 {
            self.width = width;
            self.height = height;
            self.prev_grid.clear();
            self.initialized = false;
            return Ok(());
        }

        let w_usize = width as usize;
        let h_usize = height as usize;
        let total_cells = w_usize * h_usize;
        let mut next_grid = vec![Cell::EMPTY; total_cells];

        for stalk in aquarium.scenery().seaweed() {
            let cell_color = if color { Some(stalk.color()) } else { None };
            for (col, row, ch) in stalk.cells(width, height) {
                if ch == ' ' {
                    continue;
                }
                if col >= 0 && col < width as i32 && row >= 0 && row < height as i32 {
                    let idx = (row as usize) * w_usize + (col as usize);
                    next_grid[idx] = Cell {
                        ch,
                        color: cell_color,
                    };
                }
            }
        }

        for coral in aquarium.scenery().corals() {
            let cell_color = if color { Some(coral.color()) } else { None };
            Self::draw_sprite_lines(
                &mut next_grid,
                width,
                height,
                coral.anchor_col(width),
                coral.top_row(height),
                coral.lines(),
                cell_color,
            );
        }

        const DRAW_ORDER: &[CreatureKind] = &[
            CreatureKind::LargeFish,
            CreatureKind::MediumFish,
            CreatureKind::Jellyfish,
            CreatureKind::SmallFish,
            CreatureKind::Crab,
        ];

        for &layer in DRAW_ORDER {
            for fish in aquarium.fish() {
                if fish.kind() != layer {
                    continue;
                }
                let start_x = fish.x().round() as i32;
                let start_y = fish.y().round() as i32;
                let cell_color = if color { Some(fish.color()) } else { None };

                Self::draw_sprite_lines(
                    &mut next_grid,
                    width,
                    height,
                    start_x,
                    start_y,
                    fish.sprite_lines(),
                    cell_color,
                );
            }
        }

        let size_changed =
            !self.initialized || self.width != width || self.height != height || self.prev_grid.len() != total_cells;

        if size_changed {
            queue!(out, Clear(ClearType::All))?;
        }

        let mut active_color: Option<FishColor> = None;
        let mut cursor_pos: Option<(u16, u16)> = None;

        for row in 0..height {
            let row_offset = (row as usize) * w_usize;
            for col in 0..width {
                let idx = row_offset + (col as usize);
                let cell = next_grid[idx];

                let should_draw = if size_changed {
                    cell != Cell::EMPTY
                } else {
                    cell != self.prev_grid[idx]
                };

                if !should_draw {
                    continue;
                }

                if color && cell.color != active_color {
                    match cell.color {
                        Some(fc) => queue!(out, SetForegroundColor(map_fish_color(fc)))?,
                        None => queue!(out, ResetColor)?,
                    }
                    active_color = cell.color;
                }

                if cursor_pos != Some((col, row)) {
                    queue!(out, MoveTo(col, row))?;
                }

                queue!(out, Print(cell.ch))?;
                cursor_pos = Some((col + 1, row));
            }
        }

        if color && active_color.is_some() {
            queue!(out, ResetColor)?;
        }

        out.flush()?;

        self.width = width;
        self.height = height;
        self.prev_grid = next_grid;
        self.initialized = true;

        Ok(())
    }

    fn draw_sprite_lines(
        next_grid: &mut [Cell],
        width: u16,
        height: u16,
        start_x: i32,
        start_y: i32,
        sprite_lines: &[&str],
        cell_color: Option<FishColor>,
    ) {
        let w_usize = width as usize;
        for (row_idx, line) in sprite_lines.iter().enumerate() {
            let row = start_y + row_idx as i32;
            if row < 0 || row >= height as i32 {
                continue;
            }
            for (col_idx, ch) in line.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let col = start_x + col_idx as i32;
                if col >= 0 && col < width as i32 {
                    let idx = (row as usize) * w_usize + (col as usize);
                    next_grid[idx] = Cell {
                        ch,
                        color: cell_color,
                    };
                }
            }
        }
    }
}

fn map_fish_color(color: FishColor) -> Color {
    match color {
        FishColor::Cyan => Color::DarkCyan,
        FishColor::Yellow => Color::DarkYellow,
        FishColor::Magenta => Color::DarkMagenta,
        FishColor::Green => Color::DarkGreen,
        FishColor::Blue => Color::DarkBlue,
        FishColor::Red => Color::DarkRed,
        FishColor::White => Color::White,
        FishColor::BrightCyan => Color::Cyan,
        FishColor::BrightYellow => Color::Yellow,
        FishColor::BrightMagenta => Color::Magenta,
        FishColor::BrightGreen => Color::Green,
        FishColor::BrightRed => Color::Red,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fish::{Direction, Fish};
    use crate::scenery::{Coral, Scenery, Seaweed, CORAL_SPECIES};
    use rand::{rngs::StdRng, SeedableRng};

    /// Apply monochrome ANSI output onto an existing 2D character grid (`height` rows x `width` cols).
    fn apply_ansi_screen(screen: &mut [Vec<char>], output: &str) {
        let h = screen.len();
        let w = screen.first().map_or(0, Vec::len);
        let mut cur_row = 0usize;
        let mut cur_col = 0usize;
        let bytes = output.as_bytes();
        let mut i = 0;

        while i < bytes.len() {
            if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
                i += 2;
                let start = i;
                while i < bytes.len() && !bytes[i].is_ascii_alphabetic() {
                    i += 1;
                }
                if i < bytes.len() {
                    let cmd = bytes[i] as char;
                    let params = &output[start..i];
                    i += 1;
                    if cmd == 'J' && params == "2" {
                        for row in screen.iter_mut() {
                            row.fill(' ');
                        }
                    } else if cmd == 'H' {
                        let mut parts = params.split(';');
                        let r1: usize = parts.next().and_then(|s| s.parse().ok()).unwrap_or(1);
                        let c1: usize = parts.next().and_then(|s| s.parse().ok()).unwrap_or(1);
                        cur_row = r1.saturating_sub(1);
                        cur_col = c1.saturating_sub(1);
                    }
                }
            } else {
                let ch = output[i..].chars().next().unwrap();
                i += ch.len_utf8();
                assert!(
                    cur_row < h && cur_col < w,
                    "ANSI output wrote character {ch:?} out of bounds at ({cur_col}, {cur_row}) on {w}x{h} screen"
                );
                screen[cur_row][cur_col] = ch;
                cur_col += 1;
            }
        }
    }

    /// Reconstruct a 2D character grid (`height` rows x `width` cols) from monochrome ANSI output.
    fn parse_ansi_screen(output: &str, width: u16, height: u16) -> Vec<Vec<char>> {
        let mut screen = vec![vec![' '; width as usize]; height as usize];
        apply_ansi_screen(&mut screen, output);
        screen
    }

    #[test]
    fn renders_full_fish_at_expected_coordinates() {
        let fish = Fish::new_for_test(4.0, 2.0, Direction::Right, FishColor::Cyan, "><))>");
        let aq = Aquarium::from_fish(20, 10, vec![fish], 1.0);
        let mut renderer = Renderer::new();
        let mut buf = Vec::new();

        renderer
            .render_to(&mut buf, &aq, true)
            .expect("render should succeed");
        let output = String::from_utf8(buf).expect("valid utf8 output");

        // Crossterm MoveTo(4, 2) emits 1-based ANSI CSI "\x1b[3;5H"
        assert!(
            output.contains("\x1b[3;5H><))>"),
            "expected fish sprite at row 2, col 4 in output: {output:?}"
        );
    }

    #[test]
    fn clips_fish_partially_entering_from_left_edge() {
        let fish = Fish::new_for_test(-2.0, 1.0, Direction::Right, FishColor::Yellow, "><))>");
        let aq = Aquarium::from_fish(15, 5, vec![fish], 1.0);
        let mut renderer = Renderer::new();
        let mut buf = Vec::new();

        renderer
            .render_to(&mut buf, &aq, false)
            .expect("render should succeed");
        let output = String::from_utf8(buf).expect("valid utf8 output");

        // MoveTo(0, 1) -> "\x1b[2;1H", followed only by "))>"
        assert!(
            output.contains("\x1b[2;1H))>"),
            "expected left-clipped sprite '))>' at col 0: {output:?}"
        );
        assert!(
            !output.contains("><))>"),
            "off-screen prefix '><' must be clipped: {output:?}"
        );
    }

    #[test]
    fn clips_fish_partially_exiting_right_edge_without_wrapping() {
        let width = 10u16;
        let fish = Fish::new_for_test(
            (width - 2) as f32,
            3.0,
            Direction::Right,
            FishColor::Green,
            "><))>",
        );
        let aq = Aquarium::from_fish(width, 6, vec![fish], 1.0);
        let mut renderer = Renderer::new();
        let mut buf = Vec::new();

        renderer
            .render_to(&mut buf, &aq, false)
            .expect("render should succeed");
        let output = String::from_utf8(buf).expect("valid utf8 output");

        // MoveTo(8, 3) -> "\x1b[4;9H", followed only by "><"
        assert!(
            output.contains("\x1b[4;9H><"),
            "expected right-clipped sprite '><' at col 8: {output:?}"
        );
        assert!(
            !output.contains(")"),
            "overflowing suffix '))>' must be clipped: {output:?}"
        );
    }

    #[test]
    fn handles_zero_size_terminals_safely() {
        let mut renderer = Renderer::new();

        for (w, h) in [(0, 0), (10, 0), (0, 10)] {
            let fish = Fish::new_for_test(0.0, 0.0, Direction::Right, FishColor::Cyan, "><>");
            let aq = Aquarium::from_fish(w, h, vec![fish], 1.0);
            let mut buf = Vec::new();

            renderer
                .render_to(&mut buf, &aq, true)
                .expect("zero-dimension render must not fail");
            assert!(
                buf.is_empty(),
                "zero-dimension terminal ({w}x{h}) should emit no output"
            );
        }
    }

    #[test]
    fn no_color_mode_emits_zero_sgr_color_codes() {
        let fish: Vec<Fish> = FishColor::ALL
            .iter()
            .enumerate()
            .map(|(i, &c)| Fish::new_for_test(1.0, i as f32, Direction::Right, c, "><>"))
            .collect();
        let aq = Aquarium::from_fish(20, 15, fish, 1.0);
        let mut renderer = Renderer::new();
        let mut buf = Vec::new();

        renderer
            .render_to(&mut buf, &aq, false)
            .expect("render should succeed");
        let output = String::from_utf8(buf).expect("valid utf8 output");

        assert!(!output.contains("\x1b[38;"), "must not emit 256/truecolor SGR: {output:?}");
        assert!(!output.contains("\x1b[0m"), "must not emit SGR reset: {output:?}");
        for code in 30..=37 {
            assert!(
                !output.contains(&format!("\x1b[{code}m")),
                "must not emit standard SGR {code}: {output:?}"
            );
        }
        for code in 90..=97 {
            assert!(
                !output.contains(&format!("\x1b[{code}m")),
                "must not emit bright SGR {code}: {output:?}"
            );
        }
    }

    #[test]
    fn color_mode_maps_normal_and_bright_colors_distinctly() {
        for (i, &c1) in FishColor::ALL.iter().enumerate() {
            for (j, &c2) in FishColor::ALL.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        map_fish_color(c1),
                        map_fish_color(c2),
                        "FishColor::{c1:?} and FishColor::{c2:?} must map to distinct crossterm Colors"
                    );
                }
            }
        }
    }

    #[test]
    fn consecutive_frames_diff_erase_old_cells_and_clear_only_on_resize() {
        let mut renderer = Renderer::new();

        // Frame 1: fish at (2, 1) on 20x5 screen
        let aq1 = Aquarium::from_fish(
            20,
            5,
            vec![Fish::new_for_test(
                2.0,
                1.0,
                Direction::Right,
                FishColor::Cyan,
                "><>",
            )],
            1.0,
        );
        let mut buf1 = Vec::new();
        renderer.render_to(&mut buf1, &aq1, false).unwrap();
        let out1 = String::from_utf8(buf1).unwrap();
        assert!(out1.contains("\x1b[2J"), "first frame must clear screen");
        assert!(out1.contains("\x1b[2;3H><>"));

        // Frame 2 (same size): fish moved to (3, 1); col 2 must be erased with ' ' and col 5 drawn with '>'
        let aq2 = Aquarium::from_fish(
            20,
            5,
            vec![Fish::new_for_test(
                3.0,
                1.0,
                Direction::Right,
                FishColor::Cyan,
                "><>",
            )],
            1.0,
        );
        let mut buf2 = Vec::new();
        renderer.render_to(&mut buf2, &aq2, false).unwrap();
        let out2 = String::from_utf8(buf2).unwrap();
        assert!(
            !out2.contains("\x1b[2J"),
            "subsequent frame at same dimensions must not clear entire screen: {out2:?}"
        );
        // Old col 2, row 1 (1-based \x1b[2;3H) erased with space
        assert!(
            out2.contains("\x1b[2;3H "),
            "old cell at col 2 should be erased with space: {out2:?}"
        );

        // Frame 3 (resized to 25x6): must emit full clear again
        let aq3 = Aquarium::from_fish(
            25,
            6,
            vec![Fish::new_for_test(
                3.0,
                1.0,
                Direction::Right,
                FishColor::Cyan,
                "><>",
            )],
            1.0,
        );
        let mut buf3 = Vec::new();
        renderer.render_to(&mut buf3, &aq3, false).unwrap();
        let out3 = String::from_utf8(buf3).unwrap();
        assert!(
            out3.contains("\x1b[2J"),
            "resized frame must emit full screen clear: {out3:?}"
        );
    }

    #[test]
    fn renders_multiline_creature_across_all_rows_at_expected_coordinates() {
        let mut rng = StdRng::seed_from_u64(42);
        let large = Fish::spawn_kind(CreatureKind::LargeFish, 80, 24, &mut rng);
        assert_eq!(large.height(), 5);
        let start_x = large.x().round() as usize;
        let start_y = large.y().round() as usize;
        let lines: Vec<&'static str> = large.sprite_lines().to_vec();

        let aq = Aquarium::from_fish(80, 24, vec![large], 1.0);
        let mut renderer = Renderer::new();
        let mut buf = Vec::new();
        renderer
            .render_to(&mut buf, &aq, false)
            .expect("multiline large fish render should succeed");
        let output = String::from_utf8(buf).expect("valid utf8 output");
        let screen = parse_ansi_screen(&output, 80, 24);

        for (row_idx, line) in lines.iter().enumerate() {
            let row = start_y + row_idx;
            for (col_idx, ch) in line.chars().enumerate() {
                let col = start_x + col_idx;
                assert_eq!(
                    screen[row][col], ch,
                    "mismatch at (col={col}, row={row}) for large fish row {row_idx}"
                );
            }
        }
    }

    #[test]
    fn transparent_spaces_in_multiline_sprites_do_not_erase_overlapping_creatures() {
        let mut rng = StdRng::seed_from_u64(7);

        // Case 1: SmallFish ("><>") placed inside the leading transparent whitespace of a LargeFish at (0, 0).
        // Every LargeFish species has >= 6 leading spaces on row 0.
        let large = Fish::spawn_kind(CreatureKind::LargeFish, 0, 0, &mut rng);
        assert_eq!(large.x().round() as i32, 0);
        assert_eq!(large.y().round() as i32, 0);
        assert!(
            large.sprite_lines()[0].starts_with("      "),
            "expected leading spaces on LargeFish row 0: {:?}",
            large.sprite_lines()[0]
        );
        let small_in_large_padding =
            Fish::new_for_test(1.0, 0.0, Direction::Right, FishColor::BrightYellow, "><>");

        let aq1 = Aquarium::from_fish(40, 10, vec![small_in_large_padding, large.clone()], 1.0);
        let mut renderer1 = Renderer::new();
        let mut buf1 = Vec::new();
        renderer1.render_to(&mut buf1, &aq1, false).unwrap();
        let screen1 = parse_ansi_screen(&String::from_utf8(buf1).unwrap(), 40, 10);

        // SmallFish at (1..4, 0) is preserved alongside LargeFish's dorsal fin on row 0
        assert_eq!(screen1[0][1], '>');
        assert_eq!(screen1[0][2], '<');
        assert_eq!(screen1[0][3], '>');
        for (col_idx, ch) in large.sprite_lines()[0].chars().enumerate() {
            if ch != ' ' {
                assert_eq!(
                    screen1[0][col_idx], ch,
                    "LargeFish non-space char at col {col_idx} row 0 should also appear"
                );
            }
        }

        // Case 2: Foreground multi-line creature with leading/trailing spaces (Crab at (0, 0),
        // whose row 0 is " V\\_o_o_/V " with spaces at col 0 and col 10) drawn AFTER SmallFish ("><>")
        // at (0, 0). The Crab's leading space at (0, 0) must NOT erase the SmallFish's '>' at (0, 0)!
        let crab = Fish::spawn_kind(CreatureKind::Crab, 0, 2, &mut rng);
        assert_eq!(crab.x().round() as i32, 0);
        assert_eq!(crab.y().round() as i32, 0);
        assert_eq!(crab.sprite_lines()[0].chars().next(), Some(' '));
        let small_behind_crab =
            Fish::new_for_test(0.0, 0.0, Direction::Right, FishColor::BrightCyan, "><>");

        let aq2 = Aquarium::from_fish(25, 5, vec![crab.clone(), small_behind_crab], 1.0);
        let mut renderer2 = Renderer::new();
        let mut buf2 = Vec::new();
        renderer2.render_to(&mut buf2, &aq2, false).unwrap();
        let screen2 = parse_ansi_screen(&String::from_utf8(buf2).unwrap(), 25, 5);

        // Col 0 has Crab's transparent ' ', so SmallFish's '>' at (0, 0) remains visible!
        assert_eq!(
            screen2[0][0], '>',
            "Crab's leading transparent space at (0, 0) must not erase SmallFish's '>'"
        );
        // Col 1 has Crab's non-space claw ('V' or 'v'), which renders in front of SmallFish's '<'
        let expected_claw = crab.sprite_lines()[0].chars().nth(1).unwrap();
        assert_eq!(screen2[0][1], expected_claw);
    }

    #[test]
    fn z_order_layering_renders_small_fish_and_crab_in_front_of_large_fish_regardless_of_slice_order() {
        let mut rng = StdRng::seed_from_u64(99);
        let large = Fish::spawn_kind(CreatureKind::LargeFish, 0, 0, &mut rng);
        assert_eq!(large.x().round() as i32, 0);
        assert_eq!(large.y().round() as i32, 0);

        // Row 2 of every LargeFish has non-space characters at cols 2..5
        let overlap_row = 2usize;
        let overlap_col = large.sprite_lines()[overlap_row]
            .char_indices()
            .find(|&(_, ch)| ch != ' ')
            .map(|(idx, _)| idx)
            .expect("LargeFish row 2 must have non-space characters");

        let small = Fish::new_for_test(
            overlap_col as f32,
            overlap_row as f32,
            Direction::Right,
            FishColor::BrightRed,
            "><>",
        );

        // Test both slice orderings in aquarium.fish()
        for creatures in [
            vec![large.clone(), small.clone()],
            vec![small.clone(), large.clone()],
        ] {
            let aq = Aquarium::from_fish(40, 10, creatures, 1.0);
            let mut renderer = Renderer::new();
            let mut buf = Vec::new();
            renderer.render_to(&mut buf, &aq, true).unwrap();

            // Verify in backbuffer that SmallFish won at the overlapping cells with its color
            let idx0 = overlap_row * 40 + overlap_col;
            assert_eq!(
                renderer.prev_grid[idx0],
                Cell {
                    ch: '>',
                    color: Some(FishColor::BrightRed),
                }
            );
            assert_eq!(
                renderer.prev_grid[idx0 + 1],
                Cell {
                    ch: '<',
                    color: Some(FishColor::BrightRed),
                }
            );
            assert_eq!(
                renderer.prev_grid[idx0 + 2],
                Cell {
                    ch: '>',
                    color: Some(FishColor::BrightRed),
                }
            );
        }

        // Also verify Crab (layer 5) wins over LargeFish (layer 1) and SmallFish (layer 4)
        let crab = Fish::spawn_kind(CreatureKind::Crab, 0, 2, &mut rng);
        assert_eq!(crab.x().round() as i32, 0);
        assert_eq!(crab.y().round() as i32, 0);
        let small_at_crab = Fish::new_for_test(1.0, 1.0, Direction::Right, FishColor::Green, "><>");

        for creatures in [
            vec![crab.clone(), small_at_crab.clone(), large.clone()],
            vec![large.clone(), small_at_crab.clone(), crab.clone()],
        ] {
            let aq = Aquarium::from_fish(40, 10, creatures, 1.0);
            let mut renderer = Renderer::new();
            let mut buf = Vec::new();
            renderer.render_to(&mut buf, &aq, true).unwrap();

            let expected_crab_ch = crab.sprite_lines()[1].chars().nth(1).unwrap();
            assert_ne!(expected_crab_ch, ' ');
            assert_eq!(
                renderer.prev_grid[40 + 1],
                Cell {
                    ch: expected_crab_ch,
                    color: Some(crab.color()),
                }
            );
        }
    }

    #[test]
    fn clips_multiline_creatures_at_top_bottom_left_and_right_edges_safely() {
        let mut rng = StdRng::seed_from_u64(123);
        let crab = Fish::spawn_kind(CreatureKind::Crab, 11, 6, &mut rng);
        // With screen_width=11 (== crab.width()) and screen_height=6, crab spawns at (0.0, 4.0)
        assert_eq!(crab.x().round() as i32, 0);
        assert_eq!(crab.y().round() as i32, 4);
        let crab_lines = crab.sprite_lines();

        // 1. Bottom edge via Renderer::render_to: height = 5 so crab is at y = 4 = height - 1.
        // Only row 0 of crab renders at row 4; row 1 (y = 5) is clipped cleanly.
        let aq_bottom = Aquarium::from_fish(15, 5, vec![crab.clone()], 1.0);
        let mut renderer_bottom = Renderer::new();
        let mut buf_bottom = Vec::new();
        renderer_bottom
            .render_to(&mut buf_bottom, &aq_bottom, false)
            .expect("bottom-clipped multiline render must succeed");
        let screen_bottom =
            parse_ansi_screen(&String::from_utf8(buf_bottom).unwrap(), 15, 5);
        for (col_idx, ch) in crab_lines[0].chars().enumerate() {
            assert_eq!(screen_bottom[4][col_idx], ch);
        }

        // 2. Right edge via Renderer::render_to: width = 2 so crab at x = 0 is at x = width - 2.
        // Only cols 0..2 of both rows render; cols 2..11 are clipped without wrapping.
        let aq_right = Aquarium::from_fish(2, 6, vec![crab.clone()], 1.0);
        let mut renderer_right = Renderer::new();
        let mut buf_right = Vec::new();
        renderer_right
            .render_to(&mut buf_right, &aq_right, false)
            .expect("right-clipped multiline render must succeed");
        let screen_right =
            parse_ansi_screen(&String::from_utf8(buf_right).unwrap(), 2, 6);
        for row_idx in 0..2 {
            for col_idx in 0..2 {
                let expected = crab_lines[row_idx].chars().nth(col_idx).unwrap();
                assert_eq!(screen_right[4 + row_idx][col_idx], expected);
            }
        }

        // 3. Top (y = -1), bottom (y = height - 1), left (x = -2), and right (x = width - 2)
        // multi-line 2D clipping on a 3-row MediumFish sprite:
        let medium = Fish::spawn_kind(CreatureKind::MediumFish, 0, 0, &mut rng);
        let med_lines = medium.sprite_lines();
        assert_eq!(med_lines.len(), 3);
        let w = 15u16;
        let h = 6u16;

        // Top edge (y = -1): row 0 clipped, row 1 -> screen row 0, row 2 -> screen row 1
        let mut grid_top = vec![Cell::EMPTY; (w as usize) * (h as usize)];
        Renderer::draw_sprite_lines(&mut grid_top, w, h, 2, -1, med_lines, None);
        for (out_row, src_row) in [(0usize, 1usize), (1usize, 2usize)] {
            for (col_idx, ch) in med_lines[src_row].chars().enumerate() {
                assert_eq!(grid_top[out_row * (w as usize) + (2 + col_idx)].ch, ch);
            }
        }

        // Bottom edge (y = height - 1 = 5): row 0 -> screen row 5, rows 1 & 2 clipped
        let mut grid_bot = vec![Cell::EMPTY; (w as usize) * (h as usize)];
        Renderer::draw_sprite_lines(&mut grid_bot, w, h, 2, (h as i32) - 1, med_lines, None);
        for (col_idx, ch) in med_lines[0].chars().enumerate() {
            assert_eq!(grid_bot[5 * (w as usize) + (2 + col_idx)].ch, ch);
        }

        // Left edge (x = -2): cols 0 & 1 clipped on every row, cols 2.. render at 0..
        let mut grid_left = vec![Cell::EMPTY; (w as usize) * (h as usize)];
        Renderer::draw_sprite_lines(&mut grid_left, w, h, -2, 1, med_lines, None);
        for (row_idx, line) in med_lines.iter().enumerate() {
            for (col_idx, ch) in line.chars().enumerate().skip(2) {
                assert_eq!(
                    grid_left[(1 + row_idx) * (w as usize) + (col_idx - 2)].ch,
                    ch
                );
            }
        }

        // Right edge (x = width - 2 = 13): cols 0 & 1 render at 13 & 14 on every row, cols 2.. clipped
        let mut grid_right = vec![Cell::EMPTY; (w as usize) * (h as usize)];
        Renderer::draw_sprite_lines(&mut grid_right, w, h, (w as i32) - 2, 1, med_lines, None);
        for (row_idx, line) in med_lines.iter().enumerate() {
            for (col_idx, ch) in line.chars().enumerate().take(2) {
                assert_eq!(
                    grid_right[(1 + row_idx) * (w as usize) + ((w as usize) - 2 + col_idx)].ch,
                    ch
                );
            }
        }
    }

    #[test]
    fn renders_coral_and_seaweed_at_expected_seabed_coordinates() {
        let width = 40u16;
        let height = 12u16;
        let coral = Coral::new(0.0, CORAL_SPECIES[1], FishColor::BrightMagenta);
        let seaweed = Seaweed::new(0.75, 5, FishColor::BrightGreen, 0.0, 1.5);

        let anchor_col = coral.anchor_col(width);
        let top_row = coral.top_row(height);
        let coral_lines = coral.lines();
        let sw_cells = seaweed.cells(width, height);
        assert_eq!(sw_cells[0].1, (height - 1) as i32, "seaweed root must sit on bottom row");

        let aq = Aquarium::from_fish(width, height, vec![], 1.0)
            .with_scenery(Scenery::from_parts(vec![coral.clone()], vec![seaweed.clone()]));
        let mut renderer = Renderer::new();
        let mut buf = Vec::new();
        renderer.render_to(&mut buf, &aq, false).unwrap();
        let screen = parse_ansi_screen(&String::from_utf8(buf).unwrap(), width, height);

        assert_eq!(top_row + coral.height() as i32 - 1, height as i32 - 1);
        for (row_idx, line) in coral_lines.iter().enumerate() {
            let row = (top_row + row_idx as i32) as usize;
            for (col_idx, ch) in line.chars().enumerate() {
                let col = (anchor_col + col_idx as i32) as usize;
                assert_eq!(
                    screen[row][col], ch,
                    "coral mismatch at (col={col}, row={row})"
                );
            }
        }

        for &(col, row, ch) in &sw_cells {
            assert_eq!(
                screen[row as usize][col as usize], ch,
                "seaweed mismatch at (col={col}, row={row})"
            );
        }
    }

    #[test]
    fn coral_transparent_spaces_preserve_seaweed_behind_branches() {
        let width = 40u16;
        let height = 12u16;
        let coral = Coral::new(0.0, CORAL_SPECIES[0], FishColor::Magenta);
        let anchor_col = coral.anchor_col(width);
        let top_row = coral.top_row(height);
        assert_eq!(anchor_col, 0);
        assert_eq!(top_row, 8);

        // Place seaweed root at col 3:
        // - row 11 (coral row 3 col 3 = '_') is occluded by coral branch
        // - row 10 (coral row 2 col 3 = ' ') shows seaweed through coral gap
        let seaweed = Seaweed::new(3.0 / 39.0, 5, FishColor::BrightGreen, 0.0, 0.0);
        let sw_cells = seaweed.cells(width, height);
        assert_eq!(sw_cells[0].0, 3);
        assert_eq!(sw_cells[0].1, 11);
        assert_eq!(sw_cells[1].0, 3);
        assert_eq!(sw_cells[1].1, 10);

        let aq = Aquarium::from_fish(width, height, vec![], 1.0)
            .with_scenery(Scenery::from_parts(vec![coral.clone()], vec![seaweed.clone()]));
        let mut renderer = Renderer::new();
        let mut buf = Vec::new();
        renderer.render_to(&mut buf, &aq, true).unwrap();

        let w_usize = width as usize;
        // Root at (3, 11) is covered by coral's '_'
        assert_eq!(
            renderer.prev_grid[11 * w_usize + 3],
            Cell {
                ch: '_',
                color: Some(FishColor::Magenta),
            }
        );
        // Cell at (3, 10) sits inside staghorn coral's transparent ' ' gap, keeping seaweed visible
        assert_eq!(coral.lines()[2].as_bytes()[3], b' ');
        assert_eq!(
            renderer.prev_grid[10 * w_usize + 3],
            Cell {
                ch: sw_cells[1].2,
                color: Some(FishColor::BrightGreen),
            }
        );

        // Verify all seaweed cells against coral opacity
        let mut gap_count = 0usize;
        let mut occluded_count = 0usize;
        for &(col, row, sw_ch) in &sw_cells {
            let idx = (row as usize) * w_usize + (col as usize);
            let rel_row = row - top_row;
            let rel_col = col - anchor_col;
            let coral_ch = if rel_row >= 0
                && (rel_row as usize) < coral.lines().len()
                && rel_col >= 0
                && (rel_col as usize) < coral.lines()[rel_row as usize].len()
            {
                coral.lines()[rel_row as usize].chars().nth(rel_col as usize).unwrap()
            } else {
                ' '
            };

            if coral_ch == ' ' {
                gap_count += 1;
                assert_eq!(
                    renderer.prev_grid[idx],
                    Cell {
                        ch: sw_ch,
                        color: Some(FishColor::BrightGreen),
                    }
                );
            } else {
                occluded_count += 1;
                assert_eq!(
                    renderer.prev_grid[idx],
                    Cell {
                        ch: coral_ch,
                        color: Some(FishColor::Magenta),
                    }
                );
            }
        }
        assert!(gap_count > 0 && occluded_count > 0);
    }

    #[test]
    fn creatures_render_in_front_of_coral_and_seaweed() {
        let width = 40u16;
        let height = 8u16;
        let w_usize = width as usize;

        let coral = Coral::new(0.0, CORAL_SPECIES[0], FishColor::Yellow);
        let seaweed = Seaweed::new(5.0 / 39.0, 4, FishColor::Green, 0.0, 0.0);

        // Find a seeded LargeFish at y >= 3 so its body (rows 3..8) overlaps coral (rows 4..8) and seaweed (rows 4..8)
        let mut large = None;
        for seed in 0..50u64 {
            let mut rng = StdRng::seed_from_u64(seed);
            let candidate = Fish::spawn_kind(CreatureKind::LargeFish, 20, height, &mut rng);
            if candidate.y().round() as i32 >= 3 {
                large = Some(candidate);
                break;
            }
        }
        let large = large.expect("seeded LargeFish overlapping seabed scenery");

        let mut crab_rng = StdRng::seed_from_u64(7);
        let crab = Fish::spawn_kind(CreatureKind::Crab, 11, height, &mut crab_rng);
        // Place SmallFish overlapping seaweed/coral at row 4..5, away from Crab (rows 6..8)
        let small = Fish::new_for_test(4.0, 4.0, Direction::Right, FishColor::BrightRed, "><>");

        let aq = Aquarium::from_fish(
            width,
            height,
            vec![large.clone(), small.clone(), crab.clone()],
            1.0,
        )
        .with_scenery(Scenery::from_parts(vec![coral.clone()], vec![seaweed.clone()]));

        let mut renderer = Renderer::new();
        let mut buf = Vec::new();
        renderer.render_to(&mut buf, &aq, true).unwrap();

        // Build a scenery-only grid to confirm every tested creature actually overlaps non-space scenery
        let mut scenery_grid = vec![Cell::EMPTY; w_usize * (height as usize)];
        for (col, row, ch) in seaweed.cells(width, height) {
            if ch != ' ' {
                scenery_grid[(row as usize) * w_usize + (col as usize)] = Cell {
                    ch,
                    color: Some(seaweed.color()),
                };
            }
        }
        Renderer::draw_sprite_lines(
            &mut scenery_grid,
            width,
            height,
            coral.anchor_col(width),
            coral.top_row(height),
            coral.lines(),
            Some(coral.color()),
        );

        let verify_foreground_creature = |creature: &Fish| {
            let sx = creature.x().round() as i32;
            let sy = creature.y().round() as i32;
            let mut overlapped = 0usize;
            for (r_idx, line) in creature.sprite_lines().iter().enumerate() {
                let row = sy + r_idx as i32;
                if row < 0 || row >= height as i32 {
                    continue;
                }
                for (c_idx, ch) in line.chars().enumerate() {
                    if ch == ' ' {
                        continue;
                    }
                    let col = sx + c_idx as i32;
                    if col >= 0 && col < width as i32 {
                        let idx = (row as usize) * w_usize + (col as usize);
                        if scenery_grid[idx] != Cell::EMPTY {
                            overlapped += 1;
                            assert_eq!(
                                renderer.prev_grid[idx],
                                Cell {
                                    ch,
                                    color: Some(creature.color()),
                                },
                                "creature {:?} at ({col}, {row}) must overwrite scenery",
                                creature.kind()
                            );
                        }
                    }
                }
            }
            assert!(
                overlapped > 0,
                "expected {:?} to overlap at least one non-space scenery cell",
                creature.kind()
            );
        };

        // Check LargeFish where not occluded by SmallFish or Crab
        let mut fg_grid = vec![Cell::EMPTY; w_usize * (height as usize)];
        for fg in [&small, &crab] {
            Renderer::draw_sprite_lines(
                &mut fg_grid,
                width,
                height,
                fg.x().round() as i32,
                fg.y().round() as i32,
                fg.sprite_lines(),
                Some(fg.color()),
            );
        }

        let lx = large.x().round() as i32;
        let ly = large.y().round() as i32;
        let mut large_overlaps = 0usize;
        for (r_idx, line) in large.sprite_lines().iter().enumerate() {
            let row = ly + r_idx as i32;
            for (c_idx, ch) in line.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let col = lx + c_idx as i32;
                if col >= 0 && col < width as i32 && row >= 0 && row < height as i32 {
                    let idx = (row as usize) * w_usize + (col as usize);
                    if scenery_grid[idx] != Cell::EMPTY && fg_grid[idx] == Cell::EMPTY {
                        large_overlaps += 1;
                        assert_eq!(
                            renderer.prev_grid[idx],
                            Cell {
                                ch,
                                color: Some(large.color()),
                            },
                            "LargeFish at ({col}, {row}) must render in front of scenery"
                        );
                    }
                }
            }
        }
        assert!(large_overlaps > 0, "LargeFish must overlap non-space scenery");

        verify_foreground_creature(&small);
        verify_foreground_creature(&crab);
    }

    #[test]
    fn animated_seaweed_diffs_cleanly_across_frames_without_clearing() {
        let width = 30u16;
        let height = 12u16;
        let stalk = Seaweed::new(0.5, 6, FishColor::BrightGreen, 0.0, 2.5);
        let cells_frame1 = stalk.cells(width, height);

        let mut aq = Aquarium::from_fish(width, height, vec![], 1.0)
            .with_scenery(Scenery::from_parts(vec![], vec![stalk]));
        let mut renderer = Renderer::new();

        let mut buf1 = Vec::new();
        renderer.render_to(&mut buf1, &aq, false).unwrap();
        let out1 = String::from_utf8(buf1).unwrap();
        assert!(out1.contains("\x1b[2J"), "first frame must clear screen");
        let mut screen = parse_ansi_screen(&out1, width, height);

        let mut rng = StdRng::seed_from_u64(1);
        let mut cells_frame2 = cells_frame1.clone();
        for _ in 0..40 {
            aq.update(0.1, &mut rng);
            cells_frame2 = aq.scenery().seaweed()[0].cells(width, height);
            if cells_frame2
                .iter()
                .zip(cells_frame1.iter())
                .any(|(a, b)| a.0 != b.0)
            {
                break;
            }
        }
        assert_ne!(
            cells_frame2, cells_frame1,
            "seaweed must sway to new column coordinates after updates"
        );

        let mut buf2 = Vec::new();
        renderer.render_to(&mut buf2, &aq, false).unwrap();
        let out2 = String::from_utf8(buf2).unwrap();
        assert!(
            !out2.contains("\x1b[2J"),
            "second frame must not emit full-screen clear: {out2:?}"
        );

        apply_ansi_screen(&mut screen, &out2);

        // Build expected screen for frame 2 and confirm old vacated cells are erased to ' '
        let mut expected = vec![vec![' '; width as usize]; height as usize];
        for &(col, row, ch) in &cells_frame2 {
            expected[row as usize][col as usize] = ch;
        }
        assert_eq!(screen, expected);

        for &(col, row, _) in &cells_frame1 {
            if !cells_frame2.iter().any(|&(c2, r2, _)| c2 == col && r2 == row) {
                assert_eq!(
                    screen[row as usize][col as usize], ' ',
                    "vacated seaweed cell at ({col}, {row}) must be erased"
                );
            }
        }
    }

    #[test]
    fn scenery_supports_no_color_mode_and_zero_or_tiny_sizes_safely() {
        let corals: Vec<Coral> = CORAL_SPECIES
            .iter()
            .enumerate()
            .map(|(i, &sprite)| Coral::new(i as f32 * 0.25, sprite, FishColor::ALL[i]))
            .collect();
        let seaweed = vec![
            Seaweed::new(0.2, 5, FishColor::Green, 0.0, 1.5),
            Seaweed::new(0.8, 4, FishColor::BrightGreen, 1.2, 2.0),
        ];

        let aq = Aquarium::from_fish(60, 16, vec![], 1.0)
            .with_scenery(Scenery::from_parts(corals.clone(), seaweed.clone()));
        let mut renderer = Renderer::new();
        let mut buf = Vec::new();
        renderer
            .render_to(&mut buf, &aq, false)
            .expect("no-color scenery render should succeed");
        let output = String::from_utf8(buf).expect("valid utf8");

        assert!(!output.contains("\x1b[38;"), "must not emit 256/truecolor SGR");
        assert!(!output.contains("\x1b[0m"), "must not emit SGR reset");
        for code in (30..=37).chain(90..=97) {
            assert!(
                !output.contains(&format!("\x1b[{code}m")),
                "must not emit SGR color code {code}: {output:?}"
            );
        }

        for (w, h) in [(0u16, 0u16), (10, 0), (0, 10), (4, 2)] {
            let tiny_aq = Aquarium::from_fish(w, h, vec![], 1.0)
                .with_scenery(Scenery::from_parts(corals.clone(), seaweed.clone()));
            let mut tiny_renderer = Renderer::new();
            let mut tiny_buf = Vec::new();
            tiny_renderer
                .render_to(&mut tiny_buf, &tiny_aq, true)
                .expect("render with scenery at tiny/zero size must not fail");
            if w == 0 || h == 0 {
                assert!(tiny_buf.is_empty());
            }
        }
    }
}
