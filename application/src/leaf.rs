//! The leaf the site was sketched as: its outline, and where its three tips stand and what each
//! cuts. What a leaf does with that is [`Specimen`](crate::specimen::Specimen)'s.

use lichen::{Cut, Silhouette};

use crate::theme;

/// The outline, clockwise from the tip, in the leaf's own unit space, standing: its tip at the
/// top and its stem at the bottom. It was drawn lying on its side, stem out to the right, and
/// stood up a quarter turn.
///
/// Closed implicitly: the last vertex joins the first. The stem is part of it rather than a second
/// shape, so it is dashed, filled and measured with everything else.
#[rustfmt::skip]
const OUTLINE: [(f32, f32); 75] = [
    (1.000, 0.708), (0.948, 0.773), (0.875, 0.839), (0.789, 0.904),
    (0.733, 0.953), (0.709, 0.978), (0.666, 0.989), (0.703, 1.007),
    (0.706, 1.042), (0.647, 1.068), (0.592, 1.088), (0.531, 1.104),
    (0.475, 1.117), (0.432, 1.125), (0.412, 1.232), (0.380, 1.346),
    (0.334, 1.437), (0.279, 1.491), (0.242, 1.504), (0.230, 1.477),
    (0.273, 1.445), (0.322, 1.379), (0.359, 1.265), (0.383, 1.150),
    (0.387, 1.129), (0.328, 1.114), (0.267, 1.094), (0.208, 1.068),
    (0.166, 1.045), (0.154, 1.002), (0.174, 0.970), (0.129, 0.934),
    (0.107, 0.888), (0.066, 0.826), (0.031, 0.760), (0.006, 0.708),
    (0.000, 0.655), (0.011, 0.593), (0.018, 0.531), (0.031, 0.498),
    (0.088, 0.508), (0.141, 0.514), (0.187, 0.505), (0.230, 0.521),
    (0.259, 0.478), (0.273, 0.413), (0.291, 0.351), (0.313, 0.285),
    (0.338, 0.216), (0.362, 0.151), (0.389, 0.085), (0.410, 0.039),
    (0.436, 0.007), (0.485, 0.000), (0.531, 0.013), (0.559, 0.039),
    (0.584, 0.072), (0.617, 0.085), (0.657, 0.115), (0.684, 0.154),
    (0.709, 0.213), (0.742, 0.274), (0.775, 0.324), (0.797, 0.351),
    (0.785, 0.413), (0.768, 0.486), (0.742, 0.557), (0.807, 0.544),
    (0.862, 0.549), (0.918, 0.570), (0.903, 0.596), (0.942, 0.616),
    (0.924, 0.642), (0.961, 0.662), (0.979, 0.685),
];

/// How far apart tile centres sit, as a fraction of the leaf's width. Fine enough that a tip's
/// square holds enough tiles to read as a gradient, coarse enough that the whole leaf is a few
/// hundred polygons -- which a browser draws as readily as a desktop does.
pub(crate) const PITCH: f32 = 0.040;

/// How large a tip's square is, as a fraction of the leaf's width.
pub(crate) const SQUARE: f32 = 0.16;

/// One tip, as the sketch drew it.
pub(crate) struct Tip {
    /// Where on the leaf it stands, in its unit space.
    pub(crate) at: (f32, f32),
    /// What choosing it cuts the leaf to: the line, and the tip's own side of it -- which is the
    /// tip itself.
    pub(crate) cut: Cut,
    /// Where its section's controls stand, in the leaf's unit space: a box's top-left corner and
    /// its size. In the room the cut empties, with a little of the leaf still under it.
    pub(crate) controls: ((f32, f32), (f32, f32)),
}

/// The three tips, in the order the site names them: links, showcase, herbarium.
///
/// Each stands where one of the three prongs the leaf used to link from stood, and each cut keeps
/// the lobe its tip is on and empties the rest for the controls:
///
/// - links keeps the blade's point, above a line across its widest part, and its controls stand
///   in the body of the blade below.
/// - showcase keeps the right lobe, right of a line leaning down through the middle, and its
///   controls stand in the left half of the body.
/// - herbarium keeps the left lobe's foot and the stem, below a line from the left edge down to
///   the right of the stem's foot. It opens no section beside the leaf -- the cut is the way into
///   the herbarium's page -- so its controls' box holds nothing, and stands across the middle of
///   the blade like the others'.
pub(crate) const TIPS: [Tip; 3] = [
    Tip {
        at: (0.593, 0.301),
        cut: Cut::new((0.000, 0.500), (1.000, 0.400), (0.593, 0.301)),
        controls: ((0.12, 0.58), (0.60, 0.40)),
    },
    Tip {
        at: (0.770, 0.708),
        cut: Cut::new((0.681, 0.419), (0.500, 0.999), (0.770, 0.708)),
        controls: ((0.04, 0.50), (0.44, 0.50)),
    },
    Tip {
        at: (0.316, 0.921),
        cut: Cut::new((0.000, 0.699), (0.749, 1.020), (0.316, 0.921)),
        controls: ((0.16, 0.22), (0.58, 0.46)),
    },
];

/// The leaf as lichen's specimen draws it, with `tips` -- this sketch's, brought into unit space --
/// on it.
pub(crate) fn sketch(tips: &[lichen::Tip]) -> lichen::Sketch<'_> {
    lichen::Sketch {
        outline: &OUTLINE,
        // Mostly down the leaf, a little across it: gold at the tip, deepening toward the stem,
        // and the across term is what keeps the bands from reading as stripes.
        along: (0.25, 0.75),
        edge: Some(theme::EDGE),
        pitch: PITCH,
        tip: SQUARE,
        tips,
    }
}

/// The leaf, brought into unit space: `x` over `0.0..1.0`, `y` over `0.0..` its aspect, standing
/// with its tip at the top and its stem at the bottom.
pub(crate) fn shape() -> Silhouette {
    Silhouette::new(&OUTLINE)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Standing: half again as tall as it is wide.
    #[test]
    fn stands() {
        let aspect = shape().aspect();
        assert!((1.45..1.55).contains(&aspect), "{aspect}");
    }

    /// Every tip stands on the leaf, and every cut crosses it and keeps its own tip.
    #[test]
    fn every_cut_keeps_its_tip() {
        let shape = shape();
        for tip in &TIPS {
            let (x, y) = tip.at;
            assert!((0.0..=1.0).contains(&x) && (0.0..=shape.aspect()).contains(&y));
            assert!(
                shape.crossings(tip.cut).is_some(),
                "the cut misses the leaf"
            );
            // The tip is on the side the cut keeps, and the side it cuts away faces from it.
            let (from, to) = (tip.cut.from, tip.cut.to);
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
