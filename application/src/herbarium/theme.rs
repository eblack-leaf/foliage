//! What the herbarium is painted in while it is open: one hue, a verdigris -- brass gone green --
//! and everything drawn from it.
//!
//! The suite's apps are each one hue, and this is the credentials app's. The way a scheme is drawn
//! from one hue is the suite's own and lives with it, not in lichen, so it is stated again here: a
//! breath of the hue in the grounds and the ink, the hue at its fill for the press to make, deep
//! for what is chosen, pale for what the system says, and the key's ramp pale to deep, turning a
//! few degrees as it deepens so the mosaic is not flat. Danger and caution are the two colours on
//! the page that are not the hue, set far round the wheel from it so they read.

use foliage::{Color, Palette, Scheme};

/// The spectrum the key is drawn in.
pub(crate) const SPECIMEN: usize = 0;

/// The hue, in OKLCH degrees.
const HUE: f32 = 145.0;

/// The lightness and chroma of each thing the scheme draws from the hue.
const SURFACE: (f32, f32) = (0.20, 0.012);
const RAISED: (f32, f32) = (0.265, 0.016);
const MUTED: (f32, f32) = (0.45, 0.018);
const INK: (f32, f32) = (0.955, 0.012);
const ACCENT: (f32, f32) = (0.74, 0.12);
const CHOSEN: (f32, f32) = (0.42, 0.09);
const SIGNAL: (f32, f32) = (0.88, 0.06);
const LIT: (f32, f32) = (0.42, 0.06);

/// The key, pale to deep, and how far the hue turns at each step.
const RAMP: [(f32, f32, f32); 4] = [
    (0.88, 0.07, 3.0),
    (0.74, 0.12, 0.0),
    (0.58, 0.12, -3.0),
    (0.40, 0.08, -6.0),
];

/// The blip behind a chosen tip, pale at its middle to the chosen at its end.
const BLIPS: [(f32, f32); 3] = [(0.88, 0.10), (0.66, 0.12), CHOSEN];

/// Where danger and caution are set, and at which hues.
const EXCEPTION: (f32, f32) = (0.66, 0.15);
const DANGER: f32 = 325.0;
const CAUTION: f32 = 265.0;

/// The herbarium's reading.
pub(crate) fn scheme() -> Scheme {
    let at = |(lightness, chroma): (f32, f32)| Color::oklch(lightness, chroma, HUE);
    let accent = at(ACCENT);
    let ramp = RAMP.map(|(lightness, chroma, drift)| Color::oklch(lightness, chroma, HUE + drift));
    Scheme::new()
        .set(Palette::Surface, at(SURFACE))
        .set(Palette::Raised, at(RAISED))
        .set(Palette::Muted, at(MUTED))
        .set(Palette::Ink, at(INK))
        .set(Palette::Accent, accent)
        .set(Palette::Accent.mark(), accent)
        .set(Palette::Positive, at(CHOSEN))
        .set(Palette::Signal.mark(), at(SIGNAL))
        .set(Palette::Signal, at(LIT))
        .set(
            Palette::Danger,
            Color::oklch(EXCEPTION.0, EXCEPTION.1, DANGER),
        )
        .set(
            Palette::Caution,
            Color::oklch(EXCEPTION.0, EXCEPTION.1, CAUTION),
        )
        .spectrum(SPECIMEN, &ramp)
        .spectrum(lichen::BLIP, &BLIPS.map(at))
}
