//! vte::Perform implementation: parsed VT sequences -> grid operations.

use crate::grid::{Color, Grid};
use vte::Params;

pub struct Perform {
    pub grid: Grid,
    /// Replies the child asked for (DA/DSR); drained by the main loop.
    pub replies: Vec<u8>,
    /// The child's requested mouse tracking mode (0, 1000, 1002, 1003) and
    /// SGR encoding (1006) — mirrored to the host terminal so the wheel
    /// routes to the child instead of native scrollback.
    pub mouse_mode: u16,
    pub mouse_sgr: bool,
}

impl Perform {
    pub fn new(rows: usize, cols: usize) -> Self {
        Perform {
            grid: Grid::new(rows, cols),
            replies: Vec::new(),
            mouse_mode: 0,
            mouse_sgr: false,
        }
    }

    fn reply(&mut self, s: &[u8]) {
        self.replies.extend_from_slice(s);
    }

    fn apply_sgr(&mut self, p: &[u16]) {
        let mut i = 0;
        while i < p.len() {
            match p[i] {
                0 => {
                    self.grid.cur_fg = Color::Default;
                    self.grid.cur_bg = Color::Default;
                    self.grid.cur_bold = false;
                    self.grid.cur_reverse = false;
                }
                1 => self.grid.cur_bold = true,
                22 => self.grid.cur_bold = false,
                7 => self.grid.cur_reverse = true,
                27 => self.grid.cur_reverse = false,
                30..=37 => self.grid.cur_fg = Color::Indexed((p[i] - 30) as u8),
                90..=97 => self.grid.cur_fg = Color::Indexed((p[i] - 90 + 8) as u8),
                39 => self.grid.cur_fg = Color::Default,
                40..=47 => self.grid.cur_bg = Color::Indexed((p[i] - 40) as u8),
                100..=107 => self.grid.cur_bg = Color::Indexed((p[i] - 100 + 8) as u8),
                49 => self.grid.cur_bg = Color::Default,
                38 | 48 => {
                    if i + 1 < p.len() {
                        if p[i + 1] == 5 && i + 2 < p.len() {
                            let idx = p[i + 2] as u8;
                            if p[i] == 38 {
                                self.grid.cur_fg = Color::Indexed(idx);
                            } else {
                                self.grid.cur_bg = Color::Indexed(idx);
                            }
                            i += 2;
                        } else if p[i + 1] == 2 && i + 4 < p.len() {
                            let rgb = Color::Rgb(p[i + 2] as u8, p[i + 3] as u8, p[i + 4] as u8);
                            if p[i] == 38 {
                                self.grid.cur_fg = rgb;
                            } else {
                                self.grid.cur_bg = rgb;
                            }
                            i += 4;
                        }
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }
}

fn param0(params: &Params, idx: usize) -> u16 {
    params.iter().nth(idx).and_then(|p| p.get(0).copied()).unwrap_or(0)
}

impl vte::Perform for Perform {
    fn print(&mut self, c: char) {
        self.grid.print_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            0x08 => self.grid.backspace(),
            0x09 => self.grid.tab(),
            0x0a | 0x0b | 0x0c => self.grid.lf(),
            0x0d => self.grid.cr(),
            _ => {} // BEL, CAN, SUB, NUL etc: ignore
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _action: char) {}

    fn put(&mut self, _byte: u8) {}

    fn unhook(&mut self) {}

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], _ignore: bool, action: char) {
        let p0 = param0(params, 0);
        let p1 = param0(params, 1);
        match action {
            'A' => self.grid.move_relative(-(p0.max(1) as i64), 0),
            'B' => self.grid.move_relative(p0.max(1) as i64, 0),
            'C' => self.grid.move_relative(0, p0.max(1) as i64),
            'D' => self.grid.move_relative(0, -(p0.max(1) as i64)),
            'G' => self.grid.move_to(self.grid.cursor_row, (p0.max(1) - 1) as usize),
            'd' => self.grid.move_to((p0.max(1) - 1) as usize, self.grid.cursor_col),
            'H' | 'f' => {
                let r = if p0 == 0 { 1 } else { p0 };
                let c = if p1 == 0 { 1 } else { p1 };
                self.grid.move_to((r - 1) as usize, (c - 1) as usize);
            }
            'J' => self.grid.erase_display(p0 as i64),
            'K' => self.grid.erase_line(p0 as i64),
            'X' => self.grid.erase_chars(p0.max(1) as usize),
            '@' => self.grid.insert_chars(p0.max(1) as usize),
            'P' => self.grid.delete_chars(p0.max(1) as usize),
            'L' => self.grid.scroll_down(p0.max(1) as usize),
            'M' => self.grid.scroll_up(p0.max(1) as usize),
            'S' => self.grid.scroll_up(p0.max(1) as usize),
            'T' => self.grid.scroll_down(p0.max(1) as usize),
            'b' => self.grid.repeat_last(p0.max(1) as usize),
            'r' => {
                let top = if p0 == 0 { 1 } else { p0 };
                let bottom = if p1 == 0 { self.grid.rows as u16 } else { p1 };
                self.grid.set_scroll_region((top - 1) as usize, (bottom - 1) as usize);
                self.grid.cursor_row = 0;
                self.grid.cursor_col = 0;
            }
            'm' => {
                let p: Vec<u16> = params.iter().map(|param| param.get(0).copied().unwrap_or(0)).collect();
                let empty = p.is_empty();
                let sgr_params: Vec<u16> = if empty { vec![0] } else { p };
                self.apply_sgr(&sgr_params);
            }
            'h' | 'l' => {
                let private = intermediates.first() == Some(&b'?');
                if private {
                    match p0 {
                        25 => self.grid.cursor_visible = action == 'h',
                        47 | 1049 => {
                            if action == 'h' {
                                self.grid.save_cursor();
                                self.grid.enter_alt();
                            } else {
                                self.grid.leave_alt();
                                self.grid.restore_cursor();
                            }
                        }
                        1048 => {
                            if action == 'h' {
                                self.grid.save_cursor();
                            } else {
                                self.grid.restore_cursor();
                            }
                        }
                        1000 | 1002 | 1003 => {
                            if action == 'h' {
                                self.mouse_mode = p0;
                            } else if self.mouse_mode == p0 {
                                self.mouse_mode = 0;
                            }
                        }
                        1006 => self.mouse_sgr = action == 'h',
                        _ => {} // bracketed paste, app cursor keys: accept, ignore
                    }
                }
                // non-private SM/RM (autowrap etc.): accepted, no-op
            }
            's' => self.grid.save_cursor(),
            'u' => self.grid.restore_cursor(),
            'c' => {
                // DA1 / DA2
                if intermediates.first() == Some(&b'>') {
                    self.reply(b"\x1b[>0;0;0c");
                } else {
                    self.reply(b"\x1b[?6c");
                }
            }
            'n' => {
                if p0 == 6 {
                    let r = self.grid.cursor_row + 1;
                    let c = self.grid.cursor_col + 1;
                    let msg = format!("\x1b[{};{}R", r, c);
                    self.reply(msg.as_bytes());
                }
            }
            _ => {} // DECSCUSR, window ops, CBT, etc: ignore
        }
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], _ignore: bool, byte: u8) {
        if !intermediates.is_empty() {
            return; // charset selection: ignore
        }
        match byte {
            b'7' => self.grid.save_cursor(),
            b'8' => self.grid.restore_cursor(),
            b'D' => self.grid.lf(),
            b'M' => self.grid.reverse_index(),
            b'E' => self.grid.newline(),
            b'c' => self.grid.reset(),
            _ => {}
        }
    }
}
