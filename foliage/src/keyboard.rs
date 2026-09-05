//! The soft keyboard -- raised by focus, and by nothing else.
//!
//! A phone has no keyboard until something asks for one, and the thing that asks is a field holding
//! focus. That is the whole rule, and it needs no declaration of its own: focus already rests only
//! on what said [`interactive`](crate::Place::interactive), and a [`TextInput`](crate::TextInput)
//! is the only thing in the engine that is typed *into*. So the keyboard is raised for the field
//! that holds focus and lowered for everything else, decided in the same breath as the caret's
//! visibility and from the same state.
//!
//! What a field may say is which **[`Keypad`]** it wants, because a telephone number behind a full
//! alphabet is the one thing the platform cannot be left to guess.
//!
//! # The web owns the keyboard, so the web owns the keys
//!
//! There is no way to raise a browser's keyboard except by giving DOM focus to a real input, and
//! doing that takes keyboard focus off the canvas winit is listening on -- so every key typed after
//! it would be swallowed by an input the person cannot see. The hidden input therefore hands them
//! back: what it composes arrives as text and what it does not is read from the key itself, both
//! entering [`Incoming`](crate::interaction::input::Incoming) exactly where a translated winit
//! event enters it.
//!
//! Off the web nothing is raised. Android would be the other platform with a keyboard to raise and
//! there is no Android build to raise it from, so what is here is the seam rather than a stub.

use tracing::debug;

use bevy_ecs::component::Component;

/// Which keys a soft keyboard offers.
///
/// A hint and not a constraint: it changes what is easy to type and never what a field accepts, so
/// a [`Keypad::Number`] field can still be pasted a word into and is still the app's to validate.
/// Ignored where the platform has no keyboard of its own to raise.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Keypad {
    /// Letters. What a field asks for unless it says otherwise.
    #[default]
    Text,
    /// Digits, for a quantity or a code.
    Number,
    /// A dialling pad.
    Telephone,
}

impl Keypad {
    /// What the browser is told to offer, as `inputmode` names it.
    #[cfg_attr(not(target_family = "wasm"), allow(dead_code))]
    fn mode(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Number => "numeric",
            Self::Telephone => "tel",
        }
    }
}

/// The soft keyboard, and what it is currently showing.
///
/// What is raised is engine state on every target, and raising it is the platform's -- so the rule
/// that focus decides the keyboard is proven headlessly while the DOM half stays the edge the suite
/// cannot reach, exactly as the winit translation is.
#[derive(Default)]
pub(crate) struct Keyboard {
    /// What is up, if anything is. Kept so the platform is asked only when the answer changes.
    raised: Option<Keypad>,
    /// The hidden input the browser raises its keyboard for. Absent until
    /// [`attach`](Keyboard::attach), and absent for good under the headless suite.
    #[cfg(target_family = "wasm")]
    trigger: Option<web::Trigger>,
}

impl Keyboard {
    /// What is raised, if anything is.
    ///
    /// Read by the headless suite, which is the only side of the seam that can observe it: asking
    /// for a keyboard is the engine's and raising one is the host's.
    #[cfg(test)]
    pub(crate) fn raised(&self) -> Option<Keypad> {
        self.raised
    }

    /// Builds what the platform raises its keyboard for, once, at boot.
    ///
    /// `wake` is what a key captured outside a frame rouses the loop with -- while the hidden input
    /// holds DOM focus the canvas hears nothing, so nothing else would ask for the frame that
    /// delivers it.
    #[cfg_attr(not(target_family = "wasm"), allow(unused_variables))]
    pub(crate) fn attach(&mut self, wake: &crate::queue::Wake) {
        #[cfg(target_family = "wasm")]
        {
            self.trigger = web::Trigger::build(wake.clone());
        }
    }

    /// Raises the keyboard `wanted` asks for, or lowers whatever is up.
    ///
    /// Called every frame with the whole answer rather than told about changes, because the answer
    /// is a product of focus and is recomputed like every other product.
    ///
    /// What is *up* is compared, so the mode is written and the trace is made only where it moved.
    /// DOM focus is not: the page can take it off the hidden input without the engine hearing about
    /// it -- a press on the canvas does exactly that, and is how every tap from one field to
    /// another arrives -- so it is asserted on every frame instead. Focusing what already holds
    /// focus does nothing, which is what makes asserting it free.
    pub(crate) fn raise(&mut self, wanted: Option<Keypad>) {
        let moved = wanted != self.raised;
        if moved {
            self.raised = wanted;
            debug!(keypad = ?wanted, "keyboard");
        }
        #[cfg(target_family = "wasm")]
        if let Some(trigger) = &self.trigger {
            match wanted {
                Some(keypad) => {
                    if moved {
                        trigger.mode(keypad);
                    }
                    trigger.focus();
                }
                None if moved => trigger.blur(),
                None => {}
            }
        }
    }

    /// What the hidden input captured since the last frame, in the order it arrived.
    ///
    /// Empty everywhere but the web, and empty there until something has been typed into a keyboard
    /// this raised.
    pub(crate) fn captured(&mut self) -> Vec<crate::interaction::input::Input> {
        #[cfg(target_family = "wasm")]
        if let Some(trigger) = &self.trigger {
            return trigger.captured();
        }
        Vec::new()
    }
}

/// The browser's half: one hidden input, and the keys it takes from the canvas given back.
#[cfg(target_family = "wasm")]
mod web {
    use std::cell::RefCell;
    use std::rc::Rc;

    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;

    use super::Keypad;
    use crate::interaction::input::{Input, Key, Modifiers};
    use crate::queue::Wake;

    /// The id the hidden input is found by. One element rather than one per [`Keypad`]: the pad is
    /// `inputmode`, which is written before focus moves, so there is nothing three elements would
    /// say that one cannot.
    const TRIGGER: &str = "foliage-keypad";

    /// Off screen rather than `display:none`, because an element that is not laid out cannot take
    /// focus and an element that cannot take focus raises no keyboard. Everything that would let it
    /// second-guess what is typed is off: the value is read and cleared on every event, so an
    /// autocorrection would be applied to a value the person never sees.
    const STYLE: &str = "position:absolute;left:-1px;top:-1px;width:0;height:0;\
        min-width:0;min-height:0;padding:0;border:0;opacity:0";

    /// What the hidden input has captured and no frame has taken yet.
    ///
    /// The browser is single-threaded, so an `Rc` is the whole of what sharing this with the DOM's
    /// own callbacks needs.
    #[derive(Clone, Default)]
    struct Captured(Rc<RefCell<Vec<Input>>>);

    /// The hidden input, and what it has taken.
    pub(super) struct Trigger {
        element: web_sys::HtmlInputElement,
        captured: Captured,
    }

    impl Trigger {
        /// Puts the input in the page and wires it to the input stream.
        ///
        /// `None` where there is no document to put it in, which is every host that is not a
        /// browser -- the same absence the headless suite has, reached a different way.
        pub(super) fn build(wake: Wake) -> Option<Self> {
            let document = web_sys::window()?.document()?;
            let element = document
                .create_element("input")
                .ok()?
                .dyn_into::<web_sys::HtmlInputElement>()
                .ok()?;
            element.set_id(TRIGGER);
            element.set_type("text");
            for (name, value) in [
                ("style", STYLE),
                ("autocapitalize", "off"),
                ("autocomplete", "off"),
                ("autocorrect", "off"),
                ("spellcheck", "false"),
                ("aria-hidden", "true"),
            ] {
                element.set_attribute(name, value).ok()?;
            }
            document.body()?.append_child(&element).ok()?;
            let captured = Captured::default();
            listen(&element, &captured, &wake);
            Some(Self { element, captured })
        }

        /// Says which keys to offer, before anything is offered.
        ///
        /// A mode written to an input that already holds focus changes nothing the person can see
        /// until the keyboard has been dismissed and raised again, so this is only ever written
        /// while the keyboard is down.
        pub(super) fn mode(&self, keypad: Keypad) {
            let _ = self.element.set_attribute("inputmode", keypad.mode());
        }

        /// Gives the input DOM focus, which is the only thing a browser raises a keyboard for.
        pub(super) fn focus(&self) {
            let _ = self.element.focus();
        }

        pub(super) fn blur(&self) {
            let _ = self.element.blur();
        }

        pub(super) fn captured(&self) -> Vec<Input> {
            core::mem::take(&mut *self.captured.0.borrow_mut())
        }
    }

    /// Wires the two events a hidden input has to give back.
    ///
    /// `input` is what was composed -- a plain letter, a dead-key sequence, a whole word an IME
    /// committed -- and is the only one that answers for text, because the browser is what resolved
    /// the layout. `keydown` is for the keys that produce no text and would otherwise be spent on
    /// the input's own editing, and for a character held with a modifier, which produces no `input`
    /// event because it is a command rather than something to insert.
    fn listen(element: &web_sys::HtmlInputElement, captured: &Captured, wake: &Wake) {
        let composed = {
            let (captured, wake) = (captured.clone(), wake.clone());
            Closure::wrap(Box::new(move |event: web_sys::Event| {
                let Some(input) = event
                    .target()
                    .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
                else {
                    return;
                };
                let value = input.value();
                if value.is_empty() {
                    return;
                }
                // Cleared at once. What was typed is carried as keystrokes from here on, and a
                // value left in the input would be composed against by whatever is typed next.
                input.set_value("");
                captured.push(
                    value
                        .chars()
                        // The rule `photosynthesize` reads a winit key by: a control character is
                        // what a key that means something produces where the platform hands text
                        // as well, and inserting one would put it in the value.
                        .filter(|character| !character.is_control())
                        .map(|character| Input::Keyed(Key::Typed(character)))
                        .collect(),
                );
                wake.rouse();
            }) as Box<dyn FnMut(_)>)
        };
        let pressed = {
            let (captured, wake) = (captured.clone(), wake.clone());
            Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
                // Read from this event rather than from what the engine last heard: the canvas
                // stopped receiving `ModifiersChanged` the moment this input took focus, so the
                // engine's own tracker is exactly as starved as the key events are.
                let modifiers = Modifiers {
                    shift: event.shift_key(),
                    control: event.ctrl_key() || event.meta_key(),
                };
                let Some(key) = named(&event.key(), modifiers) else {
                    return;
                };
                // The input's own editing would otherwise spend the key -- a backspace deleting
                // from a value that is always empty, an arrow moving a caret nobody can see.
                event.prevent_default();
                captured.push(vec![
                    Input::Modifiers(modifiers),
                    Input::Keyed(key),
                    Input::Modifiers(Modifiers::default()),
                ]);
                wake.rouse();
            }) as Box<dyn FnMut(_)>)
        };
        for (name, callback) in [("input", composed.as_ref()), ("keydown", pressed.as_ref())] {
            let _ = element.add_event_listener_with_callback(name, callback.unchecked_ref());
        }
        // The listeners outlive this call by exactly as long as the input does, which is the life
        // of the program: there is one hidden input and nothing ever takes it out of the page.
        composed.forget();
        pressed.forget();
    }

    /// Which engine key a browser's `KeyboardEvent.key` is, where it is one this path has to carry.
    ///
    /// Mirrors `photosynthesize`'s reading of a winit key, and for the same reason: a key that
    /// produced text is left to the `input` event that carries it, so the two paths do not both
    /// answer for the same press.
    fn named(key: &str, modifiers: Modifiers) -> Option<Key> {
        let named = match key {
            "Backspace" => Key::Backspace,
            "Delete" => Key::Delete,
            "ArrowLeft" => Key::Left,
            "ArrowRight" => Key::Right,
            "ArrowUp" => Key::Up,
            "ArrowDown" => Key::Down,
            "Home" => Key::Home,
            "End" => Key::End,
            "Enter" => Key::Enter,
            "Tab" => Key::Tab,
            "Escape" => Key::Escape,
            // A character held with control is a command and produces no `input` event, so this is
            // the only path it has. A character held with nothing is text and already has one.
            _ => {
                let mut characters = key.chars();
                let character = characters.next()?;
                if characters.next().is_some() || !modifiers.control {
                    return None;
                }
                Key::Typed(character)
            }
        };
        Some(named)
    }

    impl Captured {
        fn push(&self, inputs: Vec<Input>) {
            self.0.borrow_mut().extend(inputs);
        }
    }
}
