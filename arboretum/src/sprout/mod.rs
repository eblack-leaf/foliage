//! Parts in lichen's manner that lichen does not have yet.
//!
//! Each follows lichen's rules: the caller says where it stands; one element of it takes input
//! and every other part is intangible; what it wears is said in tones and roles; every verb is
//! safe to say again; and it reports what happened rather than handing the frame's pollen on.
//! Any one of them could move into lichen as it stands.
//!
//! | Sprout | What it is |
//! |---|---|
//! | [`Lean`] | an element drawn a little toward the pointer while it is over it |
//! | [`Glow`] | a soft light that follows wherever it is sent |
//! | [`Shade`] | a soft shadow under a card, which spreads as the card lifts |
//! | [`Typewriter`] | words typed out and taken back |
//! | [`stagger`] | things arriving one after another |
//! | [`Emerge`] | things coming in out of the dark and into focus, as a generated picture's preview does |
//! | [`Print`] | things printed in as a label printer feeds them: rules drawn across, words typed, pictures fed |
//! | [`Marquee`] | a band of words running past for ever |
//! | [`Progress`] | a bar that fills, with ticks |
//! | [`Courier`] | the mock backend: answers delivered late, as a server's are |
//! | [`Signpost`] | the way between the portfolio's pieces |
//!
//! Promoted to lichen, and used from there: `Overlay` and `Drawer`, `Hover`, `Dashes`, `Stamp`,
//! the counting `Badge` (once `Tally`), `Roll` and `Wire`. `Escaped` was too, until foliage came to
//! report `Escape` itself.

mod courier;
mod emerge;
mod glow;
mod lean;
mod marquee;
mod print;
mod progress;
mod shade;
mod signpost;
mod stagger;
mod typewriter;

pub use courier::{Courier, Ticket};
pub use emerge::{Arrival, Emerge};
pub use glow::Glow;
pub use lean::Lean;
pub use marquee::Marquee;
pub use print::{Print, Printed};
pub use progress::Progress;
pub use shade::Shade;
pub use signpost::{App, Signpost};
pub use stagger::{recede, stagger};
pub use typewriter::{Typed, Typewriter};
