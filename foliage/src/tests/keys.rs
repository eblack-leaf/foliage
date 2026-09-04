//! Keys, as an app hears them.
//!
//! A key goes to whatever holds focus and to nothing at all if focus rests nowhere. Nothing here
//! declares that it wants keys, because there is nothing to declare: focus rests only on what said
//! `interactive`, so an element that can be focused hears keys and one that cannot never does.

use crate::interaction::input::Key;
use crate::tests::{
    Observer, controlled, grove, key, stroke, tick, tick_with, typing, with_control,
};
use crate::{
    Boxed, Grove, Grow, Leaf, Location, Panel, Place, Pollen, Source, TextInput, left, top,
};

/// A box at a stated place.
fn at(x: f32, y: f32, width: f32, height: f32) -> Location {
    Location::new().xs(
        left(x.px()).width(width.px()),
        top(y.px()).height(height.px()),
    )
}

/// An element focus can rest on, which is the whole of what it takes to be sent a key.
fn listener(grove: &mut Grove) -> Leaf {
    grove.plant(Panel::new().at(at(0.0, 0.0, 80.0, 20.0)).interactive())
}

/// Runs one frame with an app in it and hands back what that frame told it.
fn frame(grove: &mut Grove) -> Pollen {
    let mut app = Observer::default();
    tick_with(grove, &mut app);
    app.last().clone()
}

/// An element that declared nothing but `interactive` is sent what was typed at it. There is no
/// second declaration, and nothing about a field anywhere near it.
#[test]
fn what_holds_focus_is_sent_the_key() {
    let mut grove = grove();
    let listener = listener(&mut grove);
    grove.focus(listener);
    tick(&mut grove);

    key(&mut grove, Key::Enter);
    // The frame the key arrives in is the frame it is dispatched and drained in, so what it
    // produced is handed over at step 3 of the next one (F7) -- the same footing as `edited`.
    assert!(frame(&mut grove).keys(listener).is_empty());
    assert_eq!(frame(&mut grove).keys(listener), [stroke(Key::Enter)]);

    // Reported once, like everything else in a drift that was taken.
    assert!(frame(&mut grove).keys(listener).is_empty());
}

/// The one ordered read in `Pollen`. Two keys in a frame mean different things in each order, which
/// is the reason F1 keeps keystrokes ordered and the reason this is not a set like everything else.
#[test]
fn keys_are_reported_in_the_order_they_arrived() {
    let mut grove = grove();
    let listener = listener(&mut grove);
    grove.focus(listener);
    tick(&mut grove);

    typing(&mut grove, "ab");
    tick(&mut grove);
    assert_eq!(
        frame(&mut grove).keys(listener),
        [stroke(Key::Typed('a')), stroke(Key::Typed('b'))]
    );

    typing(&mut grove, "ba");
    tick(&mut grove);
    assert_eq!(
        frame(&mut grove).keys(listener),
        [stroke(Key::Typed('b')), stroke(Key::Typed('a'))]
    );
}

/// What a key was held with travels with it, which is what makes a chord readable as one thing.
#[test]
fn a_modifier_travels_with_the_key_it_was_held_for() {
    let mut grove = grove();
    let listener = listener(&mut grove);
    grove.focus(listener);
    tick(&mut grove);

    controlled(&mut grove, Key::Typed('s'));
    tick(&mut grove);
    let heard = frame(&mut grove);
    assert_eq!(heard.keys(listener), [with_control(Key::Typed('s'))]);
    assert!(heard.keys(listener)[0].modifiers.control);
}

/// A key that arrived with focus nowhere is the app's own. It reaches no element, because there is
/// no element it is about -- a chord for the whole page has nothing in the page to belong to.
#[test]
fn a_key_with_nothing_focused_goes_to_the_app() {
    let mut grove = grove();
    let listener = listener(&mut grove);
    tick(&mut grove);
    assert_eq!(grove.focused(), None);

    controlled(&mut grove, Key::Typed('s'));
    tick(&mut grove);
    let heard = frame(&mut grove);
    assert_eq!(heard.root_keys(), [with_control(Key::Typed('s'))]);
    assert!(heard.keys(listener).is_empty());
}

/// The two reads are one question answered twice over: focus rests somewhere or it does not, so a
/// key is never both an element's and the app's.
#[test]
fn a_key_is_reported_to_one_place_or_the_other() {
    let mut grove = grove();
    let listener = listener(&mut grove);
    grove.focus(listener);
    tick(&mut grove);

    key(&mut grove, Key::Typed('x'));
    tick(&mut grove);
    let heard = frame(&mut grove);
    assert_eq!(heard.keys(listener), [stroke(Key::Typed('x'))]);
    assert!(heard.root_keys().is_empty());
}

/// An element that cannot take focus is never sent one. Nothing refuses the key on its behalf --
/// focus simply had nowhere to go, so the key was the app's.
#[test]
fn an_element_that_cannot_be_focused_hears_nothing() {
    let mut grove = grove();
    let quiet = grove.plant(Panel::new().at(at(0.0, 0.0, 80.0, 20.0)));
    grove.focus(quiet);
    tick(&mut grove);
    assert_eq!(grove.focused(), None);

    key(&mut grove, Key::Typed('x'));
    tick(&mut grove);
    let heard = frame(&mut grove);
    assert!(heard.keys(quiet).is_empty());
    assert_eq!(heard.root_keys(), [stroke(Key::Typed('x'))]);
}

/// A field is told what it was sent as well. What a seed makes of a key is not a claim on it, so
/// the keys that edited a value are readable beside the `edited` they produced.
#[test]
fn a_field_hears_the_keys_it_was_edited_with() {
    let mut grove = grove();
    let field = grove.plant(TextInput::new().at(at(0.0, 0.0, 200.0, 24.0)));
    grove.focus(field);
    tick(&mut grove);

    typing(&mut grove, "hi");
    tick(&mut grove);
    let heard = frame(&mut grove);
    assert!(heard.edited(field));
    assert_eq!(
        heard.keys(field),
        [stroke(Key::Typed('h')), stroke(Key::Typed('i'))]
    );
}

/// `Tab` and `Escape` are focus's own and are answered before any element is asked, so neither is
/// ever reported as a key an element was sent. An app that wants its own answer to either is asking
/// for the two to mean two things at once.
#[test]
fn the_keys_that_steer_focus_are_not_delivered() {
    let mut grove = grove();
    let listener = listener(&mut grove);
    grove.focus(listener);
    tick(&mut grove);

    key(&mut grove, Key::Tab);
    tick(&mut grove);
    let heard = frame(&mut grove);
    assert!(heard.keys(listener).is_empty());
    assert!(heard.root_keys().is_empty());

    key(&mut grove, Key::Escape);
    tick(&mut grove);
    let heard = frame(&mut grove);
    assert!(heard.keys(listener).is_empty());
    assert!(heard.root_keys().is_empty());
}
