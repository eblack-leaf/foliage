use crate::leaf::Leaf;
use crate::op::Op;
use crate::placement::grid::Grid;
use crate::placement::location::Location;
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
    #[track_caller]
    fn plant(&mut self, seed: impl Seed) -> Leaf {
        let leaf = self.allocate();
        self.queue(Op::Plant {
            leaf,
            bud: seed.bud(core::panic::Location::caller()),
        });
        leaf
    }

    /// Grows an element off `under`.
    #[track_caller]
    fn branch(&mut self, under: Leaf, seed: impl Seed) -> Leaf {
        let leaf = self.allocate();
        self.queue(Op::Branch {
            leaf,
            under,
            bud: seed.bud(core::panic::Location::caller()),
        });
        leaf
    }

    /// Takes an element and everything beneath it down. Each one is reported as
    /// [`withered`](crate::Pollen::withered).
    fn prune(&mut self, leaf: Leaf) {
        self.queue(Op::Prune(leaf));
    }

    /// Moves an element, replacing its whole placement.
    ///
    /// A placement is one value rather than a set of edges, so there is no half-written state
    /// between two of these and no question of which edge a later write meant.
    fn at(&mut self, leaf: Leaf, location: impl Into<Location>) {
        self.queue(Op::Place {
            leaf,
            location: location.into(),
        });
    }

    /// Redivides an element's box for the elements grown under it.
    fn grid(&mut self, leaf: Leaf, grid: Grid) {
        self.queue(Op::Divide { leaf, grid });
    }

    /// Points an element's placement at the one other element it may read.
    ///
    /// Replaces any anchor it already had.
    ///
    /// # Panics
    ///
    /// If the anchor would close a cycle, on the same terms as
    /// [`anchored`](crate::Place::anchored).
    #[track_caller]
    fn anchor(&mut self, leaf: Leaf, to: Leaf) {
        self.queue(Op::Anchor {
            leaf,
            to,
            at: core::panic::Location::caller(),
        });
    }
}

impl<T: Queues> Grow for T {}
