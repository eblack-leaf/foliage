//! The leaf the site was sketched as: its outline, and the patches on it the ways off the page
//! light.

use lichen::{Mosaic, Ramp, Region, Silhouette};

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

/// How far apart tile centres sit, as a fraction of the leaf's width. Fine enough that a patch
/// holds enough tiles to read as a gradient, coarse enough that the whole leaf is a few hundred
/// polygons -- which a browser draws as readily as a desktop does.
const PITCH: f32 = 0.040;

/// Which way the ramp runs: mostly down the leaf, a little across it, so it is gold at the tip and
/// deepens toward the stem, and the across term keeps the bands from reading as stripes.
const ALONG: (f32, f32) = (0.25, 0.75);

/// How large a patch is, as a fraction of the leaf's width.
const PATCH: f32 = 0.26;

/// Where on the leaf each way off the page stands, in its unit space, in the order the label lists
/// them: down the leaf from the tip, so each sits in the stretch of the ramp its row is worn in.
/// The source is last, where the blade meets the stem -- what the rest grows from.
pub(crate) const PATCHES: [(f32, f32); 4] = [(0.50, 0.20), (0.22, 0.66), (0.80, 0.70), (0.45, 0.98)];

/// The leaf, ungrown, its tiles coloured from `ramp`, and its regions the size of a patch.
pub(crate) fn mosaic(ramp: Ramp) -> Mosaic {
    Mosaic::new(Silhouette::new(&OUTLINE), ramp)
        .along(ALONG)
        .edge(theme::EDGE)
        .pitch(PITCH)
        .square(PATCH)
}

/// The patches, as regions to stand on the grown leaf.
pub(crate) fn stood() -> [Region; 4] {
    PATCHES.map(Region::Point)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Whether `(x, y)` is inside the outline: an odd number of its edges cross a ray from it.
    fn inside((x, y): (f32, f32)) -> bool {
        let mut inside = false;
        for (n, &(x0, y0)) in OUTLINE.iter().enumerate() {
            let (x1, y1) = OUTLINE[(n + 1) % OUTLINE.len()];
            if (y0 > y) != (y1 > y) && x < x0 + (y - y0) / (y1 - y0) * (x1 - x0) {
                inside = !inside;
            }
        }
        inside
    }

    /// Standing: half again as tall as it is wide.
    #[test]
    fn stands() {
        let aspect = Silhouette::new(&OUTLINE).aspect();
        assert!((1.45..1.55).contains(&aspect), "{aspect}");
    }

    /// Every patch's middle is on the leaf, they run down it in order, and no two overlap.
    #[test]
    fn patches_run_down_the_leaf() {
        for &at in &PATCHES {
            assert!(inside(at), "{at:?} is off the leaf");
        }
        let along = |(x, y): (f32, f32)| ALONG.0 * x + ALONG.1 * y;
        for pair in PATCHES.windows(2) {
            assert!(along(pair[0]) < along(pair[1]), "{pair:?} run up the ramp");
        }
        for (n, a) in PATCHES.iter().enumerate() {
            for b in &PATCHES[n + 1..] {
                let apart = (a.0 - b.0).abs().max((a.1 - b.1).abs());
                assert!(apart >= PATCH, "{a:?} and {b:?} overlap");
            }
        }
    }
}
