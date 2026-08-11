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
    const SEQ: &[u8] = b"\x1b[?25h\x1b[0m\x1b[?1049l\r\n";
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
    out.extend_from_slice(b"\x1b[?1049h\x1b[?25l");
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
                            if dial_pending {
                                // Ctrl+G then a key: live storm dials.
                                dial_pending = false;
                                match b {
                                    b']' => storm.dial_density(1.0),
                                    b'[' => storm.dial_density(-1.0),
                                    b'=' => storm.dial_speed(1.0),
                                    b'-' => storm.dial_speed(-1.0),
                                    b'.' => storm.dial_strike(1.0),
                                    b',' => storm.dial_strike(-1.0),
                                    b'b' => storm.force_strike(),
                                    _ => forwarded.push(b), // not a dial: pass through
                                }
                                continue;
                            }
                            if b == 0x07 {
                                // Ctrl+G arms the dial menu (and is otherwise
                                // the bell — the child never needs it).
                                dial_pending = true;
                                continue;
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

        // Storm + render.
        let dt = last_frame.elapsed().as_secs_f64().min(0.1);
        last_frame = Instant::now();
        storm.tick(dt, perform.grid.cols, perform.grid.rows);
        renderer.render(&perform.grid, &storm, &mut out);
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
            renderer.render(&perform.grid, &storm, &mut out);
            std::io::stdout().write_all(&out).unwrap();
            std::io::stdout().flush().unwrap();
            quit = true;
        }
    }

    let _ = child.kill();
    host_restore();
    std::io::stdout().write_all(b"\x1b[?25h\x1b[0m\x1b[?1049l\r\n").unwrap();
    std::io::stdout().flush().unwrap();
}
