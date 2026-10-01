//! The key the toy vault is drawn on: its outline, where its tips stand, what each cuts, and where
//! each one's controls stand on what it cleared.
//!
//! The suite's credentials app, sketched upright in pen -- the bow at the bottom, the blade at
//! the top with its two bits to the right. The app lays it on its side across a desk's window;
//! the herbarium is read on a phone first, so here it stands as it was drawn, its length down the
//! screen, nearly the room's height. Upright, the tips are one above another down the key, so each
//! chip has the key's width to itself; and every cut keeps the top, so a section's controls stand
//! on the ground its cut cleared and what they open runs on down from there. The bow's inner circle is a hole in the
//! outline, spliced in along a hairline seam, so the mosaic leaves it empty.
//!
//! The outline is the app's own, in the page's fractions, as the cuts are. Where the tips stand is in the shape's unit space, which is
//! the key's width across and about two and a third of them down.

use lichen::{Cut, Region, Sketch, Tip, Turn};

/// The key, as the specimen draws it.
pub(crate) const KEY: Sketch<'static> = Sketch {
    outline: &OUTLINE,
    page: PAGE,
    turn: Turn::None,
    along: (0.25, 1.0),
    edge: None,
    pitch: PITCH,
    tip: TIP,
    tips: &TIPS,
};

/// How far everything keeps from the room's edge.
pub(crate) const MARGIN: f32 = 24.0;

/// The page the outline was traced on: the photo, upright.
const PAGE: (f32, f32) = (3000.0, 4000.0);

/// The outline, as fractions of the upright page: the bow's first point, the seam, the hole, the
/// seam back, and then clockwise round the rest. Closed implicitly.
#[rustfmt::skip]
const OUTLINE: [(f32, f32); 113] = [
    (0.171, 0.765), (0.295, 0.748), (0.297, 0.773), (0.304, 0.793),
    (0.319, 0.816), (0.338, 0.834), (0.358, 0.847), (0.377, 0.855),
    (0.405, 0.863), (0.432, 0.866), (0.437, 0.869), (0.473, 0.867),
    (0.504, 0.859), (0.548, 0.840), (0.569, 0.821), (0.582, 0.804),
    (0.584, 0.802), (0.585, 0.798), (0.594, 0.781), (0.596, 0.764),
    (0.594, 0.743), (0.588, 0.721), (0.579, 0.705), (0.557, 0.682),
    (0.540, 0.669), (0.509, 0.654), (0.471, 0.644), (0.456, 0.643),
    (0.434, 0.644), (0.399, 0.649), (0.373, 0.658), (0.347, 0.672),
    (0.326, 0.688), (0.310, 0.707), (0.301, 0.720), (0.295, 0.748),
    (0.171, 0.765), (0.177, 0.720), (0.192, 0.683), (0.231, 0.633),
    (0.253, 0.615), (0.302, 0.589), (0.361, 0.568), (0.395, 0.560),
    (0.389, 0.390), (0.389, 0.388), (0.382, 0.119), (0.381, 0.099),
    (0.378, 0.020), (0.393, 0.012), (0.442, 0.005), (0.475, 0.012),
    (0.487, 0.021), (0.489, 0.040), (0.487, 0.061), (0.494, 0.113),
    (0.492, 0.126), (0.494, 0.157), (0.496, 0.178), (0.504, 0.205),
    (0.523, 0.206), (0.532, 0.204), (0.534, 0.205), (0.559, 0.205),
    (0.646, 0.198), (0.705, 0.197), (0.716, 0.210), (0.716, 0.228),
    (0.708, 0.242), (0.617, 0.246), (0.602, 0.249), (0.597, 0.247),
    (0.564, 0.248), (0.498, 0.252), (0.502, 0.302), (0.591, 0.298),
    (0.601, 0.316), (0.598, 0.331), (0.589, 0.341), (0.502, 0.345),
    (0.503, 0.386), (0.503, 0.389), (0.506, 0.564), (0.512, 0.562),
    (0.569, 0.580), (0.603, 0.596), (0.652, 0.629), (0.651, 0.631),
    (0.684, 0.667), (0.704, 0.704), (0.714, 0.737), (0.717, 0.782),
    (0.714, 0.802), (0.694, 0.836), (0.692, 0.840), (0.667, 0.872),
    (0.635, 0.900), (0.595, 0.923), (0.557, 0.938), (0.526, 0.946),
    (0.524, 0.950), (0.492, 0.956), (0.461, 0.959), (0.451, 0.959),
    (0.448, 0.955), (0.408, 0.953), (0.407, 0.950), (0.356, 0.941),
    (0.312, 0.925), (0.263, 0.900), (0.224, 0.871), (0.205, 0.848),
    (0.179, 0.805),
];

/// The three tips, in the order the app names them: browse, intake, readout.
///
/// Every cut keeps a piece of the key's top and faces down, toward the open end of a phone's
/// screen. Each tip's controls stand just under its line, the key's width, on the ground the cut
/// cleared; what they open stands under them, on down past the key. The higher a tip stands, the
/// more of the key its cut clears: browse, whose list and pane want the most room, at the blade's
/// end; intake on the shaft between the bits, its cut under them; and readout on the shaft below
/// the bits, its cut at the shaft's foot, leaving the ring for its lists. The
/// tips step from one edge of the shaft to the other rather than standing in a column, and each
/// line leans a little, so no two read as ruled.
const TIPS: [Tip; 3] = [
    Tip {
        at: Region::Point((0.54, 0.13)),
        cut: Cut::new((0.300, 0.135), (0.580, 0.105), (0.440, 0.040)),
    },
    Tip {
        at: Region::Point((0.43, 0.65)),
        cut: Cut::new((0.333, 0.396), (0.560, 0.378), (0.440, 0.100)),
    },
    Tip {
        at: Region::Point((0.56, 1.12)),
        cut: Cut::new((0.324, 0.524), (0.565, 0.540), (0.438, 0.100)),
    },
];

/// Where each tip's controls stand, in the order of the tips: a box on the ground its cut clears,
/// just under the line and the key's width -- its top-left corner and its size, in unit space.
pub(crate) const CONTROLS: [((f32, f32), (f32, f32)); 3] = [
    ((0.0, 0.34), (1.0, 0.36)),
    ((0.0, 0.99), (1.0, 0.52)),
    ((0.0, 1.34), (1.0, 0.3)),
];

/// How large a tip's square is, as a fraction of the key's width.
const TIP: f32 = 0.16;

/// How far apart tile centres sit, as a fraction of the key's width: fine enough for the shaft,
/// a fifth of the width across, to hold several.
const PITCH: f32 = 0.04;

/// How much of the room's height the key takes: most of it, so the key is what the page opens on.
pub(crate) const SHARE: f32 = 0.8;

#[cfg(test)]
mod tests {
    use super::*;
    use lichen::Silhouette;

    fn shape() -> Silhouette {
        Silhouette::traced(KEY.outline, KEY.page, KEY.turn)
    }

    /// Whether the outline holds `point`, by the crossings -- as the mosaic asks it, so the hole
    /// is outside.
    fn inside(shape: &Silhouette, (px, py): (f32, f32)) -> bool {
        let outline: Vec<(f32, f32)> = KEY.outline.iter().map(|&p| shape.at(p)).collect();
        let mut crossed = false;
        let mut previous = outline[outline.len() - 1];
        for &(x, y) in &outline {
            if (y > py) != (previous.1 > py)
                && px < (previous.0 - x) * (py - y) / (previous.1 - y) + x
            {
                crossed = !crossed;
            }
            previous = (x, y);
        }
        crossed
    }

    /// Each tip's controls stand on what its cut clears: every corner of the box is on the far side
    /// of the line, below it.
    #[test]
    fn controls_stand_on_what_the_cut_clears() {
        let shape = shape();
        for (n, tip) in KEY.tips.iter().enumerate() {
            let (a, b) = shape.crossings(tip.cut).expect("crosses");
            let ((x, y), (w, h)) = CONTROLS[n];
            assert!(
                y > a.1.max(b.1),
                "tip {n}'s controls start under its line: {y} against {a:?} {b:?}"
            );
            assert!(x >= 0.0 && x + w <= 1.0 && y + h <= shape.aspect());
        }
    }

    /// Every cut keeps the top and faces down, which is what lets a section open on what it
    /// cleared; and browse's clears the most, intake's less, readout's least.
    #[test]
    fn every_cut_faces_down_and_clears_more_the_higher_its_tip() {
        let shape = shape();
        let mut cleared = Vec::new();
        for (n, tip) in KEY.tips.iter().enumerate() {
            let (nx, ny) = shape.facing(tip.cut);
            assert!(ny > nx.abs(), "tip {n}'s cut faces down: {nx}, {ny}");
            let (a, b) = shape.crossings(tip.cut).expect("crosses");
            cleared.push(shape.aspect() - a.1.max(b.1));
        }
        assert!(
            cleared.windows(2).all(|pair| pair[0] > pair[1]),
            "{cleared:?}"
        );
    }

    /// Upright: a little over twice as tall as it is wide.
    #[test]
    fn the_key_stands_up() {
        let aspect = shape().aspect();
        assert!((2.2..2.45).contains(&aspect), "{aspect}");
    }

    /// The ring has a hole: the middle of the bow is outside the shape, and the ring round it in.
    #[test]
    fn the_bow_is_a_ring() {
        let shape = shape();
        let middle = shape.at((0.45, 0.80));
        assert!(!inside(&shape, middle), "the hole is empty");
        assert!(
            inside(&shape, shape.at((0.45, 0.92))),
            "the ring's foot is there"
        );
    }

    /// Every cut keeps some of the key and crosses it, and each tip stands on what its cut keeps.
    #[test]
    fn each_tip_stands_on_what_its_cut_keeps() {
        let shape = shape();
        for (n, tip) in KEY.tips.iter().enumerate() {
            assert!(!shape.cut(tip.cut).is_empty(), "tip {n}");
            assert!(shape.crossings(tip.cut).is_some(), "tip {n}");
            let Region::Point(middle) = tip.at else {
                panic!("tip {n} stands on a point");
            };
            let (from, to, keep) = (
                shape.at(tip.cut.from),
                shape.at(tip.cut.to),
                shape.at(tip.cut.keep),
            );
            let side = |(px, py): (f32, f32)| {
                (to.0 - from.0) * (py - from.1) - (to.1 - from.1) * (px - from.0)
            };
            assert!(
                side(middle) * side(keep) > 0.0 && inside(&shape, middle),
                "tip {n}'s square stands on what its cut keeps"
            );
        }
    }
}
