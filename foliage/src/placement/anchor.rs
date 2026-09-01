//! Reading another element's resolved box.

use crate::coordinate::Axis;
use crate::placement::source::{
    Edge, HorizontalCoordinate, Kind, Length, VerticalCoordinate, VerticalLength,
};

/// The element's anchor: the one other element its placement may read.
///
/// Set with [`anchored`](crate::Place::anchored) when the element is described, or with
/// [`anchor`](crate::Grow::anchor) at any time after. An element has at most one, and a placement
/// that reads an anchor it has not been given resolves against a zero box.
///
/// Anchoring reads one already-resolved box, which is cheap and exact, and is the right answer for
/// "sit below that thing, wherever it ended up" -- it holds up under wrapping and stacking where a
/// fixed offset does not. Sizing to content answers a different question, and neither substitutes
/// for the other.
///
/// Anchor edges are positions, so they are coordinates and never lengths. Subtracting two gives the
/// length between them; `width(anchor().left())` is refused, because an edge is not an extent.
pub fn anchor() -> Anchor {
    Anchor
}

/// What [`anchor()`] reads. Every method names one part of the anchor's box.
#[derive(Copy, Clone, Debug)]
pub struct Anchor;

impl Anchor {
    /// The anchor's left edge.
    pub fn left(self) -> HorizontalCoordinate {
        HorizontalCoordinate::anchored(Edge::Left)
    }

    /// The anchor's right edge.
    pub fn right(self) -> HorizontalCoordinate {
        HorizontalCoordinate::anchored(Edge::Right)
    }

    /// The horizontal midpoint of the anchor.
    pub fn center_x(self) -> HorizontalCoordinate {
        HorizontalCoordinate::anchored(Edge::CenterX)
    }

    /// The anchor's top edge.
    pub fn top(self) -> VerticalCoordinate {
        VerticalCoordinate::anchored(Edge::Top)
    }

    /// The anchor's bottom edge.
    pub fn bottom(self) -> VerticalCoordinate {
        VerticalCoordinate::anchored(Edge::Bottom)
    }

    /// The vertical midpoint of the anchor.
    pub fn center_y(self) -> VerticalCoordinate {
        VerticalCoordinate::anchored(Edge::CenterY)
    }

    /// How wide the anchor is.
    ///
    /// A length rather than a coordinate, so it is legal on either axis: `height(anchor().width())`
    /// is an element as tall as its anchor is wide.
    pub fn width(self) -> Length {
        Length::of(Kind::AnchorExtent(Axis::Horizontal))
    }

    /// How tall the anchor is.
    ///
    /// Vertical only, because the horizontal pass runs first and cannot read a height.
    pub fn height(self) -> VerticalLength {
        VerticalLength::of(Kind::AnchorExtent(Axis::Vertical))
    }
}
