//! Focus -- a first-class surface, not a byproduct of clicking.
//!
//! # A tap moves it, and so does the verb
//!
//! Focus goes to what a tap landed on. Nothing declares that: [`interactive`](crate::Place::interactive)
//! is already the statement that an element takes input, and focus rests only on what said it, so
//! the target of a tap is by definition somewhere focus can be. A second flag would have been the
//! same question asked twice, with the two free to disagree.
//!
//! A tap that reached nothing which receives takes focus away -- the same rule, reading that what
//! was tapped cannot hold focus. An app wanting a popover to close on a press elsewhere can still
//! ask about the [`Stem`](crate::Stem) it put behind its content, but it no longer has to.
//!
//! A gesture that became a drag was never a tap, so it moves focus nowhere.
//!
//! [`focus`](crate::Grow::focus) is the verb, and it keeps the last word without being protected: a
//! tap settles focus at dispatch, the frame before an app is handed the
//! [`clicked`](crate::Pollen::clicked) it produced, so an app writing focus elsewhere is simply the
//! later write.
//!
//! # It settles where it is decided
//!
//! Not in a pass at the end of the frame. Whether a target can take focus is *hidden or disabled
//! anywhere in its ancestry*, which is a walk of what has been declared and is therefore current as
//! of the drain -- so an app can show a drawer and focus into it in the frame it opened it. The
//! order it steps in is where elements were drawn, which is the same last-frame geometry the hit
//! test producing the gesture was resolved against.
//!
//! That is what leaves nothing downstream of focus with a pass to miss. Focus is final before
//! resolution runs, so a caret is an ordinary [`visible`](crate::Grow::visible) write inherited by
//! R7 like anything else, rather than a product patched up after the pass that composes it.
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

/// Who holds focus.
#[derive(Default)]
pub(crate) struct Focus {
    held: Option<Leaf>,
}

impl Focus {
    /// Who holds it.
    pub(crate) fn held(&self) -> Option<Leaf> {
        self.held
    }
}

/// Moves focus, where the intent asking for it was decided.
///
/// Applied at once rather than deferred to a pass at the end of the frame. Everything focus needs
/// is answerable here: whether a target can take it is a walk of what has been *declared*, which is
/// current as of the drain, and the order it steps in is where elements were drawn, which is the
/// same last-frame geometry the hit test that produced this intent was resolved against.
///
/// That is what leaves nothing downstream of focus with a pass to miss. Focus is final before
/// resolution runs, so anything that follows it -- a caret, a mark an app draws -- is an ordinary
/// write on the ordinary path, and the products are composed once from state that is already true.
pub(crate) fn moved(grove: &mut Grove, intent: Intent) {
    let was = grove.focus.held;
    // Whatever held focus may have withered, been hidden or been disabled since it took it.
    // Keyboard input arriving at an element that cannot act on it is a dead app, and it is
    // invisible to any test that uses a pointer.
    let held = was.filter(|leaf| focusable(grove, *leaf));
    let now = match intent {
        Intent::Away => None,
        Intent::To(leaf) => {
            if focusable(grove, leaf) {
                Some(leaf)
            } else {
                // Dropped like any other op naming something it does not apply to. Focus does not
                // move somewhere else instead: the engine has no way to know where that would be.
                debug!(leaf = leaf.id(), "focus dropped");
                held
            }
        }
        Intent::Next => step(grove, held, Step::Next),
        Intent::Previous => step(grove, held, Step::Previous),
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

/// Lets go of focus if whoever held it can no longer take it.
///
/// Checked once a frame, because an element can be hidden, disabled or pruned by anything at all
/// and none of those is obliged to think about focus.
pub(crate) fn sweep(grove: &mut Grove) {
    let Some(held) = grove.focus.held else {
        return;
    };
    if focusable(grove, held) {
        return;
    }
    grove.drift.unfocused.insert(held);
    grove.focus.held = None;
    debug!(leaf = held.id(), "focus let go");
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
    // Walked over what has been declared rather than read from R7's product, because focus is
    // answered before R7 runs. The walk is the same composition over the same two states -- hidden
    // or disabled anywhere in the ancestry -- and it is the *current* answer rather than the one
    // the last frame resolved, which is what lets an app show a drawer and focus into it in the
    // same frame it opened it.
    let mut step = Some(leaf);
    while let Some(at) = step {
        if !grove.tree.visible(at).0 || grove.tree.disabled(at).0 {
            return false;
        }
        step = grove.tree.trunk(at);
    }
    true
}
