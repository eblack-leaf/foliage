//! Opinionated parts on foliage: a look, and the tools that carry it.
//!
//! foliage decides nothing about how a thing looks. lichen decides a great deal, so that an app
//! built from it need not, and two apps built from it look like relatives. What it decides is
//! constant in here -- timings, proportions, how a state is dressed, how a tile is jittered -- and
//! not a knob: an app chooses its colours through the scheme, its shapes through what it traces,
//! and its [`Density`], and nothing else about the look. Where things stand is always the app's.
//!
//! # The mosaic
//!
//! Jittered five-to-seven-sided polygons, coloured along a ramp with a little drift per tile,
//! brought in seed-first and swept when emphasized. A [`Mosaic`] fills a [`Silhouette`] -- any
//! closed outline -- can stand a [`Region`] anywhere on it, and can be cropped down to any part of
//! it and put back; a [`Cut`] takes a part, and a [`Section`] draws the line. A [`Trail`] runs a
//! line of beads between two points. A [`Cluster`] puts a puff of motes in a box the caller
//! already grew. Every one is painted from a [`Ramp`] the caller states -- from a scheme's
//! spectrum, ideally -- and jittered by a [`Scatter`] the caller holds, so two of anything grown
//! from the same scatter come out as two drawings.
//!
//! # Presses, and what they wear
//!
//! A [`Chip`] -- a mark in a cell, a bar, a name -- is the one shape a press is offered in, and a
//! [`Press`] is the state it stands in: at rest, armed, inert, chosen, dangerous. The app says
//! which; the look says what that looks like, and the accent is kept for the one press to make.
//! [`gate`] turns a run of steps into what each step's press wears, which is what a chain is built
//! from. A [`Switch`] is a two-way choice, and a [`Confirm`] is the press that cannot be taken
//! back, made as two.
//!
//! # Fields, and where their words came from
//!
//! A [`Field`] holds words and a [`Pick`] one of a fixed few, both in the chip's shape. What each
//! wears says where its value came from -- see [`Origin`] and [`Holds`] -- so a glance down a form
//! says which of it the person said.
//!
//! # Being noticed
//!
//! Three ways, told apart by how long they last: a [`Ping`] -- something just happened here --
//! goes by itself; a [`Say`] -- the line answering a press -- holds until the next change; a
//! [`Badge`] -- something wants a person -- stays until the app clears it.
//!
//! Everything lichen measures in letters assumes foliage's one monospaced face.

mod badge;
mod chip;
mod cluster;
mod confirm;
mod cut;
mod field;
mod measure;
mod mosaic;
mod ping;
mod ramp;
mod say;
mod scatter;
mod silhouette;
mod switch;
mod tone;
mod trail;
pub mod words;

pub use badge::Badge;
pub use chip::{Chip, bar, caption};
pub use cluster::Cluster;
pub use confirm::{Confirm, Step};
pub use cut::{Cut, Section};
pub use field::{Field, Holds, Origin, Pick, drawn, dress, mark};
pub use measure::{Density, Measure, density, measure};
pub use mosaic::Mosaic;
pub use ping::Ping;
pub use ramp::{Ramp, Rgb, fill, rgb, shifted};
pub use say::{Say, Voice};
pub use scatter::Scatter;
pub use silhouette::{Region, Silhouette, Turn};
pub use switch::Switch;
pub use tone::{
    BAR, CHANGED, HINT, INERT, LIT, Press, REST, Reach, Tone, WELL, corner, engage, gate, timing,
};
pub use trail::Trail;
