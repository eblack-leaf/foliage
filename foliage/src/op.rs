use crate::leaf::Leaf;
use crate::stem::Stem;

/// One queued change.
pub(crate) enum Op {
    Plant { leaf: Leaf, bud: Bud },
    Branch { leaf: Leaf, under: Leaf, bud: Bud },
    Prune(Leaf),
}

/// An element formed and not yet open: what the queue carries between the call that described it
/// and the drain that grows it.
pub(crate) enum Bud {
    Stem(Stem),
}
