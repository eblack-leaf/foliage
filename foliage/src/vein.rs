use crate::coordinate::{Area, Position, Section};
use crate::elevation::Elevation;
use crate::icon::Field;
use crate::image::{Fit, Plate};
use crate::leaf::Leaf;
use crate::line::Cap;
use crate::palette::Fill;
use crate::polygon::Shape;
use crate::rounding::Corners;

/// Exactly what a frame may read of an element.
///
/// Exhaustive by construction: if it is not here, an app cannot see it. Everything an app can
/// declare is here, because a value you can set and cannot read back is a value you have to keep a
/// copy of.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
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
    /// Where a stroke's two ends landed, which is what [`Drawn`](Vein::Drawn) cannot say.
    ///
    /// A box is the rectangle around them grown by half the weight, and a rectangle has two
    /// diagonals -- so the box says how much room the stroke takes and this says where it runs.
    /// `None` on anything placed by a box.
    ///
    /// Resolved rather than declared, on the same terms as `Drawn`: the two ends as the layout
    /// answered them and scrolling moved them, not the grammar they were written in.
    Ends,
    /// How thick a stroke is drawn, in logical pixels. `None` on anything that is not a stroke.
    Weight,
    /// How a stroke's ends are finished.
    Cap,
    /// What a regular polygon looks like: its sides, its rounding and its rotation.
    ///
    /// While a [`Motion::Polygon`](crate::Motion::Polygon) is running this is where it has reached,
    /// because a blend of two shapes is a shape and is written back over the declaration -- the same
    /// standing [`Opacity`](Vein::Opacity) has.
    Shape,
    /// Which registered mark an icon draws.
    Mark,
    /// Which registered picture an image draws.
    ///
    /// The name only. Whether its pixels have arrived is not readable and is deliberately not a
    /// question an app has to ask: an element drawing a picture that has not loaded occupies its box
    /// and draws nothing, and appears when the pixels do.
    Picture,
    /// How an image's pixels are fitted into its box.
    Fit,
    /// How the element's corners are rounded, if it is something with corners.
    Rounding,
    /// What the element says, if it is a run of glyphs.
    ///
    /// What was written, not how it wrapped: where the lines fell is a function of the box the
    /// layout gave it, and the box is [`Drawn`](Vein::Drawn).
    Text,
    /// What a [`TextInput`](crate::TextInput) has selected, in characters of its value.
    ///
    /// Empty where nothing is selected, and its one end is then where the caret is -- so a caret
    /// and a selection are one read rather than two that could disagree.
    /// `None` on anything that is not a field.
    Selection,
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
    /// A set of elements, in the order the tree holds them. What [`Branches`](Vein::Branches)
    /// answers.
    Leaves(Vec<Leaf>),
    /// One element, or none where there is nothing to name. What [`Trunk`](Vein::Trunk) and
    /// [`Anchor`](Vein::Anchor) answer.
    Leaf(Option<Leaf>),
    /// A box. What [`Placed`](Vein::Placed) and [`Drawn`](Vein::Drawn) answer.
    Section(Section),
    /// Logical pixels, like every other coordinate.
    Position(Position),
    /// An extent, in logical pixels.
    Area(Area),
    /// A fraction on each axis, in `0.0..=1.0`. Held apart from a [`Position`](Sap::Position) even
    /// though it is the same pair of numbers, so a read cannot quietly take a fraction for pixels.
    Progress(Position),
    /// How far in front of its trunk an element was told to sit.
    Elevation(Elevation),
    /// What an element is filled with: the role it declared, or the color it named outright.
    Color(Fill),
    /// The radius bracket on each of the four corners.
    Rounding(Corners),
    /// A stroke's two ends, in the order they were written.
    Ends(Position, Position),
    /// A stroke's thickness, in logical pixels.
    Weight(f32),
    /// How a stroke's ends are finished.
    Cap(Cap),
    /// A regular polygon's sides, rounding and rotation.
    Shape(Shape),
    /// Which registered mark an icon draws.
    Mark(Field),
    /// Which registered picture an image draws.
    Picture(Plate),
    /// How an image's pixels are fitted into its box.
    Fit(Fit),
    /// What a run of glyphs says.
    Text(String),
    /// A span of a value, in characters. Empty means a caret at its own position.
    Selection(core::ops::Range<usize>),
    /// Whether the app has hidden the element.
    Visible(bool),
    /// How opaque the element was told to be, in `0.0..=1.0`.
    Opacity(f32),
    /// Whether the element was disabled in its own right.
    Disabled(bool),
}
