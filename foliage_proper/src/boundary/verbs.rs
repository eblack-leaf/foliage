use crate::boundary::leaf::Leaf;
use crate::boundary::op::{Motion, Op, Spec, Timing};
use crate::boundary::tween::{Channel, Tween};
use crate::coordinate::position::Position;
use crate::TextInputStyle;
use crate::{
    AssetKey, AssetSource, Color, Elevation, FontSize, GlyphColors, Location, Logical, Polygon,
    Rounding, ScrollTo,
};

/// The two things a command sink has to be able to do: take an op, and name a new element.
///
/// [`Canopy`](crate::Canopy) queues into the frame's own buffer; [`Sprig`](crate::Sprig)
/// queues into a shared one behind a lock. Everything else they can do is the same, and lives
/// on [`Grows`] rather than being written twice.
pub(crate) trait Queues {
    fn push(&mut self, op: Op);
    fn allocate(&self) -> Leaf;
}

/// Everything an app can ask the engine to do.
///
/// Carried identically by [`Canopy`](crate::Canopy) and [`Sprig`](crate::Sprig), so code that
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
    /// [`Bloom::Withered`](crate::Bloom::Withered) for each `Leaf` that goes.
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
    /// that sequence's [`Bloom::SequenceFinished`](crate::Bloom::SequenceFinished).
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
    /// finishes it emits [`Bloom::SequenceFinished`](crate::Bloom::SequenceFinished) -- the
    /// hook for chaining one stage of motion onto the next.
    fn sequence(&mut self) -> Leaf {
        let leaf = self.allocate();
        self.push(Op::Sequence(leaf));
        leaf
    }
    /// Emits [`Bloom::TimerFinished`](crate::Bloom::TimerFinished) once, `millis` from now.
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
    /// [`Bloom::Tween`](crate::Bloom::Tween) for you to apply however you like.
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
    /// announced by [`Bloom::AssetLoaded`](crate::Bloom::AssetLoaded).
    fn load_asset(&mut self, source: AssetSource) -> AssetKey {
        let key = crate::AssetLoader::generate_key();
        self.push(Op::LoadAsset { key, source });
        key
    }
}

impl<T: Queues> Grows for T {}
