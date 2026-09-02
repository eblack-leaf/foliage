use crate::elevation::Elevation;
use crate::leaf::{Growth, Leaf};
use crate::op::Op;
use crate::palette::Palette;
use crate::placement::grid::Grid;
use crate::placement::location::Location;
use crate::rounding::Corners;
use crate::seed::Seed;

/// The two things an op sink has to be able to do: take an op, and name a new element.
///
/// Naming one hands back its place in allocation order along with the name, because that order is
/// fixed where the name is asked for and not where the element is grown.
pub(crate) trait Queues {
    fn queue(&mut self, op: Op);
    fn allocate(&self) -> (Leaf, Growth);
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
        let (leaf, growth) = self.allocate();
        self.queue(Op::Plant {
            leaf,
            growth,
            bud: seed.bud(core::panic::Location::caller()),
        });
        leaf
    }

    /// Grows an element off `under`.
    #[track_caller]
    fn branch(&mut self, under: Leaf, seed: impl Seed) -> Leaf {
        let (leaf, growth) = self.allocate();
        self.queue(Op::Branch {
            leaf,
            growth,
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
    fn at(&mut self, leaf: Leaf, location: Location) {
        self.queue(Op::Place { leaf, location });
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

    /// Raises or lowers an element, and everything grown under it with it.
    ///
    /// Elevation accumulates down the tree, so this moves a whole subtree by one write and nothing
    /// inside it is touched.
    fn elevate(&mut self, leaf: Leaf, elevation: Elevation) {
        self.queue(Op::Elevate { leaf, elevation });
    }

    /// Refills an element.
    ///
    /// Dropped, like any op naming something it does not apply to, if the element draws nothing.
    fn color(&mut self, leaf: Leaf, color: Palette) {
        self.queue(Op::Recolor { leaf, color });
    }

    /// Rounds an element's corners, per corner or all at once.
    fn round(&mut self, leaf: Leaf, rounding: impl Into<Corners>) {
        self.queue(Op::Round {
            leaf,
            rounding: rounding.into(),
        });
    }
}

impl<T: Queues> Grow for T {}
