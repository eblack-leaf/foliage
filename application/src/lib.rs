//! The site foliage is proven against.
//!
//! `cargo check -p application` is a gate on the engine rather than on the site. An API that
//! cannot build a page is an incomplete API, and this is where that is found out: everything here
//! is written against `foliage`'s public surface and reaches nothing else.

mod shell;
mod site;

use foliage::{Area, Claim, Foliage};

/// Shared by every platform's entry point.
///
/// Only how a [`Foliage`] is constructed differs per platform; everything after that is the same
/// few statements.
pub fn run(mut foliage: Foliage) {
    // Where a trace goes is the app's decision -- foliage depends on `tracing` and installs no
    // subscriber of its own, so nothing is reported anywhere until something here says where.
    trace();
    foliage.title("foliage");
    foliage.app_id("foliage");
    foliage.desktop_size(Area::new(390.0, 844.0));
    // How far a gesture travels before it stops being a tap, per axis. The page scrolls down and
    // has one control that takes a drag across, so a claim across is held off until it is clearly
    // meant -- otherwise every attempt to scroll would steal into the slider it passed over.
    foliage.tune(Claim {
        horizontal: 20.0,
        vertical: 8.0,
    });
    foliage.root::<site::Site>();
    foliage.photosynthesize();
}

/// Sends the engine's own trace to stderr, at whatever `RUST_LOG` asks for.
///
/// `info` by default: boot, the adapter and the surface. `RUST_LOG=foliage=debug` adds every
/// structural change and every dropped op, which is the only account there is of an op that named
/// something no longer live.
fn trace() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
}
