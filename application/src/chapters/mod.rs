//! The visual walkthrough of how `foliage_proper` builds up a composite -- one page per
//! concept, in the order you actually need to learn them: `entity`/`location` first (an
//! entity is nothing without a position), then `grid`/`anchor` (the two ways a child's
//! `Location` resolves), then `animate`/`sequence` (motion), then `interact` (clicks),
//! then `sprout` (the authoring pattern that packages all of the above), and finally
//! `composite`, a capstone worked example. Each is currently just a placeholder shape --
//! the real infographic for each concept lands per-page later.

pub mod anchor;
pub mod animate;
pub mod composite;
pub mod entity;
pub mod grid;
pub mod interact;
pub mod location;
pub mod sequence;
pub mod sprout;
