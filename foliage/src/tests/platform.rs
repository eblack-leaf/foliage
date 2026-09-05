//! The platform edges: the clipboard, the soft keyboard, and a URL handed to the host.
//!
//! Each of them is opened by `photosynthesize` and left shut here, so what the suite runs against
//! is the engine's half alone -- the clipboard answers from its own mirror, the keyboard records
//! what it was asked to raise and raises nothing, and a URL goes nowhere. That is deliberate twice
//! over: it is the seam the platform is behind, and it is what keeps a test off the clipboard and
//! the browser of whoever is running it.
//!
//! What is proven here is therefore everything up to that seam. A clipboard read that finishes is
//! an op, so a test pushes the op a finished read pushes and everything past it is one path -- the
//! shape [`assets`](crate::tests::assets) already takes. `navigate` and `download` have nothing on
//! this side of the seam to observe: they are a line of the host's, like the winit translation, and
//! they are answered for by the app running rather than by the suite.

use crate::interaction::input::Key;
use crate::keyboard::Keypad;
use crate::leaf::Leaf;
use crate::op::Op;
use crate::tests::{Observer, controlled, grove, tick, tick_with, typing};
use crate::{
    Boxed, Grove, Grow, Location, Place, Pollen, Sap, Source, Stem, TextInput, Vein, left, top,
};

/// A field two hundred across, with focus already on it.
fn focused(grove: &mut Grove, field: TextInput) -> Leaf {
    let leaf = grove.plant(
        field.at(Location::new().xs(left(0.px()).width(200.px()), top(0.px()).height(32.px()))),
    );
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

/// Runs one frame with an app in it and hands back what that frame told it.
fn frame(grove: &mut Grove) -> Pollen {
    let mut app = Observer::default();
    tick_with(grove, &mut app);
    app.last().clone()
}

/// A clipboard read finishing, entering the pipeline where a promise or a round trip enters it.
fn answers(grove: &mut Grove, into: Option<Leaf>, text: &str) {
    grove.queue.push(Op::Pasted {
        into,
        text: text.to_string(),
    });
}

/// What is on the clipboard, asked for the way an app asks: a `paste`, and the frames it takes to
/// come back.
fn taken(grove: &mut Grove) -> String {
    grove.paste();
    tick(grove);
    tick(grove);
    frame(grove).pasted().unwrap_or_default().to_string()
}

// The clipboard.

/// What an app puts on the clipboard is what it gets back off it, which is the whole of the
/// contract -- and it holds wherever the host will not answer, because the engine's own mirror is
/// the answer there.
#[test]
fn what_was_copied_is_what_comes_back() {
    let mut grove = grove();
    grove.copy("a link");
    tick(&mut grove);
    assert_eq!(taken(&mut grove), "a link");
}

/// Never in the frame that asked. What a clipboard holds is the host's to say, and a promise on the
/// web and a round trip to whoever owns the selection off it are both later than now -- so the
/// answer is an op and is drained in the frame it arrives in, like bytes from a path.
#[test]
fn a_paste_is_answered_in_a_later_frame() {
    let mut grove = grove();
    grove.copy("copied");
    grove.paste();
    let mut app = Observer::default();
    // The frame that drains the request.
    tick_with(&mut grove, &mut app);
    assert_eq!(app.last().pasted(), None);
    // The frame that drains the answer.
    tick_with(&mut grove, &mut app);
    assert_eq!(app.last().pasted(), None);
    // The frame that hands it over, on F7's own terms like every other report.
    tick_with(&mut grove, &mut app);
    assert_eq!(app.last().pasted(), Some("copied"));
}

/// A field and an app reach one clipboard. `Ctrl+C` is not a second store the app cannot see.
#[test]
fn ctrl_c_puts_the_selection_on_the_clipboard() {
    let mut grove = grove();
    let leaf = focused(&mut grove, TextInput::new());
    typing(&mut grove, "hello");
    tick(&mut grove);

    grove.select(leaf, 1..4);
    tick(&mut grove);
    controlled(&mut grove, Key::Typed('c'));
    tick(&mut grove);
    assert_eq!(value(&grove, leaf), "hello");
    assert_eq!(taken(&mut grove), "ell");
}

/// A caret between two characters has no span to copy, and taking the whole value instead would be
/// a rule nobody asked for.
#[test]
fn ctrl_c_with_nothing_selected_leaves_the_clipboard_alone() {
    let mut grove = grove();
    grove.copy("kept");
    let leaf = focused(&mut grove, TextInput::new());
    typing(&mut grove, "abc");
    tick(&mut grove);

    controlled(&mut grove, Key::Typed('c'));
    tick(&mut grove);
    assert_eq!(value(&grove, leaf), "abc");
    assert_eq!(taken(&mut grove), "kept");
}

/// A cut is a copy and a deletion, and the deletion is the one `Backspace` already makes -- so the
/// caret lands where the span began and the app hears one `edited`.
#[test]
fn ctrl_x_takes_the_selection_out_of_the_value() {
    let mut grove = grove();
    let leaf = focused(&mut grove, TextInput::new());
    typing(&mut grove, "hello");
    tick(&mut grove);

    grove.select(leaf, 1..4);
    tick(&mut grove);
    controlled(&mut grove, Key::Typed('x'));
    tick(&mut grove);
    assert_eq!(value(&grove, leaf), "ho");
    assert_eq!(selection(&grove, leaf), 1..1);
    assert!(frame(&mut grove).edited(leaf));
    assert_eq!(taken(&mut grove), "ell");
}

/// The one keystroke a field cannot finish on its own: the key asks, and the write happens in the
/// frame the host answers in.
#[test]
fn ctrl_v_writes_the_clipboard_in_at_the_caret() {
    let mut grove = grove();
    let leaf = focused(&mut grove, TextInput::new());
    grove.copy("there");
    typing(&mut grove, "hi ");
    tick(&mut grove);

    controlled(&mut grove, Key::Typed('v'));
    tick(&mut grove);
    assert_eq!(value(&grove, leaf), "hi ");

    tick(&mut grove);
    assert_eq!(value(&grove, leaf), "hi there");
    assert_eq!(selection(&grove, leaf), 8..8);
}

/// A paste is an insertion at the caret, so it replaces a selection for the same reason typing a
/// character does.
#[test]
fn a_paste_replaces_what_was_selected() {
    let mut grove = grove();
    let leaf = focused(&mut grove, TextInput::new());
    grove.copy("XY");
    typing(&mut grove, "hello");
    tick(&mut grove);

    grove.select(leaf, 1..4);
    tick(&mut grove);
    controlled(&mut grove, Key::Typed('v'));
    tick(&mut grove);
    tick(&mut grove);
    assert_eq!(value(&grove, leaf), "hXYo");
    assert_eq!(selection(&grove, leaf), 3..3);
}

/// What the person at the keyboard did to the value, however they did it. An app reads the paste
/// where it reads the typing.
#[test]
fn a_paste_is_reported_as_an_edit() {
    let mut grove = grove();
    let leaf = focused(&mut grove, TextInput::new());
    grove.copy("pasted");
    tick(&mut grove);

    controlled(&mut grove, Key::Typed('v'));
    tick(&mut grove);
    tick(&mut grove);
    let heard = frame(&mut grove);
    assert!(heard.edited(leaf));
    // The field's paste is not the app's: nothing asked for it here.
    assert_eq!(heard.pasted(), None);
}

/// A field is one line, and a clipboard holds whatever was on it. A newline that came in that way
/// is dropped for the reason a control character a keyboard produced is: no cell can be measured
/// for it and no caret can stand between it and the next.
#[test]
fn a_line_break_on_the_clipboard_does_not_reach_a_one_line_field() {
    let mut grove = grove();
    let leaf = focused(&mut grove, TextInput::new());
    grove.copy("two\nlines");
    tick(&mut grove);

    controlled(&mut grove, Key::Typed('v'));
    tick(&mut grove);
    tick(&mut grove);
    assert_eq!(value(&grove, leaf), "twolines");
}

/// A clipboard with nothing on it writes nothing, rather than reporting an edit that changed no
/// character.
#[test]
fn a_paste_of_nothing_is_not_an_edit() {
    let mut grove = grove();
    let leaf = focused(&mut grove, TextInput::new());
    typing(&mut grove, "kept");
    tick(&mut grove);

    controlled(&mut grove, Key::Typed('v'));
    tick(&mut grove);
    tick(&mut grove);
    assert_eq!(value(&grove, leaf), "kept");
    assert!(!frame(&mut grove).edited(leaf));
}

/// An answer outlives what asked for it, because it arrives at a moment nothing chose. Dropped like
/// any op naming something that is no longer live, and not reported to the app instead -- the app
/// did not ask for this one.
#[test]
fn a_paste_for_a_field_that_is_gone_is_dropped() {
    let mut grove = grove();
    let leaf = focused(&mut grove, TextInput::new());
    grove.prune(leaf);
    tick(&mut grove);

    answers(&mut grove, Some(leaf), "late");
    tick(&mut grove);
    assert_eq!(frame(&mut grove).pasted(), None);
}

// The soft keyboard.

/// Raised for the field that holds focus and for nothing else. Nothing declares it: focus already
/// rests only on what said it receives, and a field is the only thing that is typed into.
#[test]
fn the_keyboard_follows_focus() {
    let mut grove = grove();
    let leaf = focused(&mut grove, TextInput::new());
    assert_eq!(grove.keyboard.raised(), Some(Keypad::Text));

    grove.unfocus();
    tick(&mut grove);
    assert_eq!(grove.keyboard.raised(), None);

    grove.focus(leaf);
    tick(&mut grove);
    assert_eq!(grove.keyboard.raised(), Some(Keypad::Text));
}

/// The one thing a field says about the keyboard, because a dialling pad behind a full alphabet is
/// what the platform cannot be left to guess.
#[test]
fn a_field_raises_the_keypad_it_named() {
    let mut grove = grove();
    focused(&mut grove, TextInput::new().keypad(Keypad::Telephone));
    assert_eq!(grove.keyboard.raised(), Some(Keypad::Telephone));
}

/// Focus is not the whole of it: a button takes focus and has nothing to type into, so nothing is
/// raised for one. That is what makes a keypad a fact about a field rather than about focus.
#[test]
fn nothing_is_raised_for_what_is_not_typed_into() {
    let mut grove = grove();
    let button = grove.plant(
        Stem::new()
            .interactive()
            .at(Location::new().xs(left(0.px()).width(80.px()), top(0.px()).height(32.px()))),
    );
    tick(&mut grove);
    grove.focus(button);
    tick(&mut grove);
    assert_eq!(grove.keyboard.raised(), None);
}

/// A field that stopped being somewhere focus can rest takes the keyboard down with it, because the
/// keyboard is read from focus and focus has already let go.
#[test]
fn a_field_that_was_hidden_lowers_the_keyboard() {
    let mut grove = grove();
    let leaf = focused(&mut grove, TextInput::new());
    assert_eq!(grove.keyboard.raised(), Some(Keypad::Text));

    grove.visible(leaf, false);
    tick(&mut grove);
    assert_eq!(grove.keyboard.raised(), None);
}
