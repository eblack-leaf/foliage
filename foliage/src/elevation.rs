//! Elevation -- where an element sits in the one stack.

use bevy_ecs::component::Component;

/// How far in front of its trunk an element sits.
///
/// Always relative, so raising a card carries its whole subtree with it and nothing inside it is
/// rewritten. Undeclared, an element sits at its trunk's own elevation and is separated from it by
/// the tie-break alone -- which puts it in front, because it was allocated later.
///
/// There is no form that states a layer outright and ignores its ancestry. Such a value has to be
/// chosen against every other one in the program, and two composites that pick the same number
/// collide with nothing to arbitrate between them but a tie-break neither can see. Relative
/// elevation accumulates through the tree, so two elements can only tie if they are structurally
/// related -- which is where the tie-break means something and where the code that grew them can
/// separate them.
///
/// An element that has to clear the stack it was grown in is grown somewhere else instead: the
/// trunk decides what takes an element down and what it stacks among, and
/// [`anchored`](crate::Place::anchored) decides where it sits, so a dropdown planted at top level
/// still tracks the control that opened it.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Elevation(i32);

impl Elevation {
    /// `steps` in front of its trunk.
    pub fn up(steps: i32) -> Self {
        Self(steps)
    }

    /// `steps` behind its trunk.
    ///
    /// Zero has no direction, so `down(0)` and `up(0)` are the same elevation: level with the trunk.
    pub fn down(steps: i32) -> Self {
        Self(-steps)
    }

    /// This elevation resolved against a trunk that resolved to `trunk`.
    pub(crate) fn accumulate(self, trunk: i32) -> i32 {
        trunk.saturating_add(self.0)
    }
}

/// Where an element sits in the one stack: its elevation accumulated down the tree, and the order
/// its name was allocated in.
///
/// Ordered back to front, so the largest is the front-most. The second field is what separates two
/// elements that accumulated to the same elevation, and it settles that case totally rather than
/// arbitrarily: allocation order is monotonic, and a prune cannot disturb it because a name is
/// never reused.
///
/// It is deliberately not a dense rank. An accumulated elevation changes only when the element's own
/// declaration or an ancestor's does, so growing an element leaves every other element's value
/// alone -- and extraction, which compares values, therefore has nothing to send for any of them.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ResolvedElevation {
    pub(crate) stack: i32,
    pub(crate) growth: u64,
}
