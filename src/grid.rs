//! Cell grid: the source of truth for what's on screen.
//! The storm overlay never mutates this; it only affects rendering.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Default,
    Indexed(u8),
    Rgb(u8, u8, u8),
}

impl Default for Color {
    fn default() -> Self {
        Color::Default
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub reverse: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Cell { ch: ' ', fg: Color::Default, bg: Color::Default, bold: false, reverse: false }
    }
}

pub struct Grid {
    pub rows: usize,
    pub cols: usize,
    cells: Vec<Cell>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    saved_row: usize,
    saved_col: usize,
    scroll_top: usize,
    scroll_bottom: usize, // inclusive
    pub alt: bool,
    main_cells: Vec<Cell>,
    pub cur_fg: Color,
    pub cur_bg: Color,
    pub cur_bold: bool,
    pub cur_reverse: bool,
    wrap_next: bool,
    pub cursor_visible: bool,
    pub last_char: char,
}

fn cell_index(cols: usize, r: usize, c: usize) -> usize {
    r.saturating_mul(cols).saturating_add(c.min(cols.saturating_sub(1)))
}

impl Grid {
    pub fn new(rows: usize, cols: usize) -> Self {
        let mut g = Grid {
            rows,
            cols,
            cells: vec![Cell::default(); rows * cols],
            cursor_row: 0,
            cursor_col: 0,
            saved_row: 0,
            saved_col: 0,
            scroll_top: 0,
            scroll_bottom: rows.saturating_sub(1),
            alt: false,
            main_cells: Vec::new(),
            cur_fg: Color::Default,
            cur_bg: Color::Default,
            cur_bold: false,
            cur_reverse: false,
            wrap_next: false,
            cursor_visible: true,
            last_char: ' ',
        };
        g.set_scroll_region(1, rows);
        g
    }

    pub fn cell(&self, r: usize, c: usize) -> Cell {
        if r < self.rows && c < self.cols {
            self.cells[cell_index(self.cols, r, c)]
        } else {
            Cell::default()
        }
    }

    pub fn resize(&mut self, rows: usize, cols: usize) {
        if rows == self.rows && cols == self.cols {
            return;
        }
        let mut g = Grid::new(rows, cols);
        // keep whatever fits of the old content
        for r in 0..self.rows.min(rows) {
            for c in 0..self.cols.min(cols) {
                g.cells[cell_index(g.cols, r, c)] = self.cells[cell_index(self.cols, r, c)];
            }
        }
        g.cur_fg = self.cur_fg;
        g.cur_bg = self.cur_bg;
        g.cur_bold = self.cur_bold;
        g.cur_reverse = self.cur_reverse;
        g.cursor_row = self.cursor_row.min(rows.saturating_sub(1));
        g.cursor_col = self.cursor_col.min(cols.saturating_sub(1));
        g.alt = self.alt;
        if g.alt {
            g.main_cells = self.main_cells.clone();
        }
        g.cursor_visible = self.cursor_visible;
        *self = g;
    }

    pub fn print_char(&mut self, ch: char) {
        if self.wrap_next {
            self.wrap_next = false;
            self.newline();
        }
        let cell = Cell {
            ch,
            fg: self.cur_fg,
            bg: self.cur_bg,
            bold: self.cur_bold,
            reverse: self.cur_reverse,
        };
        self.cells[cell_index(self.cols, self.cursor_row, self.cursor_col)] = cell;
        self.last_char = ch;
        if self.cursor_col + 1 >= self.cols {
            self.wrap_next = true;
        } else {
            self.cursor_col += 1;
        }
    }

    pub fn repeat_last(&mut self, n: usize) {
        let ch = self.last_char;
        for _ in 0..n.min(self.cols * 2) {
            self.print_char(ch);
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
        self.wrap_next = false;
    }

    pub fn tab(&mut self) {
        let next = (self.cursor_col / 8 + 1) * 8;
        self.cursor_col = next.min(self.cols.saturating_sub(1));
    }

    pub fn lf(&mut self) {
        if self.cursor_row == self.scroll_bottom {
            self.scroll_up(1);
        } else if self.cursor_row + 1 < self.rows {
            self.cursor_row += 1;
        }
    }

    pub fn cr(&mut self) {
        self.cursor_col = 0;
        self.wrap_next = false;
    }

    pub fn newline(&mut self) {
        self.cr();
        self.lf();
    }

    pub fn reverse_index(&mut self) {
        if self.cursor_row == self.scroll_top {
            self.scroll_down(1);
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
        }
    }

    pub fn scroll_up(&mut self, n: usize) {
        let region = self.scroll_bottom - self.scroll_top + 1;
        let n = n.min(region);
        for r in self.scroll_top..=(self.scroll_bottom - n) {
            let src = cell_index(self.cols, r + n, 0);
            let dst = cell_index(self.cols, r, 0);
            self.cells.copy_within(src..src + self.cols, dst);
        }
        for r in (self.scroll_bottom - n + 1)..=self.scroll_bottom {
            for c in 0..self.cols {
                self.cells[cell_index(self.cols, r, c)] = self.cleared();
            }
        }
    }

    pub fn scroll_down(&mut self, n: usize) {
        let region = self.scroll_bottom - self.scroll_top + 1;
        let n = n.min(region);
        for r in (self.scroll_top + n..=self.scroll_bottom).rev() {
            let src = cell_index(self.cols, r - n, 0);
            let dst = cell_index(self.cols, r, 0);
            self.cells.copy_within(src..src + self.cols, dst);
        }
        for r in self.scroll_top..(self.scroll_top + n) {
            for c in 0..self.cols {
                self.cells[cell_index(self.cols, r, c)] = self.cleared();
            }
        }
    }

    /// A cleared cell: space on the CURRENT background (real-terminal ED/EL
    /// semantics — apps like nvim clear the whole screen to their bg, so
    /// empty cells must keep it rather than fall back to default).
    fn cleared(&self) -> Cell {
        Cell { ch: ' ', fg: Color::Default, bg: self.cur_bg, bold: false, reverse: false }
    }

    pub fn erase_display(&mut self, mode: i64) {
        match mode {
            0 => {
                for c in self.cursor_col..self.cols {
                    self.cells[cell_index(self.cols, self.cursor_row, c)] = self.cleared();
                }
                for r in (self.cursor_row + 1)..self.rows {
                    for c in 0..self.cols {
                        self.cells[cell_index(self.cols, r, c)] = self.cleared();
                    }
                }
            }
            1 => {
                for c in 0..=self.cursor_col {
                    self.cells[cell_index(self.cols, self.cursor_row, c)] = self.cleared();
                }
                for r in 0..self.cursor_row {
                    for c in 0..self.cols {
                        self.cells[cell_index(self.cols, r, c)] = self.cleared();
                    }
                }
            }
            _ => {
                for r in 0..self.rows {
                    for c in 0..self.cols {
                        self.cells[cell_index(self.cols, r, c)] = self.cleared();
                    }
                }
            }
        }
    }

    pub fn erase_line(&mut self, mode: i64) {
        match mode {
            0 => {
                for c in self.cursor_col..self.cols {
                    self.cells[cell_index(self.cols, self.cursor_row, c)] = self.cleared();
                }
            }
            1 => {
                for c in 0..=self.cursor_col {
                    self.cells[cell_index(self.cols, self.cursor_row, c)] = self.cleared();
                }
            }
            _ => {
                for c in 0..self.cols {
                    self.cells[cell_index(self.cols, self.cursor_row, c)] = self.cleared();
                }
            }
        }
    }

    pub fn erase_chars(&mut self, n: usize) {
        let end = (self.cursor_col + n).min(self.cols);
        for c in self.cursor_col..end {
            self.cells[cell_index(self.cols, self.cursor_row, c)] = self.cleared();
        }
    }

    pub fn insert_chars(&mut self, n: usize) {
        let n = n.min(self.cols.saturating_sub(self.cursor_col));
        for c in (self.cursor_col..self.cols - n).rev() {
            let src = cell_index(self.cols, self.cursor_row, c);
            let dst = cell_index(self.cols, self.cursor_row, c + n);
            self.cells[dst] = self.cells[src];
        }
        for c in self.cursor_col..(self.cursor_col + n).min(self.cols) {
            self.cells[cell_index(self.cols, self.cursor_row, c)] = Cell::default();
        }
    }

    pub fn delete_chars(&mut self, n: usize) {
        let n = n.min(self.cols - self.cursor_col);
        for c in self.cursor_col..(self.cols - n) {
            let src = cell_index(self.cols, self.cursor_row, c + n);
            let dst = cell_index(self.cols, self.cursor_row, c);
            self.cells[dst] = self.cells[src];
        }
        for c in (self.cols - n)..self.cols {
            self.cells[cell_index(self.cols, self.cursor_row, c)] = Cell::default();
        }
    }

    pub fn move_to(&mut self, row: usize, col: usize) {
        self.cursor_row = row.min(self.rows - 1);
        self.cursor_col = col.min(self.cols - 1);
        self.wrap_next = false;
    }

    pub fn move_relative(&mut self, drow: i64, dcol: i64) {
        let r = self.cursor_row as i64 + drow;
        let c = self.cursor_col as i64 + dcol;
        self.move_to(r.max(0) as usize, c.max(0) as usize);
    }

    pub fn set_scroll_region(&mut self, top: usize, bottom: usize) {
        let top = top.min(self.rows.saturating_sub(1));
        let bottom = bottom.clamp(top, self.rows.saturating_sub(1));
        self.scroll_top = top;
        self.scroll_bottom = bottom;
    }

    pub fn save_cursor(&mut self) {
        self.saved_row = self.cursor_row;
        self.saved_col = self.cursor_col;
    }

    pub fn restore_cursor(&mut self) {
        self.cursor_row = self.saved_row.min(self.rows - 1);
        self.cursor_col = self.saved_col.min(self.cols - 1);
        self.wrap_next = false;
    }

    pub fn enter_alt(&mut self) {
        if !self.alt {
            self.main_cells = self.cells.clone();
            self.cells.fill(Cell::default());
            self.cursor_row = 0;
            self.cursor_col = 0;
            self.alt = true;
        }
    }

    pub fn leave_alt(&mut self) {
        if self.alt {
            self.cells = self.main_cells.clone();
            self.alt = false;
        }
    }

    pub fn reset(&mut self) {
        *self = Grid::new(self.rows, self.cols);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erase_display_uses_current_bg() {
        let mut g = Grid::new(5, 10);
        g.cur_bg = Color::Rgb(1, 2, 3);
        g.erase_display(2);
        assert_eq!(g.cell(0, 0).bg, Color::Rgb(1, 2, 3), "clear must carry the current bg (nvim paints its screen)");
        assert_eq!(g.cell(4, 9).bg, Color::Rgb(1, 2, 3));
        assert_eq!(g.cell(2, 5).ch, ' ');
        assert_eq!(g.cell(2, 5).fg, Color::Default);
    }

    #[test]
    fn erase_line_uses_current_bg() {
        let mut g = Grid::new(3, 5);
        g.cursor_row = 1;
        g.cur_bg = Color::Indexed(21);
        g.erase_line(2);
        for c in 0..5 {
            assert_eq!(g.cell(1, c).bg, Color::Indexed(21));
        }
    }

    #[test]
    fn fresh_grid_has_default_bg() {
        let g = Grid::new(5, 10);
        assert_eq!(g.cell(0, 0).bg, Color::Default);
    }
}
