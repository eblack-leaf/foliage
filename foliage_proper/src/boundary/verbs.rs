use crate::boundary::leaf::Leaf;
use crate::boundary::op::{Motion, Op, Spec, Timing};
use crate::boundary::tween::{Channel, Tween};
use crate::coordinate::position::Position;
use crate::{
    AssetKey, AssetSource, Color, Elevation, FontSize, GlyphColors, LineConstraint, Location,
    Logical, Polygon, Rounding, ScrollTo, Side,
};
use crate::{ImageView, TextInputStyle};

/// The two things a command sink has to be able to do: take an op, and name a new element.
///
/// [`Forest`](crate::Forest) queues into the frame's own buffer; [`Sprig`](crate::Sprig)
/// queues into a shared one behind a lock. Everything else they can do is the same, and lives
/// on [`Grows`] rather than being written twice.
pub(crate) trait Queues {
    fn push(&mut self, op: Op);
    fn allocate(&self) -> Leaf;
}

/// Everything an app can ask the engine to do.
///
/// Carried identically by [`Forest`](crate::Forest) and [`Sprig`](crate::Sprig), so code that
/// changes the tree reads the same whether it runs in the frame or on another thread. Sealed
/// -- `Queues` is `pub(crate)`, so this can be called but never implemented, which is what
/// keeps the set of things an app can do closed and reviewable.
#[allow(private_bounds)]
pub trait Grows: Queues {
    /// Grows a top-level element and hands back the [`Leaf`] naming it. Usable immediately,
    /// including as a parent in the same frame.
    fn leaf(&mut self, spec: impl Into<Spec>) -> Leaf {
        let leaf = self.allocate();
        self.push(Op::Grow {
            leaf,
            under: None,
            spec: spec.into(),
        });
        leaf
    }
    /// Grows an element under `under`.
    fn branch(&mut self, under: Leaf, spec: impl Into<Spec>) -> Leaf {
        let leaf = self.allocate();
        self.push(Op::Grow {
            leaf,
            under: Some(under),
            spec: spec.into(),
        });
        leaf
    }
    /// Removes an element and everything beneath it. Emits
    /// [`Moss::Withered`](crate::Moss::Withered) for each `Leaf` that goes.
    fn prune(&mut self, leaf: Leaf) {
        self.push(Op::Prune(leaf));
    }
    /// Re-enables interaction on an element and its subtree.
    fn enable(&mut self, leaf: Leaf) {
        self.push(Op::Enable(leaf));
    }
    /// Disables interaction on an element and its subtree. It still draws; it stops competing
    /// for input.
    fn disable(&mut self, leaf: Leaf) {
        self.push(Op::Disable(leaf));
    }
    /// Presses and releases the middle of an element's current section.
    ///
    /// A whole gesture rather than a state change: it is queued as real input, so the hit test
    /// picks whatever is genuinely on top at that point, focus reconciles as a side effect, the
    /// click observers fire in their usual order, and anything reading the pointer -- a text input
    /// placing its caret -- sees a position rather than nothing.
    ///
    /// This is the only way to move focus programmatically -- there is deliberately no way to
    /// hand an element the keyboard without also sending the press a real one would: nothing in
    /// the tree does anything on focus alone that a click doesn't already cover, and a text
    /// input's own caret placement/visibility runs *only* off the click path (`Engaged`), not off
    /// `Focused` -- so a focus-only op would have left the caret invisible and at column 0 with
    /// no way to place it. Landing on a child of what was named, or nothing at all if something
    /// else is over it, is the cost of that -- real input, not a shortcut around it.
    ///
    /// Queued, not immediate: the messages are read on the pass that follows, so the effect lands
    /// a frame later.
    fn click_on(&mut self, leaf: Leaf) {
        self.click_at(leaf, 0.5, 0.5);
    }
    /// The same, at a chosen point: `x` and `y` as fractions of the element's own section, from
    /// `0.0` at its left and top to `1.0` at its right and bottom.
    ///
    /// Which point is the caller's business, and has to be, because *where* a press lands is
    /// often the whole content of it. The middle is right for a button, which is why
    /// [`Self::click_on`] is that and is the one to reach for by default -- but it is wrong for a
    /// text field, where the click position becomes the caret position. Landing in the middle of a
    /// box puts the caret in the middle of whatever is written there; `(1.0, 0.5)` puts it after
    /// the last character, because a click past the end of a line clamps to the end of it.
    ///
    /// A fraction rather than an absolute position so it resolves against the section *as the op
    /// is applied*. A caller reading `section` itself is reading a frame ago, and something just
    /// built has no resolved section to read.
    fn click_at(&mut self, leaf: Leaf, x: f32, y: f32) {
        self.push(Op::Click { leaf, at: (x, y) });
    }
    /// Replaces a text element's contents.
    fn text(&mut self, leaf: Leaf, value: impl Into<String>) {
        self.push(Op::Text {
            leaf,
            value: value.into(),
        });
    }
    fn color(&mut self, leaf: Leaf, to: Color) {
        self.push(Op::Color { leaf, to });
    }
    fn opacity(&mut self, leaf: Leaf, to: f32) {
        self.push(Op::Opacity { leaf, to });
    }
    /// Shows or hides an element and everything beneath it. A hidden element keeps its state
    /// and its `Leaf`; it is skipped by drawing and hit-testing.
    fn visible(&mut self, leaf: Leaf, yes: bool) {
        self.push(Op::Visible { leaf, yes });
    }
    /// Moves and resizes an element.
    fn location(&mut self, leaf: Leaf, to: Location) {
        self.push(Op::Location { leaf, to });
    }
    /// Repoints which element `leaf`'s [`anchor()`](crate::anchor) values resolve against.
    ///
    /// The counterpart to [`anchored`](crate::Author::anchored), which could only be said once,
    /// at spawn. Nothing about the component was ever fixed -- it is a plain component with
    /// hooks that maintain the dependency both ways -- it simply had no verb, so an app that
    /// wanted a panel to follow a *different* element had to give up anchoring and compute
    /// coordinates by hand.
    ///
    /// Dropped if either end has withered, like every other op naming something gone.
    fn anchor(&mut self, leaf: Leaf, to: Leaf) {
        self.push(Op::Anchor { leaf, to });
    }
    fn elevation(&mut self, leaf: Leaf, to: Elevation) {
        self.push(Op::Elevation { leaf, to });
    }
    fn font_size(&mut self, leaf: Leaf, to: FontSize) {
        self.push(Op::FontSize { leaf, to });
    }
    /// Colors individual glyphs of a text element.
    fn glyph_colors(&mut self, leaf: Leaf, to: GlyphColors) {
        self.push(Op::GlyphColors { leaf, to });
    }
    /// Replaces a polyline's points.
    fn points(&mut self, leaf: Leaf, to: Vec<Position<Logical>>) {
        self.push(Op::Points { leaf, to });
    }
    /// How much of a polyline is drawn, 0.0..=1.0.
    fn draw_progress(&mut self, leaf: Leaf, to: f32) {
        self.push(Op::DrawProgress { leaf, to });
    }
    /// A polygon's shape: side count, corner rounding, rotation. All three are plain numbers,
    /// so driving them from a [`tween`](Grows::tween) morphs the shape.
    fn polygon(&mut self, leaf: Leaf, to: Polygon) {
        self.push(Op::Polygon { leaf, to });
    }
    /// A panel's corner-radius bracket.
    fn rounding(&mut self, leaf: Leaf, to: Rounding) {
        self.push(Op::Rounding { leaf, to });
    }
    /// Swaps which registered artwork an icon draws.
    fn icon(&mut self, leaf: Leaf, to: crate::IconId) {
        self.push(Op::Icon { leaf, to });
    }
    /// Tweens one of an element's own values.
    fn animate(&mut self, leaf: Leaf, to: Motion, timing: Timing) {
        self.push(Op::Animate {
            leaf,
            to,
            timing,
            sequence: None,
        });
    }
    /// [`animate`](Grows::animate), joined to a sequence so its completion counts toward
    /// that sequence's [`Moss::SequenceFinished`](crate::Moss::SequenceFinished).
    ///
    /// Entries keep their own timing and may overlap freely -- joining a sequence groups
    /// them, it does not order them.
    fn animate_during(&mut self, leaf: Leaf, to: Motion, timing: Timing, sequence: Leaf) {
        self.push(Op::Animate {
            leaf,
            to,
            timing,
            sequence: Some(sequence),
        });
    }
    /// Opens a sequence. Animations joined to it with
    /// [`animate_during`](Grows::animate_during) report completion, and once the last one
    /// finishes it emits [`Moss::SequenceFinished`](crate::Moss::SequenceFinished) -- the
    /// hook for chaining one stage of motion onto the next.
    fn sequence(&mut self) -> Leaf {
        let leaf = self.allocate();
        self.push(Op::Sequence(leaf));
        leaf
    }
    /// Emits [`Moss::TimerFinished`](crate::Moss::TimerFinished) once, `millis` from now.
    /// One-shot: repeating means starting another from the emission.
    fn timer(&mut self, millis: u64) -> Leaf {
        let leaf = self.allocate();
        self.push(Op::Timer { leaf, millis });
        leaf
    }
    /// A text input's placeholder, shown while it is empty.
    fn hint(&mut self, leaf: Leaf, text: impl Into<String>) {
        self.push(Op::Hint {
            leaf,
            text: text.into(),
        });
    }
    /// A text input's colors, rounding and outline, rewritten as one unit.
    fn input_style(&mut self, leaf: Leaf, style: TextInputStyle) {
        self.push(Op::InputStyle { leaf, style });
    }
    /// Scrolls a view, as a fraction of its scrollable range.
    fn scroll(&mut self, leaf: Leaf, to: ScrollTo) {
        self.push(Op::Scroll { leaf, to });
    }
    /// Names an element for later lookup.
    fn name(&mut self, leaf: Leaf, name: impl Into<String>) {
        self.push(Op::Name {
            leaf,
            name: name.into(),
        });
    }
    /// Tweens plain numbers on foliage's clock, reporting each frame's values as
    /// [`Moss::Tween`](crate::Moss::Tween) for you to apply however you like.
    ///
    /// `channels` is a start/end pair per number. Nothing is written anywhere -- this is the
    /// engine's easing and timing made available to values it has no concept of, which is
    /// what a library needs to build its own animatable properties.
    fn tween(
        &mut self,
        channels: impl IntoIterator<Item = impl Into<Channel>>,
        timing: Timing,
    ) -> Tween {
        let tween = Tween(self.allocate().0);
        self.push(Op::Tween {
            tween,
            channels: channels.into_iter().map(Into::into).collect(),
            timing,
        });
        tween
    }
    /// Starts loading an asset. The key is valid immediately; the bytes arrive later,
    /// announced by [`Moss::AssetLoaded`](crate::Moss::AssetLoaded).
    fn load_asset(&mut self, source: AssetSource) -> AssetKey {
        let key = crate::AssetLoader::generate_key();
        self.push(Op::LoadAsset { key, source });
        key
    }
    fn image_view(&mut self, leaf: Leaf, view: ImageView) {
        self.push(Op::ImageView { leaf, view });
    }
    fn rounding_side(&mut self, leaf: Leaf, side: Side) {
        self.push(Op::RoundingSide { leaf, side });
    }
    fn line_constraint(&mut self, leaf: Leaf, constraint: LineConstraint) {
        self.push(Op::LineConstraint { leaf, constraint });
    }
}

impl<T: Queues> Grows for T {}
