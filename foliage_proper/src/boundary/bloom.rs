use crate::boundary::leaf::Leaf;
use crate::boundary::tween::Tween;
use crate::coordinate::area::Area;
use crate::coordinate::position::Position;
use crate::{
    AssetKey, InteractionMethod, Key, Layout, Logical, Modifiers, PhysicalKey, TextInputAction,
};
use bevy_ecs::resource::Resource;

/// Something the tree did. The whole of what foliage reports outward.
///
/// Collected during the frame and handed over by [`Canopy::take`](crate::Canopy::take).
///
/// Two things worth knowing. A single physical click emits [`Clicked`](Bloom::Clicked) for
/// the element under the pointer *and* for every pass-through element the gesture crossed, so
/// several per frame is normal and they arrive in hit-test order. And a `Leaf` reported here
/// may already have withered by the time you act on it -- which is safe, since every command
/// naming a withered `Leaf` is a no-op.
#[derive(Clone, Debug)]
pub enum Bloom {
    /// Pressed and released on the same element without dragging.
    Clicked(Leaf),
    /// Pointer went down on this element.
    Engaged(Leaf),
    /// The pointer moved while this element held the gesture -- one per frame that carries a
    /// move, from the first pixel.
    ///
    /// This is the stream, not the threshold: it arrives below
    /// [`InteractionListener::DRAG_THRESHOLD`](crate::InteractionListener::DRAG_THRESHOLD)
    /// too, and a gesture that reports moves can still end in [`Clicked`](Bloom::Clicked).
    /// Take it for anything that follows the pointer -- a knob, a slider, a drag proxy --
    /// and [`DragStarted`](Bloom::DragStarted) for the moment the gesture commits.
    Dragged(Leaf),
    /// The gesture holding this element passed the drag threshold: it is a drag now, and the
    /// release will not [`Clicked`](Bloom::Clicked). Once per gesture, ahead of the
    /// [`Dragged`](Bloom::Dragged) for the same move.
    DragStarted(Leaf),
    /// The gesture that grabbed this element ended, however it ended. Always follows an
    /// [`Engaged`](Bloom::Engaged), whether or not a [`Clicked`](Bloom::Clicked) also fired.
    Disengaged(Leaf),
    Focused(Leaf),
    Unfocused(Leaf),
    /// A key, as the layout produces it -- what to use for typed text.
    Key {
        key: Key,
        mods: Modifiers,
    },
    /// A key by physical position, independent of layout -- what to use for chords bound to
    /// where a key sits rather than what it prints.
    PhysicalKey {
        key: PhysicalKey,
        mods: Modifiers,
    },
    /// A text input's contents changed, by typing, pasting, or a write.
    TextChanged {
        leaf: Leaf,
        value: String,
    },
    /// A text input matched a binding. Submission is `TextInputAction::Enter` on a
    /// single-line input.
    TextAction {
        leaf: Leaf,
        action: TextInputAction,
    },
    /// This frame's values for a running tween, one per channel.
    Tween {
        tween: Tween,
        values: Vec<f32>,
    },
    /// A tween reached its end and will report no further values.
    TweenDone(Tween),
    /// A countdown ran out. The `Leaf` names the timer and is spent -- it will never name
    /// anything again.
    TimerFinished(Leaf),
    /// Every animation joined to this sequence has finished.
    SequenceFinished(Leaf),
    /// An asset's bytes arrived and can now be read with
    /// [`Canopy::asset`](crate::Canopy::asset).
    AssetLoaded {
        key: AssetKey,
    },
    /// This element is gone -- pruned directly, or taken down with an ancestor. Terminal: the
    /// `Leaf` will never name anything again.
    Withered(Leaf),
    /// The window changed size, and with it possibly the breakpoint.
    Resized {
        viewport: Area<Logical>,
        layout: Layout,
        short: bool,
    },
    /// Scroll input a view turned away because [`ScrollAxes`](crate::ScrollAxes) has that axis
    /// switched off -- how far it *would* have moved, in logical pixels, on whichever axis
    /// refused it.
    ///
    /// This is what makes a locked axis usable rather than merely inert. Blocking on its own
    /// leaves an app with a dead region and nothing to offer in its place; reported, the app
    /// decides what the gesture meant. A vertically locked view can answer a wheel or a drag by
    /// turning a page, stepping a carousel, or changing a tab -- responses a continuous offset
    /// cannot express, reached by the input the reader already uses for "further down" rather
    /// than by a control put somewhere else on the screen to stand in for it.
    ///
    /// One per frame that carries refused input, so a drag arrives as a stream and a wheel notch
    /// as a single delta. It is a raw amount rather than a gesture: how much of it adds up to a
    /// step is the app's threshold to choose, because only the app knows what a step is.
    ///
    /// `method` is what makes that choosable. The two kinds of input are not the same shape and
    /// cannot share a rule: a wheel notch is already a discrete step and a reader who turns it
    /// once expects one thing to happen, while a drag is a continuous stream whose vertical
    /// component is mostly the tremor in a horizontal gesture. A single threshold serving both
    /// is either too low, and every pan judders, or too high, and a notch does nothing.
    ///
    /// The refusal still travels outward to an ancestor view unless
    /// [`OverscrollPropagation`](crate::OverscrollPropagation) is turned off, so an app meaning
    /// to own the gesture outright should turn that off too -- otherwise it acts *and* the
    /// region behind it scrolls.
    ScrollRefused {
        leaf: Leaf,
        delta: Position<Logical>,
        method: InteractionMethod,
    },
}

/// Where the funnel observers deposit emissions until the frame collects them.
#[derive(Resource, Default)]
pub(crate) struct Emissions(pub(crate) Vec<Bloom>);

impl Emissions {
    pub(crate) fn push(&mut self, bloom: Bloom) {
        self.0.push(bloom);
    }
}
