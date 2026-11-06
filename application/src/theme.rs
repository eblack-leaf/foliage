//! What the site is coloured and spaced with, decided in one place.
//!
//! foliage's dark ground, and the ember the leaf has always been drawn in.

use foliage::{Color, Font, Grove, Palette, Scheme};

/// How far everything keeps from the window's edge.
pub(crate) const MARGIN: f32 = 24.0;

/// The spectrum the leaf is drawn in.
pub(crate) const SPECIMEN: usize = 0;

/// The ramp the leaf is coloured from: gold at the tip, deep red toward the stem.
const EMBER: [Color; 4] = [
    Color::rgb(0.97, 0.76, 0.30),
    Color::rgb(0.93, 0.52, 0.17),
    Color::rgb(0.82, 0.28, 0.13),
    Color::rgb(0.58, 0.12, 0.11),
];

/// What the outline is dashed in: the ramp's middle, a little deeper.
pub(crate) const EDGE: (f32, f32, f32) = (0.80, 0.40, 0.19);

/// Each of the apps and the source, marked in its own stretch of the ramp, gold to red, in the
/// order the label lists them.
pub(crate) const HUES: [Color; 4] = EMBER;

/// The site's reading.
pub(crate) fn scheme() -> Scheme {
    Scheme::new()
        .set(Palette::Accent, EMBER[1])
        .set(Palette::Accent.mark(), Color::rgb(0.98, 0.66, 0.36))
        .spectrum(SPECIMEN, &EMBER)
}

/// The face quiet words are set in.
pub(crate) fn italic(grove: &mut Grove) -> Font {
    grove.font(include_bytes!(
        "../assets/fonts/JetBrainsMonoNL-MediumItalic.ttf"
    ))
}
