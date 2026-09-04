//! Focus: the verb, the order, and the scopes that trap it.
//!
//! Nothing here presses anything. That is the point of the slice: focus moves because it was told
//! to, and where it can go is derived from the same geometry everything else is.

use crate::tests::{Observer, grove, press, release, tick, tick_with};
use crate::{Boxed, Grove, Grow, Leaf, Location, Panel, Place, Source, Stem, left, top};

/// A box at a stated place, so reading order has something to read.
fn at(x: f32, y: f32, width: f32, height: f32) -> Location {
    Location::new().xs(
        left(x.px()).width(width.px()),
        top(y.px()).height(height.px()),
    )
}

/// An element focus can rest on, at a place.
fn field(grove: &mut Grove, x: f32, y: f32) -> Leaf {
    grove.plant(Panel::new().at(at(x, y, 80.0, 20.0)).interactive())
}

/// A page of three fields, planted out of reading order so that the order under test is the
/// derived one and not the order they were grown in.
fn page(grove: &mut Grove) -> [Leaf; 3] {
    let middle = field(grove, 0.0, 100.0);
    let last = field(grove, 0.0, 200.0);
    let first = field(grove, 0.0, 0.0);
    [first, middle, last]
}

/// The verb, and the whole of what it takes to move focus. No gesture is involved anywhere.
#[test]
fn focus_moves_by_the_verb_alone() {
    let mut grove = grove();
    let field = field(&mut grove, 0.0, 0.0);
    tick(&mut grove);

    grove.focus(field);
    tick(&mut grove);
    assert_eq!(grove.focused(), Some(field));

    let mut app = Observer::default();
    tick_with(&mut grove, &mut app);
    assert!(app.last().focused(field));
}

/// A press moves focus nowhere. The element a person pressed and the element they want to type into
/// are different questions, and an app that wants them to coincide says so itself.
#[test]
fn a_press_does_not_move_focus() {
    let mut grove = grove();
    let first = field(&mut grove, 0.0, 0.0);
    let second = field(&mut grove, 0.0, 100.0);
    grove.focus(first);
    tick(&mut grove);

    press(&mut grove, 40.0, 110.0);
    release(&mut grove, 40.0, 110.0);
    let mut app = Observer::default();
    tick_with(&mut grove, &mut app);

    assert!(app.last().clicked(second));
    assert_eq!(grove.focused(), Some(first));
}

/// Which is one line to opt into, and reads as what it is.
#[test]
fn an_app_moves_focus_from_a_tap_itself() {
    let mut grove = grove();
    let field = field(&mut grove, 0.0, 0.0);
    tick(&mut grove);

    press(&mut grove, 40.0, 10.0);
    release(&mut grove, 40.0, 10.0);
    let mut app = Observer::default();
    tick_with(&mut grove, &mut app);
    if app.last().clicked(field) {
        grove.focus(field);
    }
    tick(&mut grove);

    assert_eq!(grove.focused(), Some(field));
}

/// Only what asked to receive input can hold focus, so there is no second declaration to keep in
/// step with the first.
#[test]
fn focus_is_dropped_on_an_element_that_receives_nothing() {
    let mut grove = grove();
    let decoration = grove.plant(Panel::new().at(at(0.0, 0.0, 80.0, 20.0)));
    tick(&mut grove);

    grove.focus(decoration);
    tick(&mut grove);
    assert_eq!(grove.focused(), None);
}

/// Top to bottom, whatever order the elements were grown in.
#[test]
fn tab_order_follows_reading_order() {
    let mut grove = grove();
    let [first, middle, last] = page(&mut grove);
    tick(&mut grove);

    grove.focus_next();
    tick(&mut grove);
    assert_eq!(grove.focused(), Some(first));

    grove.focus_next();
    tick(&mut grove);
    assert_eq!(grove.focused(), Some(middle));

    grove.focus_next();
    tick(&mut grove);
    assert_eq!(grove.focused(), Some(last));

    // Wraps rather than stopping, so stepping is never a dead end.
    grove.focus_next();
    tick(&mut grove);
    assert_eq!(grove.focused(), Some(first));

    grove.focus_previous();
    tick(&mut grove);
    assert_eq!(grove.focused(), Some(last));
}

/// Left to right within a row, which is the other half of reading order.
#[test]
fn tab_order_reads_across_a_row_before_moving_down() {
    let mut grove = grove();
    let right = grove.plant(Panel::new().at(at(100.0, 0.0, 80.0, 20.0)).interactive());
    let left = grove.plant(Panel::new().at(at(0.0, 0.0, 80.0, 20.0)).interactive());
    let below = field(&mut grove, 0.0, 100.0);
    tick(&mut grove);

    grove.focus_next();
    tick(&mut grove);
    assert_eq!(grove.focused(), Some(left));

    grove.focus_next();
    tick(&mut grove);
    assert_eq!(grove.focused(), Some(right));

    grove.focus_next();
    tick(&mut grove);
    assert_eq!(grove.focused(), Some(below));
}

/// An override moves one element and renumbers nothing: everything sharing a value keeps reading
/// order among themselves.
#[test]
fn tab_order_honours_an_override() {
    let mut grove = grove();
    let middle = grove.plant(Panel::new().at(at(0.0, 100.0, 80.0, 20.0)).interactive());
    let last = grove.plant(Panel::new().at(at(0.0, 200.0, 80.0, 20.0)).interactive());
    // Drawn first and read last, which is the case an override is for.
    let pulled = grove.plant(
        Panel::new()
            .at(at(0.0, 0.0, 80.0, 20.0))
            .interactive()
            .focus_order(1),
    );
    tick(&mut grove);

    grove.focus_next();
    tick(&mut grove);
    assert_eq!(grove.focused(), Some(middle));

    grove.focus_next();
    tick(&mut grove);
    assert_eq!(grove.focused(), Some(last));

    grove.focus_next();
    tick(&mut grove);
    assert_eq!(grove.focused(), Some(pulled));
}

/// A drawer declares itself a scope. While focus is inside it, stepping cycles within it -- without
/// this, keyboard navigation inside an overlay walks off into the page behind it.
#[test]
fn focus_inside_a_scope_cycles_within_it() {
    let mut grove = grove();
    let page = field(&mut grove, 0.0, 0.0);
    let drawer = grove.plant(Stem::new().at(at(200.0, 0.0, 200.0, 300.0)).focus_scope());
    let first = grove.branch(
        drawer,
        Panel::new().at(at(200.0, 40.0, 80.0, 20.0)).interactive(),
    );
    let second = grove.branch(
        drawer,
        Panel::new().at(at(200.0, 80.0, 80.0, 20.0)).interactive(),
    );
    tick(&mut grove);

    grove.focus(first);
    tick(&mut grove);

    grove.focus_next();
    tick(&mut grove);
    assert_eq!(grove.focused(), Some(second));

    // Round the end of the scope rather than out of it, and never onto the page behind.
    grove.focus_next();
    tick(&mut grove);
    assert_eq!(grove.focused(), Some(first));

    grove.focus_previous();
    tick(&mut grove);
    assert_eq!(grove.focused(), Some(second));
    assert_ne!(grove.focused(), Some(page));
}

/// The scope traps what is in it, and nothing else. With focus on the page, stepping walks the page.
#[test]
fn a_scope_traps_only_the_focus_that_is_inside_it() {
    let mut grove = grove();
    let page = field(&mut grove, 0.0, 0.0);
    let drawer = grove.plant(Stem::new().at(at(200.0, 0.0, 200.0, 300.0)).focus_scope());
    let inside = grove.branch(
        drawer,
        Panel::new().at(at(200.0, 40.0, 80.0, 20.0)).interactive(),
    );
    tick(&mut grove);

    grove.focus(page);
    tick(&mut grove);
    grove.focus_next();
    tick(&mut grove);
    assert_eq!(grove.focused(), Some(inside));
}

/// Keyboard input arriving at an element that cannot act on it is a dead app, and it is invisible
/// to any test that uses a pointer.
#[test]
fn disabling_the_subtree_holding_focus_moves_focus_out_and_reports_it() {
    let mut grove = grove();
    let drawer = grove.plant(Stem::new().at(at(0.0, 0.0, 200.0, 300.0)));
    let field = grove.branch(
        drawer,
        Panel::new().at(at(0.0, 0.0, 80.0, 20.0)).interactive(),
    );
    grove.focus(field);
    tick(&mut grove);
    assert_eq!(grove.focused(), Some(field));

    grove.disable(drawer);
    let mut app = Observer::default();
    tick_with(&mut grove, &mut app);
    tick_with(&mut grove, &mut app);

    assert_eq!(grove.focused(), None);
    assert!(app.last().unfocused(field));
}

/// Hiding it is the same answer, for the same reason.
#[test]
fn hiding_the_element_holding_focus_moves_focus_out() {
    let mut grove = grove();
    let field = field(&mut grove, 0.0, 0.0);
    grove.focus(field);
    tick(&mut grove);

    grove.visible(field, false);
    tick(&mut grove);
    assert_eq!(grove.focused(), None);
}

/// So is pruning it, and the app hears about both the wither and the focus.
#[test]
fn pruning_the_element_holding_focus_moves_focus_out() {
    let mut grove = grove();
    let field = field(&mut grove, 0.0, 0.0);
    grove.focus(field);
    tick(&mut grove);

    grove.prune(field);
    let mut app = Observer::default();
    tick_with(&mut grove, &mut app);
    tick_with(&mut grove, &mut app);

    assert_eq!(grove.focused(), None);
    assert!(app.last().unfocused(field));
    assert!(app.pollen[app.pollen.len() - 1].withered(field));
}

/// Focus is answered against the frame's own geometry, so an app can open something and focus into
/// it in one turn.
#[test]
fn focus_lands_on_an_element_grown_in_the_same_frame() {
    let mut grove = grove();
    let field = field(&mut grove, 0.0, 0.0);
    grove.focus(field);
    tick(&mut grove);

    assert_eq!(grove.focused(), Some(field));
}

/// Taking focus off everything is its own statement, rather than focusing something arbitrary.
#[test]
fn unfocus_leaves_nothing_focused() {
    let mut grove = grove();
    let field = field(&mut grove, 0.0, 0.0);
    grove.focus(field);
    tick(&mut grove);

    grove.unfocus();
    let mut app = Observer::default();
    tick_with(&mut grove, &mut app);
    tick_with(&mut grove, &mut app);

    assert_eq!(grove.focused(), None);
    assert!(app.last().unfocused(field));
}
