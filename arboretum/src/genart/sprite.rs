//! Soft things the renderer cannot draw itself: light, shadow, gradients.
//!
//! foliage fills flat and draws no blur, so anything soft is a small picture stretched over a large
//! box, with the linear filtering doing the smoothing. Every sprite here keeps its colour constant
//! and varies only its alpha: the texture is straight alpha, and a transparent texel of a different
//! colour would bleed a fringe of that colour into its neighbours as the filter blends them.

use foliage::Color;

use super::ramp::{Ramp, bytes};

/// A round light: `color` at its middle, falling away to nothing at the edge of a `side` square.
pub fn falloff(side: u32, color: Color) -> Vec<u8> {
    let side = side.max(2);
    let [r, g, b, a] = bytes(color);
    let half = side as f32 / 2.0;
    let mut out = Vec::with_capacity((side * side * 4) as usize);
    for y in 0..side {
        for x in 0..side {
            let dx = (x as f32 + 0.5 - half) / half;
            let dy = (y as f32 + 0.5 - half) / half;
            let d = (dx * dx + dy * dy).sqrt().min(1.0);
            // Smooth to zero at the rim, with a broad shoulder, so a stretched light has no edge.
            let fall = (1.0 - d * d).powi(2);
            out.extend_from_slice(&[r, g, b, (a as f32 * fall) as u8]);
        }
    }
    out
}

/// A soft rectangle: solid in the middle `core` fraction of each axis, fading to nothing at the
/// picture's edges. Stretched under a card, it is the card's shadow.
pub fn shadow(width: u32, height: u32, core: f32, color: Color) -> Vec<u8> {
    let (width, height) = (width.max(2), height.max(2));
    let [r, g, b, a] = bytes(color);
    let core = core.clamp(0.0, 0.95);
    let mut out = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let u = ((x as f32 + 0.5) / width as f32 - 0.5).abs() * 2.0;
            let v = ((y as f32 + 0.5) / height as f32 - 0.5).abs() * 2.0;
            let du = ((u - core) / (1.0 - core)).max(0.0);
            let dv = ((v - core) / (1.0 - core)).max(0.0);
            let d = (du * du + dv * dv).sqrt().min(1.0);
            let fall = (1.0 - d * d * (3.0 - 2.0 * d)).powi(2);
            out.extend_from_slice(&[r, g, b, (a as f32 * fall) as u8]);
        }
    }
    out
}

/// A ramp laid along a strip: across if `across`, else down. Stretched over a box, a gradient.
pub fn gradient(ramp: &Ramp, width: u32, height: u32, across: bool) -> Vec<u8> {
    let (width, height) = (width.max(1), height.max(1));
    let lut = ramp.lut();
    let mut out = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let at = match across {
                true => x as f32 / (width - 1).max(1) as f32,
                false => y as f32 / (height - 1).max(1) as f32,
            };
            out.extend_from_slice(&lut.at(at));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_light_is_brightest_in_the_middle_and_gone_at_the_corner() {
        let light = falloff(32, Color::rgb(1.0, 0.5, 0.2));
        let alpha = |x: u32, y: u32| light[((y * 32 + x) * 4 + 3) as usize];
        assert!(alpha(16, 16) > 240);
        assert_eq!(alpha(0, 0), 0);
        // The colour never varies, only the alpha.
        assert!(light.chunks(4).all(|px| px[0..3] == [255, 128, 51]));
    }

    #[test]
    fn a_shadow_is_solid_at_its_core() {
        let shade = shadow(40, 20, 0.5, Color::rgba(0.0, 0.0, 0.0, 0.5));
        assert_eq!(shade[((10 * 40 + 20) * 4 + 3) as usize], 128);
        assert_eq!(shade[3], 0);
    }

    #[test]
    fn a_gradient_runs_end_to_end() {
        let ramp = Ramp::new(&[Color::rgb(0.0, 0.0, 0.0), Color::rgb(1.0, 1.0, 1.0)]);
        let strip = gradient(&ramp, 8, 1, true);
        assert_eq!(strip[0..4], [0, 0, 0, 255]);
        assert_eq!(strip[28..32], [255, 255, 255, 255]);
    }
}
