//! Line -- a straight stroke between two points.

use bevy_ecs::component::Component;

use crate::color::Color;
use crate::coordinate::{Axis, Position};
use crate::elm::{Chlorophyll, Pigment};
use crate::op::Bud;
use crate::palette::{Fill, Palette};
use crate::place::{Caller, Placement, Places};
use crate::placement::point::Point;
use crate::seed::Buds;

/// The thinnest stroke that means anything, in logical pixels.
///
/// Zero and negative weights have no drawing to do, and clamping once here is cheaper than every
/// callsite checking. The shader feathers a sub-pixel line rather than dropping it, so one logical
/// pixel is a hairline on any display rather than a line that disappears on some of them.
pub const HAIRLINE: f32 = 1.0;

/// A straight segment of fixed weight between two points.
///
/// Placed by its ends rather than by a box: a rule, a connector and a chart's series are all two
/// positions and a thickness, and a rectangle is the wrong shape to say that in. The two ends are
/// [`Point`]s in the ordinary placement grammar, so an end can sit on a grid track, half way across
/// its trunk, or at an anchor's edge -- and the line follows when any of those move.
///
/// ```no_run
/// # use foliage::{Line, Palette, Place, Point, Source};
/// Line::new()
///     .weight(2.0)
///     .color(Palette::Muted)
///     .between(Point::new(0.px(), 0.px()), Point::new(100.pct(), 0.px()));
/// ```
///
/// Its box is the rectangle around the two ends, grown by half its weight on every side -- so a
/// horizontal rule, whose ends share a `y`, still has a box to be clipped, ranked and hit-tested by.
#[derive(Clone, Debug, Default)]
pub struct Line {
    pub(crate) placement: Placement,
    pub(crate) fill: Fill,
    pub(crate) cap: Cap,
}

impl Line {
    /// A muted hairline, from its trunk's top-left corner to itself.
    ///
    /// [`between`](Line::between) is what gives it somewhere to go: a line that says nothing is a
    /// line of no length rather than one filling its trunk, because two ends have no reading that
    /// corresponds to "the whole of the parent's box".
    pub fn new() -> Self {
        Self {
            placement: Placement::default(),
            fill: Fill::Role(Palette::Muted),
            cap: Cap::default(),
        }
    }

    /// Where the two ends are.
    pub fn between(mut self, from: Point, to: Point) -> Self {
        self.placement.traced = Some(Traced { from, to });
        self
    }

    /// How thick the stroke is, in logical pixels. Clamped to [`HAIRLINE`].
    ///
    /// Placement rather than decoration, because it is what the line's box is grown by: two ends on
    /// one line describe a rectangle of no height until the weight says otherwise.
    pub fn weight(mut self, weight: f32) -> Self {
        self.placement.stroke = Some(Stroke::new(weight));
        self
    }

    /// What the stroke is filled with: a [`Palette`] role, or a [`Color`] stated outright.
    /// Undeclared, it is [`Palette::Muted`] -- the role a rule and a division are drawn in.
    pub fn color(mut self, fill: impl Into<Fill>) -> Self {
        self.fill = fill.into();
        self
    }

    /// How the stroke's two ends are finished. Undeclared, they are [`Cap::Butt`].
    pub fn cap(mut self, cap: Cap) -> Self {
        self.cap = cap;
        self
    }
}

/// How a stroke's ends are finished.
///
/// # A round cap is what makes a chain of strokes a path
///
/// A stroke is a rectangle, so two of them meeting at an angle leave a wedge open on the outside of
/// the turn. [`Round`](Cap::Round) closes it with nothing added: each stroke's cap is a half-disc of
/// half the weight centred on the shared point, so the two halves meeting there are one disc of
/// exactly the radius the gap needs. A path is then its segments and nothing else -- no joint to
/// place, nothing to keep in step with the strokes it joins, and no element count that grows twice.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Cap {
    /// Square, ending exactly on the point. What a rule and a division want: an end that reached
    /// half a weight past where it was told to stop would not line up with anything.
    #[default]
    Butt,
    /// Half-round, reaching half a weight past the point. What a path wants, and what a single
    /// stroke wants when it is a mark rather than a boundary.
    Round,
}

impl Places for Line {
    fn placement(&mut self) -> &mut Placement {
        &mut self.placement
    }
}

impl Buds for Line {
    fn bud(mut self, at: Caller) -> Bud {
        // A line always has a weight, because its box is derived from one. Everything else about a
        // placement stays absent when it was not stated.
        self.placement.stroke.get_or_insert_default();
        Bud {
            chlorophyll: Chlorophyll::Line,
            pigment: Some(Pigment::Line(LinePigment {
                fill: self.fill,
                cap: self.cap,
            })),
            placement: self.placement,
            at,
            ..Bud::bare()
        }
    }
}

/// Where a stroked element's two ends are, in place of a box.
///
/// Content rather than renderer state, in the same sense a run's [`Lettering`](crate::text::Lettering)
/// is: R2a and R2b resolve it into the element's box, so it is read by passes that have nothing to
/// do with drawing.
#[derive(Component, Clone, Debug, PartialEq)]
pub(crate) struct Traced {
    pub(crate) from: Point,
    pub(crate) to: Point,
}

/// Where a stroke's two ends actually landed, in the space [`Drawn`](crate::rowan::Drawn) is in.
///
/// Settled by R2b beside the box and moved by R4 beside it, because it is resolved geometry rather
/// than a declaration -- the same standing `Drawn` has, and for the same reason.
///
/// The box cannot stand in for it. A box is the rectangle around the two ends grown by half the
/// weight, and a rectangle has two diagonals: which of them the stroke runs along is a fact about
/// the ends that the rectangle does not carry, and it depends on where the ends resolved rather
/// than on how they were written.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq)]
pub(crate) struct Stretched {
    pub(crate) from: Position,
    pub(crate) to: Position,
}

impl Stretched {
    /// Writes one axis of both ends, which is what one pass of the resolver answers.
    pub(crate) fn set(&mut self, axis: Axis, from: f32, to: f32) {
        self.from = self.from.set(axis, from);
        self.to = self.to.set(axis, to);
    }

    /// Both ends moved by a scrolling ancestor's accumulated offset.
    pub(crate) fn less(self, offset: Position) -> Self {
        Self {
            from: Position::new(self.from.x - offset.x, self.from.y - offset.y),
            to: Position::new(self.to.x - offset.x, self.to.y - offset.y),
        }
    }
}

/// How thick a stroked element is drawn.
#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub(crate) struct Stroke {
    pub(crate) weight: f32,
}

impl Stroke {
    pub(crate) fn new(weight: f32) -> Self {
        Self {
            weight: weight.max(HAIRLINE),
        }
    }

    /// How far past its two ends the stroke reaches, which is what its box is grown by.
    pub(crate) fn half(self) -> f32 {
        self.weight / 2.0
    }
}

impl Default for Stroke {
    fn default() -> Self {
        Self::new(HAIRLINE)
    }
}

/// What the line renderer was told.
///
/// Grown alongside [`Chlorophyll::Line`] and by nothing else, so an element carries both or neither.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq)]
pub(crate) struct LinePigment {
    pub(crate) fill: Fill,
    pub(crate) cap: Cap,
}

/// One line, as extraction states it: two ends on the surface, a weight, a cap and a colour.
///
/// Deliberately *not* the form the vertex buffer takes. An axis-aligned stroke is put on whole
/// device pixels so that a one-pixel rule is one lit row rather than two half-lit ones, and only the
/// backend knows how large a device pixel is. So this is what [`Elm`](crate::elm) compares, and the
/// snapped form is derived where the density is known, exactly as a glyph's ink is.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct LineInstance {
    pub(crate) from: Position,
    pub(crate) to: Position,
    pub(crate) color: Color,
    pub(crate) weight: f32,
    pub(crate) cap: Cap,
}
