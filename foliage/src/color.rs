//! Color, in the one form every renderer reads.

use bytemuck::{Pod, Zeroable};

use crate::aspen::blend;

/// An sRGB color with an alpha channel, one channel per field.
///
/// Channels are separate floats in `0.0..=1.0` rather than packed bytes, because each is
/// interpolated independently when a color is animated and because that is the form a shader takes.
///
/// This is what a [`Palette`](crate::Palette) tone resolves to, and the form a palette is stated
/// in. An element declares the tone rather than the color, so that what a tone resolves to can be
/// changed in one place.
///
/// `#[repr(C)]` over four floats, which is the layout a shader reads a `vec4` in.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
pub struct Color {
    /// The red channel, in `0.0..=1.0`.
    pub red: f32,
    /// The green channel, in `0.0..=1.0`.
    pub green: f32,
    /// The blue channel, in `0.0..=1.0`.
    pub blue: f32,
    /// Coverage, in `0.0..=1.0`: `0.0` paints nothing and `1.0` paints over what is behind it.
    pub alpha: f32,
}

impl Color {
    /// An opaque color.
    pub fn rgb(red: f32, green: f32, blue: f32) -> Self {
        Self::rgba(red, green, blue, 1.0)
    }

    /// A color with an alpha channel. Every channel is clamped to `0.0..=1.0`.
    pub fn rgba(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red: red.clamp(0.0, 1.0),
            green: green.clamp(0.0, 1.0),
            blue: blue.clamp(0.0, 1.0),
            alpha: alpha.clamp(0.0, 1.0),
        }
    }

    /// This color a fraction `at` of the way to `other`, channel by channel.
    ///
    /// What a [`Motion::Color`](crate::Motion::Color) resolves to each frame. Channel-wise because
    /// that is what separate floats are for, and it is why a color is held as four of them rather
    /// than as packed bytes.
    pub(crate) fn blend(self, other: Self, at: f32) -> Self {
        Self::rgba(
            blend(self.red, other.red, at),
            blend(self.green, other.green, at),
            blend(self.blue, other.blue, at),
            blend(self.alpha, other.alpha, at),
        )
    }

    /// This color at a fraction of its own alpha.
    ///
    /// What an element's resolved opacity does to what it is painted in. Taken at extraction rather
    /// than written onto the element, so nothing holds a second copy of a color that a repaint
    /// would have to find.
    pub(crate) fn faded(self, opacity: f32) -> Self {
        Self {
            alpha: self.alpha * opacity.clamp(0.0, 1.0),
            ..self
        }
    }

    /// This color as OKLab lightness and its two opponent axes.
    ///
    /// The space a [`Scheme`](crate::Scheme) walks a ramp in. Lightness there is perceptual, so a
    /// fixed offset reads as the same size of change at either end of a ramp, and the two axes carry
    /// hue and chroma together -- which is what lets a step move lightness while leaving the rest of
    /// the color where the seed put it. Interpolating sRGB channels toward black or white does
    /// neither: it desaturates and drifts in hue.
    ///
    /// Alpha is not part of the conversion and is carried separately.
    pub(crate) fn oklab(self) -> (f32, f32, f32) {
        let red = linear(self.red);
        let green = linear(self.green);
        let blue = linear(self.blue);
        let long = (0.412_221_46 * red + 0.536_332_55 * green + 0.051_445_995 * blue).cbrt();
        let medium = (0.211_903_5 * red + 0.680_699_5 * green + 0.107_396_96 * blue).cbrt();
        let short = (0.088_302_46 * red + 0.281_718_85 * green + 0.629_978_7 * blue).cbrt();
        (
            0.210_454_26 * long + 0.793_617_8 * medium - 0.004_072_047 * short,
            1.977_998_5 * long - 2.428_592_2 * medium + 0.450_593_7 * short,
            0.025_904_037 * long + 0.782_771_77 * medium - 0.808_675_77 * short,
        )
    }

    /// The color at these OKLab coordinates, in sRGB.
    ///
    /// Shifting lightness at a fixed hue and chroma can leave the sRGB gamut, most often at a ramp's
    /// bright end for a saturated seed. Rather than clip a channel -- which shifts hue, the one
    /// thing a ramp holds -- chroma is backed off toward the neutral of the same lightness until the
    /// color fits, so the step lands as close to its seed's hue as sRGB can carry. The `bool` is
    /// whether it had to be, which is what a ramp traces.
    pub(crate) fn from_oklab(lightness: f32, a: f32, b: f32, alpha: f32) -> (Self, bool) {
        let direct = oklab_linear(lightness, a, b);
        if in_gamut(direct) {
            return (encode(direct, alpha), false);
        }
        // Chroma zero is the neutral of this lightness and always fits, so the low end of the search
        // is known to be good from the start and the loop only ever raises it.
        let (mut fits, mut over) = (0.0f32, 1.0f32);
        for _ in 0..16 {
            let between = 0.5 * (fits + over);
            if in_gamut(oklab_linear(lightness, a * between, b * between)) {
                fits = between;
            } else {
                over = between;
            }
        }
        (
            encode(oklab_linear(lightness, a * fits, b * fits), alpha),
            true,
        )
    }
}

/// One sRGB channel with its transfer function removed.
fn linear(channel: f32) -> f32 {
    if channel <= 0.040_45 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

/// One linear channel with the sRGB transfer function applied.
fn transfer(channel: f32) -> f32 {
    if channel <= 0.003_130_8 {
        channel * 12.92
    } else {
        1.055 * channel.powf(1.0 / 2.4) - 0.055
    }
}

/// These OKLab coordinates as linear RGB, which is where the gamut is a range rather than a shape.
fn oklab_linear(lightness: f32, a: f32, b: f32) -> (f32, f32, f32) {
    let long = (lightness + 0.396_337_78 * a + 0.215_803_76 * b).powi(3);
    let medium = (lightness - 0.105_561_346 * a - 0.063_854_17 * b).powi(3);
    let short = (lightness - 0.089_484_18 * a - 1.291_485_5 * b).powi(3);
    (
        4.076_741_7 * long - 3.307_711_6 * medium + 0.230_969_94 * short,
        -1.268_438 * long + 2.609_757_4 * medium - 0.341_319_38 * short,
        -0.004_196_086_3 * long - 0.703_418_6 * medium + 1.707_614_7 * short,
    )
}

/// Whether linear RGB names a color sRGB can carry, with the slack a round trip costs.
fn in_gamut((red, green, blue): (f32, f32, f32)) -> bool {
    const SLACK: f32 = 1e-4;
    [red, green, blue]
        .into_iter()
        .all(|channel| channel >= -SLACK && channel <= 1.0 + SLACK)
}

/// Linear RGB as a [`Color`].
fn encode((red, green, blue): (f32, f32, f32), alpha: f32) -> Color {
    Color::rgba(transfer(red), transfer(green), transfer(blue), alpha)
}
