//! The headless suite.
//!
//! Runs the whole frame against a grove with no surface: every phase an app can observe, and no
//! drawing. The clock is moved by hand rather than sampled from the platform.

mod aspen;
mod elevation;
mod elm;
mod focus;
mod frame;
mod interaction;
mod lifecycle;
mod placement;
mod root;
mod rowan;
mod text;
mod tracing;
mod views;

use core::time::Duration;

use crate::coordinate::{Area, Position, Section};
use crate::fern;
use crate::grove::Grove;
use crate::interaction::input::Input;
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

/// How opaque an element is, as an app would read it.
fn opacity(grove: &Grove, leaf: Leaf) -> f32 {
    match grove.tap(leaf, Vein::Opacity) {
        Some(Sap::Opacity(opacity)) => opacity,
        other => panic!("expected an opacity, got {other:?}"),
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
    grove.clock.advance(Duration::from_millis(millis));
}

/// Resizes the surface, taking effect at the next frame.
fn resize(grove: &mut Grove, viewport: Area) {
    grove.pending_resize = Some(viewport);
}

/// Input is written as gestures rather than as synthetic platform events.
///
/// The translation from winit is the one part of input the suite cannot cover, so pressing at a
/// point enters the pipeline exactly where a translated event would -- and everything past that is
/// one path, shared with a real press.
fn press(grove: &mut Grove, x: f32, y: f32) {
    grove.pointer.take(Input::Pressed(Position::new(x, y)));
}

/// Moves the pointer while it is down.
fn drag(grove: &mut Grove, x: f32, y: f32) {
    grove.pointer.take(Input::Moved(Position::new(x, y)));
}

fn release(grove: &mut Grove, x: f32, y: f32) {
    grove.pointer.take(Input::Released(Position::new(x, y)));
}

/// Takes the gesture away rather than finishing it.
fn cancel(grove: &mut Grove) {
    grove.pointer.take(Input::Cancelled);
}

/// A wheel notch at a point, stated as the movement a drag of the same distance would have been.
fn wheel(grove: &mut Grove, at: (f32, f32), delta: (f32, f32)) {
    grove.pointer.take(Input::Wheeled {
        at: Position::new(at.0, at.1),
        delta: Position::new(delta.0, delta.1),
    });
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
