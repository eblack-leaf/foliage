use crate::elevation::Elevation;
use crate::elm::{Chlorophyll, PanelPigment};
use crate::leaf::{Growth, Leaf};
use crate::palette::{Palette, Scheme};
use crate::place::{Caller, Placement};
use crate::placement::grid::Grid;
use crate::placement::location::Location;
use crate::rounding::Corners;

/// One queued change.
pub(crate) enum Op {
    Plant {
        leaf: Leaf,
        growth: Growth,
        bud: Bud,
    },
    Branch {
        leaf: Leaf,
        growth: Growth,
        under: Leaf,
        bud: Bud,
    },
    Prune(Leaf),
    Place {
        leaf: Leaf,
        location: Location,
    },
    Divide {
        leaf: Leaf,
        grid: Grid,
    },
    Anchor {
        leaf: Leaf,
        to: Leaf,
        at: Caller,
    },
    Elevate {
        leaf: Leaf,
        elevation: Elevation,
    },
    Recolor {
        leaf: Leaf,
        color: Palette,
    },
    Round {
        leaf: Leaf,
        rounding: Corners,
    },
    /// The one op that names no element: what every role resolves to, for the whole tree.
    Repaint(Scheme),
}

/// An element formed and not yet open: what the queue carries between the call that described it
/// and the drain that grows it.
///
/// It carries the components the element will hold rather than a second enum describing them: a bud
/// is the element before it exists, not an account of one. The pigment is present exactly when the
/// chlorophyll is a renderer that has one.
pub(crate) struct Bud {
    pub(crate) chlorophyll: Chlorophyll,
    pub(crate) pigment: Option<PanelPigment>,
    pub(crate) placement: Placement,
    pub(crate) at: Caller,
}
