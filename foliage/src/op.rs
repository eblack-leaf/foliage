use crate::aspen::{Motion, Timing, Tween};
use crate::elevation::Elevation;
use crate::elm::{Chlorophyll, Pigment};
use crate::interaction::focus::Intent;
use crate::leaf::{Growth, Leaf};
use crate::palette::{Fill, Scheme};
use crate::place::{Caller, Placement};
use crate::placement::grid::Grid;
use crate::placement::location::Location;
use crate::rounding::Corners;
use crate::text::Lettering;
use crate::view::ScrollTo;

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
        fill: Fill,
    },
    /// Rewrites what a run says. The measure follows in the same frame, because measuring is a pass
    /// and not a reaction to the write.
    Letter {
        leaf: Leaf,
        value: String,
    },
    Round {
        leaf: Leaf,
        rounding: Corners,
    },
    /// Moves a region. Recorded here and answered in R4, which is the one place that has both this
    /// frame's extent and the offset it clamps.
    Scroll {
        leaf: Leaf,
        to: ScrollTo,
    },
    Disable {
        leaf: Leaf,
        disabled: bool,
    },
    Reveal {
        leaf: Leaf,
        visible: bool,
    },
    Fade {
        leaf: Leaf,
        opacity: f32,
    },
    /// Moves a property over time. Applied here, which is what makes a write to the same property
    /// earlier in the same drain cancel it, and one later replace it.
    Animate {
        leaf: Leaf,
        motion: Motion,
        timing: Timing,
    },
    /// A scalar channel, reported outward every frame and written nowhere.
    Channel {
        tween: Tween,
        from: f32,
        to: f32,
        timing: Timing,
    },
    /// Ends a channel before it has run out.
    Stop(Tween),
    /// Where focus is to go. Applied here and answered at settle, against the geometry and the
    /// inherited state this frame resolves.
    Focus(Intent),
    /// The one op that names no element: what every role resolves to, for the whole tree.
    Repaint(Scheme),
}

/// An element formed and not yet open: what the queue carries between the call that described it
/// and the drain that grows it.
///
/// It carries the components the element will hold rather than a second enum describing them: a bud
/// is the element before it exists, not an account of one. The pigment is present exactly when the
/// chlorophyll is a renderer that has one, and the lettering exactly when there is a run to read.
pub(crate) struct Bud {
    pub(crate) chlorophyll: Chlorophyll,
    pub(crate) pigment: Option<Pigment>,
    pub(crate) lettering: Option<Lettering>,
    pub(crate) placement: Placement,
    pub(crate) at: Caller,
}
