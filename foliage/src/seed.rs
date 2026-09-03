use crate::elm::{Chlorophyll, PanelPigment, Pigment};
use crate::op::Bud;
use crate::panel::Panel;
use crate::place::{Caller, Places};
use crate::stem::Stem;

/// An element described before it exists, and what [`plant`](crate::Grow::plant) and
/// [`branch`](crate::Grow::branch) consume.
///
/// A [`Stem`] is a `Seed`, and so is every other element foliage provides. Sealed: the set of
/// things that can be grown is closed and reviewable.
#[allow(private_bounds)]
pub trait Seed: Buds + Places {}

impl<T: Buds + Places> Seed for T {}

/// Forming the [`Bud`] the queue carries.
pub(crate) trait Buds {
    fn bud(self, at: Caller) -> Bud;
}

impl Buds for Stem {
    fn bud(self, at: Caller) -> Bud {
        Bud {
            // A stem carries no renderer, which is the whole of what makes it one, and so nothing
            // for a renderer to have been told.
            chlorophyll: Chlorophyll::None,
            pigment: None,
            lettering: None,
            placement: self.placement,
            at,
        }
    }
}

impl Buds for Panel {
    fn bud(self, at: Caller) -> Bud {
        Bud {
            chlorophyll: Chlorophyll::Panel,
            pigment: Some(Pigment::Panel(PanelPigment {
                fill: self.fill,
                rounding: self.rounding,
            })),
            lettering: None,
            placement: self.placement,
            at,
        }
    }
}
