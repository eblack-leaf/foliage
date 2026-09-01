//! The resolver: declared placement in, one axis of geometry out.
//!
//! A pure function. No world, no entity names, no mutation -- given the same inputs it gives the
//! same answer, and it may be called as many times per element per frame as anything needs. Two
//! things depend on that:
//!
//! - an animating element resolves *both* of its endpoints in one context and interpolates the
//!   results, so nothing about the motion can go stale
//! - the horizontal and vertical passes are the same code called twice, which is only safe because
//!   there is no accumulated state for the second call to corrupt
//!
//! It is also why the whole placement algebra is testable as arithmetic: a struct in, a struct out,
//! with no engine anywhere near it.

use crate::coordinate::{Area, Axis, Section};
use crate::placement::grid::Tracks;
use crate::placement::role::{Config, Form};
use crate::placement::source::{Coord, Edge, Expr, Kind, Origin};

/// Everything one axis of one element resolves against.
///
/// The parent's box is complete here even during the horizontal pass, where its vertical half is
/// not yet known. Nothing can read that half: a vertical source cannot enter a horizontal role,
/// which is what the [`VerticalLength`](crate::VerticalLength) type is for.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Context {
    /// Which axis is being resolved.
    pub(crate) axis: Axis,
    /// The parent's box -- what a grid divides, and what a percentage is a fraction of.
    pub(crate) parent: Section,
    /// The anchored element's box. Zero when the element has no anchor.
    pub(crate) anchor: Section,
    /// This element's own intrinsic extent: max-content on the horizontal pass, and what it
    /// measured to at the resolved width on the vertical one.
    pub(crate) intrinsic: Area,
    /// The parent's grid, at the breakpoint in force.
    pub(crate) tracks: Tracks,
    /// This element's own character cell, for [`letters`](crate::Source::letters). An element
    /// with no font has none.
    pub(crate) cell: Area,
    /// The *parent's* character cell, for a letter-pitched track. The grid belongs to the parent,
    /// so its pitch is in the parent's font, not in the font of the children addressing it. That
    /// is what makes a column a real address into a letter-pitched grid rather than a hand-computed
    /// offset.
    pub(crate) parent_cell: Area,
}

/// One axis of a resolved box.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub(crate) struct Span {
    pub(crate) near: f32,
    pub(crate) far: f32,
}

impl Span {
    pub(crate) fn extent(&self) -> f32 {
        self.far - self.near
    }
}

/// What a source is being asked for, which is what decides how it reads.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Role {
    /// The left or top edge.
    Near,
    /// The right or bottom edge.
    Far,
    /// The midpoint.
    Center,
    /// The width or height.
    Extent,
}

/// Resolves one axis.
pub(crate) fn resolve(config: &Config, context: &Context) -> Span {
    let (near, far) = match &config.form {
        Form::NearExtent { near, extent } => {
            let near = coordinate(near, Role::Near, context);
            (near, near + clamped(extent, config, context))
        }
        Form::NearFar { near, far } => {
            let near = coordinate(near, Role::Near, context);
            let far = coordinate(far, Role::Far, context);
            let extent = clamp(far - near, config, context);
            (near, near + extent)
        }
        Form::FarExtent { far, extent } => {
            let far = coordinate(far, Role::Far, context);
            (far - clamped(extent, config, context), far)
        }
        Form::CenterExtent { center, extent } => {
            let center = coordinate(center, Role::Center, context);
            let extent = clamped(extent, config, context);
            (center - extent / 2.0, center + extent / 2.0)
        }
    };
    Span { near, far }
}

/// A position on the axis. A length in a position role is measured from the parent's near edge;
/// an anchor's edge is already a position and is measured from nothing.
fn coordinate(coordinate: &Coord, role: Role, context: &Context) -> f32 {
    let origin = match coordinate.origin {
        Origin::Parent => near_edge(context.parent, context.axis),
        Origin::Surface => 0.0,
    };
    origin + value(&coordinate.expr, role, context)
}

fn clamped(extent: &Expr, config: &Config, context: &Context) -> f32 {
    clamp(value(extent, Role::Extent, context), config, context)
}

/// Applies the declared bounds, ceiling first so a floor always wins, and refuses to hand back a
/// negative extent.
fn clamp(extent: f32, config: &Config, context: &Context) -> f32 {
    let mut extent = extent;
    if let Some(most) = &config.most {
        extent = extent.min(value(most, Role::Extent, context));
    }
    if let Some(least) = &config.least {
        extent = extent.max(value(least, Role::Extent, context));
    }
    extent.max(0.0)
}

fn value(expr: &Expr, role: Role, context: &Context) -> f32 {
    expr.terms
        .iter()
        .map(|term| term.scale * source(term.kind, role, context))
        .sum()
}

fn source(kind: Kind, role: Role, context: &Context) -> f32 {
    match kind {
        Kind::Px(px) => px,
        Kind::Pct(fraction) => fraction * extent_of(context.parent, context.axis),
        Kind::Cell { index, axis } => cell(index, axis, role, context),
        Kind::Letters(letters) => letters * extent_of_area(context.cell, context.axis),
        Kind::Content => extent_of_area(context.intrinsic, context.axis),
        Kind::AnchorEdge(edge) => match edge {
            Edge::Left => context.anchor.left(),
            Edge::Right => context.anchor.right(),
            Edge::CenterX => context.anchor.center().x,
            Edge::Top => context.anchor.top(),
            Edge::Bottom => context.anchor.bottom(),
            Edge::CenterY => context.anchor.center().y,
        },
        Kind::AnchorExtent(axis) => extent_of(context.anchor, axis),
    }
}

/// A one-based track index, read as the role asks.
///
/// A near role gives the track's near edge and a far role its far edge, so a pair of them is the
/// track itself and `n.col()` in a size role is a span of `n` tracks with the gaps between them.
fn cell(index: i32, axis: Axis, role: Role, context: &Context) -> f32 {
    let track = context.tracks.on(axis);
    let size = track.size(
        extent_of(context.parent, axis),
        extent_of_area(context.parent_cell, axis),
    );
    let index = index as f32;
    let inclusive = matches!(role, Role::Far | Role::Extent);
    let edge = (index - if inclusive { 0.0 } else { 1.0 }) * size + (index - 1.0) * track.gap;
    edge + if role == Role::Center { size / 2.0 } else { 0.0 }
}

fn near_edge(section: Section, axis: Axis) -> f32 {
    match axis {
        Axis::Horizontal => section.left(),
        Axis::Vertical => section.top(),
    }
}

fn extent_of(section: Section, axis: Axis) -> f32 {
    extent_of_area(section.area, axis)
}

fn extent_of_area(area: Area, axis: Axis) -> f32 {
    match axis {
        Axis::Horizontal => area.width,
        Axis::Vertical => area.height,
    }
}
