# Repository Guidelines

## Project Overview

**Thunderhead** (binary: `thunderhead`, project dir: `~/repository/thunderhead`) — a minimal
VT terminal emulator in Rust whose
render loop runs an ambient **storm overlay** (rain + lightning) over the live
cell grid. It spawns the user's shell in a PTY, parses its output, and
re-renders a composite of (grid + storm) to the host terminal every frame. The
goal: the storm *actively affects* everything on screen in real time while the
terminal stays fully interactive.

Project dir: `~/repository/gale`. Not a git repo (no `.git`, no `.gitignore`,
no CI). Binary name `thunderhead`.

## Architecture & Data Flow

Single-threaded, poll-driven 60 fps loop (`src/main.rs`):

```
child shell (pty) ──► pty master ──► vte::Parser ──► Perform (vte::Perform impl)
                                                          │  mutates
host stdin (raw) ──► forwarded verbatim to pty ──────────►┴─► Grid (source of truth)
                                                          │
                    Storm::tick(dt)  ─────────────────────┼─► overlay cells (read-only)
                                                          ▼
                    Renderer::render(grid, storm) ──► diff ANSI ──► host stdout (once/frame)
```

- `src/grid.rs` — `Grid`/`Cell`/`Color`: the screen's source of truth. Cursor
  and current SGR state live here; alt screen is a buffer swap (`main_cells`).
  **Invariant: the storm never mutates the grid.**
- `src/perform.rs` — `vte::Perform` impl: VT sequences → grid ops. Queues
  DA1/DA2/DSR replies back to the child.
- `src/storm.rs` — ambient rain + lightning overlay, a port of TTE's
  thunderstorm mechanics: fast single-glyph rain (`\ . ,`, light blue
  `#aaaaff`, 30-90 cells/s; half the drops are wind-responsive and lean
  `/`/`\` with gusts), a lightning FLASH — the full jagged polyline appears
  at once, white-hot for ~0.18s, fading through wake tiers — with the text
  heating by proximity (red/orange/yellow) and bezier rock sparks on impact.
  `tick()` advances drops/bolt/sparks/glow; `overlay(base, row, col) ->
  Option<Cell>` composites. Structurally read-only w.r.t. the grid. Storm
  cells inherit `base.bg` so they don't punch through painted backgrounds.
  Colors render via `38;2;` RGB — do not regress the SGR prefix bug
  (`b'3'` formats as 51 → invalid `518;2;` the terminal silently ignores).
- `src/render.rs` — diff renderer: composite = grid + storm per cell (storm
  wins by full cell replacement), reverse-video cursor overlay, emits only
  changed runs (`\x1b[r;cH` + SGR) into a per-frame buffer.
- `src/main.rs` — host tty raw mode + SIGTERM/SIGHUP/SIGINT safety net (restore
  termios + leave alt screen before dying), PTY spawn via portable-pty, resize
  propagation (host size → grid → renderer → child pty), Ctrl+Q double-tap
  (within 1.2s) exits the storm.

## Key Directories

```
src/
  main.rs     entry point + main loop (~220 lines)
  grid.rs     Grid/Cell/Color, VT cursor/editing/scroll/alt-screen ops (~350)
  perform.rs  vte::Perform dispatch (~200)
  storm.rs    storm overlay (~150)
  render.rs   diff renderer (~145)
```

## Development Commands

```sh
cargo build            # debug
cargo build --release  # optimized; binary at target/release/thunderhead
cargo run              # run (needs a real terminal; headless: pipe stdin → exits on EOF)
```

Dependencies (all caret ranges): `libc 0.2`, `vte 0.14` (VT parser; note
`Parser::advance(&mut perform, bytes)` — there is no `parse` method),
`portable-pty 0.9` (note `MasterPty::as_raw_fd()` returns `Option<RawFd>`).

## Code Conventions & Common Patterns

- **Unsafe is confined to libc calls** (termios, signal, poll, write-in-handler)
  with a rationale comment at the top of the enclosing block.
- **Error handling**: setup failures → `eprintln!` + `process::exit(1)`; fatal
  hot-path writes `.unwrap()`; non-fatal → `let _ =`.
- **Borrow discipline**: never hold `&mut self.grid` across `self`-method calls
  (E0502); index grids via the free `cell_index(cols, r, c)` function — a
  method call in index position does not compile.
- **Defensive indexing** everywhere: `saturating_*`, `.min`/`.clamp`.
- **Indexing is 0-based internally**; VT params are 1-based
  (`p0.max(1) - 1`).
- Terse WHY-comments, no doc-comment ceremony. Single `Vec<u8>` out buffer
  flushed once per frame.
- Known API gotchas (verified against vendored sources):
  `vte::Perform::hook` takes `action: char` (not `Action`);
  `portable-pty::MasterPty::as_raw_fd() -> Option<RawFd>`.

## Important Files

- `src/main.rs` — where the loop lives; add host I/O or lifecycle logic here.
- `src/perform.rs` — where new VT sequences get dispatched (match arms on the
  CSI final byte; unknown sequences are swallowed with `_ => {}`).
- `src/storm.rs` — where new ambient effects go; must stay read-only vs the grid.
- `src/render.rs` — where new composite/overlay rules go.

## Runtime/Tooling Preferences

- Rust edition 2021; stable toolchain (1.97 in use). No rust-toolchain file.
- Linux/Unix targets (host-termios + `poll` code is Unix-only; WSL2 works).
- No formatter/linter config; `cargo fmt` defaults.
- Shell spawned is `$SHELL` (fallback `/bin/bash -i`), `TERM=xterm-256color`,
  `COLORTERM=truecolor`.

## Testing & QA

- **No test infrastructure exists** — no `#[test]`, no `tests/`, no CI.
- Headless smoke: `timeout 3 ./target/release/thunderhead </dev/null` → should
  emit the takeover sequence + a rendered first frame, then exit 0 on stdin EOF.
- Interactive verification requires a real terminal (host raw mode + alt
  screen). Verify: prompt renders, typing echoes, `vim`/`less` work (alt
  screen), Ctrl+Q Ctrl+Q exits and the terminal is restored.
- Known WIP limitations (do not be surprised by these): no scrollback
  (scrolling discards rows); tab stops hardcoded every 8; application cursor
  keys (DECCKM) accepted-but-ignored; DA1 reply is `\x1b[?6c` (nonstandard,
  cosmetic); storm `overlay` is O(rows×cols×drops) per frame; wide/CJK chars
  render as width-1; renderer emits a reset SGR on every style change.
