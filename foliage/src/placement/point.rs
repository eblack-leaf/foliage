//! Points: the placement grammar, read as a position rather than as a box.
//!
//! A [`Line`](crate::Line) has no box to state. What it has is two ends, and an end is a position
//! on each axis -- which is exactly what a [`Coord`] already is. So a point is one coordinate per
//! axis in the grammar every other placement is written in, and it resolves through the same pure
//! resolver, in the same context, in the same two passes.
//!
//! That is the whole of the point mode. Nothing here is a second way to describe geometry: `px`,
//! `pct`, `col`, `letters`, `content()`, `anchor()` and every arithmetic on them read identically,
//! so a line's end can sit on a grid track, half way across its trunk, or at an anchor's edge.

use crate::placement::role::known;
use crate::placement::source::{Coord, HorizontalCoordinate, Origin, VerticalCoordinate};

/// A position stated in the placement grammar: one coordinate on each axis.
///
/// Read by whatever states its geometry as vertices rather than as a rectangle. The two halves are
/// the same [`HorizontalCoordinate`] and [`VerticalCoordinate`] a box's edges take, which is what
/// keeps one grammar rather than two.
///
/// A value rather than a role, so it is constructed rather than opened. The bare grammar openers --
/// [`left`](crate::left), [`top`](crate::top) and the rest -- are free functions because each names
/// what a value *describes* before the value is read, and returns something the grammar has yet to
/// complete. A point describes nothing and is already complete.
#[derive(Clone, Debug, PartialEq)]
pub struct Point {
    pub(crate) x: Coord,
    pub(crate) y: Coord,
}

impl Point {
    /// A position, from one coordinate on each axis.
    ///
    /// The horizontal half is written first, as it is everywhere else in the grammar, and the two
    /// are distinct types -- so the pair cannot be given the wrong way round:
    ///
    /// ```compile_fail,E0277
    /// use foliage::{Point, Source, left, top};
    /// Point::new(top(0.px()), left(0.px()));
    /// ```
    ///
    /// Bare lengths read the trunk, exactly as they do in a box's placement:
    ///
    /// ```no_run
    /// # use foliage::{Point, Source, anchor};
    /// Point::new(0.px(), 0.px());
    /// Point::new(50.pct(), 100.pct() - 8.px());
    /// Point::new(2.col(), anchor().bottom());
    /// ```
    pub fn new(x: impl Into<HorizontalCoordinate>, y: impl Into<VerticalCoordinate>) -> Self {
        Self {
            x: x.into().into(),
            y: y.into().into(),
        }
    }

    /// This point's horizontal half, as the grammar states a coordinate.
    ///
    /// What puts a box *at* a point: a marker on a series, a round joint filling the wedge two
    /// strokes leave open where they meet. Without it the same expression is written twice -- once
    /// for the point and once for the box on it -- and the two are free to drift apart.
    ///
    /// Safe in the direction it is written and in no other: the halves were separated by
    /// [`new`](Point::new) from values that were already typed, so putting one back where it came
    /// from cannot cross the axes.
    pub fn x(&self) -> HorizontalCoordinate {
        HorizontalCoordinate {
            expr: self.x.expr.clone(),
            origin: self.x.origin,
        }
    }

    /// This point's vertical half, as the grammar states a coordinate.
    pub fn y(&self) -> VerticalCoordinate {
        VerticalCoordinate {
            expr: self.y.expr.clone(),
            origin: self.y.origin,
        }
    }

    /// Whether this point's vertical half can be answered before any vertical box has resolved, and
    /// so whether an element ending here counts toward the measure of the element it is grown under.
    ///
    /// The same question [`Config::measurable`](crate::placement::role::Config::measurable) asks of
    /// a box, asked of a vertex: a point that reads a vertical extent is asking how tall something
    /// else is, and cannot be what decides how tall this is.
    pub(crate) fn measurable(&self) -> bool {
        self.y.origin != Origin::Anchor && self.y.expr.terms.iter().all(|term| known(term.kind))
    }
}
