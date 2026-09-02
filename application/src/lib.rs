//! The site foliage is proven against.
//!
//! `cargo check -p application` is a gate on the engine rather than on the site. An API that
//! cannot build a page is an incomplete API, and this is where that is found out: everything here
//! is written against `foliage`'s public surface and reaches nothing else.
//!
//! The engine has no entry point until `photosynthesize` lands, so [`run`] builds a [`Foliage`],
//! tells it what to grow, and stops. What it describes is complete.

mod shell;
mod site;

use foliage::{Area, Foliage};

/// Shared by every platform's entry point.
///
/// Only how a [`Foliage`] is constructed differs per platform; everything after that is the same
/// three statements.
pub fn run(mut foliage: Foliage) {
    foliage.desktop_size(Area::new(390.0, 844.0));
    foliage.root::<site::Site>();
    // photosynthesize() runs the loop, and lands with the platform layer.
}
