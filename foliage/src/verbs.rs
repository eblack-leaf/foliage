use crate::elevation::Elevation;
use crate::interaction::focus::Intent;
use crate::leaf::{Growth, Leaf};
use crate::op::Op;
use crate::palette::{Palette, Scheme};
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

    /// Makes an element inert without taking it out of the picture.
    ///
    /// It still draws -- a greyed control is still a control -- and it still occupies the box
    /// stack, so it **swallows**: a press on it reaches neither the element itself nor anything
    /// behind it, and a drag over it scrolls nothing. That is the whole difference between disabled
    /// and decoration, and it is what makes disabling a page enough on its own when a drawer opens
    /// over it: the page goes inert without a scrim to arrange.
    ///
    /// Cascades to everything grown under it, as a product recomputed every frame rather than a
    /// write pushed down -- so a child grown under a disabled element is disabled on its first
    /// frame. Focus leaves the subtree if it was in it.
    fn disable(&mut self, leaf: Leaf) {
        self.queue(Op::Disable {
            leaf,
            disabled: true,
        });
    }

    /// Undoes [`disable`](Grow::disable) on this element.
    ///
    /// Symmetric: an element inside the subtree that was disabled in its own right stays disabled,
    /// because what is recomputed is the product over the whole ancestry and not one inherited bit
    /// that was overwritten on the way down.
    fn enable(&mut self, leaf: Leaf) {
        self.queue(Op::Disable {
            leaf,
            disabled: false,
        });
    }

    /// Whether an element is drawn at all, as [`visible`](crate::Place::visible) states it at
    /// spawn.
    fn visible(&mut self, leaf: Leaf, visible: bool) {
        self.queue(Op::Reveal { leaf, visible });
    }

    /// How opaque an element is, as [`opacity`](crate::Place::opacity) states it at spawn.
    fn opacity(&mut self, leaf: Leaf, opacity: f32) {
        self.queue(Op::Fade { leaf, opacity });
    }

    /// Moves focus to an element.
    ///
    /// The ordinary way focus moves. It is not a byproduct of pressing anything: a press moves
    /// focus nowhere, so an app that wants a field focused when it is tapped writes that from
    /// [`clicked`](crate::Pollen::clicked), and the engine never guesses.
    ///
    /// Dropped if the element cannot take focus -- it is not
    /// [`interactive`](crate::Place::interactive), or it is hidden or disabled. Focus stays where
    /// it was rather than moving somewhere the app did not name.
    fn focus(&mut self, leaf: Leaf) {
        self.queue(Op::Focus(Intent::To(leaf)));
    }

    /// Takes focus off whatever holds it, leaving nothing focused.
    fn unfocus(&mut self) {
        self.queue(Op::Focus(Intent::Away));
    }

    /// Moves focus to the next element in reading order, wrapping within the scope focus is in.
    ///
    /// With nothing focused, this takes the first.
    fn focus_next(&mut self) {
        self.queue(Op::Focus(Intent::Next));
    }

    /// Moves focus to the previous element in reading order, wrapping within the scope focus is in.
    ///
    /// With nothing focused, this takes the last.
    fn focus_previous(&mut self) {
        self.queue(Op::Focus(Intent::Previous));
    }

    /// States what every [`Palette`] role resolves to, for the whole tree.
    ///
    /// The one write that names no element, because a role belongs to the scheme and not to any of
    /// the elements declaring it. Everything painted in a role whose color changed is re-extracted
    /// and nothing else is, which is what makes a theme one op rather than a walk.
    fn repaint(&mut self, scheme: Scheme) {
        self.queue(Op::Repaint(scheme));
    }
}

impl<T: Queues> Grow for T {}
