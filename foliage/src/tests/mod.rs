//! The headless suite.
//!
//! Runs the whole frame against a grove with no surface: every phase an app can observe, and no
//! drawing. The clock is moved by hand rather than sampled from the platform.

mod elevation;
mod elm;
mod frame;
mod lifecycle;
mod placement;
mod root;
mod rowan;
mod tracing;

use crate::coordinate::{Area, Section};
use crate::fern;
use crate::grove::Grove;
use crate::leaf::Leaf;
use crate::pollen::Pollen;
use crate::root::Rooted;
use crate::vein::{Sap, Vein};

/// A grove with no surface.
fn grove() -> Grove {
    Grove::new(Area::new(400.0, 300.0))
}

/// Where an element ended up, as an app would read it.
fn section(grove: &Grove, leaf: Leaf) -> Section {
    match grove.tap(leaf, Vein::Drawn) {
        Some(Sap::Section(section)) => section,
        other => panic!("expected a section, got {other:?}"),
    }
}

/// One whole frame, with no app to run.
fn tick(grove: &mut Grove) {
    fern::run(grove, None);
}

/// One whole frame, with `app` taking its turn in it.
fn tick_with(grove: &mut Grove, app: &mut dyn Rooted) {
    fern::run(grove, Some(app));
}

/// Moves the clock forward, to be taken up by the next frame.
fn advance(grove: &mut Grove, millis: u64) {
    grove.clock.advance(millis);
}

/// Resizes the surface, taking effect at the next frame.
fn resize(grove: &mut Grove, viewport: Area) {
    grove.pending_resize = Some(viewport);
}

/// Stands in for an app, keeping the [`Pollen`] each frame hands it.
#[derive(Default)]
struct Observer {
    pollen: Vec<Pollen>,
}

impl Rooted for Observer {
    fn frame(&mut self, _grove: &mut Grove, pollen: Pollen) {
        self.pollen.push(pollen);
    }
}

impl Observer {
    /// What the last frame handed it.
    fn last(&self) -> &Pollen {
        self.pollen.last().expect("a frame has run")
    }
}
