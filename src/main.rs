//! gale (stormterm): a minimal VT terminal emulator whose render loop runs an
//! ambient storm over the live cell grid. The grid is the truth; the storm is
//! a per-frame composite overlay, so typing/bash/vim flow through untouched.

mod grid;
mod perform;
mod render;
mod storm;

use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

use crate::grid::Color;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use vte::Parser as VteParser;

// ---------------------------------------------------------------------------
// Host terminal state + signal safety net (learned the hard way in ttfx: a
// panic or kill must never leave the user's tty in raw mode).
// ---------------------------------------------------------------------------

static HOST_SAVED: OnceLock<libc::termios> = OnceLock::new();
static CLEANUP_DONE: AtomicBool = AtomicBool::new(false);

fn host_termios_save() -> Option<libc::termios> {
    let mut t: libc::termios = unsafe { std::mem::zeroed() };
    if unsafe { libc::tcgetattr(0, &mut t) } == 0 {
        Some(t)
    } else {
        None
    }
}

fn host_raw_on() {
    if let Some(t) = host_termios_save() {
        let _ = HOST_SAVED.set(t);
        let mut raw = t;
        raw.c_lflag &= !(libc::ICANON | libc::ECHO | libc::ISIG | libc::IEXTEN);
        raw.c_iflag &= !(libc::IXON | libc::ICRNL | libc::INLCR | libc::IGNCR | libc::ISTRIP);
        raw.c_oflag &= !(libc::OPOST);
        unsafe {
            libc::tcsetattr(0, libc::TCSANOW, &raw);
        }
    }
}

fn host_restore() {
    if CLEANUP_DONE.swap(true, Ordering::SeqCst) {
        return;
    }
    if let Some(t) = HOST_SAVED.get() {
        unsafe {
            libc::tcsetattr(0, libc::TCSANOW, t);
        }
    }
}

extern "C" fn cleanup_handler(sig: libc::c_int) {
    host_restore();
    const SEQ: &[u8] = b"\x1b[?1006l\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?25h\x1b[0m\r\n";
    unsafe {
        libc::write(1, SEQ.as_ptr() as *const libc::c_void, SEQ.len());
        libc::signal(sig, libc::SIG_DFL);
        libc::raise(sig);
    }
}

fn host_size() -> (usize, usize) {
    let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
    if unsafe { libc::ioctl(1, libc::TIOCGWINSZ, &mut ws) } == 0 && ws.ws_col > 0 && ws.ws_row > 0 {
        (ws.ws_row as usize, ws.ws_col as usize)
    } else {
        (24, 80)
    }
}

/// Apply a dial key to the storm; returns true if the key was a dial.
fn dial(storm: &mut storm::Storm, b: u8) -> bool {
    match b {
        b']' => storm.dial_density(1.0),
        b'[' => storm.dial_density(-1.0),
        b'=' => storm.dial_speed(1.0),
        b'-' => storm.dial_speed(-1.0),
        b'.' => storm.dial_strike(1.0),
        b',' => storm.dial_strike(-1.0),
        b'b' => storm.force_strike(),
        b'1' => storm.dial_meteor_rate(-1.0),
        b'2' => storm.dial_meteor_rate(1.0),
        b'3' => storm.dial_meteor_size(-1.0),
        b'4' => storm.dial_meteor_size(1.0),
        _ => return false,
    }
    true
}

/// HUD text color: the blue used for labels/status.
const HUD_BLUE: Color = Color::Rgb(0x68, 0xA3, 0xE8);
/// Toggle ON: green.
const HUD_ON: Color = Color::Rgb(0x3F, 0xC9, 0x5B);
/// Toggle OFF: red.
const HUD_OFF: Color = Color::Rgb(0xF2, 0x5F, 0x5C);

/// One HUD line as (char, fg) cells.
fn hud_line(s: &str, fg: Color) -> Vec<(char, Color)> {
    s.chars().map(|ch| (ch, fg)).collect()
}

/// Refresh the live panel lines: 0-3 status block, 4 swatch, 5 fx list,
/// 7-14 keybind rows (each key+name colored by its toggle state).
fn refresh_hud(hud: &mut [Vec<(char, Color)>], storm: &storm::Storm) {
    for (i, line) in storm.status_lines().iter().enumerate() {
        hud[i] = hud_line(line, HUD_BLUE);
    }
    // 4: palette swatch — chips in the live storm colors
    let mut swatch = hud_line("colors   ● ● ● ●", HUD_BLUE);
    let sw = storm.swatch();
    for (ci, entry) in swatch.iter_mut().enumerate() {
        if entry.0 == '●' && ci >= 9 && (ci - 9) % 2 == 0 {
            let (r, g, b) = sw[(ci - 9) / 2];
            entry.1 = Color::Rgb(r, g, b);
        }
    }
    hud[4] = swatch;
    hud[5] = hud_line(&storm.fx_list(), HUD_BLUE);

    // 7-14: keybind rows, colored by state. Each pair is `key name` left
    // (padded to 12) + `key name` right; the last row has one token.
    let st = storm.toggle_states();
    let row = |hud: &mut [Vec<(char, Color)>], idx: usize, a: usize, b: Option<usize>| {
        let mut cells = Vec::with_capacity(24);
        for (i, tok) in [Some(a), b].into_iter().flatten().enumerate() {
            let (key, name, on) = st[tok];
            let fg = if on { HUD_ON } else { HUD_OFF };
            let mut tok_cells = hud_line(&format!("{key} {name}"), fg);
            if i == 0 {
                while tok_cells.len() < 12 {
                    tok_cells.push((' ', HUD_BLUE));
                }
            }
            cells.extend(tok_cells);
        }
        hud[idx] = cells;
    };
    row(hud, 7, 0, Some(1));   // r rain   t trails
    row(hud, 8, 2, Some(3));   // c corona k shake
    row(hud, 9, 4, Some(5));   // F forks  e embers
    row(hud, 10, 6, Some(7));  // s splash g fronts
    row(hud, 11, 8, Some(9));  // f fog    h hail
    row(hud, 12, 10, Some(11)); // a aurora m matrix
    // 13: M meteors + C randomize — C is a one-shot, always blue
    let mut mrow = hud_line(&format!("M meteors"), if st[12].2 { HUD_ON } else { HUD_OFF });
    while mrow.len() < 12 {
        mrow.push((' ', HUD_BLUE));
    }
    mrow.extend(hud_line("C randomize", HUD_BLUE));
    hud[13] = mrow;
    // 14: U ufo
    hud[14] = hud_line(&format!("U ufo"), if st[13].2 { HUD_ON } else { HUD_OFF });
}

#[test]
fn mouse_mode_parse_works() {
    let mut p = crate::perform::Perform::new(24, 80);
    let mut vte_parser = vte::Parser::new();
    vte_parser.advance(&mut p, b"\x1b[?1002h\x1b[?1006h");
    assert_eq!(p.mouse_mode, 1002, "tracking mode must be set");
    assert!(p.mouse_sgr, "sgr encoding must be set");
    vte_parser.advance(&mut p, b"\x1b[?1002l\x1b[?1006l");
    assert_eq!(p.mouse_mode, 0, "tracking must clear");
    assert!(!p.mouse_sgr);
}

fn main() {
    unsafe {
        libc::signal(libc::SIGTERM, cleanup_handler as *const () as usize);
        libc::signal(libc::SIGHUP, cleanup_handler as *const () as usize);
        libc::signal(libc::SIGINT, cleanup_handler as *const () as usize);
    }

    let (rows, cols) = host_size();

    // Spawn the user's shell in a pty.
    let pty_system = native_pty_system();
    let pair = match pty_system.openpty(PtySize {
        rows: rows as u16,
        cols: cols as u16,
        pixel_width: 0,
        pixel_height: 0,
    }) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("openpty failed: {e}");
            std::process::exit(1);
        }
    };
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    let mut cmd = CommandBuilder::new(&shell);
    cmd.arg("-i");
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");
    let mut child = match pair.slave.spawn_command(cmd) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("spawn failed: {e}");
            std::process::exit(1);
        }
    };
    drop(pair.slave);
    let mut reader = pair.master.try_clone_reader().expect("master reader");
    let mut writer = pair.master.take_writer().expect("master writer");
    let master_fd = pair.master.as_raw_fd().expect("master pty fd");

    // Take over the host terminal.
    host_raw_on();
    let mut out = Vec::with_capacity(1 << 16);
    out.extend_from_slice(b"\x1b[?25l"); // no alt screen: the host scrollback takes the wheel
    std::io::stdout().write_all(&out).unwrap();
    std::io::stdout().flush().unwrap();
    out.clear();

    let mut perform = perform::Perform::new(rows, cols);
    let mut storm = storm::Storm::new();
    let mut renderer = render::Renderer::new(rows, cols);
    let mut vte = VteParser::new();
    let mut stdin = std::io::stdin();
    let mut last_frame = Instant::now();
    let mut quit = false;
    let mut q_times: Vec<Instant> = Vec::new();
    let mut dial_pending = false;
    // Escape-sequence passthrough state, persists across reads so a CSI
    // sequence split by the terminal (ESC ... then the rest) is never
    // dial-matched mid-stream. 0 = none, 1 = saw ESC, 2 = inside CSI/SS3.
    let mut esc_seq: u8 = 0;
    let mut hud_on = false;
    // last mouse-mode state mirrored to the host terminal
    let mut last_mouse_mode: u16 = 0;
    let mut last_mouse_sgr = false;
    // The HUD panel: a vertical control deck, floats mid-right of the screen.
    // Line indexes are stable: 0-3 status, 4 palette swatch, 5 fx, 6-14
    // toggles, 15 dials, 16-18 blank/dial-footer. Toggle rows are rebuilt
    // by refresh_hud with per-key colors; blanks/dials are static.
    let mut hud_lines: Vec<Vec<(char, Color)>> = vec![
        Vec::new(), // 0: storm  rain  45%
        Vec::new(), // 1: speed
        Vec::new(), // 2: strike
        Vec::new(), // 3: meteor
        Vec::new(), // 4: palette swatch (refresh_hud paints the chips)
        Vec::new(), // 5: fx list
        Vec::new(), // 6: blank
        Vec::new(), // 7-14: toggle rows, colored by state
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(), // blank
        hud_line("1 / 2 meteor rate   3 / 4 meteor size", HUD_BLUE), // dials
        hud_line("] / [ density       = / - speed", HUD_BLUE), // 16
        hud_line(". / , strikes       b bolt", HUD_BLUE), // 17
        hud_line("Ctrl+G h close", HUD_BLUE), // 18
    ];
    refresh_hud(&mut hud_lines, &storm);

    while !quit {
        // Resize: propagate host size to grid, renderer, and the child pty.
        let (nrows, ncols) = host_size();
        if nrows != perform.grid.rows || ncols != perform.grid.cols {
            perform.grid.resize(nrows, ncols);
            renderer.force_redraw();
            let _ = pair.master.resize(PtySize { rows: nrows as u16, cols: ncols as u16, pixel_width: 0, pixel_height: 0 });
        }

        // Poll the pty and the host stdin (~60 fps).
        let mut fds = [
            libc::pollfd { fd: master_fd, events: libc::POLLIN, revents: 0 },
            libc::pollfd { fd: 0, events: libc::POLLIN, revents: 0 },
        ];
        let pr = unsafe { libc::poll(fds.as_mut_ptr(), 2, 16) };
        if pr > 0 {
            if fds[0].revents & libc::POLLIN != 0 {
                let mut buf = [0u8; 32768];
                match reader.read(&mut buf) {
                    Ok(0) => quit = true, // pty EOF: shell exited
                    Ok(n) => {
                        vte.advance(&mut perform, &buf[..n]);
                        if !perform.replies.is_empty() {
                            let _ = writer.write_all(&perform.replies);
                            perform.replies.clear();
                        }
                    }
                    Err(_) => quit = true,
                }
            }
            if fds[1].revents & libc::POLLIN != 0 {
                let mut buf = [0u8; 4096];
                match stdin.read(&mut buf) {
                    Ok(0) => quit = true,
                    Ok(n) => {
                        let mut forwarded = Vec::with_capacity(n);
                        for &b in &buf[..n] {
                            if esc_seq != 0 {
                                forwarded.push(b);
                                esc_seq = match (esc_seq, b) {
                                    (1, b'[') | (1, b'O') => 2,   // CSI / SS3
                                    (1, _) => 0,                  // Alt+key
                                    (2, b) if (0x40..=0x7E).contains(&b) => 0, // final byte
                                    _ => esc_seq,
                                };
                                continue;
                            }
                            if dial_pending {
                                // Ctrl+G then a key: live storm dials.
                                dial_pending = false;
                                if b == b'h' {
                                    // Ctrl+G h toggles the persistent HUD.
                                    hud_on = !hud_on;
                                    refresh_hud(&mut hud_lines, &storm);
                                } else if dial(&mut storm, b) {
                                    refresh_hud(&mut hud_lines, &storm);
                                } else {
                                    if b == 0x1B && hud_on {
                                        esc_seq = 1; // Ctrl+G then arrow: don't eat the `[`
                                    }
                                    forwarded.push(b); // not a dial: pass through
                                }
                                continue;
                            }
                            if b == 0x07 {
                                // Ctrl+G arms the dial menu (and is otherwise
                                // the bell — the child never needs it).
                                dial_pending = true;
                                continue;
                            }
                            if hud_on {
                                // HUD visible = dials armed: no Ctrl+G needed.
                                if b == 0x1B {
                                    forwarded.push(b);
                                    esc_seq = 1;
                                    continue;
                                }
                                if b == b'C' {
                                    // re-roll the palette, for shits and giggles
                                    storm.randomize_colors();
                                    continue;
                                }
                                if dial(&mut storm, b) {
                                    refresh_hud(&mut hud_lines, &storm);
                                    continue;
                                }
                                // effect toggles (r t c k F e s g f h a m M)
                                if storm.toggle_effect(b).is_some() {
                                    refresh_hud(&mut hud_lines, &storm);
                                    continue;
                                }
                            }
                            if b == 0x11 {
                                // Ctrl+Q twice within 1.2s exits the storm.
                                let now = Instant::now();
                                q_times.retain(|t| now.duration_since(*t) < std::time::Duration::from_millis(1200));
                                q_times.push(now);
                                if q_times.len() >= 2 {
                                    quit = true;
                                }
                            }
                            forwarded.push(b);
                        }
                        if !forwarded.is_empty() {
                            let _ = writer.write_all(&forwarded);
                        }
                    }
                    Err(_) => {}
                }
            }
        }

        // Mirror the child's mouse-mode request to the host terminal: with
        // tracking on, the host routes the wheel to us as SGR mouse events
        // (which we forward) instead of scrolling its native scrollback.
        let want = perform.mouse_mode;
        let want_sgr = perform.mouse_sgr;
        if want != last_mouse_mode || want_sgr != last_mouse_sgr {
            if last_mouse_mode != 0 {
                out.extend_from_slice(b"\x1b[?1006l");
                out.extend_from_slice(format!("\x1b[?{}l", last_mouse_mode).as_bytes());
            }
            if want != 0 {
                out.extend_from_slice(format!("\x1b[?{}h", want).as_bytes());
                if want_sgr {
                    out.extend_from_slice(b"\x1b[?1006h");
                }
            }
            last_mouse_mode = want;
            last_mouse_sgr = want_sgr;
        }

        // Storm + render.
        let dt = last_frame.elapsed().as_secs_f64().min(0.1);
        last_frame = Instant::now();
        storm.tick(dt, perform.grid.cols, perform.grid.rows);
        let hud_view = if hud_on { Some(hud_lines.as_slice()) } else { None };
        renderer.render(&mut perform.grid, &storm, hud_view, &mut out);
        if !out.is_empty() {
            std::io::stdout().write_all(&out).unwrap();
            std::io::stdout().flush().unwrap();
            out.clear();
        }

        // Child exited: drain, repaint, quit.
        if let Ok(Some(_)) = child.try_wait() {
            let mut buf = [0u8; 32768];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => vte.advance(&mut perform, &buf[..n]),
                }
            }
            renderer.force_redraw();
            renderer.render(&mut perform.grid, &storm, None, &mut out);
            std::io::stdout().write_all(&out).unwrap();
            std::io::stdout().flush().unwrap();
            quit = true;
        }
    }

    let _ = child.kill();
    host_restore();
    std::io::stdout()
        .write_all(b"\x1b[?1006l\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?25h\x1b[0m\r\n")
        .unwrap();
    std::io::stdout().flush().unwrap();
}
