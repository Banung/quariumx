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

use crate::fish::{Aquarium, FishColor};

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

        for fish in aquarium.fish() {
            let row = fish.y().round() as i32;
            if row < 0 || row >= height as i32 {
                continue;
            }
            let start_x = fish.x().round() as i32;
            let cell_color = if color { Some(fish.color()) } else { None };

            for (i, ch) in fish.sprite().chars().enumerate() {
                let col = start_x + i as i32;
                if col >= 0 && col < width as i32 {
                    let idx = (row as usize) * w_usize + (col as usize);
                    next_grid[idx] = Cell {
                        ch,
                        color: cell_color,
                    };
                }
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
}
