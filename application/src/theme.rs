//! What the site is coloured and spaced with, decided in one place.
//!
//! How a press is dressed and how large a chip is are lichen's. What is here is the site's own
//! reading of them: foliage's dark ground, the ember the leaf has always been drawn in, and the
//! green a chosen tip turns.

use foliage::{Color, Font, Grove, Palette, Scheme};

/// How far everything keeps from the window's edge.
pub(crate) const MARGIN: f32 = 24.0;

/// The room between two things side by side, or one above the other.
pub(crate) const GAP: f32 = 12.0;

/// How far what a tip opens stands off the column the leaf and its controls take.
pub(crate) const STANDOFF: f32 = 24.0;

/// The spectrum the leaf is drawn in.
pub(crate) const SPECIMEN: usize = 0;

/// The spectrum the blip behind a chosen tip turns: green, palest at its middle, which the chip
/// covers, so what shows round the chip is the deep end.
pub(crate) const BLIP: usize = Scheme::SPECTRA - 1;

/// The ramp the leaf is coloured from: gold at the tip, deep red toward the stem.
const EMBER: [Color; 4] = [
    Color::rgb(0.97, 0.76, 0.30),
    Color::rgb(0.93, 0.52, 0.17),
    Color::rgb(0.82, 0.28, 0.13),
    Color::rgb(0.58, 0.12, 0.11),
];

/// What the outline is dashed in: the ramp's middle, a little deeper.
pub(crate) const EDGE: (f32, f32, f32) = (0.80, 0.40, 0.19);

/// The green the blip turns, first to last.
const BLIPS: [Color; 3] = [
    Color::rgb(0.74, 0.90, 0.42),
    Color::rgb(0.46, 0.76, 0.30),
    Color::rgb(0.22, 0.52, 0.24),
];

/// What is chosen is filled with: the deep end of the blip's green, so a chosen chip and the blip
/// behind it read as one thing.
const CHOSEN: Color = Color::rgb(0.14, 0.36, 0.17);

/// The three places the site links to, each worn in its own stretch of the ramp -- so the three
/// read as the leaf's, gold to red, the way the prongs they used to be were.
pub(crate) const HUES: [Color; 3] = [EMBER[0], EMBER[1], EMBER[2]];

/// What is read on any of [`HUES`]: near-black, warmed toward them.
pub(crate) const ON_HUE: Color = Color::rgb(0.10, 0.06, 0.04);

/// The site's reading.
pub(crate) fn scheme() -> Scheme {
    let accent = EMBER[1];
    Scheme::new()
        .set(Palette::Accent, accent)
        .set(Palette::Accent.mark(), Color::rgb(0.98, 0.66, 0.36))
        .set(Palette::Contrast, ON_HUE)
        .set(Palette::Positive, CHOSEN)
        .spectrum(SPECIMEN, &EMBER)
        .spectrum(BLIP, &BLIPS)
}

/// The face quiet words are set in: what an annotation says, a byline.
pub(crate) fn italic(grove: &mut Grove) -> Font {
    grove.font(include_bytes!(
        "../assets/fonts/JetBrainsMonoNL-MediumItalic.ttf"
    ))
}
