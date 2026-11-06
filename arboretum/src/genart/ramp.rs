//! Colours along a line, read perceptually.

use foliage::Color;

/// Stops a colour is read along, evenly spaced from `0.0` to `1.0` and blended through OKLab.
///
/// Where lichen's ramp blends in RGB for tiles that each take one colour, a picture reads a ramp
/// hundreds of thousands of times across smooth fields, and an RGB blend between two saturated
/// stops greys out in its middle where the eye expects the hue to carry. So this one goes through
/// OKLab ([`Color::toward`]), and a picture reads it through a [`Lut`] rather than per pixel.
#[derive(Clone, Debug, PartialEq)]
pub struct Ramp {
    stops: Vec<Color>,
}

impl Ramp {
    /// A ramp through `stops`, first to last. One stop is a ramp of one colour.
    ///
    /// # Panics
    ///
    /// With no stops at all.
    pub fn new(stops: &[Color]) -> Self {
        assert!(!stops.is_empty(), "a ramp needs a stop");
        Self {
            stops: stops.to_vec(),
        }
    }

    /// The stops, first to last.
    pub fn stops(&self) -> &[Color] {
        &self.stops
    }

    /// The colour a fraction `at` along, clamped to the ends.
    pub fn at(&self, at: f32) -> Color {
        let last = self.stops.len() - 1;
        if last == 0 {
            return self.stops[0];
        }
        let along = at.clamp(0.0, 1.0) * last as f32;
        let index = (along.floor() as usize).min(last - 1);
        self.stops[index].toward(self.stops[index + 1], along - index as f32)
    }

    /// Every stop moved toward `other` by `at`, stop for stop -- the ramp part of the way to another
    /// one, which is what a walk between two pieces reads its colours along. Stops beyond the shorter
    /// ramp's length are read off it at the same fraction along.
    pub fn toward(&self, other: &Ramp, at: f32) -> Ramp {
        let count = self.stops.len().max(other.stops.len());
        let stops: Vec<Color> = (0..count)
            .map(|n| {
                let along = match count {
                    1 => 0.0,
                    _ => n as f32 / (count - 1) as f32,
                };
                self.at(along).toward(other.at(along), at)
            })
            .collect();
        Ramp { stops }
    }

    /// The ramp read at 256 points, as bytes, for pictures.
    pub fn lut(&self) -> Lut {
        let mut table = [[0u8; 4]; 256];
        for (n, entry) in table.iter_mut().enumerate() {
            *entry = bytes(self.at(n as f32 / 255.0));
        }
        Lut(table)
    }
}

/// A ramp at 256 points, as RGBA bytes: what a picture reads instead of blending per pixel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lut([[u8; 4]; 256]);

impl Lut {
    /// The bytes a fraction `at` along, clamped.
    #[inline]
    pub fn at(&self, at: f32) -> [u8; 4] {
        self.0[(at.clamp(0.0, 1.0) * 255.0 + 0.5) as usize]
    }
}

/// A colour as RGBA bytes, rounded.
pub fn bytes(color: Color) -> [u8; 4] {
    let byte = |channel: f32| (channel.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    [
        byte(color.red),
        byte(color.green),
        byte(color.blue),
        byte(color.alpha),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ramp_reaches_both_of_its_ends() {
        let black = Color::rgb(0.0, 0.0, 0.0);
        let white = Color::rgb(1.0, 1.0, 1.0);
        let ramp = Ramp::new(&[black, white]);
        assert_eq!(bytes(ramp.at(0.0)), [0, 0, 0, 255]);
        assert_eq!(bytes(ramp.at(1.0)), [255, 255, 255, 255]);
        assert_eq!(ramp.lut().at(-3.0), [0, 0, 0, 255]);
        assert_eq!(ramp.lut().at(9.0), [255, 255, 255, 255]);
    }

    #[test]
    fn a_ramp_toward_itself_is_itself() {
        let ramp = Ramp::new(&[Color::rgb(0.2, 0.4, 0.1), Color::rgb(0.9, 0.3, 0.5)]);
        let walked = ramp.toward(&ramp, 0.5);
        // Through OKLab and back, which is a round trip through cube roots: a byte either way.
        for (a, b) in ramp.stops().iter().zip(walked.stops()) {
            for (x, y) in bytes(*a).iter().zip(bytes(*b)) {
                assert!(x.abs_diff(y) <= 1, "{a:?} became {b:?}");
            }
        }
    }
}
