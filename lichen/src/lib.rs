//! The mosaic aesthetic: jittered five-to-seven-sided polygons, coloured along a ramp with a little
//! drift per tile, brought in seed-first and swept when emphasized.
//!
//! Three arrangements of it. A [`Mosaic`] fills a [`Silhouette`] -- any closed outline -- and can
//! stand a square region on any of the outline's vertices. A [`Trail`] runs a line of beads between
//! two points. A [`Cluster`] puts a puff of motes in a box the caller already grew.
//!
//! Every one is painted from a [`Ramp`] the caller states, and jittered by a [`Scatter`] the caller
//! holds, so two of anything grown from the same scatter come out as two drawings. Timings, the
//! polygon vocabulary, dash and gap, and how a region sits on a vertex are constants in here: that
//! is the look, and it is not a knob.

mod cluster;
mod mosaic;
mod ramp;
mod scatter;
mod silhouette;
mod trail;

pub use cluster::Cluster;
pub use mosaic::Mosaic;
pub use ramp::{Ramp, Rgb, fill, shifted};
pub use scatter::Scatter;
pub use silhouette::{Silhouette, Turn};
pub use trail::Trail;
