use crate::leaf::Leaf;
use crate::place::{Caller, Placement};
use crate::placement::grid::Grid;
use crate::placement::location::Location;

/// One queued change.
pub(crate) enum Op {
    Plant { leaf: Leaf, bud: Bud },
    Branch { leaf: Leaf, under: Leaf, bud: Bud },
    Prune(Leaf),
    Place { leaf: Leaf, location: Location },
    Divide { leaf: Leaf, grid: Grid },
    Anchor { leaf: Leaf, to: Leaf, at: Caller },
}

/// An element formed and not yet open: what the queue carries between the call that described it
/// and the drain that grows it.
pub(crate) struct Bud {
    pub(crate) sown: Sown,
    pub(crate) placement: Placement,
    pub(crate) at: Caller,
}

/// Which kind of element a bud opens into.
pub(crate) enum Sown {
    /// An element that draws nothing. It carries no renderer, which is the whole of what makes it
    /// one.
    Stem,
}
