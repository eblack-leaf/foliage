//! Opinionated `Polygon`-based motifs distilled from `application`, the `foliage` repo's
//! own demo app -- deliberately *not* part of `foliage_proper::composite`, which stays
//! unopinionated (a `Slider` has no house style; this crate's [`scrollbar`] has exactly
//! one). Depend on this only if you want this app's specific look; `foliage` alone is the
//! framework, this is one taste applied to it.
//!
//! Three pieces so far, each usable standalone or together:
//! - [`shadow_of`] -- an offset, muted `Polygon` copy anchored to a target.
//! - [`morph_in`] -- fade + step a `Polygon` through triangle -> ... -> its real shape.
//! - [`scrollbar`] -- a vertical, heptagon-knobbed scrollbar over any `View`.
//!
//! Rough first pass: plain functions, not `Sprout`-based composites with their own
//! reactive config -- extracted from one real app with each technique only used once or
//! twice so far, so guessing a config-struct-and-reactivity shape now would be guessing
//! ahead of real pressure on the API. `ScrollbarStyle` is the one exception, since
//! `scrollbar` already had several tunable constants worth naming as a group.

mod morph;
mod scrollbar;
mod shadow;

pub use morph::morph_in;
pub use scrollbar::{ScrollbarStyle, scrollbar};
pub use shadow::{SHADOW_OFFSET_PX, SHADOW_Y_OFFSET_PX, shadow_of};
