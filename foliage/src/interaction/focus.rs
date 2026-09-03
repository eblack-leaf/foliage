//! Focus -- a first-class surface, not a byproduct of clicking.
//!
//! # Focus is a verb
//!
//! [`focus`](crate::Grow::focus) moves it, and nothing else does. A press moves nothing: the engine
//! never infers from a gesture where the keyboard should be, because the element a person pressed
//! and the element they want to type into are different questions and only the app knows when they
//! coincide. An app that wants press-to-focus writes it in one line from
//! [`clicked`](crate::Pollen::clicked), and one that wants a popover to close when something else is
//! pressed asks about the [`Stem`](crate::Stem) it put behind its content.
//!
//! # Order is derived, and overridable
//!
//! Focus order is **reading order** -- top to bottom, left to right -- over the elements that
//! declared [`interactive`](crate::Place::interactive) inside the current scope.
//! [`focus_order`](crate::Place::focus_order) pulls one element earlier or later where a layout's
//! meaning differs from its geometry; everything left unstated keeps reading order among its
//! equals, so an override moves one element and renumbers nothing.
//!
//! # Scopes trap
//!
//! A drawer or a dialog declares itself a [`focus_scope`](crate::Place::focus_scope), and while
//! focus is inside it, stepping cycles within it. Without this, keyboard navigation inside an
//! overlay walks off into the page behind it, which is why most apps are mouse-only in practice.
//!
//! # Nothing is drawn
//!
//! foliage reports focus through [`Pollen`](crate::Pollen) and draws nothing. A focused element may
//! be a [`Stem`](crate::Stem) with no visible part at all, so there is no mark the engine could
//! draw that would be right.

use core::cmp::Ordering;

use tracing::debug;

use crate::grove::Grove;
use crate::leaf::Leaf;

/// What the app asked focus to do, before it is answered against resolved geometry.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Intent {
    /// To one named element.
    To(Leaf),
    /// Nowhere.
    Away,
    /// The next element in order, wrapping at the end of the scope.
    Next,
    /// The previous one.
    Previous,
}

/// Who holds focus, and what has been asked of it.
#[derive(Default)]
pub(crate) struct Focus {
    held: Option<Leaf>,
    /// Asked for at the drain and answered at settle, because both the order focus steps through
    /// and whether a target can take it at all are read from geometry this frame resolves. That is
    /// what lets an app open a drawer and focus into it in the same frame.
    pending: Option<Intent>,
}

impl Focus {
    /// Who holds it, as of the last settle.
    pub(crate) fn held(&self) -> Option<Leaf> {
        self.held
    }

    /// Takes what the drain applied. A second ask in one frame replaces the first, in arrival
    /// order, like every other write.
    pub(crate) fn ask(&mut self, intent: Intent) {
        self.pending = Some(intent);
    }
}

/// Step 7. Where focus ends up, once the frame's geometry and inherited state are settled.
pub(crate) fn settle(grove: &mut Grove) {
    let intent = grove.focus.pending.take();
    let was = grove.focus.held;
    // Whatever held focus may have withered, been hidden or been disabled since the last frame.
    // Keyboard input arriving at an element that cannot act on it is a dead app, and it is
    // invisible to any test that uses a pointer.
    let held = was.filter(|leaf| focusable(grove, *leaf));
    let now = match intent {
        None => held,
        Some(Intent::Away) => None,
        Some(Intent::To(leaf)) => {
            if focusable(grove, leaf) {
                Some(leaf)
            } else {
                // Dropped like any other op naming something it does not apply to. Focus does not
                // move somewhere else instead: the engine has no way to know where that would be.
                debug!(leaf = leaf.id(), "focus dropped");
                held
            }
        }
        Some(Intent::Next) => step(grove, held, Step::Next),
        Some(Intent::Previous) => step(grove, held, Step::Previous),
    };
    if now == was {
        return;
    }
    if let Some(leaf) = was {
        grove.drift.unfocused.insert(leaf);
    }
    if let Some(leaf) = now {
        grove.drift.focused.insert(leaf);
    }
    debug!(
        from = was.map(|leaf| leaf.id()),
        to = now.map(|leaf| leaf.id()),
        "focus moved"
    );
    grove.focus.held = now;
}

/// Which way focus is stepping.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Step {
    Next,
    Previous,
}

/// The next or previous element in order, wrapping within the scope focus is currently in.
fn step(grove: &Grove, held: Option<Leaf>, step: Step) -> Option<Leaf> {
    let order = order(grove, scope(grove, held));
    if order.is_empty() {
        return None;
    }
    let at = held.and_then(|leaf| order.iter().position(|held| *held == leaf));
    let next = match (at, step) {
        (Some(at), Step::Next) => (at + 1) % order.len(),
        (Some(at), Step::Previous) => (at + order.len() - 1) % order.len(),
        // Nothing focused yet, so a step forward takes the first and a step back takes the last.
        (None, Step::Next) => 0,
        (None, Step::Previous) => order.len() - 1,
    };
    Some(order[next])
}

/// The elements focus may step through, in the order it steps through them.
fn order(grove: &Grove, scope: Option<Leaf>) -> Vec<Leaf> {
    let mut order = grove
        .tree
        .leaves()
        .into_iter()
        .filter(|leaf| focusable(grove, *leaf))
        .filter(|leaf| scope.is_none_or(|scope| within(grove, *leaf, scope)))
        .collect::<Vec<_>>();
    order.sort_by(|left, right| {
        let (left_box, right_box) = (grove.tree.drawn(*left), grove.tree.drawn(*right));
        grove
            .tree
            .focus_order(*left)
            .cmp(&grove.tree.focus_order(*right))
            .then(left_box.top().total_cmp(&right_box.top()))
            .then(left_box.left().total_cmp(&right_box.left()))
            // Two elements in the same place with the same override are separated by the order
            // they were grown in, which is total and stable, so the walk is never arbitrary.
            .then(grove.tree.growth(*left).cmp(&grove.tree.growth(*right)))
            .then(Ordering::Equal)
    });
    order
}

/// The innermost scope focus is currently inside, or `None` for the whole tree.
///
/// Read from where focus *is*, not from what is open: a scope traps focus that is in it, and an
/// element that declares itself one while focus is elsewhere is not yet trapping anything.
fn scope(grove: &Grove, held: Option<Leaf>) -> Option<Leaf> {
    let mut step = held;
    while let Some(leaf) = step {
        if grove.tree.focus_scope(leaf) {
            return Some(leaf);
        }
        step = grove.tree.trunk(leaf);
    }
    None
}

/// Whether `leaf` is `scope` or is grown somewhere beneath it.
fn within(grove: &Grove, leaf: Leaf, scope: Leaf) -> bool {
    let mut step = Some(leaf);
    while let Some(leaf) = step {
        if leaf == scope {
            return true;
        }
        step = grove.tree.trunk(leaf);
    }
    false
}

/// Whether focus can rest on `leaf` at all.
///
/// The set that asked to receive input is the set a keyboard can reach, so there is no second
/// declaration to keep in step with the first. Hidden and disabled elements are skipped for the
/// same reason focus leaves one that becomes either.
fn focusable(grove: &Grove, leaf: Leaf) -> bool {
    if !grove.tree.is_live(leaf) || !grove.tree.gestures(leaf).receives {
        return false;
    }
    let inherited = grove.tree.inherited(leaf);
    inherited.visible && !inherited.disabled
}
