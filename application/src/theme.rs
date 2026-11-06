//! What the site is coloured and spaced with, decided in one place.
//!
//! foliage's dark ground, the ember the leaf has always been drawn in, and the green a patch of it
//! turns while the way it stands for is pointed at.

use foliage::{Color, Font, Grove, Palette, Scheme};

/// How far everything keeps from the window's edge.
pub(crate) const MARGIN: f32 = 24.0;

/// The spectrum the leaf is drawn in.
pub(crate) const SPECIMEN: usize = 0;

/// The spectrum a lit patch of the leaf is drawn in: green, palest at its middle.
pub(crate) const PATCH: usize = Scheme::SPECTRA - 1;

/// The ramp the leaf is coloured from: gold at the tip, deep red toward the stem.
const EMBER: [Color; 4] = [
    Color::rgb(0.97, 0.76, 0.30),
    Color::rgb(0.93, 0.52, 0.17),
    Color::rgb(0.82, 0.28, 0.13),
    Color::rgb(0.58, 0.12, 0.11),
];

/// What the outline is dashed in: the ramp's middle, a little deeper.
pub(crate) const EDGE: (f32, f32, f32) = (0.80, 0.40, 0.19);

/// The green a lit patch turns, first to last.
const GREEN: [Color; 3] = [
    Color::rgb(0.74, 0.90, 0.42),
    Color::rgb(0.46, 0.76, 0.30),
    Color::rgb(0.22, 0.52, 0.24),
];

/// Each way off the page, worn in its own stretch of the ramp: gold to red, in the order the
/// patches they light stand down the leaf.
pub(crate) const HUES: [Color; 4] = EMBER;

/// What a way's tile turns while it is lit: the green its patch on the leaf turns.
pub(crate) const LIT: Color = GREEN[1];

/// The site's reading.
pub(crate) fn scheme() -> Scheme {
    Scheme::new()
        .set(Palette::Accent, EMBER[1])
        .set(Palette::Accent.mark(), Color::rgb(0.98, 0.66, 0.36))
        .spectrum(SPECIMEN, &EMBER)
        .spectrum(PATCH, &GREEN)
}

/// The face quiet words are set in.
pub(crate) fn italic(grove: &mut Grove) -> Font {
    grove.font(include_bytes!(
        "../assets/fonts/JetBrainsMonoNL-MediumItalic.ttf"
    ))
}
