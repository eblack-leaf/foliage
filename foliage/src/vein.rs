use crate::coordinate::Section;
use crate::leaf::Leaf;

/// Exactly what a frame may read of an element.
///
/// Exhaustive by construction: if it is not here, an app cannot see it.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum Vein {
    /// The elements branched directly off this one.
    Branches,
    /// The element this one was branched off, if any.
    Trunk,
    /// Where the element appears: its resolved box, less every scrolling ancestor's offset. This
    /// is what was drawn, and what a hit test runs against.
    Section,
    /// Where the layout put the element, before any scrolling moved it. The two differ only inside
    /// a view that has been scrolled.
    LayoutSection,
    /// The one other element this one's placement may read, if it has been given one.
    Anchor,
}

/// What a [`Vein`] draws out.
#[derive(Clone, PartialEq, Debug)]
#[non_exhaustive]
pub enum Sap {
    Leaves(Vec<Leaf>),
    Leaf(Option<Leaf>),
    Section(Section),
}
