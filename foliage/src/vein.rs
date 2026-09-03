use crate::coordinate::{Area, Position, Section};
use crate::elevation::Elevation;
use crate::leaf::Leaf;
use crate::palette::Fill;
use crate::rounding::Corners;

/// Exactly what a frame may read of an element.
///
/// Exhaustive by construction: if it is not here, an app cannot see it. Everything an app can
/// declare is here, because a value you can set and cannot read back is a value you have to keep a
/// copy of.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum Vein {
    /// The elements branched directly off this one.
    Branches,
    /// The element this one was branched off, if any.
    Trunk,
    /// Where the layout put the element, before any scrolling moved it.
    ///
    /// What a containing view's extent is measured from, and what to read when the question is
    /// where content sits rather than where it currently appears.
    Placed,
    /// The element's box less every scrolling ancestor's offset: where it is on screen, which is
    /// what was drawn and what a hit test runs against.
    ///
    /// The two differ only inside a view that has been scrolled.
    Drawn,
    /// The one other element this one's placement may read, if it has been given one.
    Anchor,
    /// How far in front of its trunk the element was told to sit.
    ///
    /// What was declared, not where it ended up: the resolved value accumulates its whole ancestry
    /// and carries a tie-break, and neither is a number an app has any use for.
    Elevation,
    /// What the element is filled with, if it is something with a fill: the role it declared, or
    /// the color it named outright.
    ///
    /// What was declared, not what it currently paints as. A role's color is the
    /// [`Scheme`](crate::Scheme)'s answer and is resolved at extraction, so there is no resolved
    /// color held anywhere for this to report.
    Color,
    /// How the element's corners are rounded, if it is something with corners.
    Rounding,
    /// What the element says, if it is a run of glyphs.
    ///
    /// What was written, not how it wrapped: where the lines fell is a function of the box the
    /// layout gave it, and the box is [`Drawn`](Vein::Drawn).
    Text,
    /// Whether the app has hidden the element.
    ///
    /// What was declared of this element, not the product over its ancestry: an element inside a
    /// hidden subtree reads visible, because that is what it says about itself and what showing its
    /// ancestor again would leave it as.
    Visible,
    /// How opaque the element was told to be, on the same terms as [`Visible`](Vein::Visible).
    ///
    /// While a [`Motion::Opacity`](crate::Motion::Opacity) is running this is where it has reached,
    /// because a blend of two numbers is a number and is written back over the declaration. That is
    /// what is on screen, which is what a read is for.
    Opacity,
    /// Whether the element was disabled in its own right, on the same terms as
    /// [`Visible`](Vein::Visible).
    Disabled,
    /// How far a scrolling region has been moved from its content's origin, in logical pixels.
    ///
    /// The same unit it is written in, so reading it back after a
    /// [`scroll`](crate::Grow::scroll) returns the pixels it settled at. `None` on anything that
    /// does not scroll: an element with no scrolling axis has no offset, rather than an offset of
    /// zero it can never leave.
    Offset,
    /// How far a scrolling region's content reaches, from its own near edges.
    ///
    /// What a scrollbar's thumb is a fraction of. An axis the region does not scroll reads the
    /// region's own box, which is the same statement as having no extent on it.
    Extent,
    /// How far through its range a scrolling region sits, per axis, in `0.0..=1.0`.
    ///
    /// Derived from the two above rather than held, and read-only for that reason: pixels are the
    /// one unit an offset is written in. A region with nowhere to go reads zero on that axis.
    Progress,
}

/// What a [`Vein`] draws out.
#[derive(Clone, PartialEq, Debug)]
#[non_exhaustive]
pub enum Sap {
    Leaves(Vec<Leaf>),
    Leaf(Option<Leaf>),
    Section(Section),
    /// Logical pixels, like every other coordinate.
    Position(Position),
    Area(Area),
    /// A fraction on each axis, in `0.0..=1.0`. Held apart from a [`Position`](Sap::Position) even
    /// though it is the same pair of numbers, so a read cannot quietly take a fraction for pixels.
    Progress(Position),
    Elevation(Elevation),
    Color(Fill),
    Rounding(Corners),
    Text(String),
    Visible(bool),
    Opacity(f32),
    Disabled(bool),
}
