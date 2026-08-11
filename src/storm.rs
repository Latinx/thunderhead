//! Thunderhead's storm overlay — a faithful port of TTE's thunderstorm
//! mechanics (effects/effect_thunderstorm.py, ported in ttfx) adapted to a
//! live grid, plus two techniques from omp's shimmer (KITT quadratic trail,
//! three-tier palette):
//!
//! - Rain: fast single-glyph drops in light blue `#aaaaff`, ~30-90 cells/s.
//!   Half the drops are vertical (`.`, `,`) and always fall straight; half
//!   are wind-responsive — they drift sideways and lean `/` (wind left) or
//!   `\` (wind right) as gusts pass.
//! - Lightning: a `|` strike character travels DOWN as a jagged polyline
//!   (connector glyphs `|`/`\`/`/` per direction change), leaving a KITT-style
//!   quadratic-decay wake (f·f, three-tier palette). Short tendrils fork off
//!   on jogs and die after ~0.3-0.5s.
//! - Impact: struck text heats by proximity — red nearest the strike, orange
//!   mid, yellow far — and cools back; slow chunky sparks (`* o . '`) fly
//!   along beziers with ease-out, cooling from orange `#ff4d00`.
//!
//! The grid is the truth; the storm only composites at render time.

use crate::grid::{Cell, Color};

// ─── Tunables ───────────────────────────────────────────────────────────────
const RAIN_COLOR: Color = Color::Rgb(0xAA, 0xAA, 0xFF);
const SPARK_COLOR: Color = Color::Rgb(0xFF, 0x4D, 0x00);
const WIND_DRIFT: f64 = 12.0; // cells/s of horizontal drift at full wind
const WIND_EPS: f64 = 0.15; // below this the rain falls vertically (`.`/`,`)
const BOLT_TTL: f64 = 0.5; // seconds the flash line stays lit — instant to appear, but it lingers long enough to read
const GLOW_RADIUS: f64 = 12.0; // proximity radius of the strike glow (columns)
const TIER_HIGH: f64 = 0.65; // KITT wake tier thresholds (omp shimmer)
const TIER_MID: f64 = 0.22;

// ─── Pure helpers (unit-tested) ─────────────────────────────────────────────

/// Glyph for a wind-blown drop: `/` when the wind blows left, `\` when right,
/// `None` when calm (drops stay vertical).
fn lean_for(wind: f64) -> Option<char> {
    if wind < -WIND_EPS {
        Some('/')
    } else if wind > WIND_EPS {
        Some('\\')
    } else {
        None
    }
}

/// KITT wake intensity: quadratic decay `f*f` where f = 1 - age (omp shimmer).
fn kitt_intensity(age: f64) -> f64 {
    let f = (1.0 - age.clamp(0.0, 1.0)).max(0.0);
    f * f
}

/// Three-tier wake palette (omp shimmer thresholds): crest bold, mid, dim.
fn wake_tier(intensity: f64) -> (Color, bool) {
    if intensity >= TIER_HIGH {
        (Color::Rgb(0xBF, 0xD5, 0xFF), true)
    } else if intensity >= TIER_MID {
        (Color::Rgb(0x68, 0xA3, 0xE8), false)
    } else {
        (Color::Rgb(0x2A, 0x40, 0x70), false)
    }
}

/// Strike heat ramp: red nearest the strike, orange mid, yellow far.
fn heat_color(strength: f64) -> (u8, u8, u8) {
    let s = strength.clamp(0.0, 1.0);
    if s > 0.5 {
        let t = (s - 0.5) * 2.0; // orange -> red as proximity peaks
        (255, lerp(140, 60, t), lerp(40, 20, t))
    } else {
        let t = s * 2.0; // yellow -> orange
        (255, lerp(210, 140, t), lerp(80, 40, t))
    }
}

fn lerp(a: u8, b: u8, t: f64) -> u8 {
    (a as f64 + (b as f64 - a as f64) * t).clamp(0.0, 255.0) as u8
}

// ─── Storm entities ─────────────────────────────────────────────────────────

#[derive(Debug)]
struct Drop {
    col: f64,
    row: f64,
    speed: f64,
    glyph: char,
    /// '.' or ',' — the drop's glyph when falling vertical
    vertical: char,
    /// wind-responsive drops lean `/`/`\`; vertical drops always fall straight
    leanable: bool,
}

#[derive(Debug)]
struct Bolt {
    /// the flash's jagged polyline: (row, col, connector glyph, ttl)
    path: Vec<(i64, i64, char, f64)>,
}

#[derive(Debug)]
struct Spark {
    x0: f64,
    y0: f64,
    cx: f64,
    cy: f64,
    x1: f64,
    y1: f64,
    t: f64,
    dur: f64,
    glyph: char,
}

pub struct Storm {
    drops: Vec<Drop>,
    bolt: Option<Bolt>,
    sparks: Vec<Spark>,
    next_strike: f64,
    t: f64,
    rng: u64,
    /// per-cell remaining glow strength (rows*cols), resized on demand
    glow: Vec<f64>,
    glow_rows: usize,
    glow_cols: usize,
    /// wind: -1 (blowing left) .. 1 (blowing right); eases toward gusts
    wind: f64,
    wind_target: f64,
    next_gust: f64,
    spark_glyphs: [char; 4],
    /// live dials (Ctrl+G + key in the running terminal)
    rain_density: f64,          // fraction of columns with falling rain
    rain_speed: (f64, f64),     // (min, max) cells/s
    strike_interval: (f64, f64), // (min, max) seconds between strikes
}

impl Storm {
    pub fn new() -> Self {
        Storm {
            drops: Vec::new(),
            bolt: None,
            sparks: Vec::new(),
            next_strike: 2.0,
            t: 0.0,
            rng: 0x9E3779B97F4A7C15,
            glow: Vec::new(),
            glow_rows: 0,
            glow_cols: 0,
            wind: 0.0,
            wind_target: 0.0,
            next_gust: 3.0,
            spark_glyphs: ['*', '.', '\'', 'o'],
            rain_density: 0.45,
            rain_speed: (30.0, 90.0),
            strike_interval: (4.5, 8.0),
        }
    }

    fn rand(&mut self) -> f64 {
        // xorshift64*
        let mut x = self.rng;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng = x;
        let out = x.wrapping_mul(0x2545F4914F6CDD1D);
        (out >> 11) as f64 / (1u64 << 53) as f64
    }

    // ─── live dials (Ctrl+G + key) ─────────────────────────────────────────

    /// `[` / `]` — rain density, clamped to 10-90% of columns.
    pub fn dial_density(&mut self, dir: f64) {
        self.rain_density = (self.rain_density + 0.1 * dir).clamp(0.1, 0.9);
    }

    /// `-` / `=` — rain fall speed, ±25% per step, clamped to sane bounds.
    pub fn dial_speed(&mut self, dir: f64) {
        let f = 1.0 + 0.25 * dir;
        self.rain_speed =
            ((self.rain_speed.0 * f).clamp(5.0, 200.0), (self.rain_speed.1 * f).clamp(10.0, 300.0));
    }

    /// `,` / `.` — strike frequency, ∓20% on the interval per step.
    pub fn dial_strike(&mut self, dir: f64) {
        let f = 1.0 + 0.2 * dir;
        self.strike_interval = ((self.strike_interval.0 * f).clamp(1.0, 15.0), (self.strike_interval.1 * f).clamp(2.0, 25.0));
    }

    /// `b` — force a strike on the next tick.
    pub fn force_strike(&mut self) {
        self.next_strike = self.t;
    }

    /// Current dial values, for the on-screen HUD.
    pub fn status(&self) -> String {
        format!(
            "storm  rain {:>3.0}%  speed {:>3.0}-{:>3.0} c/s  strike {:.1}-{:.1}s",
            self.rain_density * 100.0,
            self.rain_speed.0,
            self.rain_speed.1,
            self.strike_interval.0,
            self.strike_interval.1,
        )
    }

    fn ensure_glow(&mut self, rows: usize, cols: usize) {
        if self.glow_rows != rows || self.glow_cols != cols {
            self.glow = vec![0.0; rows * cols];
            self.glow_rows = rows;
            self.glow_cols = cols;
        }
    }

    fn set_glow(&mut self, r: usize, c: usize, amount: f64) {
        if r < self.glow_rows && c < self.glow_cols {
            let i = r * self.glow_cols + c;
            if amount > self.glow[i] {
                self.glow[i] = amount;
            }
        }
    }

    fn glow_at(&self, r: usize, c: usize) -> f64 {
        if r < self.glow_rows && c < self.glow_cols {
            self.glow[r * self.glow_cols + c]
        } else {
            0.0
        }
    }

    fn spawn_drop(&mut self, cols: usize, rows: usize) {
        let col = self.rand() * cols as f64;
        let row = -(rows as f64) * self.rand() * 0.4; // start above the screen
        let speed = self.rain_speed.0 + self.rand() * (self.rain_speed.1 - self.rain_speed.0);
        let vertical = if self.rand() < 0.5 { '.' } else { ',' };
        // half the drops always fall straight; half lean with the wind
        let leanable = self.rand() < 0.5;
        self.drops.push(Drop { col, row, speed, glyph: vertical, vertical, leanable });
    }

    /// Strike: the whole jagged bolt appears at once (a real flash is ~1/10s),
    /// and the text heats + rocks burst the same instant it lands.
    fn strike(&mut self, cols: usize, rows: usize) {
        let mut x = (1 + (self.rand() * (cols.saturating_sub(2) as f64)) as i64).max(0);
        let mut prev = x;
        let mut path: Vec<(i64, i64, char, f64)> = Vec::new();
        for r in 0..rows {
            if r > 0 && self.rand() < 0.45 && x > 0 && x + 1 < cols as i64 {
                x += if self.rand() < 0.5 { 1 } else { -1 };
            }
            let glyph = if x == prev {
                '|'
            } else if x > prev {
                '\\'
            } else {
                '/'
            };
            path.push((r as i64, x, glyph, BOLT_TTL));
            // short horizontal tendrils fork off along the way
            if self.rand() < 0.20 {
                let dir: i64 = if self.rand() < 0.5 { 1 } else { -1 };
                let len = 2 + (self.rand() * 4.0) as i64;
                for i in 1..=len {
                    let bx = x + dir * i;
                    if bx < 0 || bx >= cols as i64 {
                        break;
                    }
                    path.push((r as i64, bx, '-', BOLT_TTL));
                }
            }
            prev = x;
        }
        self.bolt = Some(Bolt { path });
        // impact at the bottom of the bolt
        self.impact(x as f64, rows);
        let (lo, hi) = self.strike_interval;
        self.next_strike = self.t + lo + self.rand() * (hi - lo);
    }

    /// Advance the wind toward a target that re-rolls periodically (gusts).
    fn gust(&mut self, dt: f64) {
        if self.t >= self.next_gust {
            self.wind_target = (self.rand() * 2.0 - 1.0) * (0.3 + self.rand() * 0.7);
            self.next_gust = self.t + 3.0 + self.rand() * 4.0;
        }
        self.wind += (self.wind_target - self.wind) * dt * 1.2;
        if self.wind.abs() < 0.01 {
            self.wind = 0.0;
        }
    }

    /// Impact at (col, rows-1): light the text by proximity (heat ramp) and
    /// burst slow rock-like sparks.
    fn impact(&mut self, col: f64, rows: usize) {
        // text glow: strength falls off with column distance from the strike
        let ci = col as i64;
        let radius = GLOW_RADIUS as i64;
        for r in 0..self.glow_rows {
            for dc in -radius..=radius {
                let c = ci + dc;
                if c < 0 || c >= self.glow_cols as i64 {
                    continue;
                }
                let s = (1.0 - dc.abs() as f64 / GLOW_RADIUS).max(0.0);
                self.set_glow(r, c as usize, s);
            }
        }
        // rock burst: 10-20 sparks along beziers, slow and chunky
        let count = 10 + (self.rand() * 11.0) as usize;
        let y0 = rows.saturating_sub(1) as f64;
        for _ in 0..count {
            let dir: f64 = if self.rand() < 0.5 { 1.0 } else { -1.0 };
            let offset = 4.0 + self.rand() * 16.0;
            let x1 = (col + dir * offset).max(0.0);
            let cx = col - (col - x1) / 2.0;
            let cy = self.rand() * rows as f64 * 0.5;
            let dur = 0.8 + self.rand() * 1.0;
            let glyph = self.spark_glyphs[(self.rand() * 4.0) as usize];
            self.sparks.push(Spark { x0: col, y0, cx, cy, x1, y1: y0, t: 0.0, dur, glyph });
        }
    }

    /// Advance the storm by dt seconds over a cols x rows canvas.
    pub fn tick(&mut self, dt: f64, cols: usize, rows: usize) {
        self.t += dt;
        self.ensure_glow(rows, cols);

        // decay the text glow
        for g in self.glow.iter_mut() {
            *g = (*g - dt * 0.9).max(0.0); // ~1.1s to cool
        }

        self.gust(dt);

        // rain: fast TTE-style drops across a dialable share of columns
        let target = ((cols as f64) * self.rain_density).max(4.0) as usize;
        while self.drops.len() < target {
            self.spawn_drop(cols, rows);
        }
        let lean = lean_for(self.wind);
        for d in self.drops.iter_mut() {
            d.row += d.speed * dt;
            if d.leanable {
                // wind-blown: drifts sideways and leans with the gust
                d.col += self.wind * WIND_DRIFT * dt;
                d.glyph = lean.unwrap_or(d.vertical);
            } else {
                // vertical drops keep falling straight, always
                d.glyph = d.vertical;
            }
        }
        self.drops.retain(|d| d.row < rows as f64 + 1.0);

        // lightning: an instant full-line flash, then the line fades
        if self.bolt.is_none() && self.t >= self.next_strike {
            self.strike(cols, rows);
        }
        if let Some(b) = &mut self.bolt {
            for (_, _, _, ttl) in b.path.iter_mut() {
                *ttl -= dt;
            }
            b.path.retain(|&(_, _, _, ttl)| ttl > 0.0);
            if b.path.is_empty() {
                self.bolt = None;
            }
        }

        // sparks: advance along their beziers
        for s in self.sparks.iter_mut() {
            s.t += dt / s.dur;
        }
        self.sparks.retain(|s| s.t < 1.0);
    }

    /// bezier point at eased t (ease-out quint, TTE OutQuint on the path)
    fn spark_pos(&self, s: &Spark) -> (f64, f64) {
        let t = 1.0 - (1.0 - s.t).powi(5);
        let mt = 1.0 - t;
        let x = mt * mt * s.x0 + 2.0 * mt * t * s.cx + t * t * s.x1;
        let y = mt * mt * s.y0 + 2.0 * mt * t * s.cy + t * t * s.y1;
        (x, y)
    }

    /// The composite cell at (row, col): the storm's take on the base cell.
    pub fn overlay(&self, base: Cell, row: usize, col: usize) -> Option<Cell> {
        // Never draw over a wide char's second column: a width-1 glyph placed
        // on a continuation desyncs the diff (the wide char's write advances
        // the terminal cursor two columns, misplacing the next run cell).
        if base.width == 0 {
            return None;
        }
        // lightning flash: the whole jagged line appears at once, white-hot,
        // fading through the wake tiers; keeps the base background so it
        // doesn't punch through painted backgrounds (e.g. nvim)
        if let Some(b) = &self.bolt {
            for &(tr, tc, glyph, ttl) in &b.path {
                if tr == row as i64 && tc == col as i64 {
                    let age = (1.0 - ttl / BOLT_TTL).max(0.0).min(1.0);
                    let intensity = kitt_intensity(age);
                    let (fg, bold) = if intensity >= 0.8 {
                        (Color::Rgb(0xFF, 0xFF, 0xFF), true) // fresh flash: white
                    } else {
                        wake_tier(intensity)
                    };
                    return Some(Cell {
                        ch: glyph,
                        fg,
                        bg: base.bg,
                        bold,
                        reverse: false,
                        width: 1,
                    });
                }
            }
        }

        // sparks: flying glyphs cooling from orange; keep the base background
        for s in &self.sparks {
            let (x, y) = self.spark_pos(s);
            if y.floor() as usize == row && x.floor() as usize == col {
                let cool = (s.t * 1.2).min(1.0);
                let (sr, sg, sb) = match SPARK_COLOR {
                    Color::Rgb(r, g, b) => (r, g, b),
                    _ => (0xFF, 0x4D, 0x00),
                };
                return Some(Cell {
                    ch: s.glyph,
                    fg: Color::Rgb(lerp(sr, 0x28, cool), lerp(sg, 0x30, cool), lerp(sb, 0x40, cool)),
                    bg: base.bg,
                    bold: s.glyph == '*',
                    reverse: false,
                    width: 1,
                });
            }
        }

        // text glow: struck text heats by proximity — red nearest the strike,
        // orange mid, yellow far — and cools back to its base color
        let glow = self.glow_at(row, col);
        if glow > 0.0 && base.ch != ' ' {
            let (hr, hg, hb) = heat_color(glow);
            let mut lit = base;
            lit.fg = Color::Rgb(hr, hg, hb);
            lit.bold = glow > 0.55;
            return Some(lit);
        }

        // rain: fast single-glyph drops in light blue; the drop keeps the
        // base cell's background so it doesn't punch holes through app-painted
        // backgrounds (e.g. nvim) — only the glyph and color change
        for d in &self.drops {
            if d.col as usize == col && d.row as usize == row {
                return Some(Cell {
                    ch: d.glyph,
                    fg: RAIN_COLOR,
                    bg: base.bg,
                    bold: false,
                    reverse: false,
                    width: 1,
                });
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lean_maps_wind_direction() {
        assert_eq!(lean_for(-1.0), Some('/'));
        assert_eq!(lean_for(1.0), Some('\\'));
        assert_eq!(lean_for(0.05), None);
    }

    #[test]
    fn kitt_intensity_decays_quadratically() {
        assert!((kitt_intensity(0.0) - 1.0).abs() < 1e-9);
        assert!((kitt_intensity(0.5) - 0.25).abs() < 1e-9);
        assert!((kitt_intensity(1.0) - 0.0).abs() < 1e-9);
        assert!((kitt_intensity(2.0) - 0.0).abs() < 1e-9); // clamped
    }

    #[test]
    fn wake_tiers_follow_thresholds() {
        let (c_hi, b_hi) = wake_tier(0.7);
        assert_eq!(c_hi, Color::Rgb(0xBF, 0xD5, 0xFF));
        assert!(b_hi);
        let (c_mid, b_mid) = wake_tier(0.4);
        assert_eq!(c_mid, Color::Rgb(0x68, 0xA3, 0xE8));
        assert!(!b_mid);
        let (c_lo, _) = wake_tier(0.1);
        assert_eq!(c_lo, Color::Rgb(0x2A, 0x40, 0x70));
    }

    #[test]
    fn heat_ramp_runs_yellow_to_red() {
        let (r, g, b) = heat_color(1.0); // nearest the strike: red
        assert!(r > g && g > b);
        let (r2, g2, _) = heat_color(0.1); // far: yellow
        assert!(r2 >= g2 && g2 > 150);
        let (r3, _, _) = heat_color(0.55); // just past the midpoint: red-ish
        assert!(r3 == 255);
    }
}

#[cfg(test)]
mod wide_overlay_tests {
    use super::*;

    #[test]
    fn overlay_skips_wide_continuation_cells() {
        let mut s = Storm::new();
        // a drop sitting exactly on the continuation column of a wide char
        s.drops.push(Drop { col: 5.0, row: 0.0, speed: 1.0, glyph: '|', vertical: '.', leanable: false });
        let cont = Cell { ch: ' ', fg: Color::Default, bg: Color::Default, bold: false, reverse: false, width: 0 };
        assert_eq!(s.overlay(cont, 0, 5), None, "storm must not draw on a continuation");
        let normal = Cell { ch: 'a', ..Cell::default() };
        assert!(s.overlay(normal, 0, 5).is_some(), "storm draws on normal cells");
    }
}

#[cfg(test)]
mod dial_tests {
    use super::*;

    #[test]
    fn dials_clamp() {
        let mut s = Storm::new();
        for _ in 0..50 {
            s.dial_density(1.0);
        }
        assert!(s.rain_density <= 0.9);
        for _ in 0..100 {
            s.dial_density(-1.0);
        }
        assert!(s.rain_density >= 0.1);
        for _ in 0..100 {
            s.dial_speed(1.0);
        }
        assert!(s.rain_speed.1 <= 300.0);
        for _ in 0..100 {
            s.dial_strike(-1.0);
        }
        assert!(s.strike_interval.0 >= 1.0);
    }

    #[test]
    fn force_strike_arms_immediately() {
        let mut s = Storm::new();
        s.t = 100.0;
        s.next_strike = 500.0;
        s.force_strike();
        assert_eq!(s.next_strike, s.t);
    }
}
