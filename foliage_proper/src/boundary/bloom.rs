use crate::boundary::leaf::Leaf;
use crate::boundary::tween::Tween;
use crate::coordinate::area::Area;
use crate::{AssetKey, Key, Layout, Logical, Modifiers, PhysicalKey, TextInputAction};
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
    /// The gesture on this element passed the drag threshold.
    ///
    /// TODO: it does not. This is sent for *every* pointer move while this element holds the
    /// gesture, from the first pixel -- the send in `interaction::interactive_elements`'
    /// `moved` handling sits outside the `past_drag` check, which gates only the
    /// `ViewAdjustment` that pans a view. So on a mouse, where a press almost always carries
    /// some movement, nearly every gesture reports as a drag, and the threshold this line
    /// names is nowhere in what an app receives.
    ///
    /// The threshold is real and does decide something an app can see -- whether the release
    /// also produces [`Clicked`](Bloom::Clicked) -- but nothing announces it being crossed.
    /// An app that wants "this has become a drag" has to measure travel itself, per axis,
    /// against a constant it has to know: [`InteractionListener::DRAG_THRESHOLD`].
    ///
    /// Two ways out, and they are not equivalent:
    ///
    /// 1. *rename it.* Keep the per-move stream and say so -- "the pointer moved while this
    ///    element held the gesture". Truthful, costs nothing, and leaves the threshold
    ///    unobservable.
    /// 2. *move the send inside `past_drag`.* Makes the variant mean what it already claims,
    ///    at the cost of the per-move stream -- which is what a knob or a slider driving its
    ///    own drag actually consumes, so that stream needs somewhere to go before this is
    ///    safe.
    ///
    /// Doing both is likely right: this variant for the threshold, and a separate per-move
    /// report for the stream. Whichever, the doc and the send site have to agree -- an app
    /// written against this sentence today gets different behaviour than it asked for.
    Dragged(Leaf),
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
}

/// Where the funnel observers deposit emissions until the frame collects them.
#[derive(Resource, Default)]
pub(crate) struct Emissions(pub(crate) Vec<Bloom>);

impl Emissions {
    pub(crate) fn push(&mut self, bloom: Bloom) {
        self.0.push(bloom);
    }
}
