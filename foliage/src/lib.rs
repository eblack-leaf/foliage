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

mod clock;
mod coordinate;
mod fern;
mod foliage;
mod grove;
mod leaf;
mod op;
mod pollen;
mod queue;
mod root;
mod seed;
mod stem;
mod tree;
mod vein;
mod verbs;

#[cfg(test)]
mod tests;

pub use coordinate::Area;
pub use foliage::Foliage;
pub use grove::Grove;
pub use leaf::{Leaf, Presence};
pub use pollen::Pollen;
pub use root::Root;
pub use seed::Seed;
pub use stem::Stem;
pub use vein::{Sap, Vein};
pub use verbs::Grow;
