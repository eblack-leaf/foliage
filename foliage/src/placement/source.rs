//! Sources: where a number in a placement comes from.
//!
//! A source says nothing about what it is *for*. `20.px()` is neither a position nor a length until
//! a role names it, and the role is also what decides which axis it resolves on. [`content()`] is
//! the clearest case: in a width role it is the widest the element wants to be, and in a height
//! role it is how tall it turned out at the width the horizontal pass gave it.
//!
//! Four types carry that, and the split is the resolution model stated as types. The horizontal
//! axis resolves before the vertical one, so:
//!
//! - a [`Length`] is available to any role
//! - a [`VerticalLength`] is available only to a vertical role, because only the vertical pass
//!   knows the answer
//! - a coordinate never crosses axes at all, because a position on one axis has no reading on the
//!   other
//!
//! So `height(2.col())` is a two-column span used as a height, and `width(2.row())` does not
//! compile.

use core::ops::{Add, Mul, Neg, Sub};

use crate::coordinate::Axis;

/// One addend of an expression: a source, and the factor it was scaled by.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Term {
    pub(crate) scale: f32,
    pub(crate) kind: Kind,
}

impl Term {
    fn new(kind: Kind) -> Self {
        Self { scale: 1.0, kind }
    }
}

/// What a term reads.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Kind {
    /// Logical pixels, as written.
    Px(f32),
    /// A fraction of the parent's extent on the resolving axis, where `1.0` is the whole of it.
    Pct(f32),
    /// A one-based track index into the parent's grid on the named axis. Which edge of that track
    /// it means is the role's decision.
    Cell { index: i32, axis: Axis },
    /// A count of character cells on the resolving axis.
    Letters(f32),
    /// This element's own intrinsic extent on the resolving axis.
    Content,
    /// One edge of the element's anchor.
    AnchorEdge(Edge),
    /// One extent of the element's anchor.
    AnchorExtent(Axis),
}

/// Which edge of an anchor a term reads.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Edge {
    Left,
    Right,
    CenterX,
    Top,
    Bottom,
    CenterY,
}

/// A sum of scaled terms.
///
/// Every operator a source supports -- addition, subtraction, and scaling by a plain number --
/// keeps an expression linear, so this shape is total rather than a simplification. It is built
/// once, where the placement is written, and never allocates during resolution.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Expr {
    pub(crate) terms: Vec<Term>,
}

impl Expr {
    fn of(kind: Kind) -> Self {
        Self {
            terms: vec![Term::new(kind)],
        }
    }

    fn plus(mut self, other: Expr) -> Self {
        self.terms.extend(other.terms);
        self
    }

    fn minus(self, other: Expr) -> Self {
        self.plus(other.negated())
    }

    fn negated(self) -> Self {
        self.scaled(-1.0)
    }

    fn scaled(mut self, by: f32) -> Self {
        for term in &mut self.terms {
            term.scale *= by;
        }
        self
    }
}

/// Where a coordinate is measured from.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Origin {
    /// The near edge of the parent's box on the resolving axis, which is what a length placed in a
    /// position role is measured from.
    Parent,
    /// The surface, which is where an anchor's edges already are.
    Surface,
}

/// A position, with the axis already checked off by the role that took it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Coord {
    pub(crate) expr: Expr,
    pub(crate) origin: Origin,
}

impl From<HorizontalCoordinate> for Coord {
    fn from(coordinate: HorizontalCoordinate) -> Self {
        Self {
            expr: coordinate.expr,
            origin: coordinate.origin,
        }
    }
}

impl From<VerticalCoordinate> for Coord {
    fn from(coordinate: VerticalCoordinate) -> Self {
        Self {
            expr: coordinate.expr,
            origin: coordinate.origin,
        }
    }
}

/// A length: an extent, with no position of its own.
///
/// Produced by [`Source`] and by [`anchor().width()`](crate::Anchor::width). Legal in every role --
/// as a size directly, and as a position measured from the parent's near edge.
#[derive(Clone, Debug, PartialEq)]
pub struct Length(pub(crate) Expr);

/// A length only the vertical axis can answer.
///
/// Produced by [`row`](Source::row) and by [`anchor().height()`](crate::Anchor::height). The
/// horizontal axis resolves first and cannot see a vertical result, so these are legal in vertical
/// roles only. A [`Length`] converts into one, never the reverse.
///
/// ```compile_fail,E0277
/// use foliage::{Source, left};
/// left(0.px()).width(2.row());
/// ```
///
/// ```compile_fail,E0277
/// use foliage::{Source, anchor, left};
/// left(0.px()).width(anchor().height());
/// ```
///
/// The other direction is fine, and useful -- `height(2.col())` is a two-column span used as a
/// height.
#[derive(Clone, Debug, PartialEq)]
pub struct VerticalLength(pub(crate) Expr);

/// A position on the horizontal axis.
///
/// Either an anchor's edge, which is already a coordinate, or a [`Length`] measured from the
/// parent's left.
///
/// A position is not an extent, so an anchor's edge cannot be a size:
///
/// ```compile_fail,E0277
/// use foliage::{Source, anchor, left};
/// left(0.px()).width(anchor().left());
/// ```
///
/// Subtracting two of them is what gives the [`Length`] between them, which is how an anchor's
/// edges are used as a size. Adding two is not an operation:
///
/// ```compile_fail,E0308
/// use foliage::{Source, anchor, left};
/// left(anchor().left() + anchor().right()).width(10.px());
/// ```
///
/// A position on one axis has no reading on the other, so coordinates never cross:
///
/// ```compile_fail,E0277
/// use foliage::{Source, anchor, left};
/// left(anchor().bottom()).width(10.px());
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct HorizontalCoordinate {
    pub(crate) expr: Expr,
    pub(crate) origin: Origin,
}

/// A position on the vertical axis.
#[derive(Clone, Debug, PartialEq)]
pub struct VerticalCoordinate {
    pub(crate) expr: Expr,
    pub(crate) origin: Origin,
}

impl Length {
    pub(crate) fn of(kind: Kind) -> Self {
        Self(Expr::of(kind))
    }
}

impl VerticalLength {
    pub(crate) fn of(kind: Kind) -> Self {
        Self(Expr::of(kind))
    }
}

impl HorizontalCoordinate {
    pub(crate) fn anchored(edge: Edge) -> Self {
        Self {
            expr: Expr::of(Kind::AnchorEdge(edge)),
            origin: Origin::Surface,
        }
    }
}

impl VerticalCoordinate {
    pub(crate) fn anchored(edge: Edge) -> Self {
        Self {
            expr: Expr::of(Kind::AnchorEdge(edge)),
            origin: Origin::Surface,
        }
    }
}

/// Plain numbers as placement sources.
///
/// Every unit names itself at the call site, which is what keeps an expression readable when it
/// mixes them: `right(100.pct() - 16.px())`.
pub trait Source: Sized {
    /// Logical pixels.
    fn px(self) -> Length;

    /// A percentage of the parent's extent on the axis the role names. `100.pct()` is the whole of
    /// it, so `right(100.pct())` is the parent's right edge.
    fn pct(self) -> Length;

    /// A one-based column of the parent's grid.
    ///
    /// The role decides which part of that column is meant: a near role gives its left edge, a far
    /// role its right edge, a centre role its middle, and a size role the width of a span of that
    /// many columns, gaps included. So `left(1.col()).right(1.col())` is exactly the first column.
    fn col(self) -> Length;

    /// A one-based row of the parent's grid, read the same way [`col`](Source::col) is.
    fn row(self) -> VerticalLength;

    /// A count of character cells, at the element's own font size.
    ///
    /// The right tool whenever the count is genuinely known ahead of time: it costs nothing to
    /// resolve and it says what it means. Where the count is not known ahead of time,
    /// [`content()`] measures instead.
    fn letters(self) -> Length;
}

/// The element's own intrinsic extent, which is a different question per axis.
///
/// In a width role it is **max-content**: the widest the element wants to be, unwrapped. In a
/// monospaced font that is the character count times the cell width, so it is exact and free, and
/// it is available before any layout has happened.
///
/// In a height role it is what the element measured to *after* wrapping, at the width the
/// horizontal pass gave it. One word, and the axis supplies the question.
///
/// Under [`at_most`](crate::Horizontal::at_most) this is fit-content: the smaller of what the
/// content wants and what the ceiling allows.
///
/// An element with nothing in it has an intrinsic extent of zero.
pub fn content() -> Length {
    Length::of(Kind::Content)
}

macro_rules! source {
    ($($number:ty),*) => {$(
        impl Source for $number {
            fn px(self) -> Length {
                Length::of(Kind::Px(self as f32))
            }

            fn pct(self) -> Length {
                Length::of(Kind::Pct(self as f32 / 100.0))
            }

            fn col(self) -> Length {
                Length::of(Kind::Cell { index: self as i32, axis: Axis::Horizontal })
            }

            fn row(self) -> VerticalLength {
                VerticalLength::of(Kind::Cell { index: self as i32, axis: Axis::Vertical })
            }

            fn letters(self) -> Length {
                Length::of(Kind::Letters(self as f32))
            }
        }
    )*};
}

source!(i32, u32, f32, usize);

impl From<Length> for VerticalLength {
    fn from(length: Length) -> Self {
        Self(length.0)
    }
}

impl From<Length> for HorizontalCoordinate {
    fn from(length: Length) -> Self {
        Self {
            expr: length.0,
            origin: Origin::Parent,
        }
    }
}

impl From<Length> for VerticalCoordinate {
    fn from(length: Length) -> Self {
        Self {
            expr: length.0,
            origin: Origin::Parent,
        }
    }
}

impl From<VerticalLength> for VerticalCoordinate {
    fn from(length: VerticalLength) -> Self {
        Self {
            expr: length.0,
            origin: Origin::Parent,
        }
    }
}

macro_rules! length_arithmetic {
    ($name:ident) => {
        impl Add for $name {
            type Output = $name;
            fn add(self, rhs: $name) -> $name {
                $name(self.0.plus(rhs.0))
            }
        }

        impl Sub for $name {
            type Output = $name;
            fn sub(self, rhs: $name) -> $name {
                $name(self.0.minus(rhs.0))
            }
        }

        impl Mul<f32> for $name {
            type Output = $name;
            fn mul(self, rhs: f32) -> $name {
                $name(self.0.scaled(rhs))
            }
        }

        impl Neg for $name {
            type Output = $name;
            fn neg(self) -> $name {
                $name(self.0.negated())
            }
        }
    };
}

length_arithmetic!(Length);
length_arithmetic!(VerticalLength);

impl Add<VerticalLength> for Length {
    type Output = VerticalLength;
    fn add(self, rhs: VerticalLength) -> VerticalLength {
        VerticalLength(self.0.plus(rhs.0))
    }
}

impl Sub<VerticalLength> for Length {
    type Output = VerticalLength;
    fn sub(self, rhs: VerticalLength) -> VerticalLength {
        VerticalLength(self.0.minus(rhs.0))
    }
}

impl Add<Length> for VerticalLength {
    type Output = VerticalLength;
    fn add(self, rhs: Length) -> VerticalLength {
        VerticalLength(self.0.plus(rhs.0))
    }
}

impl Sub<Length> for VerticalLength {
    type Output = VerticalLength;
    fn sub(self, rhs: Length) -> VerticalLength {
        VerticalLength(self.0.minus(rhs.0))
    }
}

macro_rules! coordinate_arithmetic {
    ($name:ident, $($length:ident),*) => {$(
        impl Add<$length> for $name {
            type Output = $name;
            fn add(self, rhs: $length) -> $name {
                $name { expr: self.expr.plus(rhs.0), origin: self.origin }
            }
        }

        impl Sub<$length> for $name {
            type Output = $name;
            fn sub(self, rhs: $length) -> $name {
                $name { expr: self.expr.minus(rhs.0), origin: self.origin }
            }
        }

        impl Add<$name> for $length {
            type Output = $name;
            fn add(self, rhs: $name) -> $name {
                $name { expr: self.0.plus(rhs.expr), origin: rhs.origin }
            }
        }
    )*};
}

coordinate_arithmetic!(HorizontalCoordinate, Length);
coordinate_arithmetic!(VerticalCoordinate, Length, VerticalLength);

impl Sub for HorizontalCoordinate {
    type Output = Length;

    /// The distance between two positions on the axis, which is a length.
    fn sub(self, rhs: HorizontalCoordinate) -> Length {
        Length(self.expr.minus(rhs.expr))
    }
}

impl Sub for VerticalCoordinate {
    type Output = VerticalLength;

    /// The distance between two positions on the axis, which is a length.
    fn sub(self, rhs: VerticalCoordinate) -> VerticalLength {
        VerticalLength(self.expr.minus(rhs.expr))
    }
}
