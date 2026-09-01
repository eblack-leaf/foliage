use crate::leaf::Leaf;
use crate::op::Op;
use crate::seed::Seed;

/// The two things an op sink has to be able to do: take an op, and name a new element.
pub(crate) trait Queues {
    fn queue(&mut self, op: Op);
    fn allocate(&self) -> Leaf;
}

/// Everything an app can ask the engine to do.
///
/// A change reads identically wherever it is issued. Sealed: it can be called, never implemented.
#[allow(private_bounds)]
pub trait Grow: Queues {
    /// Grows a top-level element and hands back the [`Leaf`] naming it. Usable immediately,
    /// including as a trunk in the same frame.
    fn plant(&mut self, seed: impl Seed) -> Leaf {
        let leaf = self.allocate();
        self.queue(Op::Plant {
            leaf,
            bud: seed.bud(),
        });
        leaf
    }

    /// Grows an element off `under`.
    fn branch(&mut self, under: Leaf, seed: impl Seed) -> Leaf {
        let leaf = self.allocate();
        self.queue(Op::Branch {
            leaf,
            under,
            bud: seed.bud(),
        });
        leaf
    }

    /// Takes an element and everything beneath it down. Each one is reported as
    /// [`withered`](crate::Pollen::withered).
    fn prune(&mut self, leaf: Leaf) {
        self.queue(Op::Prune(leaf));
    }
}

impl<T: Queues> Grow for T {}
