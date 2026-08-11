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
        }
    }

    fn composite(&self, grid: &Grid, storm: &Storm, r: usize, c: usize) -> Cell {
        let base = grid.cell(r, c);
        match storm.overlay(base, r, c) {
            Some(o) => o,
            None => base,
        }
    }

    pub fn render(&mut self, grid: &Grid, storm: &Storm, out: &mut Vec<u8>) {
        if grid.rows != self.rows || grid.cols != self.cols {
            self.rows = grid.rows;
            self.cols = grid.cols;
            self.last = vec![Cell::default(); grid.rows * grid.cols];
            self.row_buf = vec![Cell::default(); grid.cols];
            self.dirty_all = true;
            out.extend_from_slice(b"\x1b[2J\x1b[H");
        }
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
                let mut cell = self.composite(grid, storm, r, c);
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
                let pos = format!("\x1b[{};{}H", r + 1, start + 1);
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
