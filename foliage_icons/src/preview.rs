//! Renders a baked `.icon` MTSDF to a PNG at an arbitrary on-screen size, reproducing the
//! shader's sampling exactly (bilinear texel fetch, median of R/G/B, screen-px-range AA), so
//! field quality can be judged without running an application.

use std::fs;
use std::path::Path;

pub fn render(icon: &Path, size: u32, px_range: f32, out: &Path) -> Result<(), String> {
    let rgba = fs::read(icon).map_err(|e| format!("reading {}: {e}", icon.display()))?;
    let texels = rgba.len() / 4;
    let field = (texels as f64).sqrt() as usize;
    if field * field * 4 != rgba.len() {
        return Err(format!(
            "{} is not a square RGBA field ({} bytes)",
            icon.display(),
            rgba.len()
        ));
    }

    // Same conversion the shader performs: how many screen pixels the field's distance
    // spread covers at this on-screen size.
    let screen_px_range = (size as f32 / field as f32 * px_range).max(1.0);

    let sample = |tx: f32, ty: f32, ch: usize| -> f32 {
        // Bilinear fetch at texture coordinate (tx, ty) in [0, 1], clamp-to-edge.
        let (fx, fy) = (tx * field as f32 - 0.5, ty * field as f32 - 0.5);
        let (x0, y0) = (fx.floor(), fy.floor());
        let (ax, ay) = (fx - x0, fy - y0);
        let clamp = |v: f32| (v.max(0.0) as usize).min(field - 1);
        let (x0i, x1i, y0i, y1i) = (clamp(x0), clamp(x0 + 1.0), clamp(y0), clamp(y0 + 1.0));
        let at = |x: usize, y: usize| rgba[(y * field + x) * 4 + ch] as f32 / 255.0;
        let top = at(x0i, y0i) * (1.0 - ax) + at(x1i, y0i) * ax;
        let bot = at(x0i, y1i) * (1.0 - ax) + at(x1i, y1i) * ax;
        top * (1.0 - ay) + bot * ay
    };

    let mut img = image::GrayImage::new(size, size);
    for y in 0..size {
        for x in 0..size {
            let (tx, ty) = (
                (x as f32 + 0.5) / size as f32,
                (y as f32 + 0.5) / size as f32,
            );
            let (r, g, b) = (sample(tx, ty, 0), sample(tx, ty, 1), sample(tx, ty, 2));
            let median = r.max(g.min(b)).min(r.min(g).max(b));
            let coverage = ((median - 0.5) * screen_px_range + 0.5).clamp(0.0, 1.0);
            // Black glyph on white -- artifacts read as light cracks inside the strokes.
            img.put_pixel(x, y, image::Luma([255 - (coverage * 255.0).round() as u8]));
        }
    }
    img.save(out)
        .map_err(|e| format!("writing {}: {e}", out.display()))?;
    println!("{} @ {size}px -> {}", icon.display(), out.display());
    Ok(())
}
