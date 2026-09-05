//! SVG geometry into a multi-channel signed distance field.
//!
//! RGB is the multi-channel field, where a median across the three reconstructs a corner that a
//! single channel would round off; A is a plain signed distance. Icon sets are commonly
//! stroke-only (`fill="none" stroke=...`), so strokes are outlined into fill contours before
//! anything is generated -- otherwise they would bake empty.
//!
//! All geometry is boolean-unioned into non-overlapping contours first. Generation assumes a
//! clean outline: an edge buried inside an overlap -- a stroke crossing another, a cap meeting a
//! line -- still contributes a coloured distance and drags the median below the edge along the
//! seam, which draws as a crack at every intersection.

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

/// Flattening tolerance, as a fraction of the mark's extent.
///
/// Paired with [`CORNER_SIN_ALPHA`]: a chord this close to the curve it came from turns by less
/// than the corner threshold at any feature the field can resolve, so a vertex left by
/// flattening stays smooth while a genuine corner still splits colours.
const FLATTEN_TOLERANCE: f64 = 5e-5;

/// Sine of the sharpest turn that is not taken for a corner while colouring edges, about 8.6
/// degrees.
const CORNER_SIN_ALPHA: f64 = 0.15;

/// How deep a curve is subdivided before its chord is accepted regardless of tolerance.
const MAX_FLATTEN_DEPTH: u32 = 12;

/// How a field is baked: the two numbers a mark is registered with.
///
/// Both describe the bake rather than the artwork, and foliage is told them at registration
/// because a field that was fetched cannot be asked.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Bake {
    /// Texels per side of the square field.
    ///
    /// One field serves every on-screen size, so this is bake-generously territory: 32 to 64
    /// keeps a small mark crisp without making the file large.
    pub side: u32,
    /// How many texels the distance spread covers.
    ///
    /// What turns a sampled distance into an edge one screen pixel wide at whatever size the
    /// mark is drawn. Two to four balances sharpness at small sizes against smoothness at large.
    pub range: f32,
}

impl Bake {
    /// A square field `side` texels across, with a `range`-texel spread.
    pub fn new(side: u32, range: f32) -> Self {
        Self { side, range }
    }
}

/// One baked field, and what it was baked with.
#[derive(Clone, Debug)]
pub struct Baked {
    /// How it was baked.
    pub spec: Bake,
    /// `side` by `side` texels of RGBA, row-major -- exactly what foliage registers.
    pub rgba: Vec<u8>,
}

/// Bakes one SVG into a square field.
///
/// The artwork is fitted to the field's interior, leaving a margin of `range` texels on each
/// edge so the spread is not clipped at the mark's own boundary. Bounds are taken from the
/// unioned geometry rather than from the `viewBox`, so a stroke that overshoots still fits.
///
/// # Errors
///
/// If the SVG cannot be parsed, or holds no fill or stroke to bake.
pub fn bake(svg: &[u8], spec: Bake) -> Result<Baked, String> {
    let range = spec.range as f64;
    let tree = usvg::Tree::from_data(svg, &usvg::Options::default()).map_err(|e| e.to_string())?;

    // Every fill and stroke-outline, in absolute user space, split by fill rule.
    let mut nonzero: Vec<SkPath> = Vec::new();
    let mut evenodd: Vec<SkPath> = Vec::new();
    collect(tree.root(), &mut nonzero, &mut evenodd);
    if nonzero.is_empty() && evenodd.is_empty() {
        return Err("no fill or stroke geometry to bake".to_string());
    }

    // The control-point extent is accurate enough to pick a flattening tolerance from.
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for path in nonzero.iter().chain(&evenodd) {
        let bounds = path.bounds();
        min_x = min_x.min(bounds.left());
        min_y = min_y.min(bounds.top());
        max_x = max_x.max(bounds.right());
        max_y = max_y.max(bounds.bottom());
    }
    let rough = ((max_x - min_x).max(max_y - min_y) as f64).max(0.0);
    let tolerance = (rough * FLATTEN_TOLERANCE).max(1e-9);

    // Flatten to polylines, normalise even-odd fills to non-overlapping nonzero contours, then
    // union everything into one outline with holes wound against their outers.
    let mut contours: Vec<Vec<[f64; 2]>> = Vec::new();
    for path in &nonzero {
        flatten(path, tolerance, &mut contours);
    }
    for path in &evenodd {
        let mut alone: Vec<Vec<[f64; 2]>> = Vec::new();
        flatten(path, tolerance, &mut alone);
        for shape in alone.simplify_shape(UnionFillRule::EvenOdd) {
            contours.extend(shape);
        }
    }
    let unioned = contours.simplify_shape(UnionFillRule::NonZero);
    if unioned.iter().flatten().next().is_none() {
        return Err("geometry produced no contours".to_string());
    }

    let (mut min_x, mut min_y, mut max_x, mut max_y) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for [x, y] in unioned.iter().flatten().flatten() {
        min_x = min_x.min(*x);
        min_y = min_y.min(*y);
        max_x = max_x.max(*x);
        max_y = max_y.max(*y);
    }
    let (width, height) = (max_x - min_x, max_y - min_y);

    // A uniform scale fitting the longer side into the interior, then centred.
    let usable = spec.side as f64 - 2.0 * range;
    let extent = width.max(height);
    let scale = match extent > 0.0 {
        true => usable / extent,
        false => 1.0,
    };
    let x = range + (usable - width * scale) * 0.5 - min_x * scale;
    let y = range + (usable - height * scale) * 0.5 - min_y * scale;
    let transformation =
        nalgebra::convert::<_, Affine2<f64>>(Similarity2::new(Vector2::new(x, y), 0.0, scale));

    // The generator wants control points already in field texels, so the shape is transformed
    // before it is coloured.
    let mut shape: Shape<Contour> = Shape::default();
    for contour in unioned.iter().flatten() {
        if contour.len() < 3 {
            continue;
        }
        let mut built = Contour::default();
        for pair in contour.windows(2) {
            built.segments.push(Segment::line(point(pair[0]), point(pair[1])));
        }
        let last = *contour.last().expect("a contour of three or more");
        built
            .segments
            .push(Segment::line(point(last), point(contour[0])));
        shape.contours.push(built);
    }
    if shape.contours.is_empty() {
        return Err("geometry produced no contours".to_string());
    }
    shape.transform(&transformation);

    // Generated as a float field because the error-correction pass reads one.
    let coloured = Shape::edge_coloring_simple(shape, CORNER_SIN_ALPHA, 69_441_337_420);
    let prepared = coloured.prepare();
    let mut field = Rgba32FImage::new(spec.side, spec.side);
    generate_mtsdf(&prepared, range, &mut field);
    correct_sign_mtsdf(&mut field, &prepared, FillRule::Nonzero);
    // Predicts and repairs the median clashes that otherwise draw as dark dots at corners and
    // stroke caps, which the raw generate leaves in.
    correct_error_mtsdf(
        &mut field,
        &coloured,
        &prepared,
        range,
        &ErrorCorrectionConfig::default(),
    );

    let rgba = field
        .into_raw()
        .into_iter()
        .map(|channel| (channel.clamp(0.0, 1.0) * 255.0).round() as u8)
        .collect();
    Ok(Baked { spec, rgba })
}

/// Walks the tree, pushing each path's fill geometry and its stroke outline in absolute user
/// coordinates.
///
/// Even-odd fills are kept apart so they can be normalised under their own rule before the
/// nonzero union.
fn collect(group: &usvg::Group, nonzero: &mut Vec<SkPath>, evenodd: &mut Vec<SkPath>) {
    for node in group.children() {
        match node {
            usvg::Node::Group(group) => collect(group, nonzero, evenodd),
            usvg::Node::Path(path) => {
                let transform = path.abs_transform();
                let data = path.data();
                if let Some(fill) = path.fill()
                    && let Some(placed) = data.clone().transform(transform)
                {
                    match fill.rule() {
                        usvg::FillRule::NonZero => nonzero.push(placed),
                        usvg::FillRule::EvenOdd => evenodd.push(placed),
                    }
                }
                if let Some(stroke) = path.stroke()
                    && let Some(outline) = data.stroke(&stroke.to_tiny_skia(), 1.0)
                    && let Some(placed) = outline.transform(transform)
                {
                    nonzero.push(placed);
                }
            }
            _ => {}
        }
    }
}

/// Flattens a path into polyline contours within `tolerance` of the curves they came from.
fn flatten(path: &SkPath, tolerance: f64, out: &mut Vec<Vec<[f64; 2]>>) {
    let mut contour: Vec<[f64; 2]> = Vec::new();
    let close = |contour: &mut Vec<[f64; 2]>, out: &mut Vec<Vec<[f64; 2]>>| {
        match contour.len() >= 3 {
            true => out.push(std::mem::take(contour)),
            false => contour.clear(),
        }
    };
    for segment in path.segments() {
        match segment {
            PathSegment::MoveTo(to) => {
                close(&mut contour, out);
                contour.push(at(to));
            }
            PathSegment::LineTo(to) => contour.push(at(to)),
            PathSegment::QuadTo(control, to) => {
                let from = *contour.last().unwrap_or(&at(to));
                quad(from, at(control), at(to), tolerance, 0, &mut contour);
            }
            PathSegment::CubicTo(first, second, to) => {
                let from = *contour.last().unwrap_or(&at(to));
                cubic(
                    from,
                    at(first),
                    at(second),
                    at(to),
                    tolerance,
                    0,
                    &mut contour,
                );
            }
            PathSegment::Close => close(&mut contour, out),
        }
    }
    if contour.len() >= 3 {
        out.push(contour);
    }
}

/// Subdivides a quadratic until its chord is within `tolerance`.
fn quad(from: [f64; 2], control: [f64; 2], to: [f64; 2], tolerance: f64, depth: u32, out: &mut Vec<[f64; 2]>) {
    if depth >= MAX_FLATTEN_DEPTH || chord(control, from, to) <= tolerance {
        out.push(to);
        return;
    }
    let first = mid(from, control);
    let second = mid(control, to);
    let middle = mid(first, second);
    quad(from, first, middle, tolerance, depth + 1, out);
    quad(middle, second, to, tolerance, depth + 1, out);
}

/// Subdivides a cubic until both its control points are within `tolerance` of its chord.
fn cubic(
    from: [f64; 2],
    first: [f64; 2],
    second: [f64; 2],
    to: [f64; 2],
    tolerance: f64,
    depth: u32,
    out: &mut Vec<[f64; 2]>,
) {
    if depth >= MAX_FLATTEN_DEPTH
        || (chord(first, from, to) <= tolerance && chord(second, from, to) <= tolerance)
    {
        out.push(to);
        return;
    }
    let (a, b, c) = (mid(from, first), mid(first, second), mid(second, to));
    let (d, e) = (mid(a, b), mid(b, c));
    let middle = mid(d, e);
    cubic(from, a, d, middle, tolerance, depth + 1, out);
    cubic(middle, e, c, to, tolerance, depth + 1, out);
}

/// The midpoint of two points.
fn mid(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5]
}

/// The distance from a point to the segment between two others.
fn chord(point: [f64; 2], from: [f64; 2], to: [f64; 2]) -> f64 {
    let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
    let length = dx * dx + dy * dy;
    let (px, py) = (point[0] - from[0], point[1] - from[1]);
    if length <= f64::EPSILON {
        return (px * px + py * py).sqrt();
    }
    let along = ((px * dx + py * dy) / length).clamp(0.0, 1.0);
    let (ex, ey) = (px - along * dx, py - along * dy);
    (ex * ex + ey * ey).sqrt()
}

/// A path point as a pair.
fn at(point: SkPoint) -> [f64; 2] {
    [point.x as f64, point.y as f64]
}

/// A pair as a generator point.
fn point(pair: [f64; 2]) -> Point {
    Point::new(pair[0], pair[1])
}

#[cfg(test)]
mod tests {
    use super::{Bake, bake};

    /// The median of a texel's three colour channels, which is what the shader reconstructs.
    fn median(texel: &[u8]) -> u8 {
        let mut channels = [texel[0], texel[1], texel[2]];
        channels.sort_unstable();
        channels[1]
    }

    /// An icon set is commonly stroke-only, so a stroke has to be outlined into a fill before
    /// anything is generated. Dropped strokes bake an all-outside field, which draws nothing.
    #[test]
    fn a_stroke_only_svg_bakes_a_field_with_an_inside() {
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24"
            viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
            stroke-linecap="round" stroke-linejoin="round">
            <rect x="4" y="4" width="16" height="16"></rect></svg>"#;
        let baked = bake(svg, Bake::new(48, 3.0)).expect("bake");
        assert_eq!(baked.rgba.len(), 48 * 48 * 4);

        let (mut inside, mut outside) = (0, 0);
        for texel in baked.rgba.chunks_exact(4) {
            match median(texel) {
                m if m > 140 => inside += 1,
                m if m < 115 => outside += 1,
                _ => {}
            }
        }
        assert!(inside > 0, "no inside texels -- the stroke was not outlined");
        assert!(outside > 0, "no outside texels -- the field is degenerate");
    }

    /// Two crossing strokes union into one outline, so no interior edge survives the
    /// intersection. Overlapping contours fed straight to the generator leave a chasm -- medians
    /// near the edge value -- through the middle of every crossing.
    #[test]
    fn crossing_strokes_have_no_chasm_at_the_intersection() {
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24"
            viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="4"
            stroke-linecap="round" stroke-linejoin="round">
            <line x1="4" y1="4" x2="20" y2="20"></line>
            <line x1="20" y1="4" x2="4" y2="20"></line></svg>"#;
        let baked = bake(svg, Bake::new(48, 3.0)).expect("bake");

        // The crossing lands at the field's centre and is wholly inside the union.
        for dy in -2i32..=2 {
            for dx in -2i32..=2 {
                let (x, y) = ((24 + dx) as usize, (24 + dy) as usize);
                let texel = &baked.rgba[(y * 48 + x) * 4..];
                assert!(
                    median(texel) > 150,
                    "texel ({x},{y}) median {} -- chasm at the intersection",
                    median(texel)
                );
            }
        }
    }

    /// An SVG with nothing to bake is an error rather than an empty field, because an empty
    /// field is indistinguishable from a mark that simply draws nothing.
    #[test]
    fn an_svg_with_no_geometry_is_refused() {
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"></svg>"#;
        assert!(bake(svg, Bake::new(48, 3.0)).is_err());
    }
}
