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

/// Three-tier wake palette (omp shimmer thresholds), derived from the rain
/// color so a re-rolled palette keeps the whole flash coherent.
fn wake_tier(intensity: f64, rain: Color) -> (Color, bool) {
    let (r, g, b) = match rain {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (0x68, 0xA3, 0xE8),
    };
    if intensity >= TIER_HIGH {
        (Color::Rgb(lerp(r, 0xFF, 0.55), lerp(g, 0xFF, 0.55), lerp(b, 0xFF, 0.55)), true)
    } else if intensity >= TIER_MID {
        (Color::Rgb(r, g, b), false)
    } else {
        (Color::Rgb(lerp(r, 0x10, 0.55), lerp(g, 0x18, 0.55), lerp(b, 0x30, 0.55)), false)
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

/// HSL -> RGB (h in degrees 0..360, s/l 0..1).
fn hsl(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let hp = (h / 60.0).rem_euclid(6.0);
    let x = c * (1.0 - (hp.rem_euclid(2.0) - 1.0).abs());
    let (r1, g1, b1) = match hp as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    (((r1 + m) * 255.0) as u8, ((g1 + m) * 255.0) as u8, ((b1 + m) * 255.0) as u8)
}

// ─── Storm entities ─────────────────────────────────────────────────────────

/// halfwidth katakana — the authentic matrix rain, width-1 in this table
const MATRIX_CHARS: [char; 30] = [
    'ｱ', 'ｲ', 'ｳ', 'ｴ', 'ｵ', 'ｶ', 'ｷ', 'ｸ', 'ｹ', 'ｺ', 'ｻ', 'ｼ', 'ｽ', 'ｾ', 'ｿ', 'ﾀ', 'ﾁ', 'ﾂ', 'ﾃ', 'ﾄ',
    'ﾅ', 'ﾆ', 'ﾇ', 'ﾈ', 'ﾉ', 'ﾊ', 'ﾋ', 'ﾌ', 'ﾍ', 'ﾎ',
];

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
    /// hail: big bright drops that fall fast and chip on impact
    hail: bool,
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

/// A meteor: a bright diagonal streak with a tapering ember trail.
#[derive(Debug)]
struct Meteor {
    x0: f64,
    y0: f64, // start (above the screen)
    dx: f64,
    dy: f64, // velocity, cells/s
    t: f64,  // 0..1 progress along the full path
    dur: f64,
    len: f64, // trail length in cells
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
    // runtime palette — C re-rolls it from a random hue
    rain_color: Color,
    spark_color: Color,
    aurora_lo: (u8, u8, u8),   // aurora top (purple-ish)
    aurora_mid: (u8, u8, u8),  // aurora mid (teal-ish)
    aurora_hi: (u8, u8, u8),   // aurora bottom (green-ish)
    corona_color: (u8, u8, u8),
    meteor_interval: f64,       // mean seconds between meteors
    meteor_len: f64,            // trail length in cells
    // per-strike personality (STORM 4.5 ports)
    strike_flash: f64,          // 1.0 normal, 1.3 powerflash (brighter bolt)
    strike_corona: f64,         // 1.0 normal, 1.6 sheet (sky flood)
    last_strike_col: i64,       // where the bolt grounded (test support)
    // effect toggles — flipped from the HUD panel (Ctrl+G h, then a key)
    fx_rain: bool,
    fx_trails: bool,
    fx_corona: bool,
    fx_shake: bool,
    fx_fog: bool,
    fx_embers: bool,
    fx_forks: bool,
    fx_splash: bool,
    fx_fronts: bool,
    fx_hail: bool,
    fx_aurora: bool,
    fx_matrix: bool,
    fx_meteor: bool,
    // effect state
    shake: f64,                  // 0..1 post-strike screen shake
    fog_t: f64,                  // fog drift clock
    aurora_t: f64,               // aurora roil clock
    front: Option<(f64, f64, f64)>, // gust front: (x, dir, speed)
    next_front: f64,
    rings: Vec<(usize, usize, f64)>, // splash rings: (row, col, t 0..1)
    embers: Vec<(f64, f64, f64, f64, f64)>, // (row, col, dir, life, dur)
    meteors: Vec<Meteor>,
    next_meteor: f64,
    rows: usize,                 // last tick's canvas height (fog band needs it)
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
            rain_color: Color::Rgb(0xAA, 0xAA, 0xFF),
            spark_color: Color::Rgb(0xFF, 0x4D, 0x00),
            aurora_lo: (0xC9, 0xA8, 0xFF),
            aurora_mid: (0x6E, 0xFF, 0xFF),
            aurora_hi: (0x80, 0xFF, 0xB0),
            corona_color: (0x9F, 0xB8, 0xFF),
            meteor_interval: 4.5,
            meteor_len: 12.0,
            strike_flash: 1.0,
            strike_corona: 1.0,
            last_strike_col: 0,
            // default on: the storm-feel set; fog/hail/aurora/matrix opt-in
            fx_rain: true,
            fx_trails: true,
            fx_corona: true,
            fx_shake: true,
            fx_fog: false,
            fx_embers: true,
            fx_forks: true,
            fx_splash: true,
            fx_fronts: true,
            fx_hail: false,
            fx_aurora: false,
            fx_matrix: false,
            fx_meteor: false,
            shake: 0.0,
            fog_t: 0.0,
            aurora_t: 0.0,
            front: None,
            next_front: 8.0,
            rings: Vec::new(),
            embers: Vec::new(),
            meteors: Vec::new(),
            next_meteor: 4.0,
            rows: 0,
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

    /// `1` / `2` — meteor spawn interval, ∓20% per step (2 = more meteors).
    pub fn dial_meteor_rate(&mut self, dir: f64) {
        let f = 1.0 - 0.2 * dir;
        self.meteor_interval = (self.meteor_interval * f).clamp(0.8, 20.0);
    }

    /// `3` / `4` — meteor trail length, ±2 cells per step.
    pub fn dial_meteor_size(&mut self, dir: f64) {
        self.meteor_len = (self.meteor_len + 2.0 * dir).clamp(4.0, 30.0);
    }

    /// `C` — re-roll the whole palette from a random hue (meteors stay
    /// white-hot — they're physical).
    pub fn randomize_colors(&mut self) {
        // vivid, not pastel — a re-rolled rain must be visibly a new color
        let h = self.rand() * 360.0;
        let rain = hsl(h, 0.62, 0.76);
        let spark = hsl((h + 40.0) % 360.0, 0.9, 0.5);
        self.rain_color = Color::Rgb(rain.0, rain.1, rain.2);
        self.spark_color = Color::Rgb(spark.0, spark.1, spark.2);
        self.aurora_lo = hsl((h + 180.0) % 360.0, 0.85, 0.68);
        self.aurora_mid = hsl((h + 240.0) % 360.0, 0.85, 0.68);
        self.aurora_hi = hsl((h + 300.0) % 360.0, 0.85, 0.68);
        self.corona_color = hsl(h, 0.7, 0.82);
    }

    /// The corona tint color, for the renderer.
    pub fn corona_color(&self) -> (u8, u8, u8) {
        self.corona_color
    }

    /// `b` — force a strike on the next tick.
    pub fn force_strike(&mut self) {
        self.next_strike = self.t;
    }

    /// `t c k F e s g f h a m` — flip an effect; returns its name if known.
    pub fn toggle_effect(&mut self, key: u8) -> Option<&'static str> {
        match key {
            b'r' => {
                self.fx_rain = !self.fx_rain;
                Some("rain")
            }
            b't' => {
                self.fx_trails = !self.fx_trails;
                Some("trails")
            }
            b'c' => {
                self.fx_corona = !self.fx_corona;
                Some("corona")
            }
            b'k' => {
                self.fx_shake = !self.fx_shake;
                Some("shake")
            }
            b'F' => {
                self.fx_forks = !self.fx_forks;
                Some("forks")
            }
            b'e' => {
                self.fx_embers = !self.fx_embers;
                Some("embers")
            }
            b's' => {
                self.fx_splash = !self.fx_splash;
                Some("splash")
            }
            b'g' => {
                self.fx_fronts = !self.fx_fronts;
                Some("fronts")
            }
            b'f' => {
                self.fx_fog = !self.fx_fog;
                Some("fog")
            }
            b'h' => {
                self.fx_hail = !self.fx_hail;
                Some("hail")
            }
            b'a' => {
                self.fx_aurora = !self.fx_aurora;
                Some("aurora")
            }
            b'M' => {
                self.fx_meteor = !self.fx_meteor;
                Some("meteors")
            }
            b'm' => {
                self.fx_matrix = !self.fx_matrix;
                self.drops.clear(); // respawn as katakana (or back to rain)
                Some("matrix")
            }
            _ => None,
        }
    }

    /// Names of the enabled effects, for the HUD fx line.
    pub fn fx_list(&self) -> String {
        let mut on: Vec<&str> = Vec::new();
        if self.fx_rain {
            on.push("rain");
        }
        if self.fx_trails {
            on.push("trails");
        }
        if self.fx_corona {
            on.push("corona");
        }
        if self.fx_shake {
            on.push("shake");
        }
        if self.fx_forks {
            on.push("forks");
        }
        if self.fx_embers {
            on.push("embers");
        }
        if self.fx_splash {
            on.push("splash");
        }
        if self.fx_fronts {
            on.push("fronts");
        }
        if self.fx_fog {
            on.push("fog");
        }
        if self.fx_hail {
            on.push("hail");
        }
        if self.fx_aurora {
            on.push("aurora");
        }
        if self.fx_meteor {
            on.push("meteors");
        }
        if self.fx_matrix {
            on.push("matrix");
        }
        if on.is_empty() {
            "fx: none".to_string()
        } else {
            format!("fx: {}", on.join(" "))
        }
    }

    /// 0..1 — how fresh the flash is (max wake intensity across the bolt).
    fn flash_level(&self) -> f64 {
        let Some(b) = &self.bolt else { return 0.0 };
        b.path
            .iter()
            .map(|&(_, _, _, ttl)| {
                kitt_intensity(1.0 - (ttl / BOLT_TTL).clamp(0.0, 1.0)).max(0.0).min(1.0)
            })
            .fold(0.0, f64::max)
    }

    /// Whole-screen cool tint while the flash is fresh (corona effect);
    /// sheet strikes flood the sky harder.
    pub fn corona_level(&self) -> f64 {
        if self.fx_corona {
            self.flash_level() * 0.45 * self.strike_corona
        } else {
            0.0
        }
    }

    pub fn shake_level(&self) -> f64 {
        if self.fx_shake {
            self.shake
        } else {
            0.0
        }
    }

    /// Deterministic per-frame jitter while the screen is shaking.
    pub fn shake_offset(&self) -> (i64, i64) {
        if !self.fx_shake || self.shake <= 0.0 {
            return (0, 0);
        }
        let dr = ((self.t * 83.0).sin() * 1.3) as i64;
        let dc = ((self.t * 127.0).sin() * 1.3) as i64;
        (dr, dc)
    }

    /// Current dial values, one per line for the vertical HUD block.
    pub fn status_lines(&self) -> Vec<String> {
        vec![
            format!("storm  rain  {:>3.0}%", self.rain_density * 100.0),
            format!("speed  {:>3.0}-{:>3.0} c/s", self.rain_speed.0, self.rain_speed.1),
            format!("strike {:.1}-{:.1}s", self.strike_interval.0, self.strike_interval.1),
            format!("meteor {:.1}s/{:.0}c", self.meteor_interval, self.meteor_len),
        ]
    }

    /// The live palette, as [rain, spark, aurora, corona] — for the HUD swatch.
    pub fn swatch(&self) -> [(u8, u8, u8); 4] {
        let rain = match self.rain_color {
            Color::Rgb(r, g, b) => (r, g, b),
            _ => (0xAA, 0xAA, 0xFF),
        };
        let spark = match self.spark_color {
            Color::Rgb(r, g, b) => (r, g, b),
            _ => (0xFF, 0x4D, 0x00),
        };
        [rain, spark, self.aurora_hi, self.corona_color]
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
        if !self.fx_rain {
            return; // rain toggled off: no new drops (matrix included)
        }
        let col = self.rand() * cols as f64;
        let row = -(rows as f64) * self.rand() * 0.4; // start above the screen
        let base_speed = self.rain_speed.0 + self.rand() * (self.rain_speed.1 - self.rain_speed.0);
        if self.fx_matrix {
            let ch = MATRIX_CHARS[(self.rand() * MATRIX_CHARS.len() as f64) as usize];
            self.drops.push(Drop { col, row, speed: base_speed * 1.6, glyph: ch, vertical: ch, leanable: false, hail: false });
            return;
        }
        // hail: big bright drops, faster; frequency rises with the density dial
        let hail = self.fx_hail && self.rand() < 0.06 + (self.rain_density - 0.35).max(0.0) * 0.5;
        let speed = if hail { base_speed * 2.5 } else { base_speed };
        let vertical = if hail { '●' } else if self.rand() < 0.5 { '.' } else { ',' };
        let leanable = !hail && self.rand() < 0.5;
        self.drops.push(Drop { col, row, speed, glyph: vertical, vertical, leanable, hail });
    }

    /// Strike: the whole jagged bolt appears at once (a real flash is ~1/10s),
    /// and the text heats + rocks burst the same instant it lands.
    fn strike(&mut self, cols: usize, rows: usize) {
        // roll the strike personality (STORM 4.5): powerflash brightens the
        // bolt, sheet floods the sky with corona, sparkfly bursts embers
        let sheet = self.rand() < 0.08;
        let power = self.rand() < 0.5;
        self.strike_flash = if power { 1.3 } else { 1.0 };
        self.strike_corona = if sheet { 1.6 } else { 1.0 };
        // aim at a grounding column: free wander early, the bolt pulls
        // toward the target as it descends and snaps home at the bottom
        let target = (cols as f64 * (0.15 + 0.7 * self.rand())) as i64;
        let mut x = (target as f64 + self.rand() * cols as f64 * 0.4 - cols as f64 * 0.2) as i64;
        x = x.clamp(1, cols as i64 - 2);
        let mut prev = x;
        let mut path: Vec<(i64, i64, char, f64)> = Vec::new();
        for r in 0..rows {
            if r > 0 {
                // free jitter early; the pull toward the target grows with depth
                if self.rand() < 0.45 {
                    x += if self.rand() < 0.5 { 1 } else { -1 };
                }
                let p = r as f64 / rows as f64;
                let pull = (target - x) as f64 * (0.05 + 0.45 * p * p);
                if pull.abs() >= 1.0 || self.rand() < 0.5 {
                    x += pull.round() as i64;
                }
                if r + 1 == rows {
                    x = target; // snap home
                }
                x = x.clamp(1, cols as i64 - 2);
            }
            let glyph = if x == prev {
                '|'
            } else if x > prev {
                '\\'
            } else {
                '/'
            };
            path.push((r as i64, x, glyph, BOLT_TTL));
            // STORM-style branch: sweeps outward with momentum (a persistent
            // directional drift per segment) and tapers with depth — branches
            // near the top are long and wild, near the bottom short and tight
            if self.rand() < 0.20 {
                let dir: i64 = if self.rand() < 0.5 { 1 } else { -1 };
                let taper = 1.0 - (r as f64 / rows as f64) * 0.65;
                let len = ((if self.fx_forks { 3.0 + self.rand() * 6.0 } else { 2.0 + self.rand() * 3.0 }) * taper) as usize;
                let propo = dir as f64 * if self.fx_forks { 1.5 } else { 1.0 };
                let mut bx = x as f64;
                let mut br = r as i64;
                let mut bprev = x;
                for _ in 0..len {
                    bx += propo + if self.rand() < 0.5 { 1.0 } else { -1.0 };
                    br += 1;
                    if bx < 0.0 || bx >= cols as f64 || br >= rows as i64 {
                        break;
                    }
                    let bg = if bx as i64 == bprev {
                        '|'
                    } else if bx as i64 > bprev {
                        '\\'
                    } else {
                        '/'
                    };
                    path.push((br, bx as i64, bg, BOLT_TTL));
                    bprev = bx as i64;
                }
            }
            prev = x;
        }
        self.last_strike_col = x;
        self.bolt = Some(Bolt { path });
        // impact at the bottom of the bolt
        self.impact(x as f64, rows);
        if self.fx_shake {
            self.shake = 1.0; // the whole screen shivers with the boom
        }
        if self.fx_embers {
            // embers crawl along the struck row as it cools; sparkfly rolls
            // a bigger burst
            let count = if self.rand() < 0.5 { 5 } else { 2 };
            for _ in 0..count {
                let dir = if self.rand() < 0.5 { 1.0 } else { -1.0 };
                let col = x as f64 + self.rand() * 6.0 - 3.0;
                let dur = 0.5 + self.rand() * 0.6;
                self.embers.push((rows.saturating_sub(1) as f64, col, dir, 0.0, dur));
            }
        }
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
        self.rows = rows;
        self.ensure_glow(rows, cols);

        // decay the text glow
        for g in self.glow.iter_mut() {
            *g = (*g - dt * 0.9).max(0.0); // ~1.1s to cool
        }

        self.gust(dt);

        // rain: fast TTE-style drops across a dialable share of columns;
        // the whole block is gated so toggling rain off can't spin the
        // spawn loop (spawn_drop early-returns while the flag is off)
        if self.fx_rain {
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
            // hail chips and splash rings where drops cross the bottom edge
            let mut chips: Vec<f64> = Vec::new();
            let mut ripples: Vec<usize> = Vec::new();
            for d in self.drops.iter() {
                if d.row >= rows as f64 - 0.5 && d.row < rows as f64 + 1.0 {
                    if d.hail && self.fx_hail {
                        chips.push(d.col);
                    } else if self.fx_splash && self.rings.len() < 16 {
                        ripples.push(d.col as usize);
                    }
                }
            }
            for col in chips {
                let x1 = col + (self.rand() * 4.0 - 2.0);
                self.sparks.push(Spark {
                    x0: col,
                    y0: rows.saturating_sub(1) as f64,
                    cx: col,
                    cy: rows as f64 * 0.5,
                    x1,
                    y1: rows.saturating_sub(1) as f64,
                    t: 0.0,
                    dur: 0.3,
                    glyph: '*',
                });
            }
            for rc in ripples {
                self.rings.push((rows.saturating_sub(1), rc, 0.0));
            }
        } else {
            self.drops.clear();
        }

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

        // screen shake decays fast (~0.15s)
        self.shake = (self.shake - dt * 7.0).max(0.0);

        // drift clocks for the slow effects
        if self.fx_fog {
            self.fog_t += dt;
        }
        if self.fx_aurora {
            self.aurora_t += dt * 0.6;
        }

        // gust fronts sweep across periodically, dragging a real gust
        if self.fx_fronts && self.front.is_none() && self.t >= self.next_front {
            let dir = if self.rand() < 0.5 { -1.0 } else { 1.0 };
            let start = if dir < 0.0 { cols as f64 + 5.0 } else { -5.0 };
            self.front = Some((start, dir, 60.0 + self.rand() * 50.0));
            self.wind_target = dir * 0.9;
            self.next_front = self.t + 6.0 + self.rand() * 8.0;
        }
        let mut front_done = false;
        if let Some((x, dir, spd)) = &mut self.front {
            *x += *dir * *spd * dt;
            if *x < -8.0 || *x > cols as f64 + 8.0 {
                front_done = true;
            }
        }
        if front_done {
            self.front = None;
        }

        // splash rings expand and fade
        for r in self.rings.iter_mut() {
            r.2 += dt / 0.35;
        }
        self.rings.retain(|r| r.2 < 1.0);

        // embers crawl along the struck row and cool
        for e in self.embers.iter_mut() {
            e.1 += e.2 * 35.0 * dt;
            e.3 += dt;
        }
        self.embers.retain(|e| e.3 < e.4);

        // meteors: occasional bright streaks, rare and dramatic
        if self.fx_meteor && self.meteors.len() < 3 && self.t >= self.next_meteor {
            let x0 = self.rand() * cols as f64;
            let y0 = -6.0 - self.rand() * 8.0;
            let dx = (self.rand() * 2.0 - 1.0) * 55.0;
            let dy = 70.0 + self.rand() * 70.0;
            let dur = 1.2 + self.rand() * 0.8;
            let len = self.meteor_len * (0.7 + self.rand() * 0.6);
            self.meteors.push(Meteor { x0, y0, dx, dy, t: 0.0, dur, len });
            self.next_meteor = self.t + self.meteor_interval * (0.6 + self.rand() * 0.8);
        }
        let mut burnouts: Vec<(f64, f64)> = Vec::new();
        for m in self.meteors.iter_mut() {
            m.t += dt / m.dur;
            if m.t >= 1.0 {
                let x1 = m.x0 + m.dx * m.dur;
                let y1 = m.y0 + m.dy * m.dur;
                if y1 > 0.0 && y1 < rows as f64 - 1.0 && x1 >= 0.0 && x1 < cols as f64 {
                    burnouts.push((x1, y1));
                }
            }
        }
        self.meteors.retain(|m| m.t < 1.0);
        for (mx, my) in burnouts {
            // a little burst where it burns out
            for _ in 0..4 {
                let dir = self.rand() * std::f64::consts::TAU;
                let off = 1.0 + self.rand() * 3.0;
                let dur = 0.4 + self.rand() * 0.4;
                let glyph = if self.rand() < 0.5 { '*' } else { '.' };
                self.sparks.push(Spark {
                    x0: mx,
                    y0: my,
                    cx: mx + dir.cos() * off,
                    cy: my + dir.sin() * off - 1.0,
                    x1: mx + dir.cos() * (off + 2.0),
                    y1: my + dir.sin() * (off + 2.0),
                    t: 0.0,
                    dur,
                    glyph,
                });
            }
        }
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
                    // powerflash: the white-hot core lingers and the wake
                    // burns brighter for the whole flash
                    let white = 0.8 - (self.strike_flash - 1.0) * 0.5;
                    let (fg, bold) = if intensity >= white {
                        (Color::Rgb(0xFF, 0xFF, 0xFF), true) // fresh flash: white
                    } else {
                        wake_tier((intensity * self.strike_flash).min(1.0), self.rain_color)
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
                // forks effect: a soft halo around the bolt while fresh
                if self.fx_forks
                    && base.ch == ' '
                    && (tr - row as i64).abs() <= 1
                    && (tc - col as i64).abs() <= 1
                {
                    let age = (1.0 - ttl / BOLT_TTL).max(0.0).min(1.0);
                    if kitt_intensity(age) > 0.5 {
                        let (hr, hg, hb) = match self.rain_color {
                            Color::Rgb(r, g, b) => (r, g, b),
                            _ => (0x9F, 0xB8, 0xFF),
                        };
                        return Some(Cell {
                            ch: '·',
                            fg: Color::Rgb(lerp(hr, 0xFF, 0.6), lerp(hg, 0xFF, 0.6), lerp(hb, 0xFF, 0.6)),
                            bg: base.bg,
                            bold: false,
                            reverse: false,
                            width: 1,
                        });
                    }
                }
            }
        }

        // sparks: flying glyphs cooling from orange; keep the base background
        for s in &self.sparks {
            let (x, y) = self.spark_pos(s);
            if y.floor() as usize == row && x.floor() as usize == col {
                let cool = (s.t * 1.2).min(1.0);
                let (sr, sg, sb) = match self.spark_color {
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

        // meteors: bright diagonal streaks, white-hot head with a tapering
        // ember trail (white -> gold -> orange -> deep red) and a soft glow
        // around the head
        for m in &self.meteors {
            let (vx, vy) = (m.dx * m.dur, m.dy * m.dur);
            let vlen2 = vx * vx + vy * vy;
            if vlen2 < 1.0 {
                continue;
            }
            // nearest point on the path at-or-behind the head
            let s = (((col as f64 - m.x0) * vx + (row as f64 - m.y0) * vy) / vlen2).clamp(0.0, m.t);
            let px = m.x0 + vx * s;
            let py = m.y0 + vy * s;
            let perp = ((col as f64 - px).powi(2) + (row as f64 - py).powi(2)).sqrt();
            if perp < 0.55 {
                let behind = (m.t - s) * vlen2.sqrt(); // cells behind the head
                if behind <= m.len {
                    let k = (1.0 - behind / m.len).max(0.0); // 1 head -> 0 tail
                    let (r, g, b) = if k > 0.7 {
                        (0xFF, lerp(0xFF, 0xD0, (1.0 - k) / 0.3), lerp(0xFF, 0x80, (1.0 - k) / 0.3))
                    } else if k > 0.35 {
                        (
                            0xFF,
                            lerp(0xD0, 0x88, (0.7 - k) / 0.35),
                            lerp(0x80, 0x30, (0.7 - k) / 0.35),
                        )
                    } else {
                        (
                            lerp(0xFF, 0x90, (0.35 - k) / 0.35),
                            lerp(0x88, 0x38, (0.35 - k) / 0.35),
                            lerp(0x30, 0x1C, (0.35 - k) / 0.35),
                        )
                    };
                    let glow = if k > 0.85 {
                        Color::Rgb(lerp(0x00, 0x66, k), lerp(0x00, 0x6E, k), lerp(0x00, 0x88, k))
                    } else {
                        base.bg
                    };
                    return Some(Cell {
                        ch: if k > 0.85 { '*' } else { '·' },
                        fg: Color::Rgb(r, g, b),
                        bg: glow,
                        bold: k > 0.7,
                        reverse: false,
                        width: 1,
                    });
                }
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

        // embers: ember crawl along the struck row, warm and fading
        for &(er, ec, _dir, life, dur) in &self.embers {
            if row == er as usize && col == ec.floor() as usize {
                let cool = (life / dur).min(1.0);
                return Some(Cell {
                    ch: '°',
                    fg: Color::Rgb(lerp(0xFF, 0x50, cool), lerp(0xC0, 0x30, cool), lerp(0x40, 0x18, cool)),
                    bg: base.bg,
                    bold: cool < 0.4,
                    reverse: false,
                    width: 1,
                });
            }
        }

        // rain: drops keep the base background; hail is big and bright, matrix
        // drops are katakana; the trails effect stretches a wake above the head
        for d in &self.drops {
            let dr = d.row as usize;
            let dc = d.col as usize;
            if dc == col && dr == row {
                let (fg, bold) = if d.hail {
                    (Color::Rgb(0xE8, 0xF0, 0xFF), true)
                } else if self.fx_matrix {
                    (Color::Rgb(0x00, 0xFF, 0x70), true)
                } else {
                    (self.rain_color, false)
                };
                return Some(Cell {
                    ch: d.glyph,
                    fg,
                    bg: base.bg,
                    bold,
                    reverse: false,
                    width: 1,
                });
            }
            if self.fx_trails && dc == col && dr >= 1 && row == dr - 1 {
                let (tg, tf) = if self.fx_matrix {
                    (d.glyph, Color::Rgb(0x00, 0x90, 0x40))
                } else {
                    let (rr, gg, bb) = match self.rain_color {
                        Color::Rgb(r, g, b) => (r, g, b),
                        _ => (0x88, 0x88, 0xCC),
                    };
                    (':', Color::Rgb((rr as f64 * 0.55) as u8, (gg as f64 * 0.55) as u8, (bb as f64 * 0.55) as u8))
                };
                return Some(Cell {
                    ch: tg,
                    fg: tf,
                    bg: base.bg,
                    bold: false,
                    reverse: false,
                    width: 1,
                });
            }
        }

        // splash rings: expanding `o` ripples where drops landed
        for &(rr, rc, t) in &self.rings {
            if row == rr && base.ch == ' ' {
                let d = (col as i64 - rc as i64).unsigned_abs() as f64;
                if (d - t * 2.0).abs() < 0.6 {
                    return Some(Cell {
                        ch: 'o',
                        fg: Color::Rgb(0x99, 0xAA, 0xFF),
                        bg: base.bg,
                        bold: false,
                        reverse: false,
                        width: 1,
                    });
                }
            }
        }

        // gust front: a dense band of wind-blown rain sweeping across — the
        // leading edge glows white-blue and fades back to normal rain behind
        if let Some((fx, dir, _spd)) = self.front {
            if base.ch == ' ' {
                let d = (col as f64 - fx).abs();
                if d < 8.0 {
                    let h = (row * 31 + col * 17) as i64 + (fx * 3.0) as i64;
                    let edge = (1.0 - d / 8.0).max(0.0); // 1 at the edge
                    let wisp = ((row.wrapping_mul(2654435761) >> 24) % 100) as f64 / 100.0;
                    let density = edge * (0.25 + 0.75 * wisp);
                    if h.rem_euclid(4) < (density * 3.0).round() as i64 {
                        // white at the edge -> normal rain behind
                        let (fr, fgg, fb) = match self.rain_color {
                            Color::Rgb(r, g, b) => (r, g, b),
                            _ => (0xAA, 0xAA, 0xFF),
                        };
                        let fg = Color::Rgb(lerp(fr, 0xE8, edge), lerp(fgg, 0xF4, edge), lerp(fb, 0xFF, edge));
                        let gk = ((density - 0.35) * 1.5).max(0.0).min(1.0);
                        let glow = Color::Rgb(
                            lerp(0x00, 0x4E, gk * 0.4),
                            lerp(0x00, 0x5C, gk * 0.4),
                            lerp(0x00, 0x90, gk * 0.4),
                        );
                        return Some(Cell {
                            ch: if dir < 0.0 { '/' } else { '\\' },
                            fg,
                            bg: glow,
                            bold: edge > 0.6,
                            reverse: false,
                            width: 1,
                        });
                    }
                }
            }
        }

        // fog: a drifting wavy band of haze — dims text under it, dusts mist
        // particles, and blooms a pale glow into the background on empty
        // cells (text keeps its own background, the sky doesn't)
        if self.fx_fog {
            let c = col as f64;
            let cy = self.rows as f64
                * (0.60 + 0.08 * (c * 0.02 + self.fog_t * 0.15).sin() + 0.05 * (c * 0.06 - self.fog_t * 0.2).sin());
            let half = 4.0 + 1.5 * (self.fog_t * 0.3 + c * 0.04).sin();
            let v = (row as f64 - cy) / half;
            if v.abs() < 1.0 {
                let wisp = ((col.wrapping_mul(2654435761) >> 24) % 100) as f64 / 100.0;
                let shimmer = 0.7 + 0.3 * (self.fog_t * 0.5 + c * 0.2).sin();
                let fade = (1.0 - v * v).max(0.0);
                let density = fade * (0.1 + 0.9 * wisp) * shimmer;
                if density >= 0.12 {
                    let gk = (density * 1.3).min(1.0);
                    let glow = Color::Rgb(lerp(0x2A, 0x9E, gk), lerp(0x30, 0xAE, gk), lerp(0x3A, 0xC8, gk));
                    if base.ch != ' ' {
                        // dim the text into the fog; keep its own background
                        let fogged = match base.fg {
                            Color::Rgb(rr, gg, bb) => Color::Rgb(
                                lerp(rr, 0x9A, gk * 0.5),
                                lerp(gg, 0xAA, gk * 0.5),
                                lerp(bb, 0xCC, gk * 0.5),
                            ),
                            _ => Color::Rgb(0x9A, 0xAA, 0xCC),
                        };
                        let mut lit = base;
                        lit.fg = fogged;
                        return Some(lit);
                    }
                    if density >= 0.30 {
                        let pch = if density > 0.6 { ':' } else { '·' };
                        return Some(Cell {
                            ch: pch,
                            fg: Color::Rgb(lerp(0x5A, 0xCE, gk), lerp(0x64, 0xDC, gk), lerp(0x74, 0xEE, gk)),
                            bg: glow,
                            bold: density > 0.7,
                            reverse: false,
                            width: 1,
                        });
                    }
                    // bg-only haze
                    return Some(Cell {
                        ch: ' ',
                        fg: Color::Default,
                        bg: glow,
                        bold: false,
                        reverse: false,
                        width: 1,
                    });
                }
            }
        }

        // aurora: roiling curtains with vertical rays — the curtain core is a
        // stack of sine waves undulating with column and time, each column has
        // a fixed ray personality, intensity fades parabolically toward the
        // edges, and the color runs a continuous green->teal->purple gradient
        // that drifts. It flares briefly when lightning strikes.
        if self.fx_aurora && base.ch == ' ' && row < self.rows * 2 / 3 {
            let t = self.aurora_t;
            let c = col as f64;
            let center = 2.5 + 2.2 * (c * 0.045 + t * 0.6).sin() + 1.6 * (c * 0.09 - t * 0.35).sin();
            let half = 2.8 + 1.2 * (c * 0.03 - t * 0.2).sin();
            let v = (row as f64 - center) / half; // -1..1 across the curtain
            if v.abs() < 1.0 {
                let ray = ((col.wrapping_mul(2654435761) >> 24) % 100) as f64 / 100.0;
                let shimmer = 0.75 + 0.25 * (t * 1.8 + c * 0.25).sin();
                let fade = (1.0 - v * v).max(0.0); // bright core, soft edges
                let intensity = fade * (0.05 + 0.95 * ray) * shimmer * (1.0 + self.flash_level() * 0.7);
                // smooth gradient, bottom green -> mid teal -> top purple
                let g = ((v + 1.0) / 2.0 + t * 0.04).fract();
                let (hi_r, hi_g, hi_b) = self.aurora_hi;
                let (mid_r, mid_g, mid_b) = self.aurora_mid;
                let (lo_r, lo_g, lo_b) = self.aurora_lo;
                let (cr, cg, cb) = if g < 0.5 {
                    let k = g * 2.0;
                    (lerp(hi_r, mid_r, k), lerp(hi_g, mid_g, k), lerp(hi_b, mid_b, k))
                } else {
                    let k = (g - 0.5) * 2.0;
                    (lerp(mid_r, lo_r, k), lerp(mid_g, lo_g, k), lerp(mid_b, lo_b, k))
                };
                // the curtain does NOT inherit the base bg — it paints a dim
                // aurora hue into the background, so it blooms as a glow field
                let glow_k = (intensity * 1.4).min(1.0);
                let glow = Color::Rgb(
                    lerp(0x00, cr, glow_k * 0.45),
                    lerp(0x00, cg, glow_k * 0.45),
                    lerp(0x00, cb, glow_k * 0.45),
                );
                if intensity >= 0.40 {
                    let ch = if intensity > 0.72 { '~' } else { '·' };
                    return Some(Cell {
                        ch,
                        fg: Color::Rgb(
                            lerp(0x34, cr, glow_k),
                            lerp(0x3C, cg, glow_k),
                            lerp(0x44, cb, glow_k),
                        ),
                        bg: glow,
                        bold: intensity > 0.85,
                        reverse: false,
                        width: 1,
                    });
                }
                // halo: bg-only glow, no glyph
                if intensity >= 0.18 {
                    return Some(Cell {
                        ch: ' ',
                        fg: Color::Default,
                        bg: glow,
                        bold: false,
                        reverse: false,
                        width: 1,
                    });
                }
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
    fn wake_tiers_follow_thresholds_and_rain() {
        let rain = Color::Rgb(0xAA, 0xAA, 0xFF);
        let (c_hi, b_hi) = wake_tier(0.7, rain);
        assert!(b_hi);
        // high tier = rain lightened toward white
        assert_eq!(c_hi, Color::Rgb(lerp(0xAA, 0xFF, 0.55), lerp(0xAA, 0xFF, 0.55), 0xFF));
        let (c_mid, b_mid) = wake_tier(0.4, rain);
        assert_eq!(c_mid, rain); // mid tier = the rain color itself
        assert!(!b_mid);
        let (c_lo, _) = wake_tier(0.1, rain);
        assert_ne!(c_lo, rain); // low tier = dimmed
        // a different rain color changes the wake (palette coherence)
        let (c_hi2, _) = wake_tier(0.7, Color::Rgb(0xFF, 0x80, 0x80));
        assert_ne!(c_hi, c_hi2);
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
        s.drops.push(Drop { col: 5.0, row: 0.0, speed: 1.0, glyph: '|', vertical: '.', leanable: false, hail: false });
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

#[cfg(test)]
mod effect_tests {
    use super::*;

        #[test]
    fn toggles_flip_flags_and_report_names() {
        let mut s = Storm::new();
        assert_eq!(s.toggle_effect(b't'), Some("trails"));
        assert!(!s.fx_trails);
        assert_eq!(s.toggle_effect(b'm'), Some("matrix"));
        assert!(s.fx_matrix);
        assert_eq!(s.toggle_effect(b'z'), None);
        assert!(s.fx_list().contains("matrix"));
        assert!(s.fx_list().contains("trails") == false); // trails was flipped off
    }

        #[test]
    fn matrix_respawns_as_katakana() {
        let mut s = Storm::new();
        s.toggle_effect(b'm');
        s.spawn_drop(80, 24);
        assert!(s.drops.iter().all(|d| MATRIX_CHARS.contains(&d.glyph) && !d.leanable));
        // back to rain: glyphs are the vertical set again
        s.toggle_effect(b'm');
        s.drops.clear();
        s.spawn_drop(80, 24);
        assert!(s.drops.iter().all(|d| d.glyph == '.' || d.glyph == ','));
    }

        #[test]
    fn hail_spawns_with_the_flag_on() {
        let mut s = Storm::new();
        s.fx_hail = true;
        s.rain_density = 0.9;
        for _ in 0..200 {
            s.spawn_drop(80, 24);
        }
        assert!(s.drops.iter().any(|d| d.hail), "fixed-seed rng: hail must appear");
    }

        #[test]
    fn corona_tracks_the_flash_and_fades() {
        let mut s = Storm::new();
        assert_eq!(s.corona_level(), 0.0);
        s.strike(80, 24);
        assert!(s.corona_level() > 0.0);
        s.bolt = Some(Bolt { path: vec![(0, 0, '|', 0.0)] }); // fully faded
        assert_eq!(s.corona_level(), 0.0);
    }

        #[test]
    fn shake_offsets_are_small() {
        let mut s = Storm::new();
        assert_eq!(s.shake_offset(), (0, 0));
        s.shake = 1.0;
        let (dr, dc) = s.shake_offset();
        assert!(dr.abs() <= 1 && dc.abs() <= 1);
    }

    
    
    
    
    
    
        #[test]
    fn bolt_aims_and_snaps_home() {
        let mut s = Storm::new();
        s.strike(80, 24);
        let b = s.bolt.as_ref().unwrap();
        // the main bolt lands exactly on the target column at the bottom row
        assert!(
            b.path.iter().any(|&(r, c, _, _)| r == 23 && c == s.last_strike_col),
            "bolt must ground on its target"
        );
        assert!(s.last_strike_col >= 1 && s.last_strike_col <= 78);
        // every path cell stays in bounds
        assert!(b.path.iter().all(|&(r, c, _, _)| r >= 0 && r < 24 && c >= 0 && c < 80));
    }

        #[test]
    fn forks_sweep_longer_branches() {
        let mut plain = Storm::new();
        plain.fx_forks = false; // forks default ON; compare against the plain bolt
        plain.strike(80, 24);
        let mut forked = Storm::new();
        forked.fx_forks = true;
        forked.strike(80, 24);
        let n_plain = plain.bolt.as_ref().unwrap().path.len();
        let n_forked = forked.bolt.as_ref().unwrap().path.len();
        assert!(
            n_forked > n_plain,
            "forks must add longer momentum branches ({n_forked} vs {n_plain})"
        );
    }

        #[test]
    fn strike_rolls_a_personality() {
        let mut s = Storm::new();
        s.strike(80, 24);
        assert!(s.strike_flash == 1.0 || s.strike_flash == 1.3);
        assert!(s.strike_corona == 1.0 || s.strike_corona == 1.6);
    }

    #[test]
    fn hsl_maps_primary_hues() {
        assert_eq!(hsl(0.0, 1.0, 0.5), (255, 0, 0));
        assert_eq!(hsl(120.0, 1.0, 0.5), (0, 255, 0));
        assert_eq!(hsl(240.0, 1.0, 0.5), (0, 0, 255));
    }

        #[test]
    fn randomize_colors_rerolls_the_palette() {
        let mut s = Storm::new();
        let before = s.rain_color;
        s.randomize_colors();
        assert_ne!(s.rain_color, before, "rain color must change");
        let (_r, _g, _b) = s.corona_color();
        // the aurora anchors follow the same hue family
        assert!(s.aurora_hi != s.aurora_lo);
    }

        #[test]
    fn meteor_dials_clamp() {
        let mut s = Storm::new();
        for _ in 0..50 {
            s.dial_meteor_rate(1.0);
        }
        assert!(s.meteor_interval >= 0.8);
        for _ in 0..50 {
            s.dial_meteor_rate(-1.0);
        }
        assert!(s.meteor_interval <= 20.0);
        for _ in 0..50 {
            s.dial_meteor_size(1.0);
        }
        assert!(s.meteor_len <= 30.0);
        for _ in 0..50 {
            s.dial_meteor_size(-1.0);
        }
        assert!(s.meteor_len >= 4.0);
        assert!(s.status_lines()[3].contains("meteor"));
    }

    #[test]
    fn meteors_spawn_streak_and_burn_out() {
        let mut s = Storm::new();
        assert_eq!(s.toggle_effect(b'M'), Some("meteors"));
        assert!(s.fx_meteor);
        assert!(s.fx_list().contains("meteors"));
        s.rows = 24;
        let blank = Cell { ch: ' ', fg: Color::Default, bg: Color::Default, bold: false, reverse: false, width: 1 };

        // spawn path: a due meteor spawns
        s.next_meteor = 0.0;
        s.tick(0.016, 80, 24);
        assert!(!s.meteors.is_empty(), "a due meteor must spawn");
        s.meteors.clear();
        s.drops.clear(); // no rain interfering with the streak check
        s.next_meteor = 1e9;

        // deterministic streak across the screen: overlay must paint it
        s.meteors.push(Meteor { x0: 10.0, y0: -5.0, dx: 20.0, dy: 60.0, t: 0.45, dur: 1.0, len: 12.0 });
        let mut hit = 0;
        for r in 0..24 {
            for c in 0..80 {
                if let Some(cell) = s.overlay(blank, r, c) {
                    assert!(cell.ch == '*' || cell.ch == '\u{b7}', "meteor glyphs are * head and middle-dot trail");
                    hit += 1;
                }
            }
        }
        assert!(hit > 0, "meteor must paint a visible streak");

        // advancing past the duration burns it out (no new spawns due)
        s.tick(3.0, 80, 24);
        assert!(s.meteors.is_empty(), "meteors must burn out");
    }

    #[test]
    fn rain_toggle_stops_spawning_without_spinning() {
        let mut s = Storm::new();
        s.tick(0.1, 80, 24);
        assert!(!s.drops.is_empty(), "rain spawns by default");
        s.toggle_effect(b'r');
        assert!(!s.fx_rain);
        s.tick(0.1, 80, 24); // must terminate: no infinite spawn loop
        assert!(s.drops.is_empty(), "rain off clears drops");
        s.tick(0.1, 80, 24); // stays empty, still no hang
        assert!(s.drops.is_empty());
        s.toggle_effect(b'r');
        s.tick(0.1, 80, 24);
        assert!(!s.drops.is_empty(), "rain on respawns");
    }

    #[test]
    fn aurora_is_curtains_not_a_lattice() {
        let mut s = Storm::new();
        s.fx_aurora = true;
        s.rows = 24;
        s.aurora_t = 5.0;
        let blank = Cell { ch: ' ', fg: Color::Default, bg: Color::Default, bold: false, reverse: false, width: 1 };
        let mut total = 0usize;
        let mut glyphs = 0usize;
        let mut halos = 0usize;
        let mut per_row = vec![0usize; 24];
        let mut colors = std::collections::HashSet::new();
        for r in 0..24 {
            for c in 0..80 {
                if let Some(cell) = s.overlay(blank, r, c) {
                    if cell.ch == '~' || cell.ch == '\u{b7}' {
                        glyphs += 1;
                        colors.insert(cell.fg);
                    } else if cell.ch == ' ' && cell.bg != Color::Default {
                        halos += 1; // bg-only glow cell
                        colors.insert(cell.bg);
                    } else {
                        panic!("unexpected aurora cell {cell:?}");
                    }
                    per_row[r] += 1;
                    total += 1;
                }
            }
        }
        assert!(glyphs > 0, "aurora must render glyphs");
        assert!(halos > 0, "aurora must bloom a bg glow halo");
        // sparse: no row may be a solid lattice, and the whole thing stays wispy
        // the curtain is confined to the sky: nothing below the top third
        assert!(per_row[8..].iter().all(|&n| n == 0), "aurora must stay in the sky (got {per_row:?})");
        assert!(total < 80 * 24 / 3, "aurora must stay under a third of the screen");
        // ray structure: column fill varies — not a uniform lattice
        let mut per_col = vec![0usize; 80];
        for r in 0..8 {
            for c in 0..80 {
                if let Some(cell) = s.overlay(blank, r, c) {
                    if cell.ch != ' ' {
                        per_col[c] += 1;
                    }
                }
            }
        }
        assert!(*per_col.iter().max().unwrap() < 8, "no column may be solid");
        assert!(per_col.iter().filter(|&&n| n == 0).count() > 0, "the curtain must have gaps");
        assert!(colors.len() >= 2, "the gradient must vary color");
    }

    #[test]
    fn gust_front_is_a_band_not_a_scan_line() {
        // a gust front must be a dense region of leaning rain, never a hard
        // full-height line at one column (the old jitter read as a scan line)
        let mut s = Storm::new();
        s.rows = 24;
        let blank = Cell { ch: ' ', fg: Color::Default, bg: Color::Default, bold: false, reverse: false, width: 1 };
        for dir in [-1.0f64, 1.0] {
            s.front = Some((40.0, dir, 70.0));
            let mut per_col = vec![0usize; 80];
            let mut total = 0;
            for r in 0..24 {
                for c in 0..80 {
                    if let Some(cell) = s.overlay(blank, r, c) {
                        assert!(cell.ch == '/' || cell.ch == '\\', "front glyph must lean with the wind");
                        per_col[c] += 1;
                        total += 1;
                    }
                }
            }
            assert!(total > 0, "the front band must render some glyphs");
            let cols_used = per_col.iter().filter(|&&n| n > 0).count();
            assert!(cols_used >= 4, "the band must span several columns, got {cols_used}");
            assert!(per_col.iter().all(|&n| n < 24), "no column may be a full-height line");
        }
    }

    #[test]
    fn effects_never_paint_continuation_cells() {
        let mut s = Storm::new();
        s.fx_fog = true;
        s.fx_aurora = true;
        s.front = Some((40.0, 1.0, 70.0));
        s.rings.push((5, 5, 0.5));
        s.embers.push((5.0, 5.0, 1.0, 0.0, 0.5));
        s.meteors.push(Meteor { x0: 0.0, y0: 0.0, dx: 40.0, dy: 60.0, t: 0.5, dur: 1.0, len: 10.0 });
        s.rows = 24;
        let cont = Cell { ch: ' ', fg: Color::Default, bg: Color::Default, bold: false, reverse: false, width: 0 };
        for r in 0..24 {
            for c in 0..80 {
                assert!(s.overlay(cont, r, c).is_none(), "continuation cell painted at {r},{c}");
            }
        }
    }
}
