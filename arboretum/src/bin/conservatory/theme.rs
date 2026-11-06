//! What the exhibition is printed in: a dark, shaded ground that everything on the page comes out
//! of, bone type, and one vermilion signal.
//!
//! Nothing is simply put on the wall: type and presses come up out of its dark and into focus, and
//! the works develop out of it as a generator's preview does -- dim and soft first, then lit and
//! sharp. The works are dark-grounded too, so their darks are the wall's and only their light
//! stands out of it.

use foliage::{Color, FontSize, Horizontal, Palette, Scheme, Source, center_x};

use arboretum::genart::{Ramp, bytes};

/// The ground, the panels on it, the grey between, and the ink.
const WALL: (f32, f32, f32) = (0.175, 0.005, 250.0);
const PANEL: (f32, f32, f32) = (0.225, 0.006, 250.0);
const GREY: (f32, f32, f32) = (0.62, 0.008, 80.0);
const INK: (f32, f32, f32) = (0.93, 0.012, 85.0);
/// The signal.
const VERMILION: (f32, f32, f32) = (0.67, 0.2, 34.0);

const BLIPS: [(f32, f32, f32); 3] = [(0.80, 0.12, 40.0), (0.67, 0.2, 34.0), INK];

/// How wide the page's column is at most.
pub const COLUMN: f32 = 1240.0;

/// The room between the page's edges and anything on it.
pub const GUTTER: f32 = 22.0;

fn oklch((l, c, h): (f32, f32, f32)) -> Color {
    Color::oklch(l, c, h)
}

/// The exhibition's reading.
pub fn scheme() -> Scheme {
    Scheme::new()
        .set(Palette::Surface, oklch(WALL))
        .set(Palette::Raised, oklch(PANEL))
        .set(Palette::Muted, oklch(GREY))
        .set(Palette::Ink, oklch(INK))
        .set(Palette::Accent, oklch(VERMILION))
        .set(Palette::Accent.mark(), oklch(VERMILION))
        .set(Palette::Positive, oklch(INK))
        .set(Palette::Signal, oklch((0.30, 0.006, 250.0)))
        .set(Palette::Signal.mark(), oklch(INK))
        .set(Palette::Danger, oklch((0.64, 0.19, 28.0)))
        .set(Palette::Caution, oklch((0.76, 0.13, 75.0)))
        // The four rooms, each in the pigment its works lean toward.
        .set(Palette::hue(0), oklch((0.80, 0.12, 105.0)))
        .set(Palette::hue(1), oklch((0.74, 0.11, 155.0)))
        .set(Palette::hue(2), oklch((0.74, 0.09, 230.0)))
        .set(Palette::hue(3), oklch((0.72, 0.14, 45.0)))
        .spectrum(0, &[oklch(VERMILION), oklch(INK)])
        .spectrum(lichen::BLIP, &BLIPS.map(oklch))
}

/// The light on the wall: lifted under the title and falling away to the edges and the foot. Smooth
/// on purpose, and small, since it is only ever stretched; `(pixels, width, height)`.
pub fn shading() -> (Vec<u8>, u32, u32) {
    let (width, height) = (160u32, 100u32);
    let (wall, c, h) = WALL;
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let (u, v) = (x as f32 / (width - 1) as f32, y as f32 / (height - 1) as f32);
            // A broad pool of light high on the wall, a little left of the middle.
            let (dx, dy) = ((u - 0.42) * 1.25, v + 0.12);
            let pool = (-(dx * dx + dy * dy) / 0.42).exp();
            // And the edges and the foot falling away into the dark.
            let edge = ((u - 0.5).abs() * 2.0).powf(3.0) * 0.6 + v.powf(2.0) * 0.4;
            let l = wall + 0.042 * pool - 0.03 * edge;
            pixels.extend_from_slice(&bytes(oklch((l, c, h))));
        }
    }
    (pixels, width, height)
}

/// The column everything stands in: the page less its gutters, and no wider than [`COLUMN`].
pub fn column() -> Horizontal {
    center_x(50.pct())
        .width(100.pct() - (2.0 * GUTTER).px())
        .at_most(COLUMN.px())
}

/// The sizes the display face is set at. Its letters are wide -- three quarters of their size --
/// so the title takes two lines wherever one would not fit.
pub fn title() -> FontSize {
    FontSize::new().xs(44).sm(60).md(84).lg(104).xl(124)
}

pub fn heading() -> FontSize {
    FontSize::new().xs(26).md(34)
}

pub fn quote() -> FontSize {
    FontSize::new().xs(19).md(26)
}

pub fn lead() -> FontSize {
    FontSize::new().xs(16).md(20)
}

pub fn body() -> FontSize {
    FontSize::new().xs(14).md(15)
}

/// A ramp from OKLCH stops: lightness, chroma, hue.
pub fn ramp(stops: &[(f32, f32, f32)]) -> Ramp {
    Ramp::new(&stops.iter().map(|&stop| oklch(stop)).collect::<Vec<_>>())
}

/// The ramp a visitor's words call for: the first of the colours it names, or ink and vermilion
/// where it names none. Every one rises out of the dark, as the works' do.
pub fn ramp_for(prompt: &str) -> Ramp {
    let prompt = prompt.to_lowercase();
    let has = |words: &[&str]| words.iter().any(|word| prompt.contains(word));
    let stops: &[(f32, f32, f32)] = if has(&["moss", "fern", "leaf", "green", "forest", "lichen"]) {
        &[(0.16, 0.01, 160.0), (0.38, 0.07, 155.0), (0.66, 0.11, 140.0), (0.90, 0.07, 120.0)]
    } else if has(&["ember", "rust", "fire", "autumn", "copper", "decay"]) {
        &[(0.16, 0.02, 35.0), (0.40, 0.11, 35.0), (0.66, 0.15, 50.0), (0.92, 0.07, 80.0)]
    } else if has(&["tide", "glass", "sea", "rain", "ice", "river"]) {
        &[(0.16, 0.02, 235.0), (0.38, 0.07, 220.0), (0.70, 0.08, 200.0), (0.95, 0.03, 185.0)]
    } else if has(&["neon", "city", "night", "electric", "synth"]) {
        &[(0.15, 0.01, 270.0), (0.36, 0.09, 260.0), (0.68, 0.11, 200.0), (0.93, 0.06, 165.0)]
    } else if has(&["sun", "gold", "honey", "wheat", "dawn"]) {
        &[(0.17, 0.01, 80.0), (0.45, 0.09, 75.0), (0.76, 0.14, 85.0), (0.95, 0.06, 98.0)]
    } else {
        &[(0.16, 0.01, 40.0), (0.38, 0.12, 34.0), (0.66, 0.19, 36.0), (0.94, 0.05, 60.0)]
    };
    ramp(stops)
}

/// `text` cut to `most` characters, with an ellipsis where anything was cut: for a line that has
/// a fixed number of lines to stand in.
pub fn clip(text: &str, most: usize) -> String {
    if text.chars().count() <= most {
        return text.to_string();
    }
    let cut: String = text.chars().take(most.saturating_sub(1)).collect();
    format!("{}…", cut.trim_end())
}
