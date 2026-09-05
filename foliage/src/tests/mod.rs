//! The headless suite.
//!
//! Runs the whole frame against a grove with no surface: every phase an app can observe, and no
//! drawing. The clock is moved by hand rather than sampled from the platform.

mod aspen;
mod assets;
mod elevation;
mod elm;
mod focus;
mod frame;
mod interaction;
mod keys;
mod lifecycle;
mod placement;
mod platform;
mod renderers;
mod root;
mod rowan;
mod sprig;
mod text;
mod text_input;
mod tracing;
mod views;

use core::time::Duration;

use crate::coordinate::{Area, Position, Section};
use crate::fern;
use crate::grove::Grove;
use crate::interaction::input::{Input, Key, Keystroke, Modifiers};
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

/// Moves the clock past the duration a press has to be down to be held, at whatever the engine is
/// tuned to.
fn past_the_hold(grove: &mut Grove) {
    let after = grove.hold.after.as_millis() as u64;
    advance(grove, after);
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
    grove.incoming.take(Input::Pressed(Position::new(x, y)));
}

/// Moves the pointer while it is down.
fn drag(grove: &mut Grove, x: f32, y: f32) {
    grove.incoming.take(Input::Moved(Position::new(x, y)));
}

fn release(grove: &mut Grove, x: f32, y: f32) {
    grove.incoming.take(Input::Released(Position::new(x, y)));
}

/// Takes the gesture away rather than finishing it.
fn cancel(grove: &mut Grove) {
    grove.incoming.take(Input::Cancelled);
}

/// A wheel notch at a point, stated as the movement a drag of the same distance would have been.
fn wheel(grove: &mut Grove, at: (f32, f32), delta: (f32, f32)) {
    grove.incoming.take(Input::Wheeled {
        at: Position::new(at.0, at.1),
        delta: Position::new(delta.0, delta.1),
    });
}

/// A key pressed on its own.
fn stroke(key: Key) -> Keystroke {
    Keystroke {
        key,
        modifiers: Modifiers::default(),
    }
}

/// The same key with shift held.
fn with_shift(key: Key) -> Keystroke {
    Keystroke {
        key,
        modifiers: Modifiers {
            shift: true,
            ..Modifiers::default()
        },
    }
}

/// The same with control held.
fn with_control(key: Key) -> Keystroke {
    Keystroke {
        key,
        modifiers: Modifiers {
            control: true,
            ..Modifiers::default()
        },
    }
}

/// One key press, entering the pipeline where a translated one would.
fn key(grove: &mut Grove, key: Key) {
    grove.incoming.take(Input::Keyed(key));
}

/// One key press with modifiers held, which is two events: what is held, and then the key.
///
/// The same pair a window sends, so a test and a platform reach the engine the same way.
fn held(grove: &mut Grove, modifiers: Modifiers, key: Key) {
    grove.incoming.take(Input::Modifiers(modifiers));
    grove.incoming.take(Input::Keyed(key));
    grove.incoming.take(Input::Modifiers(Modifiers::default()));
}

/// The same, with shift held.
fn shifted(grove: &mut Grove, key: Key) {
    held(
        grove,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
        key,
    );
}

/// The same, with control held.
fn controlled(grove: &mut Grove, key: Key) {
    held(
        grove,
        Modifiers {
            control: true,
            ..Modifiers::default()
        },
        key,
    );
}

/// A run of characters, one keystroke each, in the order they were written.
fn typing(grove: &mut Grove, value: &str) {
    for character in value.chars() {
        key(grove, Key::Typed(character));
    }
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
