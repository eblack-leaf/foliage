//! Bases: the elements a placement may read, and the one vocabulary each is read in.
//!
//! An element resolves against three things -- itself, the trunk it was grown under, and the one
//! element it anchors to. [`trunk()`] and [`anchor()`] open the last two, and both carry the same
//! readings, so nothing is sayable about one that is not sayable about the other.
//!
//! The element's own two readings have no opener: its box is what is being solved for, so
//! [`content()`](crate::content) and [`letters`](crate::Source::letters) are the whole of what it
//! can ask about itself.

use crate::coordinate::Axis;
use crate::placement::source::{
    Against, Edge, HorizontalCoordinate, Kind, Length, Origin, VerticalCoordinate, VerticalLength,
};

/// The element this one was grown under.
///
/// The bare sources on [`Source`](crate::Source) already read the trunk, so this is for the two
/// readings they cannot spell -- [`content`](Trunk::content) and [`letters`](Trunk::letters), which
/// belong to the element that owns them and are not the same as the reader's own.
pub fn trunk() -> Trunk {
    Trunk
}

/// The one other element this one's placement may read.
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
/// It carries the whole vocabulary a trunk does, which is what makes growing an element elsewhere a
/// real move rather than a downgrade: an element that leaves its trunk to escape a stack or a clip
/// anchors back to it and goes on addressing the same grid.
pub fn anchor() -> Anchor {
    Anchor
}

/// What [`trunk()`] reads.
#[derive(Copy, Clone, Debug)]
pub struct Trunk;

/// What [`anchor()`] reads.
#[derive(Copy, Clone, Debug)]
pub struct Anchor;

macro_rules! basis {
    ($name:ident, $against:expr, $origin:expr) => {
        impl $name {
            /// Its left edge.
            pub fn left(self) -> HorizontalCoordinate {
                HorizontalCoordinate::edge(Edge::Left, $against)
            }

            /// Its right edge.
            pub fn right(self) -> HorizontalCoordinate {
                HorizontalCoordinate::edge(Edge::Right, $against)
            }

            /// Its horizontal midpoint.
            pub fn center_x(self) -> HorizontalCoordinate {
                HorizontalCoordinate::edge(Edge::CenterX, $against)
            }

            /// Its top edge.
            pub fn top(self) -> VerticalCoordinate {
                VerticalCoordinate::edge(Edge::Top, $against)
            }

            /// Its bottom edge.
            pub fn bottom(self) -> VerticalCoordinate {
                VerticalCoordinate::edge(Edge::Bottom, $against)
            }

            /// Its vertical midpoint.
            pub fn center_y(self) -> VerticalCoordinate {
                VerticalCoordinate::edge(Edge::CenterY, $against)
            }

            /// How wide it is.
            ///
            /// A length rather than a coordinate, so it is legal on either axis:
            /// `height(anchor().width())` is an element as tall as its anchor is wide.
            pub fn width(self) -> Length {
                Length::of(Kind::Extent {
                    axis: Axis::Horizontal,
                    against: $against,
                })
            }

            /// How tall it is.
            ///
            /// Vertical only, because the horizontal pass runs first and cannot read a height.
            pub fn height(self) -> VerticalLength {
                VerticalLength::of(Kind::Extent {
                    axis: Axis::Vertical,
                    against: $against,
                })
            }

            /// A one-based column of its grid, read the way [`col`](crate::Source::col) is.
            ///
            /// A position rather than a length, because a track of someone else's grid is somewhere
            /// on the surface rather than an offset into this element's own parent.
            pub fn col(self, index: i32) -> HorizontalCoordinate {
                HorizontalCoordinate::cell(index, $against, $origin)
            }

            /// A one-based row of its grid, read the way [`row`](crate::Source::row) is.
            pub fn row(self, index: i32) -> VerticalCoordinate {
                VerticalCoordinate::cell(index, $against, $origin)
            }

            /// A count of character cells in *its* font, which is not the reader's.
            ///
            /// An element composes in its own font, so [`letters`](crate::Source::letters) is
            /// always the reader's. This is the other one, and it is what a letter-pitched grid is
            /// measured in.
            pub fn letters(self, count: f32) -> Length {
                Length::of(Kind::Letters {
                    letters: count,
                    against: $against,
                })
            }

            /// Its measured intrinsic extent on the resolving axis.
            ///
            /// A different number from its box whenever it was given more room than it asked for,
            /// which is what makes "as wide as that label's text" sayable at all.
            pub fn content(self) -> Length {
                Length::of(Kind::Content { against: $against })
            }
        }
    };
}

basis!(Trunk, Against::Trunk, Origin::Trunk);
basis!(Anchor, Against::Anchor, Origin::Anchor);
