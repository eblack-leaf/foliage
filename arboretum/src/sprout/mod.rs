//! Parts in lichen's manner that lichen does not have yet.
//!
//! Each follows lichen's rules: the caller says where it stands; one element of it takes input
//! and every other part is intangible; what it wears is said in tones and roles; every verb is
//! safe to say again; and it reports what happened rather than handing the frame's pollen on.
//! Any one of them could move into lichen as it stands.
//!
//! | Sprout | What it is |
//! |---|---|
//! | [`Hover`] | which of a set of elements the pointer rests on, and when that changes |
//! | [`Lean`] | an element drawn a little toward the pointer while it is over it |
//! | [`Glow`] | a soft light that follows wherever it is sent |
//! | [`Shade`] | a soft shadow under a card, which spreads as the card lifts |
//! | [`Roll`] | odometer figures |
//! | [`Typewriter`] | words typed out and taken back |
//! | [`stagger`] | things arriving one after another |
//! | [`Dashes`] | lichen's dashed line, round any box |
//! | [`Emerge`] | things coming in out of the dark and into focus, as a generated picture's preview does |
//! | [`Print`] | things printed in as a label printer feeds them: rules drawn across, words typed, pictures fed |
//! | [`Wire`] | a connector between two points, routed, stroked and pulsed |
//! | [`Drawer`] | a sheet in from an edge, over a scrim, closed by Escape |
//! | [`Escaped`] | Escape, told apart from a stray press and from focus swept off what went out of reach |
//! | [`Marquee`] | a band of words running past for ever |
//! | [`Stamp`] | a rubber stamp |
//! | [`Progress`] | a bar that fills, with ticks |
//! | [`Tally`] | a count on a press's corner |
//! | [`Courier`] | the mock backend: answers delivered late, as a server's are |
//! | [`Signpost`] | the way between the portfolio's pieces |

mod courier;
mod dashes;
mod drawer;
mod emerge;
mod escaped;
mod glow;
mod hover;
mod lean;
mod marquee;
mod print;
mod progress;
mod roll;
mod shade;
mod signpost;
mod stagger;
mod stamp;
mod tally;
mod typewriter;
pub mod wire;

pub use courier::{Courier, Ticket};
pub use dashes::Dashes;
pub use drawer::{Closed, Drawer, Edge};
pub use emerge::{Arrival, Emerge};
pub use escaped::Escaped;
pub use glow::Glow;
pub use hover::{Hover, topmost};
pub use lean::Lean;
pub use marquee::Marquee;
pub use print::{Print, Printed};
pub use progress::Progress;
pub use roll::Roll;
pub use shade::Shade;
pub use signpost::{App, Signpost};
pub use stagger::{recede, stagger};
pub use stamp::Stamp;
pub use tally::Tally;
pub use typewriter::{Typed, Typewriter};
pub use wire::{Look, Route, Stroke, Wire};
