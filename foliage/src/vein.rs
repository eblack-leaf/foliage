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
}

/// What a [`Vein`] draws out.
#[derive(Clone, PartialEq, Debug)]
#[non_exhaustive]
pub enum Sap {
    Leaves(Vec<Leaf>),
    Leaf(Option<Leaf>),
}
