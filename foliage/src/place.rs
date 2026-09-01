//! Stating an element's placement when it is described, before it exists.

use bevy_ecs::component::Component;

use crate::leaf::Leaf;
use crate::placement::grid::Grid;
use crate::placement::location::Location;

/// Where the caller was standing. Carried from the call that wrote a placement to the drain that
/// applies it, so a refusal names the write rather than the pass that noticed it.
pub(crate) type Caller = &'static core::panic::Location<'static>;

/// What a seed carries about where it will sit. Anything left unsaid takes its default.
#[derive(Clone, Debug, Default)]
pub(crate) struct Placement {
    pub(crate) location: Option<Location>,
    pub(crate) grid: Option<Grid>,
    pub(crate) anchor: Option<Anchored>,
}

/// The one other element a placement may read, and where it was named.
#[derive(Component, Copy, Clone, Debug)]
pub(crate) struct Anchored {
    pub(crate) to: Leaf,
    pub(crate) at: Caller,
}

/// Reaching a seed's placement.
pub(crate) trait Places {
    fn placement(&mut self) -> &mut Placement;
}

/// Stating where an element will sit.
///
/// Implemented by every [`Seed`](crate::Seed), so placement reads identically whatever is being
/// grown. Sealed: it can be called, never implemented.
#[allow(private_bounds)]
pub trait Place: Places + Sized {
    /// Where the element sits.
    ///
    /// Takes one [`Horizontal`](crate::Horizontal) and one [`Vertical`](crate::Vertical) for the
    /// common case, or a [`Location`] when the placement changes with the breakpoint.
    ///
    /// An element that says nothing fills its parent.
    fn at(mut self, location: impl Into<Location>) -> Self {
        self.placement().location = Some(location.into());
        self
    }

    /// How this element's box is divided for the elements grown under it.
    ///
    /// Undeclared, it is a single column and a single row.
    fn grid(mut self, grid: Grid) -> Self {
        self.placement().grid = Some(grid);
        self
    }

    /// The one other element this one's placement may read through [`anchor()`](crate::anchor).
    ///
    /// # Panics
    ///
    /// If the anchor would close a cycle. A ↔ B is a contradiction rather than an ordering problem:
    /// an element whose placement cannot resolve has no box, so there is no state for it to fall
    /// back to. It is refused when the op is applied, naming both elements and the write that did
    /// it, which leaves the tree acyclic by construction.
    #[track_caller]
    fn anchored(mut self, to: Leaf) -> Self {
        self.placement().anchor = Some(Anchored {
            to,
            at: core::panic::Location::caller(),
        });
        self
    }
}

impl<T: Places> Place for T {}
