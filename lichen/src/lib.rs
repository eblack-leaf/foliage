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
//! A [`Specimen`] is all of that put to work: a traced shape with tips on it, each cutting the
//! shape to open a section, as a [`Sketch`] states it.
//!
//! # Presses, and what they wear
//!
//! A [`Chip`] -- a mark in a cell, a bar, a name, or the cell alone -- is the one shape a press is
//! offered in, and a
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
//! [`Badge`] -- something wants a person -- stays until the app clears it, and where how many
//! matters it [counts](Badge::counting). A [`Stamp`] is a state pressed onto a page -- PAID,
//! DRAFT -- that lands like one.
//!
//! # Over the page
//!
//! An [`Overlay`] comes up over a page where it stands, and a [`Drawer`] slides in over it from
//! an edge; both take the page out of reach, hold focus, and close on `Escape`. foliage sends
//! `Escape` to nothing -- it only takes focus away -- so [`Escaped`] is how anything that closes on
//! it reads it, told apart from a stray press and from focus swept off what went out of reach.
//! [`Hover`] says which of a set of elements the pointer rests on, which foliage does not.
//!
//! # Lines and figures
//!
//! [`Dashes`] draws a section's dashed line round any box, and moves it from box to box. A
//! [`Wire`] connects two points -- straight, elbowed or curved, solid, dashed or dotted, with an
//! arrowhead or without -- and re-aims every frame for the cost of a few writes; its routing is
//! in [`wire`]. A [`Roll`] shows a figure on an odometer, each glyph sliding to the next.
//!
//! Everything lichen measures in letters assumes foliage's one monospaced face.

mod badge;
mod chip;
mod cluster;
mod confirm;
mod cut;
mod dashes;
mod escaped;
mod field;
mod hover;
mod measure;
mod mosaic;
mod overlay;
mod ping;
mod ramp;
mod roll;
mod say;
mod scatter;
mod silhouette;
mod specimen;
mod stamp;
mod switch;
mod tone;
mod trail;
pub mod wire;
pub mod words;

pub use badge::Badge;
pub use chip::{Chip, bar, caption};
pub use cluster::Cluster;
pub use confirm::{Confirm, Step};
pub use cut::{Cut, Section};
pub use dashes::Dashes;
pub use escaped::Escaped;
pub use field::{Field, Holds, Origin, Pick, drawn, dress, mark};
pub use hover::{Hover, topmost};
pub use measure::{Density, Measure, density, measure};
pub use mosaic::Mosaic;
pub use overlay::{Closed, Drawer, Edge, Overlay};
pub use ping::Ping;
pub use ramp::{Ramp, Rgb, fill, rgb, shifted};
pub use roll::Roll;
pub use say::{Say, Voice};
pub use scatter::Scatter;
pub use silhouette::{Region, Silhouette};
pub use specimen::{BLIP, Cleared, Sketch, Specimen, Tip};
pub use stamp::Stamp;
pub use switch::Switch;
pub use tone::{
    BAR, CHANGED, HINT, INERT, LIT, Press, REST, Reach, Tone, WELL, corner, engage, gate, timing,
};
pub use trail::Trail;
pub use wire::{Look, Route, Stroke, Wire};
