use crate::op::Bud;
use crate::stem::Stem;

/// An element described before it exists, and what [`plant`](crate::Grow::plant) and
/// [`branch`](crate::Grow::branch) consume.
///
/// A [`Stem`] is a `Seed`, and so is every other element foliage provides. Sealed: the set of
/// things that can be grown is closed and reviewable.
#[allow(private_bounds)]
pub trait Seed: Buds {}

impl<T: Buds> Seed for T {}

/// Forming the [`Bud`] the queue carries.
pub(crate) trait Buds {
    fn bud(self) -> Bud;
}

impl Buds for Stem {
    fn bud(self) -> Bud {
        Bud::Stem(self)
    }
}
