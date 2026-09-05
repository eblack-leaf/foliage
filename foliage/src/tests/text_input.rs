//! TextInput: what a keystroke does, where the caret lands, and the four elements behind one name.
//!
//! The first half is pure -- a value, a caret and a key in, a value and a caret out -- because that
//! is where every off-by-one in editing lives and none of it needs a tree. The second half runs the
//! whole frame, and is where the caret becomes a box, focus decides whether it is drawn, and a
//! gesture reaches a character.

use crate::elevation::ResolvedElevation;
use crate::interaction::input::{Key, Keystroke};
use crate::tests::{
    Observer, controlled, drag, grove, key, past_the_hold, press, release, section, shifted,
    stroke, tick, tick_with, typing, with_control, with_shift,
};
use crate::text_input::{Applied, Editing, applied};
use crate::{
    Boxed, Grove, Grow, Leaf, Location, Palette, Place, Sap, Source, Stem, TextInput, Vein, left,
    top,
};

// Editing, as arithmetic over character indices.

/// Applies a key and reports what the value and the caret became, for the cases where both matter.
fn edit(value: &str, editing: Editing, stroke: Keystroke) -> (String, Editing) {
    match applied(value, editing, stroke) {
        Applied::Wrote(written, editing) => (written, editing),
        Applied::Moved(editing) => (value.to_string(), editing),
        other => panic!(
            "expected a write or a move, got {}",
            match other {
                Applied::Submitted => "a submission",
                _ => "nothing",
            }
        ),
    }
}

fn at(index: usize) -> Editing {
    Editing::at(index)
}

fn selecting(anchor: usize, caret: usize) -> Editing {
    Editing { anchor, caret }
}

#[test]
fn a_character_lands_where_the_caret_is_and_carries_it_along() {
    assert_eq!(
        edit("helo", at(2), stroke(Key::Typed('l'))),
        ("hello".to_string(), at(3))
    );
}

/// The caret is in characters and not in bytes, so a value that is not ASCII indexes the same way
/// the run that draws it does.
#[test]
fn indices_are_characters_and_not_bytes() {
    let (value, editing) = edit("héllo", at(2), stroke(Key::Typed('X')));
    assert_eq!(value, "héXllo");
    assert_eq!(editing, at(3));
}

/// Typing over a selection replaces it, which is the same rule as removing it and then inserting.
#[test]
fn typing_over_a_selection_replaces_it() {
    assert_eq!(
        edit("hello", selecting(1, 4), stroke(Key::Typed('a'))),
        ("hao".to_string(), at(2))
    );
}

#[test]
fn backspace_takes_the_character_before_the_caret() {
    assert_eq!(
        edit("hello", at(3), stroke(Key::Backspace)),
        ("helo".to_string(), at(2))
    );
}

/// One rule, not two: backspace removes the span, and an empty span reaches back by one first.
#[test]
fn backspace_takes_the_selection_where_there_is_one() {
    assert_eq!(
        edit("hello", selecting(4, 1), stroke(Key::Backspace)),
        ("ho".to_string(), at(1))
    );
}

#[test]
fn backspace_at_the_start_does_nothing() {
    assert!(matches!(
        applied("hello", at(0), stroke(Key::Backspace)),
        Applied::Nothing
    ));
}

#[test]
fn delete_takes_the_character_after_the_caret() {
    assert_eq!(
        edit("hello", at(0), stroke(Key::Delete)),
        ("ello".to_string(), at(0))
    );
    assert!(matches!(
        applied("hello", at(5), stroke(Key::Delete)),
        Applied::Nothing
    ));
}

#[test]
fn the_arrows_move_by_one_and_stop_at_the_ends() {
    assert_eq!(edit("hi", at(1), stroke(Key::Left)).1, at(0));
    assert_eq!(edit("hi", at(0), stroke(Key::Left)).1, at(0));
    assert_eq!(edit("hi", at(1), stroke(Key::Right)).1, at(2));
    assert_eq!(edit("hi", at(2), stroke(Key::Right)).1, at(2));
}

#[test]
fn home_and_end_go_to_the_ends() {
    assert_eq!(edit("hello", at(3), stroke(Key::Home)).1, at(0));
    assert_eq!(edit("hello", at(3), stroke(Key::End)).1, at(5));
}

/// Shift keeps the anchor where it was, which is what makes a selection grow from one end.
#[test]
fn a_shifted_arrow_extends_from_the_anchor() {
    assert_eq!(
        edit("hello", at(2), with_shift(Key::Right)).1,
        selecting(2, 3)
    );
    assert_eq!(
        edit("hello", selecting(2, 4), with_shift(Key::Right)).1,
        selecting(2, 5)
    );
    assert_eq!(
        edit("hello", selecting(2, 4), with_shift(Key::Home)).1,
        selecting(2, 0)
    );
}

/// An unshifted arrow against a selection collapses to the edge it points at rather than stepping
/// from the caret. The selection is the thing being moved away from.
#[test]
fn an_arrow_collapses_a_selection_to_the_edge_it_points_at() {
    assert_eq!(edit("hello", selecting(1, 4), stroke(Key::Left)).1, at(1));
    assert_eq!(edit("hello", selecting(1, 4), stroke(Key::Right)).1, at(4));
    // And from the other direction, where the caret is the low end.
    assert_eq!(edit("hello", selecting(4, 1), stroke(Key::Right)).1, at(4));
}

/// Neither an edit nor a caret. The app is told, and what it means is the app's.
#[test]
fn enter_is_a_submission_and_changes_nothing() {
    assert!(matches!(
        applied("hello", at(2), stroke(Key::Enter)),
        Applied::Submitted
    ));
}

/// A caret past the end of the value is held inside it before anything is asked of it, so a value
/// rewritten from under a caret cannot address a character that is not there.
#[test]
fn a_caret_past_the_value_is_held_inside_it() {
    assert_eq!(edit("hi", at(9), stroke(Key::Left)).1, at(1));
    assert_eq!(
        edit("hi", at(9), stroke(Key::Typed('!'))),
        ("hi!".to_string(), at(3))
    );
}

// Through the frame.

/// The default font's cell at the default size, which every box below is a multiple of.
const CELL: f32 = 10.0;
const LINE: f32 = 22.0;

/// A field two hundred across and thirty-two tall, at the origin.
fn field(grove: &mut Grove) -> Leaf {
    grove.plant(
        TextInput::new()
            .placeholder("search")
            .at(Location::new().xs(left(0.px()).width(200.px()), top(0.px()).height(32.px()))),
    )
}

/// A field with focus already on it, ready to be typed into.
fn focused(grove: &mut Grove) -> Leaf {
    let leaf = field(grove);
    tick(grove);
    grove.focus(leaf);
    tick(grove);
    leaf
}

fn value(grove: &Grove, leaf: Leaf) -> String {
    match grove.tap(leaf, Vein::Text) {
        Some(Sap::Text(value)) => value,
        other => panic!("expected a value, got {other:?}"),
    }
}

/// How far the field has been scrolled across.
fn offset(grove: &Grove, leaf: Leaf) -> f32 {
    match grove.tap(leaf, Vein::Offset) {
        Some(Sap::Position(offset)) => offset.x,
        other => panic!("expected an offset, got {other:?}"),
    }
}

fn selection(grove: &Grove, leaf: Leaf) -> core::ops::Range<usize> {
    match grove.tap(leaf, Vein::Selection) {
        Some(Sap::Selection(range)) => range,
        other => panic!("expected a selection, got {other:?}"),
    }
}

/// The parts, in the order they were grown: selection, run, hint, caret.
fn parts(grove: &Grove, leaf: Leaf) -> Vec<Leaf> {
    match grove.tap(leaf, Vein::Branches) {
        Some(Sap::Leaves(leaves)) => leaves,
        other => panic!("expected branches, got {other:?}"),
    }
}

fn caret(grove: &Grove, leaf: Leaf) -> Leaf {
    parts(grove, leaf)[3]
}

fn selection_box(grove: &Grove, leaf: Leaf) -> Leaf {
    parts(grove, leaf)[0]
}

fn hint(grove: &Grove, leaf: Leaf) -> Leaf {
    parts(grove, leaf)[2]
}

/// Where a part sits in the one stack, which is what decides who is drawn over whom.
fn rank(grove: &Grove, leaf: Leaf) -> ResolvedElevation {
    grove.tree.rank(leaf)
}

/// Whether a part is actually drawn, which is the resolved product and not the declaration.
///
/// `Vein::Visible` is what an element was *told* to be. A mark that follows focus is told one thing
/// by the field and gated by another, and what a reader sees is the two composed -- so that is what
/// these ask.
fn shown(grove: &Grove, leaf: Leaf) -> bool {
    grove.tree.inherited(leaf).visible
}

/// One name to the app, four elements underneath it -- and the frame that plants a field is the
/// frame the whole of it is live in, because the parts are grown in the drain that grew it.
#[test]
fn a_field_is_one_name_and_four_elements() {
    let mut grove = grove();
    let leaf = field(&mut grove);
    tick(&mut grove);
    assert_eq!(parts(&grove, leaf).len(), 4);
    assert_eq!(value(&grove, leaf), "");
}

/// Typing reaches the field holding focus, and what it says is read back from the field itself
/// rather than from anything it is made of.
#[test]
fn typing_reaches_the_field_holding_focus() {
    let mut grove = grove();
    let leaf = focused(&mut grove);

    typing(&mut grove, "hello");
    tick(&mut grove);
    assert_eq!(value(&grove, leaf), "hello");
    assert_eq!(selection(&grove, leaf), 5..5);
}

/// Keystrokes are the one thing whose order is part of what they mean, and several arriving in one
/// frame keep it.
#[test]
fn keystrokes_in_one_frame_keep_their_order() {
    let mut grove = grove();
    let leaf = focused(&mut grove);

    typing(&mut grove, "abc");
    key(&mut grove, Key::Left);
    key(&mut grove, Key::Typed('X'));
    tick(&mut grove);
    assert_eq!(value(&grove, leaf), "abXc");
}

/// What the person typed, reported to the app on F7's own terms.
#[test]
fn what_was_typed_is_reported() {
    let mut grove = grove();
    let leaf = focused(&mut grove);

    typing(&mut grove, "a");
    tick(&mut grove);
    let mut app = Observer::default();
    tick_with(&mut grove, &mut app);
    assert!(app.last().edited(leaf));
}

/// A value the app wrote is not reported back: it already knows what it wrote. The caret goes to
/// the end of it, which is where a field that was just filled in is ready to be typed into.
#[test]
fn a_value_the_app_wrote_is_not_reported_as_edited() {
    let mut grove = grove();
    let leaf = focused(&mut grove);

    grove.text(leaf, "written");
    tick(&mut grove);
    let mut app = Observer::default();
    tick_with(&mut grove, &mut app);
    assert_eq!(value(&grove, leaf), "written");
    assert_eq!(selection(&grove, leaf), 7..7);
    assert!(!app.last().edited(leaf));
}

#[test]
fn enter_is_reported_and_leaves_the_value_alone() {
    let mut grove = grove();
    let leaf = focused(&mut grove);
    typing(&mut grove, "query");
    tick(&mut grove);

    key(&mut grove, Key::Enter);
    tick(&mut grove);
    let mut app = Observer::default();
    tick_with(&mut grove, &mut app);
    assert!(app.last().submitted(leaf));
    assert_eq!(value(&grove, leaf), "query");
}

/// A keystroke with nothing focused reaches nothing, and one with something focused that is not a
/// field reaches nothing either.
#[test]
fn a_keystroke_with_no_field_focused_goes_nowhere() {
    let mut grove = grove();
    let leaf = field(&mut grove);
    let elsewhere = grove.plant(
        Stem::new()
            .at(Location::new().xs(left(0.px()).width(50.px()), top(100.px()).height(50.px())))
            .interactive(),
    );
    tick(&mut grove);

    typing(&mut grove, "a");
    tick(&mut grove);
    assert_eq!(value(&grove, leaf), "");

    grove.focus(elsewhere);
    tick(&mut grove);
    typing(&mut grove, "b");
    tick(&mut grove);
    assert_eq!(value(&grove, leaf), "");
}

/// `Tab` is focus's own, wherever focus is -- including nowhere, which is what makes a keyboard
/// reach a page that has not been pressed yet.
#[test]
fn tab_steps_focus() {
    let mut grove = grove();
    let first = field(&mut grove);
    let second = grove.plant(
        TextInput::new()
            .at(Location::new().xs(left(0.px()).width(200.px()), top(100.px()).height(32.px()))),
    );
    tick(&mut grove);

    key(&mut grove, Key::Tab);
    tick(&mut grove);
    assert_eq!(grove.focused(), Some(first));

    key(&mut grove, Key::Tab);
    tick(&mut grove);
    assert_eq!(grove.focused(), Some(second));

    shifted(&mut grove, Key::Tab);
    tick(&mut grove);
    assert_eq!(grove.focused(), Some(first));
}

/// Tab does not reach the value, even with the field holding focus.
#[test]
fn tab_is_not_typed_into_a_field() {
    let mut grove = grove();
    let leaf = focused(&mut grove);
    key(&mut grove, Key::Tab);
    tick(&mut grove);
    assert_eq!(value(&grove, leaf), "");
}

/// The caret is a box at a character, and it is placed in the ordinary grammar: a count of the
/// run's own cells from the run's own left edge.
#[test]
fn the_caret_sits_at_its_character() {
    let mut grove = grove();
    let leaf = focused(&mut grove);
    typing(&mut grove, "abcd");
    tick(&mut grove);

    let mark = caret(&grove, leaf);
    assert_eq!(section(&grove, mark).left(), 4.0 * CELL);
    assert_eq!(section(&grove, mark).area.height, LINE);

    key(&mut grove, Key::Left);
    key(&mut grove, Key::Left);
    tick(&mut grove);
    assert_eq!(section(&grove, mark).left(), 2.0 * CELL);
}

/// A selection is the span between its two ends, whichever way round it was made.
#[test]
fn a_selection_covers_the_span_it_names() {
    let mut grove = grove();
    let leaf = focused(&mut grove);
    typing(&mut grove, "abcdef");
    tick(&mut grove);

    grove.select(leaf, 1..4);
    tick(&mut grove);
    let box_of = section(&grove, selection_box(&grove, leaf));
    assert_eq!(box_of.left(), 1.0 * CELL);
    assert_eq!(box_of.area.width, 3.0 * CELL);
    assert!(shown(&grove, selection_box(&grove, leaf)));
}

/// Nothing selected is nothing drawn, rather than a box of no width sitting in the stack.
#[test]
fn a_collapsed_selection_is_not_drawn() {
    let mut grove = grove();
    let leaf = focused(&mut grove);
    typing(&mut grove, "abc");
    tick(&mut grove);
    assert!(!shown(&grove, selection_box(&grove, leaf)));
}

/// A range past the value selects to the end of it.
#[test]
fn a_selection_past_the_value_is_held_inside_it() {
    let mut grove = grove();
    let leaf = focused(&mut grove);
    typing(&mut grove, "abc");
    tick(&mut grove);

    grove.select(leaf, 1..99);
    tick(&mut grove);
    assert_eq!(selection(&grove, leaf), 1..3);
}

/// Both marks are drawn while the field holds focus and at no other time. A span highlighted on a
/// field nothing is typing into reads as a field that is still live.
///
/// What is selected is state and survives, so coming back finds it as it was -- focus gates whether
/// the mark is drawn and never what the field holds.
#[test]
fn the_selection_is_drawn_only_while_the_field_holds_focus() {
    let mut grove = grove();
    let leaf = focused(&mut grove);
    typing(&mut grove, "abcdef");
    tick(&mut grove);
    grove.select(leaf, 1..4);
    tick(&mut grove);
    assert!(shown(&grove, selection_box(&grove, leaf)));

    grove.unfocus();
    tick(&mut grove);
    assert!(!shown(&grove, selection_box(&grove, leaf)));
    // Still selected, and still saying so.
    assert_eq!(selection(&grove, leaf), 1..4);

    grove.focus(leaf);
    tick(&mut grove);
    assert!(shown(&grove, selection_box(&grove, leaf)));
    assert_eq!(selection(&grove, leaf), 1..4);
}

/// A tap collapses what was selected, because a tap says where the caret goes. So the span a field
/// kept while it was away survives being *stepped* back into and not being *tapped* back into --
/// which is the same answer every editor gives, and worth pinning because the two ways back in
/// differ.
#[test]
fn tapping_back_into_a_field_collapses_what_it_kept() {
    let mut grove = grove();
    let leaf = focused(&mut grove);
    typing(&mut grove, "abcdef");
    tick(&mut grove);
    grove.select(leaf, 1..4);
    grove.unfocus();
    tick(&mut grove);

    // Stepped back into: the span is as it was left.
    grove.focus(leaf);
    tick(&mut grove);
    assert_eq!(selection(&grove, leaf), 1..4);

    grove.unfocus();
    tick(&mut grove);
    // Tapped back into: focus returns and the caret goes where the tap landed.
    press(&mut grove, 5.0 * CELL, 16.0);
    release(&mut grove, 5.0 * CELL, 16.0);
    tick(&mut grove);
    assert_eq!(grove.focused(), Some(leaf));
    assert_eq!(selection(&grove, leaf), 5..5);
}

/// And focus gates the mark rather than replacing what the field said about it: a collapsed caret
/// selects nothing, so a focused field with no span draws no band.
#[test]
fn a_focused_field_with_nothing_selected_draws_no_selection() {
    let mut grove = grove();
    let leaf = focused(&mut grove);
    typing(&mut grove, "abcdef");
    tick(&mut grove);
    assert_eq!(selection(&grove, leaf), 6..6);
    assert!(!shown(&grove, selection_box(&grove, leaf)));
}

/// The caret is drawn while the field holds focus and at no other time, and it is answered in the
/// frame focus moved rather than in the one after it.
#[test]
fn the_caret_is_drawn_only_while_the_field_holds_focus() {
    let mut grove = grove();
    let leaf = field(&mut grove);
    tick(&mut grove);
    assert!(!shown(&grove, caret(&grove, leaf)));

    grove.focus(leaf);
    tick(&mut grove);
    assert!(shown(&grove, caret(&grove, leaf)));

    grove.unfocus();
    tick(&mut grove);
    assert!(!shown(&grove, caret(&grove, leaf)));
}

/// The placeholder is read in the field's place while it says nothing, and is never part of what it
/// says.
#[test]
fn the_placeholder_shows_only_while_the_field_is_empty() {
    let mut grove = grove();
    let leaf = focused(&mut grove);
    assert!(shown(&grove, hint(&grove, leaf)));
    assert_eq!(value(&grove, leaf), "");

    typing(&mut grove, "a");
    tick(&mut grove);
    assert!(!shown(&grove, hint(&grove, leaf)));

    key(&mut grove, Key::Backspace);
    tick(&mut grove);
    assert!(shown(&grove, hint(&grove, leaf)));
}

/// A press puts the caret at the character it landed on, and rounds to the nearer edge -- which is
/// where a hand aiming between two characters means.
#[test]
fn a_press_puts_the_caret_where_it_landed() {
    let mut grove = grove();
    let leaf = focused(&mut grove);
    typing(&mut grove, "abcdef");
    tick(&mut grove);

    press(&mut grove, 2.0 * CELL + 1.0, 16.0);
    release(&mut grove, 2.0 * CELL + 1.0, 16.0);
    tick(&mut grove);
    assert_eq!(selection(&grove, leaf), 2..2);

    press(&mut grove, 3.0 * CELL + 6.0, 16.0);
    release(&mut grove, 3.0 * CELL + 6.0, 16.0);
    tick(&mut grove);
    assert_eq!(selection(&grove, leaf), 4..4);
}

/// A drag across a field scrolls its value and selects nothing. The field declares no drags, so the
/// gesture is the region's -- and the region is the field.
#[test]
fn a_drag_across_a_field_scrolls_its_value() {
    let mut grove = grove();
    let leaf = focused(&mut grove);
    // Thirty characters through a box twenty wide, so there is a hundred pixels to scroll.
    typing(&mut grove, "abcdefghijklmnopqrstuvwxyz0123");
    tick(&mut grove);
    // Back to the start, where typing to the end had carried the field away from.
    grove.select(leaf, 0..0);
    tick(&mut grove);
    assert_eq!(offset(&grove, leaf), 0.0);

    press(&mut grove, 15.0 * CELL, 16.0);
    tick(&mut grove);
    drag(&mut grove, 5.0 * CELL, 16.0);
    tick(&mut grove);
    release(&mut grove, 5.0 * CELL, 16.0);
    tick(&mut grove);

    assert_eq!(offset(&grove, leaf), 100.0);
    assert_eq!(selection(&grove, leaf), 0..0);
}

/// The same motion out of a press that was held selects instead. The hold settled that the field is
/// holding the gesture, so the drag is the field's whatever it declared about drags -- which is the
/// only thing separating these two, because nothing in the motions themselves does.
#[test]
fn a_drag_out_of_a_hold_selects() {
    let mut grove = grove();
    let leaf = focused(&mut grove);
    typing(&mut grove, "abcdefgh");
    tick(&mut grove);

    press(&mut grove, 1.0 * CELL, 16.0);
    tick(&mut grove);
    past_the_hold(&mut grove);
    tick(&mut grove);
    // The hold puts the caret where it landed, which a tap would have done and a drag never does.
    assert_eq!(selection(&grove, leaf), 1..1);

    drag(&mut grove, 6.0 * CELL, 16.0);
    tick(&mut grove);
    release(&mut grove, 6.0 * CELL, 16.0);
    tick(&mut grove);
    assert_eq!(selection(&grove, leaf), 1..6);
    assert_eq!(offset(&grove, leaf), 0.0);
}

/// A drag that runs past the edge keeps the end it started from. The anchor is a character and the
/// hold put it there; a point on the screen is not one, because the value scrolls under it as the
/// field follows the caret -- which is what used to walk the start of the selection along with it.
#[test]
fn a_drag_past_the_edge_keeps_the_end_it_started_from() {
    let mut grove = grove();
    let leaf = focused(&mut grove);
    typing(&mut grove, "abcdefghijklmnopqrstuvwxyz0123");
    tick(&mut grove);
    grove.select(leaf, 0..0);
    tick(&mut grove);

    press(&mut grove, 3.0 * CELL, 16.0);
    tick(&mut grove);
    past_the_hold(&mut grove);
    tick(&mut grove);
    assert_eq!(selection(&grove, leaf), 3..3);

    // Out past the right edge of the box, which carries the value along under the pointer.
    drag(&mut grove, 24.0 * CELL, 16.0);
    tick(&mut grove);
    assert!(offset(&grove, leaf) > 0.0, "the field followed the caret");
    let reached = selection(&grove, leaf);
    assert_eq!(reached.start, 3);

    drag(&mut grove, 25.0 * CELL, 16.0);
    tick(&mut grove);
    let further = selection(&grove, leaf);
    assert_eq!(further.start, 3, "the anchor stayed the character it was");
    assert!(further.end > reached.end);
}

/// A drag that reached the edge and stopped moving keeps going. A pointer held still produces no
/// event of any kind, so a field that worked only when one arrived would stop at its own edge and
/// wait to be jiggled -- and the frames it takes to get there are its own to ask for.
#[test]
fn a_drag_held_past_the_edge_keeps_scrolling() {
    let mut grove = grove();
    let leaf = focused(&mut grove);
    typing(&mut grove, "abcdefghijklmnopqrstuvwxyz0123");
    tick(&mut grove);
    grove.select(leaf, 0..0);
    tick(&mut grove);

    press(&mut grove, 3.0 * CELL, 16.0);
    tick(&mut grove);
    past_the_hold(&mut grove);
    tick(&mut grove);

    // Out past the right edge, once.
    drag(&mut grove, 22.0 * CELL, 16.0);
    tick(&mut grove);
    let (moved, selected) = (offset(&grove, leaf), selection(&grove, leaf));
    assert!(moved > 0.0);
    assert!(grove.again, "the field is asking for the frames");

    // Nothing arrives at all. The pointer is where it was, and the frame runs because the field
    // asked for it.
    tick(&mut grove);
    assert!(offset(&grove, leaf) > moved, "it kept going");
    assert!(selection(&grove, leaf).end > selected.end);
    assert_eq!(selection(&grove, leaf).start, 3);

    // Back inside, and it stops asking.
    drag(&mut grove, 10.0 * CELL, 16.0);
    tick(&mut grove);
    let settled = offset(&grove, leaf);
    assert!(!grove.again);
    tick(&mut grove);
    assert_eq!(offset(&grove, leaf), settled);
}

/// A hold takes focus with the caret, and the field writes that itself: a press that was held is not
/// a tap, and a tap is the only gesture the engine moves focus on.
#[test]
fn a_hold_focuses_the_field_it_landed_in() {
    let mut grove = grove();
    let leaf = field(&mut grove);
    tick(&mut grove);
    grove.text(leaf, "abcdef");
    tick(&mut grove);
    assert_eq!(grove.focused(), None);

    press(&mut grove, 2.0 * CELL, 16.0);
    tick(&mut grove);
    past_the_hold(&mut grove);
    tick(&mut grove);

    assert_eq!(grove.focused(), Some(leaf));
    assert_eq!(selection(&grove, leaf), 2..2);
    assert!(shown(&grove, caret(&grove, leaf)));
}

/// The field is what a press reaches, never one of the parts drawn on it. Every part is out of the
/// stack, because the hit test reads the top of it and stops.
#[test]
fn a_press_reaches_the_field_and_not_its_parts() {
    let mut grove = grove();
    let leaf = focused(&mut grove);
    typing(&mut grove, "abc");
    tick(&mut grove);

    press(&mut grove, 5.0, 16.0);
    release(&mut grove, 5.0, 16.0);
    let mut app = Observer::default();
    tick_with(&mut grove, &mut app);
    assert!(app.last().clicked(leaf));
}

/// A value longer than the box scrolls, and the caret is what the field is kept showing -- so
/// typing past the right edge brings it back into view in the frame it was typed in.
#[test]
fn the_field_follows_its_caret() {
    let mut grove = grove();
    let leaf = focused(&mut grove);
    // Thirty characters is three hundred logical pixels in a box two hundred wide.
    typing(&mut grove, "abcdefghijklmnopqrstuvwxyz0123");
    tick(&mut grove);

    let offset = match grove.tap(leaf, Vein::Offset) {
        Some(Sap::Position(offset)) => offset,
        other => panic!("expected an offset, got {other:?}"),
    };
    assert!(offset.x > 0.0, "the field scrolled to its caret");
    // And the caret is inside the box rather than past its right edge.
    let mark = section(&grove, caret(&grove, leaf));
    assert!(mark.right() <= 200.0 + 0.01);

    key(&mut grove, Key::Home);
    tick(&mut grove);
    let home = match grove.tap(leaf, Vein::Offset) {
        Some(Sap::Position(offset)) => offset,
        other => panic!("expected an offset, got {other:?}"),
    };
    assert_eq!(home.x, 0.0);
}

/// A field is one thing to take down, parts and all.
#[test]
fn pruning_a_field_takes_its_parts() {
    let mut grove = grove();
    let leaf = focused(&mut grove);
    let parts = parts(&grove, leaf);

    grove.prune(leaf);
    tick(&mut grove);
    let mut app = Observer::default();
    tick_with(&mut grove, &mut app);
    for part in parts {
        assert!(app.last().withered(part));
    }
}

/// Neither the selection nor the caret is a field, so a verb meant for one is dropped rather than
/// half-applied to whatever it landed on.
#[test]
fn selecting_something_that_is_not_a_field_is_dropped() {
    let mut grove = grove();
    let panel = grove.plant(
        Stem::new()
            .at(Location::new().xs(left(0.px()).width(10.px()), top(0.px()).height(10.px()))),
    );
    tick(&mut grove);
    grove.select(panel, 0..2);
    tick(&mut grove);
    assert_eq!(grove.tap(panel, Vein::Selection), None);
}

/// A field states its own colours, and they are ordinary fills -- so a role follows a repaint and a
/// literal does not, exactly as anywhere else.
#[test]
fn a_fields_parts_carry_the_fills_it_was_given() {
    let mut grove = grove();
    let leaf = grove.plant(
        TextInput::new()
            .color(Palette::Ink)
            .caret(Palette::Accent)
            .at(Location::new().xs(left(0.px()).width(200.px()), top(0.px()).height(32.px()))),
    );
    tick(&mut grove);
    assert_eq!(
        grove.tap(caret(&grove, leaf), Vein::Color),
        Some(Sap::Color(Palette::Accent.into()))
    );
}

/// Control makes a key a command rather than a character: it says what to do with the value instead
/// of what to put in it, and nothing is inserted.
#[test]
fn control_and_a_selects_the_whole_value() {
    assert_eq!(
        edit("hello", at(2), with_control(Key::Typed('a'))).1,
        selecting(0, 5)
    );
    // The character itself is never written, whichever case the layout reported.
    assert_eq!(
        edit("hello", at(2), with_control(Key::Typed('A'))).0,
        "hello"
    );
    // And a control chord a field has no answer for is nothing a field does, rather than a
    // character it inserts.
    assert!(matches!(
        applied("hello", at(2), with_control(Key::Typed('q'))),
        Applied::Nothing
    ));
}

/// The same, through the whole pipeline: what is held arrives as its own event, so a test and a
/// window reach the engine by the same path.
#[test]
fn control_travels_as_its_own_event() {
    let mut grove = grove();
    let leaf = focused(&mut grove);
    typing(&mut grove, "hello");
    tick(&mut grove);

    controlled(&mut grove, Key::Typed('a'));
    tick(&mut grove);
    assert_eq!(selection(&grove, leaf), 0..5);

    // And it is released with the chord, so the next character is a character again.
    typing(&mut grove, "x");
    tick(&mut grove);
    assert_eq!(value(&grove, leaf), "x");
}

/// A tap takes focus, because a field declares that it does. The engine still infers nothing -- a
/// panel that said nothing is left exactly as it was.
#[test]
fn a_tap_on_a_field_takes_focus() {
    let mut grove = grove();
    let leaf = field(&mut grove);
    tick(&mut grove);
    assert_eq!(grove.focused(), None);

    press(&mut grove, 3.0 * CELL, 16.0);
    release(&mut grove, 3.0 * CELL, 16.0);
    tick(&mut grove);
    assert_eq!(grove.focused(), Some(leaf));
    // And the caret went where the tap landed, in the same frame.
    assert_eq!(selection(&grove, leaf), 0..0);
    assert!(shown(&grove, caret(&grove, leaf)));
}

/// An app that wants focus somewhere else writes it from `clicked` and wins: the tap settled focus
/// before the frame the app reads the tap in, so the app's write is simply the later one.
#[test]
fn an_app_writing_focus_beats_the_tap() {
    let mut grove = grove();
    let leaf = field(&mut grove);
    let elsewhere = grove.plant(
        Stem::new()
            .interactive()
            .at(Location::new().xs(left(0.px()).width(10.px()), top(60.px()).height(10.px()))),
    );
    tick(&mut grove);

    press(&mut grove, 3.0 * CELL, 16.0);
    release(&mut grove, 3.0 * CELL, 16.0);
    let mut app = Observer::default();
    tick_with(&mut grove, &mut app);
    grove.focus(elsewhere);
    tick(&mut grove);
    assert_eq!(grove.focused(), Some(elsewhere));
    assert!(!shown(&grove, caret(&grove, leaf)));
}

/// `Escape` takes focus away, wherever it is. Focus's own, like `Tab`, and answered before any
/// field is asked -- so it works on a page with nothing focused at all.
#[test]
fn escape_takes_focus_away() {
    let mut grove = grove();
    let leaf = focused(&mut grove);
    assert_eq!(grove.focused(), Some(leaf));

    key(&mut grove, Key::Escape);
    tick(&mut grove);
    assert_eq!(grove.focused(), None);
    assert!(!shown(&grove, caret(&grove, leaf)));
    // And it inserted nothing on the way past.
    assert_eq!(value(&grove, leaf), "");
}

/// A gesture that turned into a scroll was never a statement about what it began on, so it leaves
/// the caret and the selection exactly as it found them -- the same rule that denies it a tap.
#[test]
fn a_drag_that_scrolls_away_leaves_the_caret_alone() {
    let mut grove = grove();
    let leaf = focused(&mut grove);
    typing(&mut grove, "abcdefgh");
    tick(&mut grove);
    grove.select(leaf, 2..6);
    tick(&mut grove);

    // Down, which the field does not take: it yields, and the gesture is no longer about it.
    press(&mut grove, 3.0 * CELL, 16.0);
    tick(&mut grove);
    drag(&mut grove, 3.0 * CELL, 120.0);
    tick(&mut grove);
    release(&mut grove, 3.0 * CELL, 120.0);
    tick(&mut grove);
    assert_eq!(selection(&grove, leaf), 2..6);
}

/// The run is drawn in front of both marks. A caret is as wide as it needs to be seen and sits on
/// the boundary between two cells, so one drawn over the run would take a bite out of the glyph it
/// stands before.
#[test]
fn the_run_is_drawn_in_front_of_the_caret_and_the_selection() {
    let mut grove = grove();
    let leaf = focused(&mut grove);
    let parts = parts(&grove, leaf);
    let (selection, run, hint, caret) = (parts[0], parts[1], parts[2], parts[3]);
    assert!(rank(&grove, run) > rank(&grove, caret));
    assert!(rank(&grove, caret) > rank(&grove, selection));
    // The hint reads in the run's own place, so it stands at the same elevation as the run --
    // separated only by the allocation order that separates any two equals.
    assert_eq!(rank(&grove, hint).stack, rank(&grove, run).stack);
}
