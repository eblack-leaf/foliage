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
//! The outline is the app's own, upright. It, the cuts and where the tips stand are all in the
//! shape's unit space, which is the key's width across and about two and a third of them down.

use lichen::{Cut, Region, Sketch, Tip};

/// The key, as the specimen draws it.
pub(crate) const KEY: Sketch<'static> = Sketch {
    outline: &OUTLINE,
    along: (0.25, 1.0),
    edge: None,
    pitch: PITCH,
    tip: TIP,
    tips: &TIPS,
};

/// How far everything keeps from the room's edge.
pub(crate) const MARGIN: f32 = 24.0;

/// The outline, upright, in the key's own unit space: the bow's first point, the seam, the hole,
/// the seam back, and then clockwise round the rest. Closed implicitly.
#[rustfmt::skip]
const OUTLINE: [(f32, f32); 113] = [
    (0.000, 1.856), (0.227, 1.814), (0.231, 1.875), (0.244, 1.924),
    (0.271, 1.980), (0.306, 2.024), (0.342, 2.056), (0.377, 2.076),
    (0.429, 2.095), (0.478, 2.103), (0.487, 2.110), (0.553, 2.105),
    (0.610, 2.085), (0.690, 2.039), (0.729, 1.993), (0.753, 1.951),
    (0.756, 1.946), (0.758, 1.937), (0.775, 1.895), (0.778, 1.853),
    (0.775, 1.802), (0.764, 1.748), (0.747, 1.709), (0.707, 1.653),
    (0.676, 1.621), (0.619, 1.585), (0.549, 1.560), (0.522, 1.558),
    (0.482, 1.560), (0.418, 1.573), (0.370, 1.595), (0.322, 1.629),
    (0.284, 1.668), (0.255, 1.714), (0.238, 1.746), (0.227, 1.814),
    (0.000, 1.856), (0.011, 1.746), (0.038, 1.656), (0.110, 1.534),
    (0.150, 1.490), (0.240, 1.426), (0.348, 1.375), (0.410, 1.355),
    (0.399, 0.940), (0.399, 0.935), (0.386, 0.278), (0.385, 0.230),
    (0.379, 0.037), (0.407, 0.017), (0.496, 0.000), (0.557, 0.017),
    (0.579, 0.039), (0.582, 0.085), (0.579, 0.137), (0.592, 0.264),
    (0.588, 0.295), (0.592, 0.371), (0.595, 0.422), (0.610, 0.488),
    (0.645, 0.491), (0.661, 0.486), (0.665, 0.488), (0.711, 0.488),
    (0.870, 0.471), (0.978, 0.469), (0.998, 0.501), (0.998, 0.545),
    (0.984, 0.579), (0.817, 0.589), (0.789, 0.596), (0.780, 0.591),
    (0.720, 0.593), (0.599, 0.603), (0.606, 0.725), (0.769, 0.716),
    (0.788, 0.759), (0.782, 0.796), (0.766, 0.821), (0.606, 0.830),
    (0.608, 0.930), (0.608, 0.938), (0.614, 1.365), (0.625, 1.360),
    (0.729, 1.404), (0.791, 1.443), (0.881, 1.524), (0.879, 1.529),
    (0.940, 1.617), (0.976, 1.707), (0.995, 1.788), (1.000, 1.897),
    (0.995, 1.946), (0.958, 2.029), (0.954, 2.039), (0.908, 2.117),
    (0.850, 2.186), (0.777, 2.242), (0.707, 2.278), (0.650, 2.298),
    (0.647, 2.308), (0.588, 2.322), (0.531, 2.330), (0.513, 2.330),
    (0.507, 2.320), (0.434, 2.315), (0.432, 2.308), (0.339, 2.286),
    (0.258, 2.247), (0.168, 2.186), (0.097, 2.115), (0.062, 2.059),
    (0.015, 1.954),
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
        cut: Cut::new((0.236, 0.317), (0.749, 0.244), (0.493, 0.085)),
    },
    Tip {
        at: Region::Point((0.43, 0.65)),
        cut: Cut::new((0.297, 0.955), (0.712, 0.911), (0.493, 0.232)),
    },
    Tip {
        at: Region::Point((0.56, 1.12)),
        cut: Cut::new((0.280, 1.267), (0.722, 1.306), (0.489, 0.232)),
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
        Silhouette::new(KEY.outline)
    }

    /// Whether the outline holds `point`, by the crossings -- as the mosaic asks it, so the hole
    /// is outside.
    fn inside((px, py): (f32, f32)) -> bool {
        let outline = KEY.outline;
        let mut crossed = false;
        let mut previous = outline[outline.len() - 1];
        for &(x, y) in outline {
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
        assert!(!inside((0.511, 1.941)), "the hole is empty");
        assert!(inside((0.511, 2.234)), "the ring's foot is there");
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
            let (from, to, keep) = (tip.cut.from, tip.cut.to, tip.cut.keep);
            let side = |(px, py): (f32, f32)| {
                (to.0 - from.0) * (py - from.1) - (to.1 - from.1) * (px - from.0)
            };
            assert!(
                side(middle) * side(keep) > 0.0 && inside(middle),
                "tip {n}'s square stands on what its cut keeps"
            );
        }
    }
}
