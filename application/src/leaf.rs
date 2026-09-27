//! The leaf the site was sketched as: its outline, and where its three tips stand and what each
//! cuts. What a leaf does with that is [`Specimen`](crate::specimen::Specimen)'s.

use lichen::{Cut, Silhouette, Turn};

/// The page the outline was traced on.
const PAGE: (f32, f32) = (2000.0, 1500.0);

/// The outline, clockwise from the tip, as fractions of the page it was traced on.
///
/// Closed implicitly: the last vertex joins the first. The stem is part of it rather than a second
/// shape, so it is dashed, filled and measured with everything else.
///
/// Stated exactly as it was drawn -- lying on its side, stem out to the right -- and stood up by
/// [`Turn::Quarter`], so what was traced and which way it faces stay separable.
#[rustfmt::skip]
const OUTLINE: [(f32, f32); 75] = [
    (0.480, 0.073),
    (0.520, 0.115),
    (0.560, 0.175),
    (0.600, 0.245),
    (0.630, 0.290),
    (0.645, 0.310),
    (0.652, 0.345),
    (0.663, 0.315),
    (0.684, 0.312),
    (0.700, 0.360),
    (0.712, 0.405),
    (0.722, 0.455),
    (0.730, 0.500),
    (0.735, 0.535),
    (0.800, 0.552),
    (0.870, 0.578),
    (0.925, 0.615),
    (0.958, 0.660),
    (0.966, 0.690),
    (0.950, 0.700),
    (0.930, 0.665),
    (0.890, 0.625),
    (0.820, 0.595),
    (0.750, 0.575),
    (0.737, 0.572),
    (0.728, 0.620),
    (0.716, 0.670),
    (0.700, 0.718),
    (0.686, 0.752),
    (0.660, 0.762),
    (0.640, 0.745),
    (0.618, 0.782),
    (0.590, 0.800),
    (0.552, 0.833),
    (0.512, 0.862),
    (0.480, 0.882),
    (0.448, 0.887),
    (0.410, 0.878),
    (0.372, 0.872),
    (0.352, 0.862),
    (0.358, 0.815),
    (0.362, 0.772),
    (0.356, 0.735),
    (0.366, 0.700),
    (0.340, 0.676),
    (0.300, 0.665),
    (0.262, 0.650),
    (0.222, 0.632),
    (0.180, 0.612),
    (0.140, 0.592),
    (0.100, 0.570),
    (0.072, 0.553),
    (0.052, 0.532),
    (0.048, 0.492),
    (0.056, 0.455),
    (0.072, 0.432),
    (0.092, 0.412),
    (0.100, 0.385),
    (0.118, 0.352),
    (0.142, 0.330),
    (0.178, 0.310),
    (0.215, 0.283),
    (0.246, 0.256),
    (0.262, 0.238),
    (0.300, 0.248),
    (0.345, 0.262),
    (0.388, 0.283),
    (0.380, 0.230),
    (0.383, 0.185),
    (0.396, 0.140),
    (0.412, 0.152),
    (0.424, 0.120),
    (0.440, 0.135),
    (0.452, 0.105),
    (0.466, 0.090),
];

/// How far apart tile centres sit, as a fraction of the leaf's width. Fine enough that a tip's
/// square holds enough tiles to read as a gradient, coarse enough that the whole leaf is a few
/// hundred polygons -- which a browser draws as readily as a desktop does.
pub(crate) const PITCH: f32 = 0.040;

/// How large a tip's square is, as a fraction of the leaf's width.
pub(crate) const SQUARE: f32 = 0.16;

/// One tip, as the sketch drew it.
pub(crate) struct Tip {
    /// Where on the leaf it stands, as a point of the page the outline was traced on.
    pub(crate) at: (f32, f32),
    /// What choosing it cuts the leaf to: the line, and the tip's own side of it -- which is the
    /// tip itself.
    pub(crate) cut: Cut,
    /// Where its section's controls stand, in the leaf's unit space: a box's top-left corner and
    /// its size. In the room the cut empties, with a little of the leaf still under it.
    pub(crate) controls: ((f32, f32), (f32, f32)),
}

/// The three tips, in the order the site names them: links, showcase, internals.
///
/// Each stands where one of the three prongs the leaf used to link from stood, and each cut keeps
/// the lobe its tip is on and empties the rest for the controls:
///
/// - links keeps the blade's point, above a line across its widest part, and its controls stand
///   in the body of the blade below.
/// - showcase keeps the right lobe, right of a line leaning down through the middle, and its
///   controls stand in the left half of the body.
/// - internals keeps the left lobe's foot and the stem, below a line from the left edge down to
///   the right of the stem's foot, and its controls stand across the middle of the blade. The stem
///   is the internals', because what holds a leaf up is what the engine is to a page.
pub(crate) const TIPS: [Tip; 3] = [
    Tip {
        at: (0.232, 0.404),
        cut: Cut::new((0.353, 0.887), (0.292, 0.073), (0.232, 0.404)),
        controls: ((0.12, 0.58), (0.60, 0.40)),
    },
    Tip {
        at: (0.480, 0.260),
        cut: Cut::new((0.304, 0.333), (0.658, 0.480), (0.480, 0.260)),
        controls: ((0.04, 0.50), (0.44, 0.50)),
    },
    Tip {
        at: (0.610, 0.630),
        cut: Cut::new((0.475, 0.887), (0.671, 0.277), (0.610, 0.630)),
        controls: ((0.16, 0.22), (0.58, 0.46)),
    },
];

/// The leaf, brought into unit space: `x` over `0.0..1.0`, `y` over `0.0..` its aspect, standing
/// with its tip at the top and its stem at the bottom.
pub(crate) fn shape() -> Silhouette {
    Silhouette::traced(&OUTLINE, PAGE, Turn::Quarter)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The quarter turn lichen applies is the one the site's own tracing used to: a point `(x, y)`
    /// of the page stood up as `(-y, x)`, both axes scaled by the page first. Traced that way by
    /// hand, the leaf is as tall per unit of width as lichen makes it.
    #[test]
    fn stands_as_it_used_to() {
        let turned: Vec<(f32, f32)> = OUTLINE
            .iter()
            .map(|&(x, y)| (-y * PAGE.1, x * PAGE.0))
            .collect();
        let span = |axis: fn(&(f32, f32)) -> f32| {
            let low = turned.iter().map(axis).fold(f32::MAX, f32::min);
            let high = turned.iter().map(axis).fold(f32::MIN, f32::max);
            high - low
        };
        let aspect = span(|point| point.1) / span(|point| point.0);
        assert!((shape().aspect() - aspect).abs() < 1e-4);
    }

    /// Every tip stands on the leaf, and every cut crosses it and keeps its own tip.
    #[test]
    fn every_cut_keeps_its_tip() {
        let shape = shape();
        for tip in &TIPS {
            let (x, y) = shape.at(tip.at);
            assert!((0.0..=1.0).contains(&x) && (0.0..=shape.aspect()).contains(&y));
            assert!(
                shape.crossings(tip.cut).is_some(),
                "the cut misses the leaf"
            );
            // The tip is on the side the cut keeps, and the side it cuts away faces from it.
            let (from, to) = (shape.at(tip.cut.from), shape.at(tip.cut.to));
            let side = (to.0 - from.0) * (y - from.1) - (to.1 - from.1) * (x - from.0);
            assert!(side.abs() > 0.05, "the tip sits on its own cut");
            let (nx, ny) = shape.facing(tip.cut);
            assert!(nx * (x - from.0) + ny * (y - from.1) < 0.0);
            // Every corner of the controls' box is in the room the cut empties.
            let ((cx, cy), (width, height)) = tip.controls;
            for (px, py) in [
                (cx, cy),
                (cx + width, cy),
                (cx, cy + height),
                (cx + width, cy + height),
            ] {
                assert!(
                    nx * (px - from.0) + ny * (py - from.1) > 0.0,
                    "the controls cross the cut"
                );
            }
        }
    }
}
