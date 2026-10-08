//! TextArea: a field whose value wraps, and the keys that are about lines.
//!
//! Everything a field does, an area does the same way -- what is proven in `text_input` holds and
//! is not restated. What is proven here is the difference: a line is a fact about the wrap, and the
//! caret, the selection and the keys that move by line are read off it. The first half is pure, with
//! the wrap fed in; the second runs the whole frame, where the wrap comes from a real box and the
//! caret becomes a box against it.

use crate::coordinate::Area;
use crate::interaction::input::{Key, Keystroke};
use crate::tests::{
    controlled, drag, grove, key, past_the_hold, press, release, resize, section, shifted, stroke,
    tick, typing, with_control, with_shift,
};
use crate::text::shape::shape;
use crate::text_input::{Applied, Editing, Lines, applied, inserted, word_at};
use crate::{Boxed, Grove, Grow, Leaf, Location, Sap, Source, TextArea, Vein, left, top};

// Editing, against a wrap.

/// Applies a key against `value` wrapped at `columns`, and reports what the value and the caret
/// became.
///
/// The caret is reported as where it stands and what it selects. The column a line move carries is
/// left out, because what it is for is the move after -- which is [`moves`]'s to show.
fn edit(value: &str, columns: usize, editing: Editing, stroke: Keystroke) -> (String, Editing) {
    let shaped = shape(value, Area::new(10.0, 22.0));
    let placed = |editing: Editing| Editing {
        goal: None,
        ..editing
    };
    match applied(value, editing, stroke, shaped.wrap(columns), Lines::Many) {
        Applied::Wrote(written, editing) => (written, placed(editing)),
        Applied::Moved(editing) => (value.to_string(), placed(editing)),
        _ => panic!("expected a write or a move"),
    }
}

/// Applies keys one after another against `value` wrapped at `columns`, each from wherever the last
/// left the caret -- the column it carries included -- and reports where they left it.
fn moves(value: &str, columns: usize, from: Editing, strokes: &[Keystroke]) -> Editing {
    let shaped = shape(value, Area::new(10.0, 22.0));
    let ended = strokes.iter().fold(from, |editing, stroke| {
        match applied(value, editing, *stroke, shaped.wrap(columns), Lines::Many) {
            Applied::Moved(editing) => editing,
            _ => panic!("expected a move"),
        }
    });
    Editing {
        goal: None,
        ..ended
    }
}

/// Applies a key to a field of one line.
fn one(value: &str, editing: Editing, stroke: Keystroke) -> Applied {
    let shaped = shape(value, Area::new(10.0, 22.0));
    applied(value, editing, stroke, shaped.wrap(usize::MAX), Lines::One)
}

fn at(index: usize) -> Editing {
    Editing::at(index)
}

fn selecting(anchor: usize, caret: usize) -> Editing {
    Editing {
        anchor,
        caret,
        goal: None,
    }
}

/// `Enter` is the one character a key cannot otherwise put in the value, so on an area it is that
/// character; the chord is what submits, and it submits without touching the value.
#[test]
fn enter_writes_a_newline_and_control_enter_submits() {
    assert_eq!(
        edit("ab", 20, at(1), stroke(Key::Enter)),
        ("a\nb".to_string(), at(2))
    );
    // Over a selection, like any character.
    assert_eq!(
        edit("hello", 20, selecting(1, 4), stroke(Key::Enter)),
        ("h\no".to_string(), at(2))
    );
    let shaped = shape("ab", Area::new(10.0, 22.0));
    assert!(matches!(
        applied(
            "ab",
            at(1),
            with_control(Key::Enter),
            shaped.wrap(20),
            Lines::Many
        ),
        Applied::Submitted
    ));
    // And the chord means the same thing on a field, so it is one chord wherever it is pressed.
    assert!(matches!(
        one("ab", at(1), with_control(Key::Enter)),
        Applied::Submitted
    ));
}

/// `Up` and `Down` go to the same column of the line above or below, as the value wrapped: "hello"
/// over "world" in five cells, so the caret before 'r' is below the caret before 'l'.
#[test]
fn up_and_down_move_by_a_wrapped_line() {
    assert_eq!(edit("hello world", 5, at(8), stroke(Key::Up)).1, at(2));
    assert_eq!(edit("hello world", 5, at(2), stroke(Key::Down)).1, at(8));
    // A hard line reads the same way.
    assert_eq!(edit("ab\ncd", 20, at(4), stroke(Key::Up)).1, at(1));
    assert_eq!(edit("ab\ncd", 20, at(1), stroke(Key::Down)).1, at(4));
}

/// Past the first line up is the start of the value, and past the last line down is its end --
/// which is where every editor goes, and where a reader stepping through a value expects to stop.
#[test]
fn up_and_down_stop_at_the_ends_of_the_value() {
    assert_eq!(edit("hello world", 5, at(3), stroke(Key::Up)).1, at(0));
    assert_eq!(edit("hello world", 5, at(8), stroke(Key::Down)).1, at(11));
}

/// A column past the end of a shorter line is that line's end, because the column is not somewhere
/// the caret can stand there -- and the line after it has the caret back at the column it set out
/// from, because a run of line moves keeps the column the first of them left.
#[test]
fn a_shorter_line_passed_through_keeps_the_column() {
    let (up, down) = (stroke(Key::Up), stroke(Key::Down));
    // "hello" over "hi" over "world", setting out from after "hell".
    assert_eq!(moves("hello\nhi\nworld", 20, at(4), &[down]), at(8));
    assert_eq!(moves("hello\nhi\nworld", 20, at(4), &[down, down]), at(13));
    assert_eq!(moves("hello\nhi\nworld", 20, at(13), &[up]), at(8));
    assert_eq!(moves("hello\nhi\nworld", 20, at(13), &[up, up]), at(4));
    // And back down from the top again: going past the ends of the value is still a line move.
    assert_eq!(
        moves("hello\nhi\nworld", 20, at(4), &[up, down, down]),
        at(13)
    );
}

/// Any other move is a new place to set out from: the column a horizontal move leaves is the one
/// the next line move keeps.
#[test]
fn any_other_move_sets_out_from_where_it_left_the_caret() {
    let (down, left) = (stroke(Key::Down), stroke(Key::Left));
    assert_eq!(
        moves("hello\nhi\nworld", 20, at(4), &[down, left, down]),
        at(10)
    );
    // Shift keeps the column as it keeps the anchor.
    let held = with_shift(Key::Down);
    assert_eq!(
        moves("hello\nhi\nworld", 20, at(4), &[held, held]),
        selecting(4, 13)
    );
}

/// A word move is not about lines: a newline is space like any other, so the caret goes on across
/// the end of a line to the word beyond it.
#[test]
fn a_word_move_crosses_the_end_of_a_line() {
    let (back, on) = (with_control(Key::Left), with_control(Key::Right));
    assert_eq!(edit("ab\ncd", 20, at(5), back).1, at(3));
    assert_eq!(edit("ab\ncd", 20, at(3), back).1, at(0));
    assert_eq!(edit("ab\ncd", 20, at(2), on).1, at(5));
}

/// A word is chosen within its own line. A tap past the end of a line, wrapped or broken, means the
/// last word on it, and a line with nothing on it has no word.
#[test]
fn a_word_under_a_tap_is_read_off_the_wrap() {
    let shaped = shape("hello world", Area::new(10.0, 22.0));
    let run = shaped.wrap(5);
    // The space the wrap carried to the end of the first line is past its last word.
    assert_eq!(word_at(run, 5), 0..5);
    assert_eq!(word_at(run, 11), 6..11);
    let shaped = shape("ab  \n\ncd", Area::new(10.0, 22.0));
    let run = shaped.wrap(20);
    // Space at the end of a line is a word of its own, and does not take the newline with it.
    assert_eq!(word_at(run, 3), 2..4);
    assert_eq!(word_at(run, 4), 0..2);
    assert_eq!(word_at(run, 5), 5..5);
}

/// Shift holds the anchor, so a line-wise move is a line-wise selection.
#[test]
fn a_shifted_line_move_extends_from_the_anchor() {
    assert_eq!(
        edit("hello world", 5, at(2), with_shift(Key::Down)).1,
        selecting(2, 8)
    );
    assert_eq!(
        edit("hello world", 5, selecting(2, 8), with_shift(Key::Up)).1,
        selecting(2, 2)
    );
}

/// `Home` and `End` are the ends of the line the caret is on, as it wrapped -- not of the value, and
/// not of the hard line, because the line a reader sees is the wrapped one.
#[test]
fn home_and_end_go_to_the_ends_of_the_wrapped_line() {
    assert_eq!(edit("hello world", 5, at(8), stroke(Key::Home)).1, at(6));
    assert_eq!(edit("hello world", 5, at(8), stroke(Key::End)).1, at(11));
    // The end of the first line is before the space the break took, not before the next word.
    assert_eq!(edit("hello world", 5, at(2), stroke(Key::End)).1, at(5));
    assert_eq!(
        edit("hello world", 5, at(8), with_shift(Key::Home)).1,
        selecting(8, 6)
    );
}

/// The keys a field has no line for do nothing on it, exactly as before an area existed.
#[test]
fn a_field_of_one_line_still_has_no_line_to_move_to() {
    assert!(matches!(
        one("hello", at(2), stroke(Key::Up)),
        Applied::Nothing
    ));
    assert!(matches!(
        one("hello", at(2), stroke(Key::Enter)),
        Applied::Submitted
    ));
}

/// A newline on the clipboard is a character an area holds and a field does not, and a carriage
/// return is a newline's other spelling.
#[test]
fn a_pasted_line_break_is_kept_by_an_area_and_dropped_by_a_field() {
    let kept = match inserted("ab", at(1), "x\r\ny\nz", Lines::Many) {
        Applied::Wrote(written, editing) => (written, editing),
        _ => panic!("expected a write"),
    };
    assert_eq!(kept, ("ax\ny\nzb".to_string(), at(6)));
    let dropped = match inserted("ab", at(1), "x\r\ny\nz", Lines::One) {
        Applied::Wrote(written, editing) => (written, editing),
        _ => panic!("expected a write"),
    };
    assert_eq!(dropped, ("axyzb".to_string(), at(4)));
}

// Through the frame.

/// The default font's cell at the default size, which every box below is a multiple of.
const CELL: f32 = 10.0;
const LINE: f32 = 22.0;

/// An area five cells wide and two lines tall, at the origin, with focus on it.
fn area(grove: &mut Grove) -> Leaf {
    let leaf = grove.plant(TextArea::new().placeholder("notes").at(Location::new().xs(
        left(0.px()).width((5.0 * CELL).px()),
        top(0.px()).height((2.0 * LINE).px()),
    )));
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

fn selection(grove: &Grove, leaf: Leaf) -> core::ops::Range<usize> {
    match grove.tap(leaf, Vein::Selection) {
        Some(Sap::Selection(range)) => range,
        other => panic!("expected a selection, got {other:?}"),
    }
}

/// How far the area has been scrolled down.
fn offset(grove: &Grove, leaf: Leaf) -> f32 {
    match grove.tap(leaf, Vein::Offset) {
        Some(Sap::Position(offset)) => offset.y,
        other => panic!("expected an offset, got {other:?}"),
    }
}

/// The parts, in the order they were grown: the selection's head, body and tail, then the run, the
/// hint and the caret.
fn parts(grove: &Grove, leaf: Leaf) -> Vec<Leaf> {
    match grove.tap(leaf, Vein::Branches) {
        Some(Sap::Leaves(leaves)) => leaves,
        other => panic!("expected branches, got {other:?}"),
    }
}

fn caret(grove: &Grove, leaf: Leaf) -> Leaf {
    parts(grove, leaf)[5]
}

fn head(grove: &Grove, leaf: Leaf) -> Leaf {
    parts(grove, leaf)[0]
}

fn body(grove: &Grove, leaf: Leaf) -> Leaf {
    parts(grove, leaf)[1]
}

fn tail(grove: &Grove, leaf: Leaf) -> Leaf {
    parts(grove, leaf)[2]
}

fn shown(grove: &Grove, leaf: Leaf) -> bool {
    grove.elements.inherited(leaf).visible
}

/// Whether a piece of the selection puts anything on the surface: shown, and with a box that has
/// both a width and a height. A piece a span does not need is still shown and resolves to nothing.
fn drawn(grove: &Grove, leaf: Leaf) -> bool {
    let area = section(grove, leaf).area;
    shown(grove, leaf) && area.width > 0.0 && area.height > 0.0
}

/// The same six elements a field is, under one name, and the value it says is read the same way.
#[test]
fn an_area_is_a_field_that_holds_newlines() {
    let mut grove = grove();
    let leaf = area(&mut grove);
    assert_eq!(parts(&grove, leaf).len(), 6);

    typing(&mut grove, "ab");
    key(&mut grove, Key::Enter);
    typing(&mut grove, "cd");
    tick(&mut grove);
    assert_eq!(value(&grove, leaf), "ab\ncd");
    assert_eq!(selection(&grove, leaf), 5..5);
}

/// The caret stands in the cell the run wrapped its character into: a line down for a newline, and
/// a line down again when the value runs out of the box's width.
#[test]
fn the_caret_stands_on_the_line_its_character_wrapped_to() {
    let mut grove = grove();
    let leaf = area(&mut grove);
    typing(&mut grove, "ab");
    key(&mut grove, Key::Enter);
    typing(&mut grove, "cd");
    tick(&mut grove);
    let mark = section(&grove, caret(&grove, leaf));
    assert_eq!((mark.left(), mark.top()), (2.0 * CELL, LINE));
    assert_eq!(mark.area.height, LINE);

    // Back to the end of the first line, which is before the newline.
    key(&mut grove, Key::Up);
    tick(&mut grove);
    assert_eq!(selection(&grove, leaf), 2..2);
    let mark = section(&grove, caret(&grove, leaf));
    assert_eq!((mark.left(), mark.top()), (2.0 * CELL, 0.0));

    // "hello world" in five cells is "hello" over "world", and the caret after 'd' is at the end
    // of the second line.
    grove.text(leaf, "hello world");
    tick(&mut grove);
    let mark = section(&grove, caret(&grove, leaf));
    assert_eq!((mark.left(), mark.top()), (5.0 * CELL, LINE));
}

/// The caret is read off the wrap on the frame the wrap changes, not off where it was last seen: a
/// box that narrows under a value moves the caret down in the same frame the value wraps.
#[test]
fn the_caret_follows_a_wrap_that_moved_under_it_in_the_same_frame() {
    let mut grove = grove();
    let leaf = grove.plant(TextArea::new().at(Location::new().xs(
        left(0.px()).right(100.pct()),
        top(0.px()).height((4.0 * LINE).px()),
    )));
    tick(&mut grove);
    grove.focus(leaf);
    grove.text(leaf, "hello world");
    tick(&mut grove);
    // Forty cells across: one line, and the caret after the last character.
    let mark = section(&grove, caret(&grove, leaf));
    assert_eq!((mark.left(), mark.top()), (11.0 * CELL, 0.0));

    resize(&mut grove, Area::new(5.0 * CELL, 300.0));
    tick(&mut grove);
    let mark = section(&grove, caret(&grove, leaf));
    assert_eq!((mark.left(), mark.top()), (5.0 * CELL, LINE));
}

/// An area scrolls down, and the caret is what it is kept showing: typing past the bottom of the
/// box brings the caret back into view in the frame it was typed in, and going back to the top
/// brings the value with it.
#[test]
fn the_area_follows_its_caret_down() {
    let mut grove = grove();
    let leaf = area(&mut grove);
    // Four lines in a box two lines tall.
    typing(&mut grove, "a");
    key(&mut grove, Key::Enter);
    typing(&mut grove, "b");
    key(&mut grove, Key::Enter);
    typing(&mut grove, "c");
    key(&mut grove, Key::Enter);
    typing(&mut grove, "d");
    tick(&mut grove);
    assert!(offset(&grove, leaf) > 0.0, "the area scrolled to its caret");
    let mark = section(&grove, caret(&grove, leaf));
    assert!(mark.bottom() <= 2.0 * LINE + 0.01);
    assert!(mark.top() >= -0.01);

    grove.select(leaf, 0..0);
    tick(&mut grove);
    assert_eq!(offset(&grove, leaf), 0.0);
}

/// A press puts the caret on the line it landed on and at the nearer cell boundary across, read
/// off the wrap.
#[test]
fn a_press_puts_the_caret_on_the_line_it_landed_on() {
    let mut grove = grove();
    let leaf = area(&mut grove);
    grove.text(leaf, "hello world");
    tick(&mut grove);

    // Halfway down the second line, just past the middle of its second cell.
    press(&mut grove, 1.0 * CELL + 6.0, 1.5 * LINE);
    release(&mut grove, 1.0 * CELL + 6.0, 1.5 * LINE);
    tick(&mut grove);
    assert_eq!(selection(&grove, leaf), 8..8);

    // Past the right edge of the first line is its end, which is before the space.
    press(&mut grove, 4.0 * CELL + 9.0, 0.5 * LINE);
    release(&mut grove, 4.0 * CELL + 9.0, 0.5 * LINE);
    tick(&mut grove);
    assert_eq!(selection(&grove, leaf), 5..5);
}

/// A selection within one line is one box; across two it is the rest of the first and the start of
/// the second, and nothing between.
#[test]
fn a_selection_across_two_lines_is_drawn_in_two_pieces() {
    let mut grove = grove();
    let leaf = area(&mut grove);
    grove.text(leaf, "hello world");
    tick(&mut grove);

    grove.select(leaf, 1..3);
    tick(&mut grove);
    let first = section(&grove, head(&grove, leaf));
    assert_eq!((first.left(), first.right()), (1.0 * CELL, 3.0 * CELL));
    assert_eq!(first.top(), 0.0);
    assert!(drawn(&grove, head(&grove, leaf)));
    assert!(!drawn(&grove, body(&grove, leaf)));
    assert!(!drawn(&grove, tail(&grove, leaf)));

    grove.select(leaf, 2..8);
    tick(&mut grove);
    let first = section(&grove, head(&grove, leaf));
    assert_eq!((first.left(), first.right()), (2.0 * CELL, 5.0 * CELL));
    assert_eq!(first.top(), 0.0);
    let last = section(&grove, tail(&grove, leaf));
    assert_eq!((last.left(), last.right()), (0.0, 2.0 * CELL));
    assert_eq!(last.top(), LINE);
    assert!(drawn(&grove, head(&grove, leaf)));
    assert!(!drawn(&grove, body(&grove, leaf)));
    assert!(drawn(&grove, tail(&grove, leaf)));
}

/// Across more than two lines, every whole line between the ends is one box the width of the run.
#[test]
fn a_selection_across_many_lines_fills_the_lines_between() {
    let mut grove = grove();
    let leaf = area(&mut grove);
    grove.text(leaf, "aaaa bbbb cccc dddd");
    tick(&mut grove);

    grove.select(leaf, 1..16);
    tick(&mut grove);
    let middle = section(&grove, body(&grove, leaf));
    assert!(drawn(&grove, body(&grove, leaf)));
    assert_eq!((middle.left(), middle.right()), (0.0, 5.0 * CELL));
    // Placed against the run, which has scrolled up under the caret: two whole lines, starting one
    // line under the head.
    let first = section(&grove, head(&grove, leaf));
    assert_eq!(middle.top(), first.top() + LINE);
    assert_eq!(middle.area.height, 2.0 * LINE);
    let last = section(&grove, tail(&grove, leaf));
    assert_eq!(last.top(), middle.bottom());
    assert_eq!((last.left(), last.right()), (0.0, 1.0 * CELL));
}

/// A selection is divided into lines by the wrap the frame lays out, not the one it last drew: a
/// box that narrows under a selection on one line draws it across two in the same frame. Nothing
/// is owed the frame after, so a frame behind would be a selection left wrong until the next input.
#[test]
fn a_selection_follows_a_wrap_that_moved_under_it_in_the_same_frame() {
    let mut grove = grove();
    let leaf = grove.plant(TextArea::new().at(Location::new().xs(
        left(0.px()).right(100.pct()),
        top(0.px()).height((4.0 * LINE).px()),
    )));
    tick(&mut grove);
    grove.focus(leaf);
    grove.text(leaf, "hello world");
    tick(&mut grove);
    grove.select(leaf, 2..8);
    tick(&mut grove);
    // Forty cells across: one line, and the head is the whole of it.
    let first = section(&grove, head(&grove, leaf));
    assert_eq!((first.left(), first.right()), (2.0 * CELL, 8.0 * CELL));
    assert!(!drawn(&grove, tail(&grove, leaf)));

    resize(&mut grove, Area::new(5.0 * CELL, 300.0));
    tick(&mut grove);
    let first = section(&grove, head(&grove, leaf));
    assert_eq!((first.left(), first.right()), (2.0 * CELL, 5.0 * CELL));
    let last = section(&grove, tail(&grove, leaf));
    assert_eq!((last.left(), last.right()), (0.0, 2.0 * CELL));
    assert_eq!(last.top(), LINE);
    assert!(drawn(&grove, tail(&grove, leaf)));
    assert!(!drawn(&grove, body(&grove, leaf)));
}

/// A drag out of a hold that runs below the box keeps scrolling, because down is the axis an area
/// reaches along -- and it keeps the end it started from, exactly as a field does across.
#[test]
fn a_drag_held_below_the_area_keeps_scrolling() {
    let mut grove = grove();
    let leaf = area(&mut grove);
    grove.text(leaf, "aaaa bbbb cccc dddd eeee");
    tick(&mut grove);
    grove.select(leaf, 0..0);
    tick(&mut grove);
    assert_eq!(offset(&grove, leaf), 0.0);

    press(&mut grove, 1.0 * CELL, 0.5 * LINE);
    tick(&mut grove);
    past_the_hold(&mut grove);
    tick(&mut grove);
    assert_eq!(selection(&grove, leaf), 1..1);

    drag(&mut grove, 1.0 * CELL, 3.0 * LINE);
    tick(&mut grove);
    let (moved, selected) = (offset(&grove, leaf), selection(&grove, leaf));
    assert!(moved > 0.0, "the area followed the caret");
    assert_eq!(selected.start, 1);
    assert!(grove.again, "the area is asking for the frames");

    tick(&mut grove);
    assert!(offset(&grove, leaf) > moved, "it kept going");
    assert!(selection(&grove, leaf).end > selected.end);
    assert_eq!(selection(&grove, leaf).start, 1);
}

/// A press is a new place to set out from: the line move after it keeps the column the press left
/// the caret at, not the one the moves before it were keeping.
#[test]
fn a_press_sets_out_afresh() {
    let mut grove = grove();
    let leaf = grove.plant(TextArea::new().at(Location::new().xs(
        left(0.px()).width((20.0 * CELL).px()),
        top(0.px()).height((4.0 * LINE).px()),
    )));
    tick(&mut grove);
    grove.focus(leaf);
    grove.text(leaf, "hello\nhi\nworld");
    grove.select(leaf, 4..4);
    tick(&mut grove);

    key(&mut grove, Key::Down);
    key(&mut grove, Key::Down);
    tick(&mut grove);
    assert_eq!(selection(&grove, leaf), 13..13);

    // After the 'h' of "hi", on the line between.
    press(&mut grove, 1.0 * CELL, 1.5 * LINE);
    release(&mut grove, 1.0 * CELL, 1.5 * LINE);
    tick(&mut grove);
    assert_eq!(selection(&grove, leaf), 7..7);
    key(&mut grove, Key::Down);
    tick(&mut grove);
    assert_eq!(selection(&grove, leaf), 10..10);
}

/// `Ctrl+A` and the clipboard chords are a field's, whichever kind it is.
#[test]
fn the_chords_reach_an_area() {
    let mut grove = grove();
    let leaf = area(&mut grove);
    grove.text(leaf, "ab\ncd");
    tick(&mut grove);
    controlled(&mut grove, Key::Typed('a'));
    tick(&mut grove);
    assert_eq!(selection(&grove, leaf), 0..5);
    // A selection reaching backwards, from a shifted arrow, reads the same way.
    grove.select(leaf, 5..5);
    tick(&mut grove);
    shifted(&mut grove, Key::Up);
    tick(&mut grove);
    assert_eq!(selection(&grove, leaf), 2..5);
}
