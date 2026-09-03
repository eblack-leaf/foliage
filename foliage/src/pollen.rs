use crate::aspen::{Sequence, Tween};
use crate::coordinate::{Area, Position};
use crate::interaction::Drag;
use crate::leaf::Leaf;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;

/// What the tree put out this frame.
///
/// A set you interrogate, not a list you walk: ask it about the elements you own. There is no
/// order to read, apart from the sequences that are ordered in their own right.
#[derive(Clone, Default)]
pub struct Pollen(Arc<Drift>);

impl Pollen {
    /// Whether `leaf` was taken down. Reported once, in the frame after it went.
    pub fn withered(&self, leaf: Leaf) -> bool {
        self.0.withered.contains(&leaf)
    }

    /// The surface's new size, if it changed this frame.
    pub fn resized(&self) -> Option<Area> {
        self.0.resized
    }

    /// Whether a gesture went down on `leaf`.
    ///
    /// The hook for a pressed visual. It says a gesture is being held, not what it will turn out to
    /// be -- [`clicked`](Pollen::clicked) and [`drag_started`](Pollen::drag_started) are the two
    /// things it can become, and [`disengaged`](Pollen::disengaged) is where a pressed visual is
    /// put back however it ended.
    pub fn engaged(&self, leaf: Leaf) -> bool {
        self.0.engaged.contains(&leaf)
    }

    /// Whether `leaf` stopped holding a gesture, however it stopped.
    ///
    /// It ended, or it became a drag this element does not take and passed to a region containing
    /// it. Either way this element is no longer holding anything, so there is always somewhere to
    /// put a pressed visual back.
    pub fn disengaged(&self, leaf: Leaf) -> bool {
        self.0.disengaged.contains(&leaf)
    }

    /// Whether `leaf` was tapped: a gesture that began on it and ended without ever becoming a
    /// drag.
    ///
    /// Not a click that was issued and then taken back. Nothing is emitted while a gesture is
    /// resolving, so there is nothing to retract when it turns out to be a drag.
    pub fn clicked(&self, leaf: Leaf) -> bool {
        self.0.clicked.contains(&leaf)
    }

    /// Whether the gesture `leaf` is holding has become a drag.
    ///
    /// Reported once per gesture, and only to an element that declared
    /// [`drags`](crate::Place::drags) on the axis the gesture went. An element that takes no drags
    /// hears [`disengaged`](Pollen::disengaged) instead, because it has let the gesture go.
    pub fn drag_started(&self, leaf: Leaf) -> bool {
        self.0.drag_started.contains(&leaf)
    }

    /// How far the drag `leaf` is holding has moved, if it moved this frame.
    pub fn dragged(&self, leaf: Leaf) -> Option<Drag> {
        self.0.dragged.get(&leaf).copied()
    }

    /// Whether a tween on `leaf` reached its end.
    ///
    /// The hook for whatever happens next -- pruning what has faded out, hiding what has slid away.
    /// There is nothing to settle: what the element declares was already the target from the moment
    /// the motion started, so this reports an arrival rather than asking for one.
    ///
    /// One report for the element rather than one per property. Two motions ending together are one
    /// arrival to an app, and an app that needs a single value's own end runs it as a
    /// [`tween`](crate::Grow::tween) instead.
    pub fn landed(&self, leaf: Leaf) -> bool {
        self.0.landed.contains(&leaf)
    }

    /// This frame's value of a scalar channel, for as long as it is running.
    ///
    /// The frame it ends reports its end value and [`finished`](Pollen::finished) together, so
    /// there is never an end value to infer from an absence. After that there is nothing to read.
    pub fn tween(&self, tween: Tween) -> Option<f32> {
        self.0.tweens.get(&tween).copied()
    }

    /// Whether a channel or a [`timer`](crate::Grow::timer) reached its end this frame.
    pub fn finished(&self, tween: Tween) -> bool {
        self.0.finished.contains(&tween)
    }

    /// Whether the last tween running under a [`Sequence`](crate::Sequence) ended this frame.
    ///
    /// The group's own arrival, as against each member's. Keyed on the sequence rather than on a
    /// [`Leaf`], because a group is not about an element -- its whole purpose is to time things
    /// together that have nothing else in common.
    ///
    /// However the members ended: landed, cancelled by a direct write, or taken down with their
    /// element. A group being over is one fact and gets one report.
    pub fn sequence_finished(&self, sequence: Sequence) -> bool {
        self.0.sequences.contains(&sequence)
    }

    /// Whether `leaf` took focus.
    pub fn focused(&self, leaf: Leaf) -> bool {
        self.0.focused.contains(&leaf)
    }

    /// Whether `leaf` lost focus.
    ///
    /// It was moved elsewhere, or `leaf` stopped being something focus can rest on -- it withered,
    /// was hidden, or was disabled.
    pub fn unfocused(&self, leaf: Leaf) -> bool {
        self.0.unfocused.contains(&leaf)
    }

    pub(crate) fn seal(drift: Drift) -> Self {
        Self(Arc::new(drift))
    }
}

impl fmt::Debug for Pollen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// What the frame is collecting, before it is sealed into a [`Pollen`] and handed over.
#[derive(Default, Debug, PartialEq)]
pub(crate) struct Drift {
    pub(crate) withered: HashSet<Leaf>,
    pub(crate) resized: Option<Area>,
    pub(crate) engaged: HashSet<Leaf>,
    pub(crate) disengaged: HashSet<Leaf>,
    pub(crate) clicked: HashSet<Leaf>,
    pub(crate) drag_started: HashSet<Leaf>,
    pub(crate) dragged: HashMap<Leaf, Drag>,
    pub(crate) focused: HashSet<Leaf>,
    pub(crate) unfocused: HashSet<Leaf>,
    pub(crate) landed: HashSet<Leaf>,
    pub(crate) tweens: HashMap<Tween, f32>,
    pub(crate) finished: HashSet<Tween>,
    pub(crate) sequences: HashSet<Sequence>,
}

impl Drift {
    /// Whether there is anything here to tell the app.
    ///
    /// A frame is owed while this is true (F9): steps 4 through 7 emit into the drift, and step 3
    /// of the *next* frame is where an app is handed it (F7). Without this the loop could idle
    /// holding a report nothing would ever be run to deliver.
    ///
    /// Compared against an empty drift rather than field by field, so a field added later is
    /// counted without anything having to remember to count it.
    pub(crate) fn pending(&self) -> bool {
        *self != Drift::default()
    }

    /// Adds one frame's worth of movement to what `leaf` is being told about its drag.
    ///
    /// Several moves can arrive in one frame, and a reader is handed one answer: where the drag is
    /// now, and how far it came this frame. Which platform events that was made of is engine
    /// bookkeeping.
    pub(crate) fn dragged(&mut self, leaf: Leaf, drag: Drag) {
        let carried = self.dragged.entry(leaf).or_insert(Drag {
            delta: Position::default(),
            ..drag
        });
        carried.current = drag.current;
        carried.delta = Position::new(
            carried.delta.x + drag.delta.x,
            carried.delta.y + drag.delta.y,
        );
    }
}
