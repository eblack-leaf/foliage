//! Cross-platform, ECS-backed UI.
//!
//! Everything crossing the seam between an app and the engine is plain data: ops in, emissions
//! out, and read-only taps taken at the app's own callsite. No engine type is handed out and
//! nothing an app holds borrows from the world.
//!
//! - [`Foliage`] is the engine. [`Grove`] is the surface it hands a frame, and [`Root`] is the app
//!   it hands it to.
//! - [`Leaf`] names an element and [`Seed`] describes one before it exists. [`Grow`] carries every
//!   write, [`Pollen`] is what comes back out.
//! - [`Vein`] asks a read for one property; [`Sap`] is what the tap draws out.
//! - [`Place`] states where an element sits, and [`Location`] carries the grammar it is said in.
//! - [`Panel`] is a filled rectangle; [`Palette`] names what fills it and [`Corners`] how it is
//!   rounded.

mod ash;
mod clock;
mod color;
mod coordinate;
mod elevation;
mod elm;
mod fern;
mod foliage;
mod ginkgo;
mod grove;
mod layout;
mod leaf;
mod op;
mod palette;
mod panel;
mod photosynthesize;
mod place;
mod placement;
mod pollen;
mod queue;
mod root;
mod rounding;
mod rowan;
mod seed;
mod stem;
mod tree;
mod vein;
mod verbs;
mod willow;

#[cfg(test)]
mod tests;

pub use color::Color;
pub use coordinate::{Area, Position, Section};
pub use elevation::Elevation;
pub use foliage::Foliage;
pub use grove::Grove;
pub use layout::{Layout, Short};
pub use leaf::{Leaf, Presence};
pub use palette::{Palette, Scheme};
pub use panel::Panel;
pub use place::Place;
pub use placement::basis::{Anchor, Trunk, anchor, trunk};
pub use placement::grid::{Columns, Divide, Grid, Rows};
pub use placement::location::Location;
pub use placement::role::{
    Bottom, CenterX, CenterY, Horizontal, Left, Right, Top, Vertical, bottom, center_x, center_y,
    left, right, top,
};
pub use placement::source::{
    HorizontalCoordinate, Length, Source, VerticalCoordinate, VerticalLength, content,
};
pub use pollen::Pollen;
pub use root::Root;
pub use rounding::{Corner, Corners, Rounding, Side};
pub use seed::Seed;
pub use stem::Stem;
pub use vein::{Sap, Vein};
pub use verbs::Grow;
