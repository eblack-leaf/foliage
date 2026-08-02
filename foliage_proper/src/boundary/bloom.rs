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
