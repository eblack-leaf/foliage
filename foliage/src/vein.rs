use crate::coordinate::Section;
use crate::elevation::Elevation;
use crate::leaf::Leaf;
use crate::palette::Palette;
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
    /// The role the element is filled with, if it is something with a fill.
    Color,
    /// How the element's corners are rounded, if it is something with corners.
    Rounding,
}

/// What a [`Vein`] draws out.
#[derive(Clone, PartialEq, Debug)]
#[non_exhaustive]
pub enum Sap {
    Leaves(Vec<Leaf>),
    Leaf(Option<Leaf>),
    Section(Section),
    Elevation(Elevation),
    Color(Palette),
    Rounding(Corners),
}
