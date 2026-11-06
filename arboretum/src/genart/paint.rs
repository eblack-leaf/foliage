//! Pictures from seeds: eight styles, painted a band at a time.
//!
//! A picture is painted by a [`Painter`] in bounded steps rather than in one call, because the
//! browser this runs in has one thread and a frame is sixteen milliseconds: a 320-pixel warped
//! field is tens of milliseconds of arithmetic in wasm, which painted whole would be a stall. A
//! step is a few thousand pixels, or a few thousand points of an attractor; an app takes as many
//! as its frame can spare and loads the picture when the painter says it is done.

use core::f32::consts::TAU;

use super::noise::{fbm, gradient, hash, white};
use super::ramp::{Lut, Ramp};
use super::seed::Seed;

/// The ways a picture can be grown.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Style {
    /// A field folded into itself three times: soft, cloudlike, organic.
    Bloom,
    /// Streaks laid down by particles following a noise field, like iron filings or grass in wind.
    Flow,
    /// Cells around jittered points with dark seams between them: stained glass, or a mosaic's
    /// cousin.
    Lattice,
    /// The density of a strange attractor: fine threads wound into a figure.
    Orbit,
    /// Rings from two centres beating against each other.
    Moire,
    /// Light through moving water.
    Caustic,
    /// Curtains of light hanging over a dark ground.
    Aurora,
    /// Blobs that merge and part.
    Lava,
}

impl Style {
    /// Every style, in the order they are offered.
    pub const ALL: [Style; 8] = [
        Style::Bloom,
        Style::Flow,
        Style::Lattice,
        Style::Orbit,
        Style::Moire,
        Style::Caustic,
        Style::Aurora,
        Style::Lava,
    ];

    /// The style's name, as a placard prints it.
    pub fn name(self) -> &'static str {
        match self {
            Style::Bloom => "bloom",
            Style::Flow => "flow",
            Style::Lattice => "lattice",
            Style::Orbit => "orbit",
            Style::Moire => "moire",
            Style::Caustic => "caustic",
            Style::Aurora => "aurora",
            Style::Lava => "lava",
        }
    }

    /// Whether the style is laid down by accumulation -- points or particles -- rather than read
    /// per pixel. Accumulated styles need every step before any pixel is right.
    fn accumulates(self) -> bool {
        matches!(self, Style::Orbit | Style::Flow)
    }
}

/// The numbers a picture is grown from: everything a seed decides, as values that can be walked
/// from one picture's toward another's.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Params {
    /// Which noise field. Not walked: a walk keeps the field it started in and moves through it.
    pub key: u32,
    /// How many features cross the picture.
    pub scale: f32,
    /// How far the field is folded into itself.
    pub warp: f32,
    /// Where in the field the picture is taken from.
    pub drift: (f32, f32),
    /// The style's own shape, in `0.0..=1.0`.
    pub twist: f32,
    /// How hard the values are pushed toward the ramp's ends.
    pub contrast: f32,
    /// Four more numbers a style may read: an attractor's coefficients, the blobs' phases.
    pub coeffs: [f32; 4],
}

impl Params {
    /// The parameters `seed` grows in `style`.
    pub fn of(style: Style, seed: Seed) -> Self {
        let mut rng = seed.fork(style as u64 + 1);
        let key = rng.next_u64() as u32;
        let (scale, warp) = match style {
            Style::Bloom => (rng.between(1.1, 2.2), rng.between(0.6, 1.1)),
            Style::Flow => (rng.between(1.2, 2.6), rng.between(0.6, 1.4)),
            Style::Lattice => (rng.between(5.0, 9.0), rng.between(0.1, 0.4)),
            Style::Orbit => (1.0, rng.between(0.0, 0.3)),
            Style::Moire => (rng.between(0.8, 1.6), rng.between(0.05, 0.25)),
            Style::Caustic => (rng.between(0.8, 1.4), rng.between(0.4, 1.0)),
            Style::Aurora => (rng.between(1.4, 2.6), rng.between(0.5, 1.0)),
            Style::Lava => (rng.between(0.9, 1.3), rng.between(0.3, 0.8)),
        };
        // An attractor is only interesting in parts of its parameter space; these ranges keep to
        // the parts that wind rather than collapse to a point or fill the square.
        let coeffs = match style {
            Style::Orbit => [
                rng.between(1.4, 2.4) * sign(&mut rng),
                rng.between(1.4, 2.4) * sign(&mut rng),
                rng.between(1.4, 2.4) * sign(&mut rng),
                rng.between(1.4, 2.4) * sign(&mut rng),
            ],
            _ => [rng.unit(), rng.unit(), rng.unit(), rng.unit()],
        };
        Self {
            key,
            scale,
            warp,
            drift: (rng.between(-40.0, 40.0), rng.between(-40.0, 40.0)),
            twist: rng.unit(),
            contrast: rng.between(0.85, 1.25),
            coeffs,
        }
    }

    /// These parameters a fraction `at` of the way to `other`'s, in `self`'s field.
    pub fn toward(&self, other: &Params, at: f32) -> Params {
        let at = at.clamp(0.0, 1.0);
        let mix = |a: f32, b: f32| a + (b - a) * at;
        Params {
            key: self.key,
            scale: mix(self.scale, other.scale),
            warp: mix(self.warp, other.warp),
            drift: (
                mix(self.drift.0, other.drift.0),
                mix(self.drift.1, other.drift.1),
            ),
            twist: mix(self.twist, other.twist),
            contrast: mix(self.contrast, other.contrast),
            coeffs: [
                mix(self.coeffs[0], other.coeffs[0]),
                mix(self.coeffs[1], other.coeffs[1]),
                mix(self.coeffs[2], other.coeffs[2]),
                mix(self.coeffs[3], other.coeffs[3]),
            ],
        }
    }
}

fn sign(rng: &mut Seed) -> f32 {
    match rng.unit() < 0.5 {
        true => -1.0,
        false => 1.0,
    }
}

/// How many pixels a step of a per-pixel style paints.
const STEP_PIXELS: u32 = 6144;

/// How many attractor points a step of [`Style::Orbit`] plots.
const STEP_POINTS: u32 = 30_000;

/// How many particles a step of [`Style::Flow`] traces, and how far each goes.
const STEP_PARTICLES: u32 = 96;
const PARTICLE_STEPS: u32 = 56;

/// A picture being painted, a bounded step at a time.
pub struct Painter {
    style: Style,
    params: Params,
    lut: Lut,
    width: u32,
    height: u32,
    /// Time, in seconds, for the styles that move.
    t: f32,
    pixels: Vec<u8>,
    /// Per-pixel styles: the next row to paint.
    row: u32,
    /// Accumulated styles: what has landed where, and how much work is done of how much.
    density: Vec<f32>,
    done_work: u32,
    total_work: u32,
    point: (f32, f32),
    rng: Seed,
    finished: bool,
}

impl Painter {
    /// A painter for a `width` × `height` picture of `params` in `style`, coloured along `ramp`, at
    /// time `t`. Nothing is painted until it is stepped.
    pub fn new(style: Style, params: Params, ramp: &Ramp, width: u32, height: u32, t: f32) -> Self {
        let width = width.max(1);
        let height = height.max(1);
        let pixels = vec![0; (width * height * 4) as usize];
        let total_work = match style {
            Style::Orbit => (width * height * 3).max(STEP_POINTS),
            Style::Flow => (width * height / 36).max(STEP_PARTICLES),
            _ => height,
        };
        let density = match style.accumulates() {
            true => vec![0.0; (width * height) as usize],
            false => Vec::new(),
        };
        Self {
            style,
            params,
            lut: ramp.lut(),
            width,
            height,
            t,
            pixels,
            row: 0,
            density,
            done_work: 0,
            total_work,
            point: (0.1, 0.1),
            rng: Seed::new(params.key as u64 ^ 0xa5a5),
            finished: false,
        }
    }

    /// The picture's size, in pixels.
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Whether every step has been taken.
    pub fn done(&self) -> bool {
        self.finished
    }

    /// How much of the work is done, `0.0` to `1.0`.
    pub fn progress(&self) -> f32 {
        match self.finished {
            true => 1.0,
            false => self.done_work as f32 / self.total_work.max(1) as f32,
        }
    }

    /// The picture as far as it has been painted: RGBA, row-major.
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// The finished picture, painting whatever is left first.
    pub fn finish(mut self) -> Vec<u8> {
        while !self.step() {}
        self.pixels
    }

    /// Takes the picture, as far as it has been painted.
    pub fn into_pixels(self) -> Vec<u8> {
        self.pixels
    }

    /// Paints one bounded step. Whether that finished the picture.
    pub fn step(&mut self) -> bool {
        if self.finished {
            return true;
        }
        match self.style {
            Style::Orbit => self.orbit(),
            Style::Flow => self.flow(),
            _ => self.rows(),
        }
        self.finished
    }

    fn rows(&mut self) {
        let rows = (STEP_PIXELS / self.width).max(1);
        let end = (self.row + rows).min(self.height);
        let aspect = self.width as f32 / self.height as f32;
        for y in self.row..end {
            for x in 0..self.width {
                let u = (x as f32 + 0.5) / self.width as f32 * aspect;
                let v = (y as f32 + 0.5) / self.height as f32;
                let value = shade(self.style, &self.params, self.t, u, v, aspect);
                let mut rgba = self.lut.at(curve(value, self.params.contrast));
                grain(&mut rgba, x, y, self.params.key);
                let at = ((y * self.width + x) * 4) as usize;
                self.pixels[at..at + 4].copy_from_slice(&rgba);
            }
        }
        self.row = end;
        self.done_work = end;
        self.finished = end >= self.height;
    }

    fn orbit(&mut self) {
        let [a, b, c, d] = self.params.coeffs;
        let warp = self.params.warp;
        let (w, h) = (self.width as f32, self.height as f32);
        let side = w.min(h);
        let (ox, oy) = ((w - side) * 0.5, (h - side) * 0.5);
        let count = STEP_POINTS.min(self.total_work - self.done_work);
        let (mut x, mut y) = self.point;
        for _ in 0..count {
            let nx = (a * y).sin() - (b * x).cos();
            let ny = (c * x).sin() - (d * y).cos();
            // A little of the field folded in, so two attractors with near coefficients still
            // read as two hands.
            x = nx + warp * 0.05 * gradient(ny * 3.0, nx * 3.0, self.params.key);
            y = ny;
            let px = ox + (x + 2.2) / 4.4 * side;
            let py = oy + (y + 2.2) / 4.4 * side;
            if px >= 0.0 && py >= 0.0 && px < w && py < h {
                let at = py as usize * self.width as usize + px as usize;
                self.density[at] += 1.0;
            }
        }
        self.point = (x, y);
        self.done_work += count;
        if self.done_work >= self.total_work {
            self.develop(0.35);
        }
    }

    fn flow(&mut self) {
        let count = STEP_PARTICLES.min(self.total_work - self.done_work);
        let (w, h) = (self.width as f32, self.height as f32);
        let aspect = w / h;
        let p = self.params;
        for _ in 0..count {
            let mut x = self.rng.unit() * w;
            let mut y = self.rng.unit() * h;
            for _ in 0..PARTICLE_STEPS {
                let u = x / w * aspect * p.scale + p.drift.0;
                let v = y / h * p.scale + p.drift.1;
                let angle = fbm(u, v, p.key, 3) * TAU * (1.2 + p.twist * 1.6) * p.warp;
                x += angle.cos() * 1.4;
                y += angle.sin() * 1.4;
                if x < 0.0 || y < 0.0 || x >= w || y >= h {
                    break;
                }
                let at = y as usize * self.width as usize + x as usize;
                self.density[at] += 1.0;
            }
        }
        self.done_work += count;
        if self.done_work >= self.total_work {
            self.develop(0.12);
        }
    }

    /// Tone-maps what accumulated into the picture: log density along the ramp, over the ramp's
    /// own dark end lifted by `ground`.
    fn develop(&mut self, ground: f32) {
        let peak = self.density.iter().copied().fold(0.0f32, f32::max).max(1.0);
        let scale = (1.0 + peak).ln();
        for y in 0..self.height {
            for x in 0..self.width {
                let at = (y * self.width + x) as usize;
                let lit = (1.0 + self.density[at]).ln() / scale;
                let value = ground * 0.25 + lit.powf(0.8) * (1.0 - ground * 0.25);
                let mut rgba = self.lut.at(curve(value, self.params.contrast));
                grain(&mut rgba, x, y, self.params.key);
                self.pixels[at * 4..at * 4 + 4].copy_from_slice(&rgba);
            }
        }
        self.done_work = self.total_work;
        self.finished = true;
    }
}

/// The finished picture, painted in one call: for small pictures and for tests.
pub fn paint(style: Style, params: Params, ramp: &Ramp, width: u32, height: u32, t: f32) -> Vec<u8> {
    Painter::new(style, params, ramp, width, height, t).finish()
}

/// Pushes a value toward the ramp's ends: a soft S around the middle, harder as contrast rises.
fn curve(value: f32, contrast: f32) -> f32 {
    let v = value.clamp(0.0, 1.0);
    let centred = (v - 0.5) * contrast;
    (0.5 + centred.signum() * centred.abs().powf(0.9)).clamp(0.0, 1.0)
}

/// Print grain: a byte or two either way, fixed per pixel, so a flat field reads as a surface.
fn grain(rgba: &mut [u8; 4], x: u32, y: u32, key: u32) {
    let n = (white(x as i32, y as i32, key ^ 0x6a09) - 0.5) * 6.0;
    for channel in rgba.iter_mut().take(3) {
        *channel = (*channel as f32 + n).clamp(0.0, 255.0) as u8;
    }
}

fn smoothstep(low: f32, high: f32, x: f32) -> f32 {
    let t = ((x - low) / (high - low)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A per-pixel style's value at `(u, v)` -- `u` across `0.0..aspect`, `v` down `0.0..1.0` -- in
/// `0.0..=1.0`, to be read along the ramp.
fn shade(style: Style, p: &Params, t: f32, u: f32, v: f32, aspect: f32) -> f32 {
    let k = p.key;
    match style {
        Style::Bloom => {
            let x = u * p.scale + p.drift.0 + t * 0.03;
            let y = v * p.scale + p.drift.1 - t * 0.02;
            let w = 4.0 * p.warp;
            let qx = fbm(x, y, k, 4);
            let qy = fbm(x + 5.2, y + 1.3, k, 4);
            let rx = fbm(x + w * qx + 1.7, y + w * qy + 9.2, k, 4);
            let ry = fbm(x + w * qx + 8.3, y + w * qy + 2.8, k, 4);
            let f = fbm(x + w * rx, y + w * ry, k, 4);
            let swirl = (qx * qx + qy * qy).sqrt();
            let fold = (rx * rx + ry * ry).sqrt();
            0.48 + 0.95 * f + 0.4 * (swirl - 0.35) * (0.5 + p.twist) + 0.25 * (fold - 0.3)
        }
        Style::Lattice => {
            let x = u * p.scale + p.drift.0;
            let y = v * p.scale + p.drift.1;
            let wx = x + p.warp * gradient(x * 0.7, y * 0.7, k);
            let wy = y + p.warp * gradient(x * 0.7 + 31.0, y * 0.7, k);
            let (cx, cy) = (wx.floor() as i32, wy.floor() as i32);
            let mut first = f32::MAX;
            let mut second = f32::MAX;
            let mut cell = 0u32;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let (gx, gy) = (cx + dx, cy + dy);
                    let jx = gx as f32 + 0.15 + 0.7 * white(gx, gy, k);
                    let jy = gy as f32 + 0.15 + 0.7 * white(gx, gy, k ^ 0x51);
                    let d = (jx - wx).powi(2) + (jy - wy).powi(2);
                    if d < first {
                        second = first;
                        first = d;
                        cell = hash(gx, gy, k ^ 0x77);
                    } else if d < second {
                        second = d;
                    }
                }
            }
            let seam = second.sqrt() - first.sqrt();
            let tone = (cell >> 8) as f32 / (1u32 << 24) as f32;
            let glass = 0.18 + 0.74 * tone + 0.08 * fbm(x * 3.0, y * 3.0, k, 2);
            let lead = smoothstep(0.02, 0.09 + 0.05 * p.twist, seam);
            glass * lead + 0.02 * (1.0 - lead)
        }
        Style::Moire => {
            let (ax, ay) = (p.coeffs[0] * aspect, p.coeffs[1]);
            let (bx, by) = (p.coeffs[2] * aspect, p.coeffs[3]);
            let wobble = p.warp * fbm(u * 2.0 + p.drift.0, v * 2.0 + p.drift.1, k, 3);
            let d1 = ((u - ax).powi(2) + (v - ay).powi(2)).sqrt() + wobble;
            let d2 = ((u - bx).powi(2) + (v - by).powi(2)).sqrt() - wobble;
            let f = 60.0 * p.scale;
            let beat = (d1 * f + t).sin() + (d2 * f * (1.0 + 0.1 * p.twist) - t * 0.7).sin();
            0.5 + 0.25 * beat
        }
        Style::Caustic => {
            // Light through water: a point is pushed round by the waves a few times, and what
            // gathers where the pushes agree is the bright web.
            let px = (u * TAU * p.scale).rem_euclid(TAU) - 250.0 + p.drift.0 * 0.01;
            let py = (v * TAU * p.scale).rem_euclid(TAU) - 250.0 + p.drift.1 * 0.01;
            let (mut ix, mut iy) = (px, py);
            let mut c = 1.0;
            let strength = 0.005;
            for n in 0..5 {
                let time = (t * 0.5 + p.coeffs[0] * 20.0) * (1.0 - 3.5 / (n as f32 + 1.0));
                let nx = px + (time - ix).cos() + (time + iy).sin();
                let ny = py + (time - iy).sin() + (time + ix).cos();
                ix = nx;
                iy = ny;
                let lx = px / ((ix + time).sin() / strength);
                let ly = py / ((iy + time).cos() / strength);
                c += 1.0 / (lx * lx + ly * ly).sqrt().max(1e-4);
            }
            c /= 5.0;
            let c = 1.17 - c.powf(1.4);
            let web = c.abs().powf(7.0 + 3.0 * p.twist).clamp(0.0, 1.0);
            0.08 + 0.12 * (1.0 - v) + web * 0.85
        }
        Style::Aurora => {
            // A dark sky, and curtains hanging from a wavering lower hem: bright at the hem, rays
            // rising out of it and fading upward, nothing below it but a little glow.
            let x = u * p.scale + p.drift.0;
            let sky = 0.02 + 0.1 * v;
            let mut light = 0.0;
            for fold in 0..2 {
                let key = k.wrapping_add(fold * 0x51ed);
                let hem = 0.62 - 0.14 * fold as f32 + 0.16 * fbm(x * 0.9 + t * 0.04, fold as f32 * 7.3, key, 3);
                let rays = 0.35 + 0.65 * (0.5 + 0.5 * fbm(x * 14.0 + t * 0.15, 3.1, key ^ 0x33, 2));
                let height = 0.1 + 0.12 * p.warp;
                let rising = match v < hem {
                    true => (-(hem - v) / height).exp(),
                    false => (-((v - hem) / 0.025).powi(2)).exp(),
                };
                light += rising * rays * (0.85 - 0.3 * fold as f32);
            }
            (sky + light * 0.9).clamp(0.0, 1.0)
        }
        Style::Lava => {
            let mut field = 0.0;
            for n in 0..6 {
                let phase = p.coeffs[n % 4] * TAU + n as f32 * 1.7;
                let speed = 0.25 + 0.1 * n as f32;
                let cx = aspect * (0.5 + 0.32 * (t * speed + phase).sin() * (0.6 + 0.4 * p.twist));
                let cy = 0.5 + 0.34 * (t * speed * 0.8 + phase * 1.3).cos();
                let r = 0.13 + 0.04 * ((n * 7 % 5) as f32 / 5.0);
                let d2 = (u - cx).powi(2) + (v - cy).powi(2);
                field += r * r / d2.max(1e-4);
            }
            let inside = smoothstep(0.9, 1.25, field);
            let core = ((field - 1.25) * 0.25).clamp(0.0, 1.0);
            0.12 + 0.18 * field.min(1.0) * (1.0 - inside) + inside * (0.58 + 0.38 * core)
        }
        // Accumulated styles are never shaded per pixel.
        Style::Orbit | Style::Flow => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foliage::Color;

    fn ramp() -> Ramp {
        Ramp::new(&[
            Color::rgb(0.05, 0.04, 0.08),
            Color::rgb(0.55, 0.35, 0.80),
            Color::rgb(0.98, 0.92, 0.85),
        ])
    }

    #[test]
    fn every_style_paints_a_whole_picture() {
        for style in Style::ALL {
            let params = Params::of(style, Seed::new(42));
            let pixels = paint(style, params, &ramp(), 48, 32, 0.0);
            assert_eq!(pixels.len(), 48 * 32 * 4, "{style:?}");
            assert!(pixels.chunks(4).all(|px| px[3] == 255), "{style:?} left a pixel unpainted");
            let first = &pixels[0..4];
            assert!(
                pixels.chunks(4).any(|px| px != first),
                "{style:?} painted a flat picture"
            );
        }
    }

    #[test]
    fn a_seed_paints_the_same_picture_every_time() {
        for style in Style::ALL {
            let params = Params::of(style, Seed::new(9));
            assert_eq!(
                paint(style, params, &ramp(), 24, 24, 0.5),
                paint(style, params, &ramp(), 24, 24, 0.5),
                "{style:?}"
            );
        }
    }

    #[test]
    fn stepping_reaches_the_same_picture_as_painting_whole() {
        let params = Params::of(Style::Bloom, Seed::new(3));
        let mut painter = Painter::new(Style::Bloom, params, &ramp(), 160, 100, 0.0);
        let mut steps = 0;
        while !painter.step() {
            steps += 1;
            assert!(painter.progress() < 1.0);
        }
        assert!(steps > 0, "a picture this size takes more than one step");
        assert_eq!(painter.into_pixels(), paint(Style::Bloom, params, &ramp(), 160, 100, 0.0));
    }

    #[test]
    fn a_walk_keeps_its_field_and_lands_on_the_other_parameters() {
        let a = Params::of(Style::Bloom, Seed::new(1));
        let b = Params::of(Style::Bloom, Seed::new(2));
        let there = a.toward(&b, 1.0);
        assert_eq!(there.key, a.key);
        assert_eq!(there.scale, b.scale);
        assert_eq!(a.toward(&b, 0.0), a);
    }
}
