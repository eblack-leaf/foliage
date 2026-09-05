use crate::asset::{Destination, Retrieved};
use crate::aspen::{Motion, Timing, Tween};
use crate::coordinate::Area;
use crate::elevation::Elevation;
use crate::elm::{Chlorophyll, Pigment};
use crate::frond::Sprouts;
use crate::image::Plate;
use crate::interaction::focus::Intent;
use crate::interaction::input::Keystroke;
use crate::leaf::{Growth, Leaf};
use crate::palette::{Fill, Scheme};
use crate::place::{Caller, Placement};
use crate::placement::grid::Grid;
use crate::placement::location::Location;
use crate::placement::point::Point;
use crate::rounding::Corners;
use crate::text::{Lettering, Tints};
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
    /// Moves the two ends of a stroke. The point-mode counterpart to [`Place`](Op::Place), and
    /// separate for the same reason the declarations are: an element states a box or two ends, and
    /// an op that could write either would be able to write the one the element does not have.
    Trace {
        leaf: Leaf,
        from: Point,
        to: Point,
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
    /// One keystroke, against whatever holds focus. Which element that is is dispatch's answer;
    /// what the key does is the drain's.
    ///
    /// `target` is `None` where focus rests nowhere. The key is still carried, still ordered against
    /// everything else in the queue, and still reported -- to the app itself rather than to an
    /// element, which is what lets a chord reach a page nobody has pressed.
    Keyed {
        target: Option<Leaf>,
        stroke: Keystroke,
    },
    /// Selects a span of a field's value outright.
    ///
    /// Anchor-then-caret rather than low-then-high, so a span whose end precedes its start is a
    /// selection reaching backwards -- which is what a drag leftwards is. The one op a gesture in a
    /// field produces, because where a gesture landed is a position and what a field makes of one is
    /// a span: the field converts, and nothing needs an op that speaks in pixels.
    Select {
        leaf: Leaf,
        range: core::ops::Range<usize>,
    },
    /// Refills part of a run, over a range of its own index space.
    Tint {
        leaf: Leaf,
        tints: Tints,
    },
    Round {
        leaf: Leaf,
        rounding: Corners,
    },
    /// Reshapes a regular polygon: how many sides, how round, how far turned.
    Reshape {
        leaf: Leaf,
        shape: crate::polygon::Shape,
    },
    /// Fills a registered picture's name with pixels.
    ///
    /// An op like any other, rather than a call on the engine, because a picture may arrive at any
    /// frame -- from a fetch, a decode on another thread, a re-render at a higher resolution -- and
    /// arriving is a change to the tree's contents in exactly the way a rewritten run is.
    Load {
        plate: Plate,
        pixels: Vec<u8>,
        size: Area,
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
    /// Bytes that were read from somewhere outside the frame, and the name they were read for.
    ///
    /// Pushed by whatever finished the retrieval rather than by an app, which is the whole reason it
    /// is an op: it arrives at a moment nothing chose, and the queue is what gives it a place in the
    /// order anyway.
    Arrived {
        destination: Destination,
        bytes: Retrieved,
    },
    /// Text to put on the clipboard.
    Copy(String),
    /// A request for what the clipboard holds, and who it is for: the field that was asked to paste
    /// into itself, or the app where it asked for itself.
    ///
    /// Separate from the answer because reading a clipboard is not instant on either target -- a
    /// promise on the web, a round trip to whoever owns the selection off it.
    Paste { into: Option<Leaf> },
    /// What the clipboard turned out to hold, and who asked for it.
    ///
    /// Pushed by whatever finished the read rather than by an app, which is what makes it an op for
    /// the reason [`Arrived`](Op::Arrived) is one: it lands at a moment nothing chose, and the queue
    /// is what gives it a place in the order anyway.
    Pasted { into: Option<Leaf>, text: String },
    /// A URL to go to.
    Navigate(String),
    /// A URL to hand the host to save.
    Download(String),
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
    /// Present only where part of a run is filled differently from the rest of it.
    pub(crate) tints: Option<Tints>,
    /// Present only on a [`Frond`](crate::frond) -- a leaf that is divided. The drain takes this and
    /// grows the leaflets underneath what it just grew.
    pub(crate) sprout: Option<Box<dyn Sprouts>>,
    pub(crate) placement: Placement,
    pub(crate) at: Caller,
}

impl Bud {
    /// An element carrying nothing but a placement: no renderer and nothing to read.
    ///
    /// What every seed fills in from, so a seed states what it *is* rather than restating what it
    /// is not.
    pub(crate) fn bare() -> Self {
        Self {
            chlorophyll: Chlorophyll::None,
            pigment: None,
            lettering: None,
            tints: None,
            sprout: None,
            placement: Placement::default(),
            // Overwritten by every real callsite, which takes the caller's own.
            at: core::panic::Location::caller(),
        }
    }
}
