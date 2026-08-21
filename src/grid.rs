//! Cell grid: the source of truth for what's on screen.
//! The storm overlay never mutates this; it only affects rendering.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    /// cell width: 2 = wide char, 1 = normal, 0 = continuation of a wide char
    pub width: u8,
}

impl Default for Cell {
    fn default() -> Self {
        Cell { ch: ' ', fg: Color::Default, bg: Color::Default, bold: false, reverse: false, width: 1 }
    }
}

/// Cell width for a char: 2 for wide ranges (CJK, Hangul, fullwidth, emoji),
/// 1 otherwise. A static table rather than wcwidth — wcwidth depends on the
/// process locale and returns 1 for CJK unless a UTF-8 locale is set.
fn char_width(ch: char) -> u8 {
    let c = ch as u32;
    let wide = (0x1100..=0x115F).contains(&c) // Hangul jamo
        || (0x2E80..=0xA4CF).contains(&c) // CJK radicals, punctuation, Hira/Kana, Han, Yi
        || (0xAC00..=0xD7A3).contains(&c) // Hangul syllables
        || (0xF900..=0xFAFF).contains(&c) // CJK compatibility
        || (0xFE30..=0xFE4F).contains(&c) // CJK compatibility forms
        || (0xFF00..=0xFF60).contains(&c) // fullwidth forms
        || (0xFFE0..=0xFFE6).contains(&c)
        || (0x1F000..=0x1FAFF).contains(&c) // emoji
        || (0x2600..=0x27BF).contains(&c) // misc symbols
        || (0x20000..=0x3FFFD).contains(&c); // CJK Ext B+
    if wide {
        2
    } else {
        1
    }
}

/// How many scrolled-off lines the grid retains for scrollback replay.
const HISTORY_LIMIT: usize = 4096;

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
    /// scrolled-off lines (oldest first), replayed into the terminal so its
    /// scrollback captures the full history — not just the visible rows
    pub history: Vec<Vec<Cell>>,
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
            history: Vec::new(),
        };
        g.set_scroll_region(0, rows.saturating_sub(1));
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
        let w = char_width(ch);
        if w == 2 && self.cursor_col + 1 >= self.cols {
            // a wide char doesn't fit at the last column; terminals drop it
            self.wrap_next = true;
            return;
        }
        let cell = Cell {
            ch,
            fg: self.cur_fg,
            bg: self.cur_bg,
            bold: self.cur_bold,
            reverse: self.cur_reverse,
            width: w,
        };
        self.set(self.cursor_row, self.cursor_col, cell);
        self.last_char = ch;
        self.cursor_col += w as usize;
        if self.cursor_col >= self.cols {
            // DEC autowrap: only a write to the actual last column arms the
            // wrap. Writing at cols-2 leaves the cursor at cols-1 with NO
            // pending wrap — arming early made full-width lines emit a
            // spurious LF after every redraw (omp's cascade).
            self.cursor_col = self.cols - 1;
            self.wrap_next = true;
        } else {
            self.wrap_next = false;
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
        // retain the lines about to scroll off, oldest first, so the renderer
        // can replay them into the host terminal's scrollback
        for r in self.scroll_top..self.scroll_top + n {
            let start = cell_index(self.cols, r, 0);
            self.history.push(self.cells[start..start + self.cols].to_vec());
        }
        if self.history.len() > HISTORY_LIMIT {
            let excess = self.history.len() - HISTORY_LIMIT;
            self.history.drain(0..excess);
        }
        for r in self.scroll_top..=(self.scroll_bottom - n) {
            let src = cell_index(self.cols, r + n, 0);
            let dst = cell_index(self.cols, r, 0);
            self.cells.copy_within(src..src + self.cols, dst);
        }
        for r in (self.scroll_bottom - n + 1)..=self.scroll_bottom {
            for c in 0..self.cols {
                self.set(r, c, self.cleared());
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
                self.set(r, c, self.cleared());
            }
        }
    }

    /// A cleared cell: space on the CURRENT background (real-terminal ED/EL
    /// semantics — apps like nvim clear the whole screen to their bg, so
    /// empty cells must keep it rather than fall back to default).
    fn cleared(&self) -> Cell {
        Cell { ch: ' ', fg: Color::Default, bg: self.cur_bg, bold: false, reverse: false, width: 1 }
    }

    /// The blank second column of a wide char — never independently writable.
    fn continuation() -> Cell {
        Cell { ch: ' ', fg: Color::Default, bg: Color::Default, bold: false, reverse: false, width: 0 }
    }

    /// The single write path. Maintains the wide-char invariant:
    ///   - writing into a continuation first clears the wide char to its left
    ///   - replacing a wide char clears its continuation
    ///   - placing a width-2 char lays down its continuation
    fn set(&mut self, r: usize, c: usize, cell: Cell) {
        let i = cell_index(self.cols, r, c);
        if c > 0 && cell.width >= 1 && self.cells[i].width == 0 {
            // target is the second column of a wide char: clear the wide char
            self.cells[cell_index(self.cols, r, c - 1)] = self.cleared();
        }
        if self.cells[i].width == 2 && c + 1 < self.cols {
            // replacing a wide char: its continuation must go too
            self.cells[cell_index(self.cols, r, c + 1)] = self.cleared();
        }
        self.cells[i] = cell;
        if cell.width == 2 && c + 1 < self.cols {
            self.cells[cell_index(self.cols, r, c + 1)] = Self::continuation();
        }
    }

    pub fn erase_display(&mut self, mode: i64) {
        match mode {
            0 => {
                for c in self.cursor_col..self.cols {
                    self.set(self.cursor_row, c, self.cleared());
                }
                for r in (self.cursor_row + 1)..self.rows {
                    for c in 0..self.cols {
                        self.set(r, c, self.cleared());
                    }
                }
            }
            1 => {
                for c in 0..=self.cursor_col {
                    self.set(self.cursor_row, c, self.cleared());
                }
                for r in 0..self.cursor_row {
                    for c in 0..self.cols {
                        self.set(r, c, self.cleared());
                    }
                }
            }
            _ => {
                for r in 0..self.rows {
                    for c in 0..self.cols {
                        self.set(r, c, self.cleared());
                    }
                }
            }
        }
    }

    pub fn erase_line(&mut self, mode: i64) {
        match mode {
            0 => {
                for c in self.cursor_col..self.cols {
                    self.set(self.cursor_row, c, self.cleared());
                }
            }
            1 => {
                for c in 0..=self.cursor_col {
                    self.set(self.cursor_row, c, self.cleared());
                }
            }
            _ => {
                for c in 0..self.cols {
                    self.set(self.cursor_row, c, self.cleared());
                }
            }
        }
    }

    pub fn erase_chars(&mut self, n: usize) {
        let end = (self.cursor_col + n).min(self.cols);
        for c in self.cursor_col..end {
            self.set(self.cursor_row, c, self.cleared());
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
            self.set(self.cursor_row, c, self.cleared());
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
            self.set(self.cursor_row, c, self.cleared());
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
        self.history.clear();
        if !self.alt {
            self.main_cells = self.cells.clone();
            self.cells.fill(Cell::default());
            self.cursor_row = 0;
            self.cursor_col = 0;
            self.alt = true;
        }
    }

    pub fn leave_alt(&mut self) {
        self.history.clear();
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

#[cfg(test)]
mod wide_tests {
    use super::*;

    #[test]
    fn wide_char_places_continuation() {
        let mut g = Grid::new(3, 10);
        g.print_char('界'); // width 2
        assert_eq!(g.cell(0, 0).ch, '界');
        assert_eq!(g.cell(0, 0).width, 2);
        assert_eq!(g.cell(0, 1).width, 0, "second column must be a continuation");
        assert_eq!(g.cursor_col, 2, "cursor advances by the char's width");
    }

    #[test]
    fn overwriting_wide_char_clears_continuation() {
        let mut g = Grid::new(3, 10);
        g.print_char('界');
        g.move_to(0, 0);
        g.print_char('a');
        assert_eq!(g.cell(0, 0).ch, 'a');
        assert_eq!(g.cell(0, 0).width, 1);
        assert_eq!(g.cell(0, 1).width, 1, "continuation must be cleared");
        assert_ne!(g.cell(0, 1).width, 0);
    }

    #[test]
    fn writing_into_continuation_clears_wide_char() {
        let mut g = Grid::new(3, 10);
        g.print_char('界');
        g.move_to(0, 1);
        g.print_char('b');
        assert_eq!(g.cell(0, 0).width, 1, "wide char to the left must be cleared");
        assert_eq!(g.cell(0, 1).ch, 'b');
        assert_eq!(g.cell(0, 1).width, 1);
    }

    #[test]
    fn erase_over_wide_char_clears_both() {
        let mut g = Grid::new(3, 10);
        g.print_char('界');
        g.erase_line(2);
        for c in 0..10 {
            assert_eq!(g.cell(0, c).width, 1, "no stale continuations after erase");
        }
    }

    #[test]
    fn ascii_is_width_one() {
        let mut g = Grid::new(2, 5);
        g.print_char('|');
        assert_eq!(g.cell(0, 0).width, 1);
        assert_eq!(g.cursor_col, 1);
    }
}

#[cfg(test)]
mod scroll_tests {
    use super::*;

    #[test]
    fn scroll_up_captures_history_in_order() {
        let mut g = Grid::new(24, 80);
        // fill row 0 with 'x', row 1 with 'y', then scroll them off
        for c in 0..80 {
            g.set(0, c, Cell { ch: 'x', fg: Color::Default, bg: Color::Default, bold: false, reverse: false, width: 1 });
            g.set(1, c, Cell { ch: 'y', fg: Color::Default, bg: Color::Default, bold: false, reverse: false, width: 1 });
        }
        g.scroll_up(2);
        assert_eq!(g.history.len(), 2, "two lines scrolled off");
        assert_eq!(g.history[0][0].ch, 'x', "oldest line first");
        assert_eq!(g.history[1][0].ch, 'y', "then the next");
        // bounded: a huge scroll keeps only HISTORY_LIMIT lines
        let mut big = Grid::new(24, 80);
        for _ in 0..HISTORY_LIMIT + 100 {
            big.scroll_up(1);
        }
        assert_eq!(big.history.len(), HISTORY_LIMIT, "history is bounded");
        // alt transition clears history
        g.enter_alt();
        assert!(g.history.is_empty(), "alt transition clears history");
    }
}

#[cfg(test)]
mod wrap_tests {
    use super::*;
    

    #[test]
    fn autowrap_arms_only_at_last_column() {
        let mut g = Grid::new(24, 80);
        // writing at col 78 (second-to-last) leaves the cursor at 78 with no
        // pending wrap — the next char must land on col 79, not wrap
        for _ in 0..78 {
            g.print_char('x');
        }
        assert_eq!(g.cursor_col, 78);
        assert!(!g.wrap_next);
        g.print_char('y');
        assert_eq!(g.cursor_col, 79);
        assert!(!g.wrap_next);
        assert_eq!(g.cell(0, 78).ch, 'y');
        // writing the last column parks the cursor there and arms the wrap
        g.print_char('z');
        assert_eq!(g.cursor_col, 79);
        assert!(g.wrap_next);
        // the next char wraps to the following row
        g.print_char('w');
        assert_eq!(g.cursor_row, 1);
        assert_eq!(g.cursor_col, 1);
        assert!(!g.wrap_next);
        assert_eq!(g.cell(1, 0).ch, 'w');
    }

    #[test]
    fn full_width_line_redraw_stays_in_place() {
        // the omp cascade repro: fill the whole row, then \r + reprint —
        // the reprint must land on the SAME row (no spurious LF)
        let mut g = Grid::new(24, 80);
        for _ in 0..80 {
            g.print_char('─');
        }
        assert!(g.wrap_next);
        g.cr();
        assert!(!g.wrap_next);
        assert_eq!(g.cursor_col, 0);
        g.print_char('x');
        assert_eq!(g.cursor_row, 0);
        assert_eq!(g.cursor_col, 1);
        assert_eq!(g.cell(0, 0).ch, 'x');
        assert_eq!(g.cell(1, 0).ch, ' '); // next row untouched
    }

    #[test]
    fn default_scroll_region_covers_full_screen() {
        let g = Grid::new(24, 80);
        assert_eq!(g.scroll_top, 0);
        assert_eq!(g.scroll_bottom, 23);
    }
}
