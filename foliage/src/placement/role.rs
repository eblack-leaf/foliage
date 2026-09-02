//! Roles: what a source means for this element.
//!
//! A role is written first, because it says what the value is describing before the value has to be
//! parsed. `left(anchor().right())` is read in the order it is understood.
//!
//! Each opener returns a type carrying only the completions that are legal with it, so an axis has
//! exactly four spellings and no illegal pairing can be written at all. `left(..).center_x(..)` and
//! `width(..).width(..)` are not rejected -- the method is not there.

use crate::placement::source::{
    Coord, Expr, HorizontalCoordinate, Length, VerticalCoordinate, VerticalLength,
};

/// How one axis is pinned down: two of its coordinates, and any clamp on the extent between them.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Config {
    pub(crate) form: Form,
    pub(crate) least: Option<Expr>,
    pub(crate) most: Option<Expr>,
}

impl Config {
    fn new(form: Form) -> Self {
        Self {
            form,
            least: None,
            most: None,
        }
    }
}

/// The four legal ways to pin an axis down.
///
/// Near is left or top, far is right or bottom, and extent is width or height.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Form {
    NearExtent { near: Coord, extent: Expr },
    NearFar { near: Coord, far: Coord },
    FarExtent { far: Coord, extent: Expr },
    CenterExtent { center: Coord, extent: Expr },
}

/// One axis of a placement, complete: the horizontal half.
///
/// Complete, so there is nothing left to say about the axis. A second extent has no method to be
/// written with:
///
/// ```compile_fail,E0599
/// use foliage::{Source, left};
/// left(0.px()).width(10.px()).width(20.px());
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Horizontal(pub(crate) Config);

/// One axis of a placement, complete: the vertical half.
///
/// A distinct type from [`Horizontal`], so the two cannot be given in the wrong order:
///
/// ```compile_fail,E0308
/// use foliage::{Location, Source, left, top};
/// Location::new().xs(top(0.px()).height(10.px()), left(0.px()).width(10.px()));
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Vertical(pub(crate) Config);

impl Horizontal {
    /// Refuses to resolve narrower than this.
    ///
    /// Named for the bound rather than `min`, because `width(content()).min(300)` puts two
    /// unrelated senses of the word in one expression: one meaning the content's natural size, the
    /// other a floor.
    pub fn at_least(mut self, extent: impl Into<Length>) -> Self {
        self.0.least = Some(extent.into().0);
        self
    }

    /// Refuses to resolve wider than this. Over [`content()`](crate::content) this is fit-content:
    /// whichever of the two is smaller.
    pub fn at_most(mut self, extent: impl Into<Length>) -> Self {
        self.0.most = Some(extent.into().0);
        self
    }
}

impl Vertical {
    /// Refuses to resolve shorter than this.
    pub fn at_least(mut self, extent: impl Into<VerticalLength>) -> Self {
        self.0.least = Some(extent.into().0);
        self
    }

    /// Refuses to resolve taller than this.
    pub fn at_most(mut self, extent: impl Into<VerticalLength>) -> Self {
        self.0.most = Some(extent.into().0);
        self
    }
}

/// The element's left edge, as a coordinate in its parent's space.
///
/// Complete it with a [`width`](Left::width) or with the [`right`](Left::right) edge.
pub fn left(coordinate: impl Into<HorizontalCoordinate>) -> Left {
    Left(coordinate.into().into())
}

/// The element's right edge, as a coordinate in its parent's space.
///
/// Every edge role is a coordinate, including this one, so sixteen in from the right is
/// `right(100.pct() - 16.px())`. Insets would read more nicely for that one case, but anchor
/// sources are already positions, so an inset would put two coordinate spaces in one expression.
pub fn right(coordinate: impl Into<HorizontalCoordinate>) -> Right {
    Right(coordinate.into().into())
}

/// The element's horizontal midpoint.
pub fn center_x(coordinate: impl Into<HorizontalCoordinate>) -> CenterX {
    CenterX(coordinate.into().into())
}

/// The element's top edge, as a coordinate in its parent's space.
pub fn top(coordinate: impl Into<VerticalCoordinate>) -> Top {
    Top(coordinate.into().into())
}

/// The element's bottom edge, as a coordinate in its parent's space.
pub fn bottom(coordinate: impl Into<VerticalCoordinate>) -> Bottom {
    Bottom(coordinate.into().into())
}

/// The element's vertical midpoint.
pub fn center_y(coordinate: impl Into<VerticalCoordinate>) -> CenterY {
    CenterY(coordinate.into().into())
}

/// A [`left`] awaiting the value that completes the axis.
///
/// It carries only the two completions that are legal with it, so an illegal pairing is not
/// rejected -- it cannot be written:
///
/// ```compile_fail,E0599
/// use foliage::{Source, left};
/// left(0.px()).center_x(50.pct());
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Left(Coord);

/// A [`right`] awaiting the value that completes the axis.
///
/// A far edge takes an extent. The near-and-far pair is spelled from the near edge, so each of the
/// four forms has exactly one spelling:
///
/// ```compile_fail,E0599
/// use foliage::{Source, right};
/// right(100.pct()).left(0.px());
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Right(Coord);

/// A [`center_x`] awaiting the value that completes the axis.
#[derive(Clone, Debug, PartialEq)]
pub struct CenterX(Coord);

/// A [`top`] awaiting the value that completes the axis.
#[derive(Clone, Debug, PartialEq)]
pub struct Top(Coord);

/// A [`bottom`] awaiting the value that completes the axis.
#[derive(Clone, Debug, PartialEq)]
pub struct Bottom(Coord);

/// A [`center_y`] awaiting the value that completes the axis.
#[derive(Clone, Debug, PartialEq)]
pub struct CenterY(Coord);

impl Left {
    /// How wide the element is, measured rightward from its left edge.
    pub fn width(self, extent: impl Into<Length>) -> Horizontal {
        Horizontal(Config::new(Form::NearExtent {
            near: self.0,
            extent: extent.into().0,
        }))
    }

    /// The element's right edge, stretching it between the two coordinates.
    pub fn right(self, coordinate: impl Into<HorizontalCoordinate>) -> Horizontal {
        Horizontal(Config::new(Form::NearFar {
            near: self.0,
            far: coordinate.into().into(),
        }))
    }
}

impl Right {
    /// How wide the element is, measured leftward from its right edge.
    pub fn width(self, extent: impl Into<Length>) -> Horizontal {
        Horizontal(Config::new(Form::FarExtent {
            far: self.0,
            extent: extent.into().0,
        }))
    }
}

impl CenterX {
    /// How wide the element is, spread either side of its midpoint.
    pub fn width(self, extent: impl Into<Length>) -> Horizontal {
        Horizontal(Config::new(Form::CenterExtent {
            center: self.0,
            extent: extent.into().0,
        }))
    }
}

impl Top {
    /// How tall the element is, measured downward from its top edge.
    pub fn height(self, extent: impl Into<VerticalLength>) -> Vertical {
        Vertical(Config::new(Form::NearExtent {
            near: self.0,
            extent: extent.into().0,
        }))
    }

    /// The element's bottom edge, stretching it between the two coordinates.
    pub fn bottom(self, coordinate: impl Into<VerticalCoordinate>) -> Vertical {
        Vertical(Config::new(Form::NearFar {
            near: self.0,
            far: coordinate.into().into(),
        }))
    }
}

impl Bottom {
    /// How tall the element is, measured upward from its bottom edge.
    pub fn height(self, extent: impl Into<VerticalLength>) -> Vertical {
        Vertical(Config::new(Form::FarExtent {
            far: self.0,
            extent: extent.into().0,
        }))
    }
}

impl CenterY {
    /// How tall the element is, spread either side of its midpoint.
    pub fn height(self, extent: impl Into<VerticalLength>) -> Vertical {
        Vertical(Config::new(Form::CenterExtent {
            center: self.0,
            extent: extent.into().0,
        }))
    }
}
