//! Diff renderer: composite = grid + storm overlay; emit only cells that
//! changed since the last frame, so the terminal gets a minimal ANSI stream
//! even at 60 fps with a fullscreen storm running.

use crate::grid::{Cell, Color, Grid};
use crate::storm::Storm;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Style {
    fg: Color,
    bg: Color,
    bold: bool,
    reverse: bool,
}

impl Style {
    fn of(cell: Cell) -> Style {
        Style { fg: cell.fg, bg: cell.bg, bold: cell.bold, reverse: cell.reverse }
    }
}

fn lerp(a: u8, b: u8, t: f64) -> u8 {
    (a as f64 + (b as f64 - a as f64) * t).clamp(0.0, 255.0) as u8
}

fn color_code(out: &mut Vec<u8>, fg: bool, c: Color) {
    // NOTE: prefix must be a string ("3"/"4"), NOT the byte value — formatting
    // b'3' as {} prints 51, producing the invalid SGR 518;2;... that the
    // terminal silently ignores (rain used to render in default gray).
    let p = if fg { "3" } else { "4" };
    match c {
        Color::Default => out.extend_from_slice(format!("\x1b[{}9m", p).as_bytes()),
        Color::Indexed(n) => out.extend_from_slice(format!("\x1b[{}8;5;{}m", p, n).as_bytes()),
        Color::Rgb(r, g, b) => {
            out.extend_from_slice(format!("\x1b[{}8;2;{};{};{}m", p, r, g, b).as_bytes());
        }
    }
}

fn is_default(s: Style) -> bool {
    s.fg == Color::Default && s.bg == Color::Default && !s.bold && !s.reverse
}

fn write_sgr(out: &mut Vec<u8>, style: Style, prev: Style) {
    if style == prev {
        return;
    }
    if is_default(style) {
        out.extend_from_slice(b"\x1b[0m");
        return;
    }
    out.extend_from_slice(b"\x1b[0m");
    if style.bold {
        out.extend_from_slice(b"\x1b[1m");
    }
    if style.reverse {
        out.extend_from_slice(b"\x1b[7m");
    }
    color_code(out, true, style.fg);
    color_code(out, false, style.bg);
}

pub struct Renderer {
    last: Vec<Cell>,
    pub rows: usize,
    pub cols: usize,
    row_buf: Vec<Cell>,
    style: Style,
    /// when set, the next render treats every cell as changed (full repaint)
    dirty_all: bool,
    /// the previous frame was shaking — keeps dirty_all armed one extra frame
    /// so the screen repaints clean once the shake ends
    was_shaking: bool,
}

impl Renderer {
    pub fn new(rows: usize, cols: usize) -> Self {
        Renderer {
            last: vec![Cell::default(); rows * cols],
            rows,
            cols,
            row_buf: vec![Cell::default(); cols],
            style: Style::default(),
            dirty_all: true,
            was_shaking: false,
        }
    }

    fn composite(&self, grid: &Grid, storm: &Storm, hud: Option<&[String]>, corona: f64, r: usize, c: usize) -> Cell {
        let base = grid.cell(r, c);
        // the HUD panel floats mid-right of the screen: only its text span is
        // replaced (inheriting the base bg like rain) — everything else keeps
        // the base + storm composite. The palette swatch line renders its
        // chips in the live storm colors.
        if let Some(lines) = hud {
            let panel_rows = lines.len();
            if panel_rows <= grid.rows {
                let top = (grid.rows - panel_rows) / 2;
                let max_w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
                let left = grid.cols.saturating_sub(max_w + 2);
                if r >= top && r < top + panel_rows && c >= left {
                    let line = &lines[r - top];
                    let ci = c - left;
                    let len = line.chars().count();
                    if ci < len {
                        let ch = line.chars().nth(ci).expect("in text span");
                        // swatch line: index 4, chips at chars 9, 11, 13, 15
                        if r - top == 4 && ch == '●' && ci >= 9 && (ci - 9) % 2 == 0 {
                            let sw = storm.swatch();
                            let (sr, sg, sb) = sw[(ci - 9) / 2];
                            return Cell {
                                ch,
                                fg: Color::Rgb(sr, sg, sb),
                                bg: base.bg,
                                bold: true,
                                reverse: false,
                                width: 1,
                            };
                        }
                        return Cell {
                            ch,
                            fg: Color::Rgb(0x68, 0xA3, 0xE8),
                            bg: base.bg,
                            bold: true,
                            reverse: false,
                            width: 1,
                        };
                    }
                }
            }
        }
        match storm.overlay(base, r, c) {
            Some(o) => o,
            None => {
                if corona > 0.0 {
                    // corona: the whole screen tints toward the storm's
                    // palette while the flash is fresh — a moment, not a line
                    let (cr, cg, cb) = storm.corona_color();
                    let mut lit = base;
                    if let Color::Rgb(red, green, blue) = base.fg {
                        lit.fg = Color::Rgb(
                            lerp(red, cr, corona),
                            lerp(green, cg, corona),
                            lerp(blue, cb, corona),
                        );
                    }
                    return lit;
                }
                base
            }
        }
    }

    pub fn render(&mut self, grid: &mut Grid, storm: &Storm, hud: Option<&[String]>, out: &mut Vec<u8>) {
        if grid.rows != self.rows || grid.cols != self.cols {
            self.rows = grid.rows;
            self.cols = grid.cols;
            self.last = vec![Cell::default(); grid.rows * grid.cols];
            self.row_buf = vec![Cell::default(); grid.cols];
            self.dirty_all = true;
            out.extend_from_slice(b"\x1b[2J\x1b[H");
        }
        // Replay the grid's scrolled-off lines into the terminal so its
        // scrollback captures the FULL history. A terminal only pushes the
        // rows visible at scroll time into history, so a single big scroll
        // event loses the content; instead we replay the lines the way a live
        // stream would: fill top-to-bottom, then write at the bottom row and
        // LF. Each LF pushes the current top row into the scrollback, so the
        // lines land in order. The live screen is repainted afterwards.
        let history = std::mem::take(&mut grid.history);
        if !history.is_empty() {
            out.extend_from_slice(b"\x1b[r"); // full-screen region for the LFs
            // every line streams through the bottom row + LF, so each one
            // scrolls into the terminal's history in order — a top-fill would
            // let a later chunk overwrite the previous chunk's tail (losing
            // ~24 lines per frame boundary during fast bursts)
            for line in &history {
                out.extend_from_slice(format!("\x1b[{};1H", grid.rows).as_bytes());
                let mut cc = 0;
                while cc < line.len() {
                    let cell = line[cc];
                    if cell.width == 0 {
                        cc += 1;
                        continue; // wide-char continuation: part of the pair
                    }
                    write_sgr(out, Style::of(cell), self.style);
                    self.style = Style::of(cell);
                    let mut buf = [0u8; 4];
                    out.extend_from_slice(cell.ch.encode_utf8(&mut buf).as_bytes());
                    cc += 1;
                }
                out.extend_from_slice(b"\n"); // push a line into the scrollback
            }
            self.dirty_all = true; // replay scrambled the live screen; repaint
        }

        // corona tint level (0 when off) and screen shake — shake arms a full
        // repaint, so it must run BEFORE `full` is snapshotted below
        let corona = storm.corona_level();
        let shaking = storm.shake_level() > 0.001;
        if shaking || self.was_shaking {
            self.dirty_all = true;
        }
        self.was_shaking = shaking;
        let (dr, dc) = storm.shake_offset();

        let full = self.dirty_all;
        self.dirty_all = false;

        // composite the whole grid into row_buf, diffing against `last`
        let mut cursor_cell = None;
        if grid.cursor_visible && grid.cursor_row < grid.rows && grid.cursor_col < grid.cols {
            let base = grid.cell(grid.cursor_row, grid.cursor_col);
            let mut cc = base;
            cc.reverse = !cc.reverse;
            cursor_cell = Some((grid.cursor_row, grid.cursor_col, cc));
        }

        for r in 0..grid.rows {
            // fill row_buf with the composite for this row
            for c in 0..grid.cols {
                let mut cell = self.composite(grid, storm, hud, corona, r, c);
                if let Some((cr, cc, ccell)) = cursor_cell {
                    if cr == r && cc == c {
                        cell = ccell;
                    }
                }
                self.row_buf[c] = cell;
            }
            // emit changed runs; continuation cells (width 0) are never
            // written on their own — they're part of the wide char to their
            // left, so skip them in the scan
            let mut c = 0;
            while c < grid.cols {
                if self.row_buf[c].width == 0 {
                    c += 1;
                    continue;
                }
                if !full && self.row_buf[c] == self.last[r * grid.cols + c] {
                    c += 1;
                    continue;
                }
                let start = c;
                while c < grid.cols
                    && self.row_buf[c].width != 0
                    && (full || self.row_buf[c] != self.last[r * grid.cols + c])
                {
                    c += 1;
                }
                let pos = format!(
                    "\x1b[{};{}H",
                    (r as i64 + 1 + dr).clamp(1, grid.rows as i64),
                    // the shake's dc offset must never push a run past the
                    // last column — an overflow wraps and pushes a line into
                    // the terminal's scrollback (every strike grew it by 3)
                    (start as i64 + 1 + dc)
                        .clamp(1, grid.cols as i64 - (c - start) as i64 + 1)
                );
                out.extend_from_slice(pos.as_bytes());
                for cc in start..c {
                    let cell = self.row_buf[cc];
                    write_sgr(out, Style::of(cell), self.style);
                    self.style = Style::of(cell);
                    let mut buf = [0u8; 4];
                    out.extend_from_slice(cell.ch.encode_utf8(&mut buf).as_bytes());
                    self.last[r * grid.cols + cc] = cell;
                    if cell.width == 2 && cc + 1 < grid.cols {
                        // the terminal renders the second column as part of
                        // the glyph — record the continuation in `last`
                        self.last[r * grid.cols + cc + 1] = self.row_buf[cc + 1];
                    }
                }
            }
        }
    }

    /// Full repaint on demand (used after resize or the shell exiting).
    pub fn force_redraw(&mut self) {
        self.dirty_all = true;
    }
}
