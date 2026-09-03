//! Color, in the one form every renderer reads.

use bytemuck::{Pod, Zeroable};

/// An sRGB color with an alpha channel, one channel per field.
///
/// Channels are separate floats in `0.0..=1.0` rather than packed bytes, because each is
/// interpolated independently when a color is animated and because that is the form a shader takes.
///
/// This is what a [`Palette`](crate::Palette) role resolves to, and the form a palette is stated
/// in. An element declares the role rather than the color, so that what a role resolves to can be
/// changed in one place.
///
/// `#[repr(C)]` over four floats, which is the layout a shader reads a `vec4` in.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
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
}
