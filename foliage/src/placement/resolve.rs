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

use crate::aspen::blend;
use crate::coordinate::{Area, Axis, Section};
use crate::placement::grid::Tracks;
use crate::placement::role::{Config, Form};
use crate::placement::source::{Against, Coord, Edge, Expr, Kind, Origin};

/// Everything one element offers a placement that reads it.
///
/// One shape per element rather than a field per reading, so a term names which element it is
/// asking and every element answers the same questions. It is what stops the grammar being able to
/// describe a trunk and not an anchor.
#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct Basis {
    /// Its box: the edges a coordinate reads, and the extent a percentage is a fraction of.
    pub(crate) section: Section,
    /// Its measured extent, for [`content`](crate::content).
    pub(crate) intrinsic: Area,
    /// Its grid, at the breakpoint in force, for a track index.
    pub(crate) tracks: Tracks,
    /// Its character cell, for a count of letters and for a letter-pitched track. An element with
    /// no font has none.
    pub(crate) cell: Area,
}

/// Everything one axis of one element resolves against.
///
/// The trunk's box is complete here even during the horizontal pass, where its vertical half is
/// not yet known. Nothing can read that half: a vertical source cannot enter a horizontal role,
/// which is what the [`VerticalLength`](crate::VerticalLength) type is for.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Context {
    /// Which axis is being resolved.
    pub(crate) axis: Axis,
    /// The element itself. Only its measured extent and its character cell are readable -- its box
    /// is the answer being computed.
    pub(crate) own: Basis,
    /// The element it was grown under, and what it fills when it says nothing.
    pub(crate) trunk: Basis,
    /// The one other element it may read. Zero throughout when it has no anchor.
    pub(crate) anchor: Basis,
}

impl Context {
    /// The element a term is reading.
    fn basis(&self, against: Against) -> &Basis {
        match against {
            Against::Own => &self.own,
            Against::Trunk => &self.trunk,
            Against::Anchor => &self.anchor,
        }
    }
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

    /// One axis of a box that is already settled.
    ///
    /// What an endpoint that is a snapshot rather than a placement offers: it has no configuration
    /// to resolve, because it is already an answer.
    pub(crate) fn of(section: Section, axis: Axis) -> Self {
        match axis {
            Axis::Horizontal => Self {
                near: section.left(),
                far: section.right(),
            },
            Axis::Vertical => Self {
                near: section.top(),
                far: section.bottom(),
            },
        }
    }

    /// A fraction `at` of the way from this span to `other`.
    ///
    /// Both edges move together, which is the same thing as moving the near edge and the extent:
    /// the blend is affine, so there is no third reading of it to disagree with.
    pub(crate) fn blend(self, other: Self, at: f32) -> Self {
        Self {
            near: blend(self.near, other.near, at),
            far: blend(self.far, other.far, at),
        }
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

/// A position on the axis: one origin, and a sum of deltas measured from it.
///
/// The origin is whichever element the coordinate was opened against, and an edge supplies its own
/// -- it is already a position on the surface, so nothing is added to it.
fn coordinate(coordinate: &Coord, role: Role, context: &Context) -> f32 {
    let origin = match coordinate.origin {
        Origin::Trunk => near_edge(context.trunk.section, context.axis),
        Origin::Anchor => near_edge(context.anchor.section, context.axis),
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
        Kind::Pct { fraction, against } => {
            fraction * extent_of(context.basis(against).section, context.axis)
        }
        Kind::Extent { axis, against } => extent_of(context.basis(against).section, axis),
        Kind::Cell {
            index,
            axis,
            against,
        } => cell(index, axis, role, context.basis(against)),
        Kind::Letters { letters, against } => {
            letters * extent_of_area(context.basis(against).cell, context.axis)
        }
        Kind::Content { against } => extent_of_area(context.basis(against).intrinsic, context.axis),
        Kind::Edge { edge, against } => {
            let section = context.basis(against).section;
            match edge {
                Edge::Left => section.left(),
                Edge::Right => section.right(),
                Edge::CenterX => section.center().x,
                Edge::Top => section.top(),
                Edge::Bottom => section.bottom(),
                Edge::CenterY => section.center().y,
            }
        }
    }
}

/// A one-based track index into `basis`'s grid, read as the role asks.
///
/// A near role gives the track's near edge and a far role its far edge, so a pair of them is the
/// track itself and `n.col()` in a size role is a span of `n` tracks with the gaps between them.
///
/// The grid belongs to `basis`, so its extent and its character cell come from `basis` too -- a
/// letter-pitched track is in the font of the element the grid is on, not in the font of whatever
/// is addressing it.
fn cell(index: i32, axis: Axis, role: Role, basis: &Basis) -> f32 {
    let track = basis.tracks.on(axis);
    let size = track.size(
        extent_of(basis.section, axis),
        extent_of_area(basis.cell, axis),
    );
    let index = index as f32;
    let inclusive = matches!(role, Role::Far | Role::Extent);
    let edge = (index - if inclusive { 0.0 } else { 1.0 }) * size + (index - 1.0) * track.gap;
    edge + if role == Role::Center {
        size / 2.0
    } else {
        0.0
    }
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
