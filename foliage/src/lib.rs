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
//! - [`Place`] states where an element sits and how it behaves, and [`Location`] carries the
//!   grammar the first of those is said in.
//! - [`Panel`] is a filled rectangle; [`Fill`] is what fills it -- a [`Palette`] role or a [`Color`]
//!   stated outright -- and [`Corners`] is how it is rounded.
//! - A gesture goes to the top of the box stack: [`interactive`](Place::interactive) says who
//!   receives one, [`pass_through`](Place::pass_through) says who is never the top, and
//!   [`Drag`] is what one reports while it is held.
//! - [`Text`] is a run of monospaced glyphs, [`Font`] names one an app registered and [`FontSize`]
//!   says how large it is; [`content()`] is a box measured rather than declared.
//! - [`Motion`] is what can be animated and [`Timing`] how long it takes; [`Ease`] is the shape it
//!   moves in, [`Tween`] names a value the engine only reports, and [`Sequence`] names a group of
//!   them arriving.
//! - An element scrolls because it said so: [`scrolls`](Place::scrolls) declares which axes and
//!   whether each [`Scroll`]s outward or contains, [`pinned`](Place::pinned) keeps a child out of
//!   the movement, [`ScrollTo`] is how a region is moved by name, and [`Momentum`] is how a release
//!   coasts.

mod ash;
mod aspen;
mod clock;
mod color;
mod coordinate;
mod elevation;
mod elm;
mod fern;
mod foliage;
mod ginkgo;
mod grove;
mod icon;
mod image;
mod interaction;
mod layout;
mod leaf;
mod lifecycle;
mod line;
mod op;
mod palette;
mod panel;
mod photosynthesize;
mod place;
mod placement;
mod pollen;
mod polygon;
mod queue;
mod root;
mod rounding;
mod rowan;
mod seed;
mod stem;
mod text;
mod tree;
mod vein;
mod verbs;
mod view;
mod willow;

#[cfg(test)]
mod tests;

pub use aspen::{Ease, Motion, Sequence, Timing, Tween};
pub use color::Color;
pub use coordinate::{Area, Axes, Position, Section};
pub use elevation::Elevation;
pub use foliage::Foliage;
pub use grove::Grove;
pub use icon::{Field, Icon};
pub use image::{Fit, Image, Plate};
pub use interaction::{Claim, Drag};
pub use layout::{Layout, Short};
pub use leaf::{Leaf, Presence};
pub use line::{Cap, HAIRLINE, Line};
pub use palette::{Fill, Palette, Scheme};
pub use panel::Panel;
pub use place::{Boxed, Place};
pub use placement::basis::{Anchor, Trunk, anchor, trunk};
pub use placement::grid::{Columns, Divide, Grid, Rows};
pub use placement::location::Location;
pub use placement::point::Point;
pub use polygon::{Polygon, Shape};
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
pub use text::{Font, FontSize, Text};
pub use vein::{Sap, Vein};
pub use verbs::Grow;
pub use view::{Escape, Momentum, Scroll, ScrollTo};
