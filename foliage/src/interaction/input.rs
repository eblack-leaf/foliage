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
}

/// The one pointer: what has arrived since the last frame, and the gesture in progress.
///
/// One of these for the whole engine, because there is one pointer. Multi-touch resolves to a
/// primary pointer at the platform edge; a second finger is not a second gesture here.
#[derive(Default)]
pub(crate) struct Pointer {
    /// Events waiting for the next frame's dispatch, in arrival order.
    pub(crate) pending: Vec<Input>,
    /// The gesture that is open, from the press that opened it until it ends.
    pub(crate) gesture: Option<super::Gesture>,
}

impl Pointer {
    /// Takes an event from the platform. Nothing is resolved here: dispatch is a frame phase, and
    /// input that arrived between frames is handled in the frame it arrived in.
    pub(crate) fn take(&mut self, input: Input) {
        self.pending.push(input);
    }
}
