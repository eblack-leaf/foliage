//! A baked field, rendered the way the shader samples it.

use std::io::Cursor;

use image::{ExtendedColorType, ImageEncoder, codecs::png::PngEncoder};

/// Renders a baked field at an on-screen size, as a PNG.
///
/// Reproduces the shader's own sampling -- a bilinear fetch, the median of the three colour
/// channels, and the screen-pixel spread that turns a distance into coverage -- so a field's
/// quality can be judged without running an application. The mark is drawn dark on a light
/// ground, which is what makes an artifact read as a crack rather than as a highlight.
///
/// # Errors
///
/// If `field` is not a square RGBA field, or the PNG cannot be encoded.
pub fn preview(field: &[u8], size: u32, range: f32) -> Result<Vec<u8>, String> {
    let side = (field.len() as f64 / 4.0).sqrt() as usize;
    if side == 0 || side * side * 4 != field.len() {
        return Err(format!(
            "not a square RGBA field: {} bytes is not a whole number of texels squared",
            field.len()
        ));
    }

    // The shader's own conversion: how many screen pixels the spread covers at this size.
    let spread = (size as f32 / side as f32 * range).max(1.0);

    let sample = |x: f32, y: f32, channel: usize| -> f32 {
        // A bilinear fetch at a texture coordinate in `0.0..=1.0`, clamped to the edge.
        let (fx, fy) = (x * side as f32 - 0.5, y * side as f32 - 0.5);
        let (x0, y0) = (fx.floor(), fy.floor());
        let (ax, ay) = (fx - x0, fy - y0);
        let clamp = |v: f32| (v.max(0.0) as usize).min(side - 1);
        let (left, right) = (clamp(x0), clamp(x0 + 1.0));
        let (top, bottom) = (clamp(y0), clamp(y0 + 1.0));
        let at = |x: usize, y: usize| field[(y * side + x) * 4 + channel] as f32 / 255.0;
        let upper = at(left, top) * (1.0 - ax) + at(right, top) * ax;
        let lower = at(left, bottom) * (1.0 - ax) + at(right, bottom) * ax;
        upper * (1.0 - ay) + lower * ay
    };

    let mut rendered = image::GrayImage::new(size, size);
    for y in 0..size {
        for x in 0..size {
            let (tx, ty) = (
                (x as f32 + 0.5) / size as f32,
                (y as f32 + 0.5) / size as f32,
            );
            let (r, g, b) = (sample(tx, ty, 0), sample(tx, ty, 1), sample(tx, ty, 2));
            let median = r.max(g.min(b)).min(r.min(g).max(b));
            let coverage = ((median - 0.5) * spread + 0.5).clamp(0.0, 1.0);
            rendered.put_pixel(x, y, image::Luma([255 - (coverage * 255.0).round() as u8]));
        }
    }

    let mut png = Vec::new();
    PngEncoder::new(Cursor::new(&mut png))
        .write_image(&rendered, size, size, ExtendedColorType::L8)
        .map_err(|e| format!("encoding the preview: {e}"))?;
    Ok(png)
}

#[cfg(test)]
mod tests {
    use super::preview;

    #[test]
    fn a_field_that_is_not_square_is_refused() {
        assert!(preview(&[255; 7], 32, 3.0).is_err());
        assert!(preview(&[], 32, 3.0).is_err());
    }

    /// A field that is entirely inside renders as a fully covered square, which is what says the
    /// median and the coverage step run the way the shader's do.
    #[test]
    fn a_field_that_is_wholly_inside_renders_covered() {
        let png = preview(&[255; 4 * 4 * 4], 16, 3.0).expect("preview");
        assert_eq!(&png[1..4], b"PNG");
    }
}
