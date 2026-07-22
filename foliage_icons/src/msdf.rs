//! SVG -> MTSDF (multi-channel + true signed distance field) bake, replacing the old
//! fixed-size coverage rasterizer. Produces one resolution-independent RGBA field per icon:
//! RGB is the multi-channel SDF (median reconstructs sharp corners at any scale), A is a plain
//! SDF. Feather-style icons are stroke-only (`fill="none" stroke=...`), so strokes are outlined
//! into fill contours before generation -- otherwise they'd bake empty.
//!
//! All geometry is boolean-unioned into non-overlapping contours before the field is
//! generated. MSDF generation assumes a clean outline: edges buried inside an overlap (a
//! stroke crossing another, a cap meeting a line) otherwise still contribute colored
//! distances and drag the median below 0.5 along the seam, which renders as a crack at every
//! intersection. This is the same preprocessing msdfgen performs via Skia's `Simplify`.

use fdsm::bezier::scanline::FillRule;
use fdsm::bezier::{Point, Segment};
use fdsm::correct_error::{ErrorCorrectionConfig, correct_error_mtsdf};
use fdsm::generate::generate_mtsdf;
use fdsm::render::correct_sign_mtsdf;
use fdsm::shape::{Contour, Shape};
use fdsm::transform::Transform as _;
use i_overlay::core::fill_rule::FillRule as UnionFillRule;
use i_overlay::float::simplify::SimplifyShape;
use image::Rgba32FImage;
use nalgebra::{Affine2, Similarity2, Vector2};
use usvg::tiny_skia_path::{Path as SkPath, PathSegment, Point as SkPoint};

/// Flattening tolerance as a fraction of the icon's extent. Paired with `CORNER_SIN_ALPHA`:
/// chords this close to their source curve turn by less than the corner threshold at any
/// feature the field can resolve, so flattening vertices stay "smooth" (one edge color)
/// while genuine corners still split colors.
const FLATTEN_TOLERANCE: f64 = 5e-5;

/// Sine of the sharpest turn that is *not* a corner during edge coloring (~8.6 degrees,
/// msdfgen's own default of sin(3 rad) is ~0.141).
const CORNER_SIN_ALPHA: f64 = 0.15;

pub struct MsdfField {
    pub size: u32,
    pub px_range: f32,
    /// `size * size * 4` RGBA bytes.
    pub rgba: Vec<u8>,
}

/// Bakes `svg_bytes` into a square `field_size`×`field_size` MTSDF with a `px_range`-texel
/// distance spread (and a matching margin so the field isn't clipped at the icon's own edges).
pub fn generate_field(svg_bytes: &[u8], field_size: u32, px_range: f64) -> Result<MsdfField, String> {
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_data(svg_bytes, &opt).map_err(|e| e.to_string())?;

    // Flatten every fill + stroke-outline into absolute-user-space paths, split by fill rule.
    let mut nonzero: Vec<SkPath> = Vec::new();
    let mut evenodd: Vec<SkPath> = Vec::new();
    collect(tree.root(), &mut nonzero, &mut evenodd);
    if nonzero.is_empty() && evenodd.is_empty() {
        return Err("no fill or stroke geometry to bake".to_string());
    }

    // Rough (control-point) extent is plenty accurate for picking a flattening tolerance.
    let (mut min_x, mut min_y, mut max_x, mut max_y) =
        (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for p in nonzero.iter().chain(&evenodd) {
        let b = p.bounds();
        min_x = min_x.min(b.left());
        min_y = min_y.min(b.top());
        max_x = max_x.max(b.right());
        max_y = max_y.max(b.bottom());
    }
    let rough_extent = ((max_x - min_x).max(max_y - min_y) as f64).max(0.0);
    let tolerance = (rough_extent * FLATTEN_TOLERANCE).max(1e-9);

    // Flatten to polylines, normalize even-odd fills to non-overlapping nonzero contours,
    // then union everything into one clean outline (holes wound opposite to outers).
    let mut contours: Vec<Vec<[f64; 2]>> = Vec::new();
    for p in &nonzero {
        flatten_path(p, tolerance, &mut contours);
    }
    for p in &evenodd {
        let mut alone: Vec<Vec<[f64; 2]>> = Vec::new();
        flatten_path(p, tolerance, &mut alone);
        for shape in alone.simplify_shape(UnionFillRule::EvenOdd) {
            contours.extend(shape);
        }
    }
    let unioned = contours.simplify_shape(UnionFillRule::NonZero);

    // Exact bounds of the unioned geometry -- not the viewBox, so a stroke that overshoots
    // the viewBox still fits.
    let (mut min_x, mut min_y, mut max_x, mut max_y) =
        (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for [x, y] in unioned.iter().flatten().flatten() {
        min_x = min_x.min(*x);
        min_y = min_y.min(*y);
        max_x = max_x.max(*x);
        max_y = max_y.max(*y);
    }
    if unioned.iter().flatten().next().is_none() {
        return Err("geometry produced no contours".to_string());
    }
    let (bw, bh) = (max_x - min_x, max_y - min_y);

    // Uniform scale that fits the longer side into the usable interior (field minus a
    // `px_range` margin on each edge), then centered.
    let usable = field_size as f64 - 2.0 * px_range;
    let extent = bw.max(bh);
    let scale = if extent > 0.0 { usable / extent } else { 1.0 };
    let tx = px_range + (usable - bw * scale) * 0.5 - min_x * scale;
    let ty = px_range + (usable - bh * scale) * 0.5 - min_y * scale;
    let transformation = nalgebra::convert::<_, Affine2<f64>>(Similarity2::new(
        Vector2::new(tx, ty),
        0.0,
        scale,
    ));

    // Build the fdsm shape from the unioned polylines, then pre-transform its control points
    // to pixel space (fdsm expects coordinates already in distance-field texels).
    let mut shape: Shape<Contour> = Shape::default();
    for contour in unioned.iter().flatten() {
        if contour.len() < 3 {
            continue;
        }
        let mut c = Contour::default();
        for pair in contour.windows(2) {
            c.segments.push(Segment::line(cvt(pair[0]), cvt(pair[1])));
        }
        c.segments
            .push(Segment::line(cvt(*contour.last().unwrap()), cvt(contour[0])));
        shape.contours.push(c);
    }
    if shape.contours.is_empty() {
        return Err("geometry produced no contours".to_string());
    }
    shape.transform(&transformation);

    // Color edges (median can then reconstruct sharp corners), prepare, generate, fix sign.
    // Generated as a *float* field because the error-correction pass below needs it.
    let colored = Shape::edge_coloring_simple(shape, CORNER_SIN_ALPHA, 69_441_337_420);
    let prepared = colored.prepare();
    let mut img = Rgba32FImage::new(field_size, field_size);
    generate_mtsdf(&prepared, px_range, &mut img);
    correct_sign_mtsdf(&mut img, &prepared, FillRule::Nonzero);
    // Error correction: predicts and fixes the median "clashes" that otherwise show up as
    // spurious dark dots at corners and stroke caps -- the standard `msdfgen` pass, which the
    // raw generate omits.
    correct_error_mtsdf(
        &mut img,
        &colored,
        &prepared,
        px_range,
        &ErrorCorrectionConfig::default(),
    );

    // Quantize the float field to 8-bit RGBA (0.5 -> 128; the shader medians R/G/B).
    let rgba = img
        .into_raw()
        .into_iter()
        .map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8)
        .collect();
    Ok(MsdfField {
        size: field_size,
        px_range: px_range as f32,
        rgba,
    })
}

/// Depth-first over the usvg tree, pushing each path's fill geometry and its stroke *outline*
/// (strokes converted to fillable contours) in absolute user coordinates. Even-odd fills are
/// kept apart so they can be normalized under their own rule before the nonzero union.
fn collect(group: &usvg::Group, nonzero: &mut Vec<SkPath>, evenodd: &mut Vec<SkPath>) {
    for node in group.children() {
        match node {
            usvg::Node::Group(g) => collect(g, nonzero, evenodd),
            usvg::Node::Path(p) => {
                let ts = p.abs_transform();
                let data = p.data();
                if let Some(fill) = p.fill() {
                    if let Some(t) = data.clone().transform(ts) {
                        match fill.rule() {
                            usvg::FillRule::NonZero => nonzero.push(t),
                            usvg::FillRule::EvenOdd => evenodd.push(t),
                        }
                    }
                }
                if let Some(stroke) = p.stroke() {
                    if let Some(outline) = data.stroke(&stroke.to_tiny_skia(), 1.0) {
                        if let Some(t) = outline.transform(ts) {
                            nonzero.push(t);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// Flattens a tiny_skia path into polyline contours within `tolerance` of the true curves.
fn flatten_path(path: &SkPath, tolerance: f64, out: &mut Vec<Vec<[f64; 2]>>) {
    let mut contour: Vec<[f64; 2]> = Vec::new();
    for seg in path.segments() {
        match seg {
            PathSegment::MoveTo(p) => {
                if contour.len() >= 3 {
                    out.push(std::mem::take(&mut contour));
                } else {
                    contour.clear();
                }
                contour.push(pt(p));
            }
            PathSegment::LineTo(p) => contour.push(pt(p)),
            PathSegment::QuadTo(c, p) => {
                let last = *contour.last().unwrap_or(&pt(p));
                flatten_quad(last, pt(c), pt(p), tolerance, 0, &mut contour);
            }
            PathSegment::CubicTo(c1, c2, p) => {
                let last = *contour.last().unwrap_or(&pt(p));
                flatten_cubic(last, pt(c1), pt(c2), pt(p), tolerance, 0, &mut contour);
            }
            PathSegment::Close => {
                if contour.len() >= 3 {
                    out.push(std::mem::take(&mut contour));
                } else {
                    contour.clear();
                }
            }
        }
    }
    if contour.len() >= 3 {
        out.push(contour);
    }
}

const MAX_FLATTEN_DEPTH: u32 = 12;

fn flatten_quad(p0: [f64; 2], c: [f64; 2], p1: [f64; 2], tol: f64, depth: u32, out: &mut Vec<[f64; 2]>) {
    if depth >= MAX_FLATTEN_DEPTH || dist_to_chord(c, p0, p1) <= tol {
        out.push(p1);
        return;
    }
    let a = mid(p0, c);
    let b = mid(c, p1);
    let m = mid(a, b);
    flatten_quad(p0, a, m, tol, depth + 1, out);
    flatten_quad(m, b, p1, tol, depth + 1, out);
}

fn flatten_cubic(
    p0: [f64; 2],
    c1: [f64; 2],
    c2: [f64; 2],
    p1: [f64; 2],
    tol: f64,
    depth: u32,
    out: &mut Vec<[f64; 2]>,
) {
    if depth >= MAX_FLATTEN_DEPTH
        || (dist_to_chord(c1, p0, p1) <= tol && dist_to_chord(c2, p0, p1) <= tol)
    {
        out.push(p1);
        return;
    }
    let a1 = mid(p0, c1);
    let a2 = mid(c1, c2);
    let a3 = mid(c2, p1);
    let b1 = mid(a1, a2);
    let b2 = mid(a2, a3);
    let m = mid(b1, b2);
    flatten_cubic(p0, a1, b1, m, tol, depth + 1, out);
    flatten_cubic(m, b2, a3, p1, tol, depth + 1, out);
}

fn mid(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5]
}

/// Distance from `p` to the segment `a`-`b`.
fn dist_to_chord(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len_sq = dx * dx + dy * dy;
    let (px, py) = (p[0] - a[0], p[1] - a[1]);
    if len_sq <= f64::EPSILON {
        return (px * px + py * py).sqrt();
    }
    let t = ((px * dx + py * dy) / len_sq).clamp(0.0, 1.0);
    let (ex, ey) = (px - t * dx, py - t * dy);
    (ex * ex + ey * ey).sqrt()
}

fn pt(p: SkPoint) -> [f64; 2] {
    [p.x as f64, p.y as f64]
}

fn cvt(p: [f64; 2]) -> Point {
    Point::new(p[0], p[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stroke-only SVG (the Feather shape: `fill="none" stroke=...`) must bake to a
    /// non-degenerate field -- strokes get outlined into fill contours, so there are both
    /// clearly-inside (median > 0.5) and clearly-outside (median < 0.5) texels. A regression
    /// where strokes were dropped would produce an all-outside (empty) field.
    #[test]
    fn a_stroke_only_svg_bakes_a_non_empty_field() {
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24"
            viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
            stroke-linecap="round" stroke-linejoin="round">
            <rect x="4" y="4" width="16" height="16"></rect></svg>"#;
        let field = generate_field(svg, 48, 3.0).expect("bake");
        assert_eq!(field.rgba.len(), 48 * 48 * 4);

        let (mut inside, mut outside) = (0, 0);
        for px in field.rgba.chunks_exact(4) {
            let mut rgb = [px[0], px[1], px[2]];
            rgb.sort_unstable();
            let median = rgb[1];
            if median > 140 {
                inside += 1;
            } else if median < 115 {
                outside += 1;
            }
        }
        assert!(inside > 0, "no inside texels -- the stroke was not outlined into a fill");
        assert!(outside > 0, "no outside texels -- the field is degenerate");
    }

    /// Two crossing strokes must union into one outline: no interior edges survive at the
    /// intersection, so the median along the crossing stays clearly inside. The regression
    /// this guards against (overlapping contours fed straight to the generator) leaves a
    /// "chasm" -- median texels near 0.5 -- through the middle of every intersection.
    #[test]
    fn crossing_strokes_have_no_chasm_at_the_intersection() {
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24"
            viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="4"
            stroke-linecap="round" stroke-linejoin="round">
            <line x1="4" y1="4" x2="20" y2="20"></line>
            <line x1="20" y1="4" x2="4" y2="20"></line></svg>"#;
        let field = generate_field(svg, 48, 3.0).expect("bake");

        // The X center lands at the field center; the crossing is fully inside the union, so
        // the medians there must all be decisively inside (well above the 0.5 edge).
        for dy in -2i32..=2 {
            for dx in -2i32..=2 {
                let (x, y) = ((24 + dx) as usize, (24 + dy) as usize);
                let i = (y * 48 + x) * 4;
                let mut rgb = [field.rgba[i], field.rgba[i + 1], field.rgba[i + 2]];
                rgb.sort_unstable();
                assert!(
                    rgb[1] > 150,
                    "texel ({x},{y}) median {} -- chasm at the intersection",
                    rgb[1]
                );
            }
        }
    }
}
