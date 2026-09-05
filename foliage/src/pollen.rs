use crate::aspen::{Sequence, Tween};
use crate::asset::Arrival;
use crate::coordinate::{Area, Position};
use crate::interaction::Drag;
use crate::interaction::input::Keystroke;
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
    /// be -- [`clicked`](Pollen::clicked), [`held`](Pollen::held) and
    /// [`drag_started`](Pollen::drag_started) are what it can become, and
    /// [`disengaged`](Pollen::disengaged) is where a pressed visual is put back however it ended.
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
        self.0.clicked.contains_key(&leaf)
    }

    /// Where `leaf` was tapped, if it was.
    ///
    /// The point the gesture *began* at, which is the point it was hit-tested against and therefore
    /// the only one that is certainly on the element. A gesture that wandered and came back is
    /// still a tap, and it is a tap where it started.
    pub fn clicked_at(&self, leaf: Leaf) -> Option<Position> {
        self.0.clicked.get(&leaf).copied()
    }

    /// Whether the press `leaf` is holding has been held past [`Hold::after`](crate::Hold) without
    /// becoming a drag.
    ///
    /// A gesture fact of its own, and the second way a press stops being one that might still be a
    /// tap -- so a hold is never also a [`clicked`](Pollen::clicked), and a press released after one
    /// moves no focus. Reported once, in the frame the duration passed in.
    ///
    /// What follows is the holder's. The drag out of a hold belongs to whoever took it, whatever
    /// that element declared with [`drags`](crate::Place::drags) and whichever way it goes, so a
    /// press-and-hold that turns into a drag reports [`drag_started`](Pollen::drag_started) and then
    /// [`dragged`](Pollen::dragged) like any other.
    pub fn held(&self, leaf: Leaf) -> bool {
        self.0.held.contains_key(&leaf)
    }

    /// Where `leaf` was held, if it was.
    ///
    /// The point the press landed at, which is the point it was hit-tested against. A hold is where
    /// a menu opens and where a selection begins, and neither has anywhere else to take its place
    /// from -- the pointer has not moved since, by definition of the gesture still resolving.
    pub fn held_at(&self, leaf: Leaf) -> Option<Position> {
        self.0.held.get(&leaf).copied()
    }

    /// Whether the gesture `leaf` is holding has become a drag.
    ///
    /// Reported once per gesture, and only to an element that declared
    /// [`drags`](crate::Place::drags) on the axis the gesture went, or that took the hold this drag
    /// came out of -- a hold has already settled who is holding the gesture, so there is nothing
    /// left for an axis to decide. An element that takes no drags hears
    /// [`disengaged`](Pollen::disengaged) instead, because it has let the gesture go.
    pub fn drag_started(&self, leaf: Leaf) -> bool {
        self.0.drag_started.contains(&leaf)
    }

    /// How far the drag `leaf` is holding has moved, if it moved this frame.
    pub fn dragged(&self, leaf: Leaf) -> Option<Drag> {
        self.0.dragged.get(&leaf).copied()
    }

    /// Whether a motion on `leaf` reached its end.
    ///
    /// The element settling, and the cheap hook for whatever happens next -- pruning what has faded
    /// out, hiding what has slid away. There is nothing to settle: what the element declares was
    /// already the target from the moment the motion started, so this reports an arrival rather than
    /// asking for one.
    ///
    /// One report for the element rather than one per property, so it says an element arrived and
    /// not which of its properties did. Two motions ending together are one arrival to an app, and
    /// telling one of an element's motions from another is what the name
    /// [`animate`](crate::Grow::animate) hands back is for.
    pub fn landed(&self, leaf: Leaf) -> bool {
        self.0.landed.contains(&leaf)
    }

    /// How far a tween has come this frame, for as long as it is running.
    ///
    /// A channel's value between the two ends it was given, and a motion's eased progress from `0.0`
    /// to `1.0` -- the eased progress and not the fraction of the duration elapsed, because what a
    /// motion is part way through is what it looks like, and the two differ under every
    /// [`Ease`](crate::Ease) but [`Linear`](crate::Ease::Linear).
    ///
    /// The frame it ends reports its end value and [`finished`](Pollen::finished) together, so
    /// there is never an end value to infer from an absence. After that there is nothing to read.
    pub fn tween(&self, tween: Tween) -> Option<f32> {
        self.0.tweens.get(&tween).copied()
    }

    /// Whether the motion, channel or [`timer`](crate::Grow::timer) `tween` names ended this frame
    /// **on the end it was going to**.
    ///
    /// An arrival, and the hook a chain hangs off: it reports a tween that ran its course, and one
    /// [`finish`](crate::Grow::finish) ended early, which is the app declaring the same arrival. It
    /// does not report a name that stopped being the one running -- [`stop`](crate::Grow::stop)ped,
    /// cancelled by a direct write, replaced by a second motion on the property, or taken down with
    /// its element. A chain waiting on any of those never runs, which is what makes stopping a
    /// motion the way to break one.
    pub fn finished(&self, tween: Tween) -> bool {
        self.0.finished.contains(&tween)
    }

    /// Whether the last tween running under a [`Sequence`](crate::Sequence) ended this frame.
    ///
    /// The group's own arrival, as against each member's. Keyed on the sequence rather than on a
    /// [`Leaf`], because a group is not about an element -- its whole purpose is to time things
    /// together that have nothing else in common.
    ///
    /// However the members ended: landed, stopped, cancelled by a direct write, or taken down with
    /// their element. A group being over is one fact and gets one report -- whether a *particular*
    /// motion arrived is [`finished`](Pollen::finished), which is a different question and is
    /// answered separately, so neither report has to carry the other's meaning.
    pub fn sequence_finished(&self, sequence: Sequence) -> bool {
        self.0.sequences.contains(&sequence)
    }

    /// Whether the person at the keyboard changed what `leaf` says.
    ///
    /// A [`TextInput`](crate::TextInput) only, and only what was typed into it: a value the app
    /// wrote with [`text`](crate::Grow::text) is not reported back, because an app that wrote one
    /// already knows what it wrote. What it now says is [`Vein::Text`](crate::Vein::Text).
    pub fn edited(&self, leaf: Leaf) -> bool {
        self.0.edited.contains(&leaf)
    }

    /// Whether `Enter` was pressed in `leaf`.
    ///
    /// What that means is the app's: a field says the key was pressed and holds no opinion about
    /// whether anything is to be submitted.
    pub fn submitted(&self, leaf: Leaf) -> bool {
        self.0.submitted.contains(&leaf)
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

    /// Whether what `of` names has arrived and been registered.
    ///
    /// One question over a [`Font`](crate::Font), a [`Field`](crate::Field) and a
    /// [`Plate`](crate::Plate) alike, because one road fills all three. Reported once, in the frame
    /// the bytes landed in.
    ///
    /// Mostly nothing has to wait on it: an element drawing a picture or a mark that has not arrived
    /// occupies its box and draws nothing, and appears the frame it does. A **font** is the one worth
    /// waiting for, because a run composed in one that has yet to land is measured in the bundled
    /// face and reflows when the real one arrives.
    pub fn loaded(&self, of: impl Into<Arrival>) -> bool {
        self.0.loaded.contains(&of.into())
    }

    /// Whether what `of` names could not be read, or could not be used once it was.
    ///
    /// A path that is not there, a fetch that answered with a status, a picture in a format foliage
    /// does not decode, a font that turned out to be proportional. One report, because there is one
    /// thing an app can do about any of them.
    ///
    /// The name stays valid and unfilled: what drew nothing goes on drawing nothing, and a second
    /// attempt is a second call.
    pub fn missing(&self, of: impl Into<Arrival>) -> bool {
        self.0.missing.contains(&of.into())
    }

    /// What the clipboard held, if a [`paste`](crate::Grow::paste) this app asked for was answered
    /// this frame.
    ///
    /// Never in the frame that asked for it. Empty text is the answer for an empty clipboard and
    /// for one the host would not let the engine read, because there is one thing an app does about
    /// either -- the reason it could not be read is traced rather than reported.
    ///
    /// A [`TextInput`](crate::TextInput) answers `Ctrl+V` for itself, and what it wrote is
    /// [`edited`](Pollen::edited) like anything else typed into it. Nothing arrives here for that.
    pub fn pasted(&self) -> Option<&str> {
        self.0.pasted.as_deref()
    }

    /// What was typed at `leaf` this frame, in the order it arrived.
    ///
    /// A key goes to whatever holds focus, and focus rests only on what declared
    /// [`interactive`](crate::Place::interactive) -- so an element that can be focused hears keys
    /// with nothing further declared, and one that cannot never does. What an element makes of a
    /// key is the app's: an activation on [`Key::Enter`], a step through a list on
    /// [`Key::Up`]/[`Key::Down`], anything else it wants to answer.
    ///
    /// ```
    /// # use foliage::{Grove, Grow, Key, Leaf, Pollen};
    /// # fn f(grove: &mut Grove, pollen: &Pollen, button: Leaf, menu: Leaf) {
    /// for stroke in pollen.keys(button) {
    ///     match stroke.key {
    ///         Key::Enter => grove.prune(menu),
    ///         Key::Down => grove.focus_next(),
    ///         _ => {}
    ///     }
    /// }
    /// # }
    /// ```
    ///
    /// **Ordered, which nothing else here is.** [`Pollen`] is a set to interrogate rather than a
    /// list to walk, and keystrokes are the one exception, because two keys in a frame mean
    /// different things in each order.
    ///
    /// A [`TextInput`](crate::TextInput) is told what it was sent as well, so a field's own keys are
    /// readable beside the [`edited`](Pollen::edited) they produced.
    pub fn keys(&self, leaf: Leaf) -> &[Keystroke] {
        self.0.keys.get(&leaf).map_or(&[], Vec::as_slice)
    }

    /// What was typed this frame while focus rested nowhere, in the order it arrived.
    ///
    /// Keys reach an element by holding focus, and a page that has never been pressed holds none --
    /// so a chord that is about the whole app rather than about anything in it would have nowhere
    /// to land. It lands here instead.
    ///
    /// A key is never reported both here and to an element: focus rests somewhere or it does not.
    pub fn root_keys(&self) -> &[Keystroke] {
        &self.0.root_keys
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
    /// Where each tap landed, which is where its gesture began. Carried rather than counted,
    /// because an element that has somewhere to put a point needs the point.
    pub(crate) clicked: HashMap<Leaf, Position>,
    /// Where each hold landed, which is where its press did. Carried for the reason a tap's point
    /// is: a menu and a selection both begin at a point, and neither has another to use.
    pub(crate) held: HashMap<Leaf, Position>,
    pub(crate) drag_started: HashSet<Leaf>,
    pub(crate) dragged: HashMap<Leaf, Drag>,
    /// What each focused element was sent, in arrival order -- the one ordered thing here, because
    /// two keys in a frame mean different things in each order.
    pub(crate) keys: HashMap<Leaf, Vec<Keystroke>>,
    /// The same for keys that arrived while focus rested nowhere, which are the app's own.
    pub(crate) root_keys: Vec<Keystroke>,
    /// What a `paste` the app asked for came back with. A field's own paste is an `edited` instead,
    /// because what the person at the keyboard did to a value is one report however they did it.
    pub(crate) pasted: Option<String>,
    /// What arrived from a path or a URL this frame, and what could not.
    pub(crate) loaded: HashSet<Arrival>,
    pub(crate) missing: HashSet<Arrival>,
    pub(crate) edited: HashSet<Leaf>,
    pub(crate) submitted: HashSet<Leaf>,
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
