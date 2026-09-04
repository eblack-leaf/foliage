//! What arrives from the platform, before anything has been decided about it.
//!
//! One shape for every device. A mouse, a finger and a stylus produce the same four pointer
//! statements, and the wheel is separate because it is a discrete pulse rather than a gesture with a
//! lifecycle: it moves what is under it and is over.
//!
//! Nothing here reads the tree. Translation from the platform's own events happens at the edge, and
//! the headless suite writes these directly -- which is what makes the two paths the same path, and
//! what keeps the winit translation layer the only part of input the suite cannot reach.

use crate::coordinate::Position;

/// One input event, as the platform reported it.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Input {
    /// The pointer went down. Opens a gesture.
    Pressed(Position),
    /// The pointer moved while down.
    Moved(Position),
    /// The pointer came up.
    Released(Position),
    /// The gesture was taken away rather than completed -- the window lost focus, a touch was
    /// cancelled by the system. Never produces a tap.
    Cancelled,
    /// A wheel notch at a point, as the movement a drag of the same distance would have been.
    Wheeled { at: Position, delta: Position },
    /// A key went down. Ordered against everything else here, because keystrokes are the one thing
    /// in the engine whose order is part of what they mean.
    ///
    /// It names the key alone. What was held with it is [`Modifiers`], which arrives as its own
    /// event in this same stream -- so what a key was pressed with is read from the order the two
    /// arrived in rather than carried alongside.
    Keyed(Key),
    /// What is now held down. Ordered here with everything else for the reason above: a modifier is
    /// only ever a statement about the keys pressed after it.
    Modifiers(Modifiers),
}

/// One keystroke: which key, and what was held with it.
///
/// What [`Pollen::keys`](crate::Pollen::keys) hands an element, and what
/// [`Pollen::root_keys`](crate::Pollen::root_keys) hands an app with nothing focused.
///
/// Assembled at dispatch from the key and whatever [`Modifiers`] the stream had reached by then,
/// which is what makes it a single value the drain can carry.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Keystroke {
    /// Which key it was.
    pub key: Key,
    /// What was held down with it.
    pub modifiers: Modifiers,
}

/// A key, as the platform's own layout resolved it.
///
/// [`Typed`](Key::Typed) is a character to insert and nothing more: which key produced it, whether
/// it took a dead key or two, and what the layout is are all the platform's, answered before this.
/// Everything else here is a key that means something rather than says something.
///
/// Non-exhaustive, because a key set grows: an app matches the keys it acts on and lets the rest
/// fall through.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Key {
    /// A character the layout produced, to be taken as itself.
    Typed(char),
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    Backspace,
    Delete,
    Enter,
    Tab,
    Escape,
}

/// What was held down with a key.
///
/// One flag per modifier that changes what a key means rather than which character it produced --
/// a layout has already answered the second before anything arrives here. `shift` extends a
/// selection where a bare arrow moves a caret; `control` is what makes a key a command rather than
/// a character.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    /// Whether shift was down.
    pub shift: bool,
    /// Whether control was down.
    pub control: bool,
}

/// Everything input has arrived with and everything it has built up: what is waiting for the next
/// frame, the gesture in progress, and what the keyboard is holding.
///
/// One of these for the whole engine. There is one pointer -- multi-touch resolves to a primary one
/// at the platform edge, and a second finger is not a second gesture here -- and one keyboard, so
/// what a key was pressed with and where a hand is are the same object's business.
#[derive(Default)]
pub(crate) struct Incoming {
    /// Events waiting for the next frame's dispatch, in arrival order.
    pub(crate) pending: Vec<Input>,
    /// The gesture that is open, from the press that opened it until it ends.
    pub(crate) gesture: Option<super::Gesture>,
    /// What is held down as of the last [`Input::Modifiers`] dispatch reached.
    ///
    /// Engine state rather than the platform's, so the headless suite reaches it by writing the
    /// same event a window does and there is no second path for a test to miss.
    pub(crate) modifiers: Modifiers,
}

impl Incoming {
    /// Takes an event from the platform. Nothing is resolved here: dispatch is a frame phase, and
    /// input that arrived between frames is handled in the frame it arrived in.
    pub(crate) fn take(&mut self, input: Input) {
        self.pending.push(input);
    }

    /// Whether the open gesture could still become a hold, which is a frame the loop owes (F9).
    ///
    /// A press that is not moving is the one gesture that changes with nothing arriving to change
    /// it, so it is the one that has to be waited on rather than woken for.
    pub(crate) fn awaiting_hold(&self) -> bool {
        self.gesture
            .as_ref()
            .is_some_and(super::Gesture::awaiting_hold)
    }
}
