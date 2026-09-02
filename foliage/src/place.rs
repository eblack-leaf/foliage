//! Stating an element's placement when it is described, before it exists.

use bevy_ecs::component::Component;

use crate::elevation::Elevation;
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
    pub(crate) elevation: Option<Elevation>,
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
    /// An element that says nothing fills its parent.
    fn at(mut self, location: Location) -> Self {
        self.placement().location = Some(location);
        self
    }

    /// How this element's box is divided for the elements grown under it.
    ///
    /// Undeclared, it is a single column and a single row.
    fn grid(mut self, grid: Grid) -> Self {
        self.placement().grid = Some(grid);
        self
    }

    /// How far in front of its trunk the element sits, from [`Elevation::up`] or
    /// [`Elevation::down`].
    ///
    /// Undeclared, it sits at its trunk's own elevation, which leaves it just in front of it.
    ///
    /// On every element, not only on one that draws: what is grown under an element accumulates
    /// from its elevation, so a wrapper that carried none would flatten the subtree beneath it.
    fn elevate(mut self, elevation: Elevation) -> Self {
        self.placement().elevation = Some(elevation);
        self
    }

    /// The one other element this one's placement may read through [`anchor()`](crate::anchor).
    ///
    /// It carries every reading a trunk does, so an element grown away from what it describes --
    /// to clear a stack, or a clip -- goes on addressing that element's grid, box, font and measure
    /// in the same words.
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
