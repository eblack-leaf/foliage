//! The loop, and the law that governs it.
//!
//! [`Fern::run`](crate::fern::run) is steps 1 through 8 and is the whole of what the headless suite
//! reaches. This is the other side: the platform's events on the way in, step 9 on the way out, and
//! F9's question -- whether a frame is owed at all.

use core::time::Duration;

use winit::application::ApplicationHandler;
use winit::event::{
    ElementState, KeyEvent, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent,
};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key as Named, NamedKey};
use winit::window::WindowId;

use crate::ash::Ash;
use crate::coordinate::Position;
use crate::fern;
use crate::foliage::Foliage;
use crate::ginkgo::Ginkgo;
use crate::interaction::input::{Input, Key, Modifiers};
use crate::palette::Palette;

/// The most time one frame is allowed to be told has passed.
///
/// Real gaps between frames are unbounded and have nothing to do with the app: the machine slept,
/// a tab was in the background, a window was held mid-drag, a breakpoint was sitting in the loop.
/// Reported honestly, the frame that wakes the loop hands every running tween the whole gap at
/// once, which is a glitch in the machine becoming a glitch in the interface -- what is on screen
/// tears its way to wherever wall time says it should be.
///
/// So the gap is not skipped, it is **waited on**: everything advances by the capped delta, which
/// defers the whole engine by the same amount rather than moving any part of it. A tween resumes
/// where it was and takes longer in wall time; it does not arrive at its end value having never
/// been drawn. Playing late is the better failure, and it is only available because F6 gives the
/// frame one clock -- there is no second time source for a subsystem to fall out of step against.
///
/// This is the platform's ceiling and not the [`Clock`](crate::clock::Clock)'s. The headless suite
/// advances by hand and has to be exact: a tween told to advance five seconds advances five.
const HITCH: Duration = Duration::from_millis(100);

/// What one key press becomes, in the engine's own vocabulary.
///
/// The platform has already answered everything about layout, dead keys and composition, so a key
/// that produced text is taken as the text it produced and nothing here asks which key that was.
/// A key that produced none is one of the few that mean something instead, and anything that is
/// neither is not a keystroke this engine has a use for.
///
/// Several characters at once is a real case -- a composed sequence commits as a word -- so this
/// answers with as many keys as the press produced rather than with one.
///
/// What was held with them is not read here. Modifiers travel as their own event in the same
/// stream, so what a key was pressed with is the order the two arrived in.
fn keys(event: &KeyEvent) -> Vec<Key> {
    let named = |key: Key| vec![key];
    match &event.logical_key {
        Named::Named(NamedKey::Backspace) => named(Key::Backspace),
        Named::Named(NamedKey::Delete) => named(Key::Delete),
        Named::Named(NamedKey::ArrowLeft) => named(Key::Left),
        Named::Named(NamedKey::ArrowRight) => named(Key::Right),
        Named::Named(NamedKey::ArrowUp) => named(Key::Up),
        Named::Named(NamedKey::ArrowDown) => named(Key::Down),
        Named::Named(NamedKey::Home) => named(Key::Home),
        Named::Named(NamedKey::End) => named(Key::End),
        Named::Named(NamedKey::Enter) => named(Key::Enter),
        Named::Named(NamedKey::Tab) => named(Key::Tab),
        Named::Named(NamedKey::Escape) => named(Key::Escape),
        _ => event
            .text
            .iter()
            .flat_map(|text| text.chars())
            // A control character is what a key that means something produces where the platform
            // gives it text as well, and inserting one would put it in the value.
            .filter(|character| !character.is_control())
            .map(Key::Typed)
            .collect(),
    }
}

/// How far one wheel notch scrolls, where the platform reports notches rather than pixels.
///
/// A notch is a count and the engine works in logical pixels, so something has to say how far one
/// of them is. Roughly three lines of text, which is the convention every platform reporting them
/// this way expects.
const NOTCH: f32 = 48.0;

impl Foliage {
    /// Runs the app. Does not return.
    ///
    /// The platform owns the loop, which is why this consumes the engine: everything after it is
    /// the [`Root`](crate::Root)'s, called once per frame with the [`Grove`](crate::Grove) it is
    /// lent.
    pub fn photosynthesize(self) {
        let event_loop = EventLoop::new().expect("event loop");
        // The engine idles when nothing is owed, so the loop sleeps until the platform has
        // something to say. F9's remaining clause -- an app that asked for another frame -- is
        // answered by requesting a paint at the end of the frame that asked.
        event_loop.set_control_flow(ControlFlow::Wait);
        #[cfg(target_family = "wasm")]
        {
            use winit::platform::web::EventLoopExtWebSys;
            event_loop.spawn_app(self);
        }
        #[cfg(not(target_family = "wasm"))]
        {
            let mut foliage = self;
            event_loop.run_app(&mut foliage).expect("event loop");
        }
    }

    /// F9. Whether a frame is owed.
    ///
    /// The loop's own question, answered from state the engine already holds. It is not a method on
    /// [`Grove`](crate::Grove) or on anything else an app can reach, because nothing outside the
    /// loop has cause to ask -- [`again`](crate::Grove::again) is the app's half and the only half
    /// an app can see.
    fn owed(&self) -> bool {
        // The tree has not been grown yet: `take_root` runs inside the first frame.
        self.grove.frames == 0
            || self.grove.again
            || self.grove.pending_resize.is_some()
            || !self.grove.queue.is_empty()
            || !self.grove.incoming.pending.is_empty()
            || !self.grove.aspen.idle()
            // A press that is not moving becomes a hold on its own, with nothing arriving to say
            // so. Idling under a finger is how that duration would pass unremarked, so the frames
            // that would notice it are owed until the gesture is past being able to become one.
            || self.grove.incoming.awaiting_hold()
            // A coasting region is pending work in its own right: it is not a tween, and there is
            // nothing on the app's side asking for the frames it needs. So frames keep running
            // until it settles, and stop afterwards.
            || !self.grove.coasting.idle()
            // Steps 4 through 7 emit into the drift and step 3 of the *next* frame is where an app
            // is handed it, so a report that has been made and not yet delivered owes the frame
            // that delivers it. Without this the loop could idle holding one.
            || self.grove.drift.pending()
    }

    /// One platform input event, in the form dispatch takes.
    ///
    /// The whole of the translation layer, and the whole of what the headless suite cannot reach:
    /// past this call there is one path, and a scripted press and a real one are the same event.
    fn input(&mut self, input: Input) {
        self.grove.incoming.take(input);
    }

    /// Where the platform's physical coordinates land in logical ones.
    fn at(&self, x: f64, y: f64) -> Position {
        let scale = self.willow.scale();
        Position::new(x as f32 / scale, y as f32 / scale)
    }

    /// Moves the clock by what elapsed, up to [`HITCH`].
    fn advance(&mut self) {
        let now = web_time::Instant::now();
        let delta = match self.sampled.replace(now) {
            Some(last) => now.saturating_duration_since(last).min(HITCH),
            None => Duration::ZERO,
        };
        self.grove.clock.advance(delta);
    }

    /// The window's size and density as the engine reads them, taken after the platform has
    /// changed either.
    ///
    /// A changed **density** drops what the backend is held to. Extraction compares logical values
    /// and the backend holds instances derived from them in device pixels -- a cut glyph, a snapped
    /// stroke, a mark's screen-space range -- so a display that changed density leaves those correct
    /// against a density that is gone, while the logical values they came from are untouched and
    /// would compare equal forever. Moving a window between two displays is exactly that case: the
    /// logical size can be identical on both.
    fn reconfigure(&mut self) {
        let area = self.willow.area();
        let scale = self.willow.scale();
        if let Some(ginkgo) = &mut self.ginkgo {
            if ginkgo.scale() != scale {
                self.grove.elm.recut();
            }
            ginkgo.resize(area, scale);
        }
        self.grove.pending_resize = Some(area);
        self.willow.repaint();
    }

    /// The device is up: build the renderers and ask for the first frame.
    fn boot(&mut self, ginkgo: Ginkgo) {
        self.ash = Some(Ash::new(&ginkgo));
        self.ginkgo = Some(ginkgo);
        self.willow.repaint();
    }

    /// Runs the frame if one is owed, then paints.
    ///
    /// The two are separate questions. A frame runs when the engine has something to do; a paint
    /// happens when the platform asks for one, and it can ask for reasons of its own -- a window
    /// exposed, a compositor redrawing. Painting again from what the renderers hold is always
    /// possible and always correct, so the second question never forces the first.
    ///
    /// Absorbing the batch sits inside the first branch, immediately after the extraction that
    /// produced it. That is what makes the cache in [`Elm`](crate::elm) safe to keep: every batch
    /// is applied exactly where it is made, and a paint that fails afterwards costs a picture.
    fn paint(&mut self) {
        #[cfg(target_family = "wasm")]
        self.receive();
        if self.ginkgo.is_none() {
            return;
        }
        if self.owed() {
            self.advance();
            fern::run(&mut self.grove, self.root.as_deref_mut());
            let ginkgo = self.ginkgo.as_ref().expect("device");
            let ash = self.ash.as_mut().expect("backend");
            ash.absorb(
                &self.grove.elm,
                &self.grove.fonts,
                &self.grove.fields,
                &self.grove.plates,
                ginkgo,
            );
        }
        // Nothing of the engine's own sits behind the tree, so the ground is the app's: whatever
        // the scheme currently resolves the ordinary fill to.
        let clear = self.grove.scheme().color(Palette::Surface);
        let ginkgo = self.ginkgo.as_ref().expect("device");
        let ash = self.ash.as_ref().expect("backend");
        ash.draw(ginkgo, clear);
        if self.grove.again {
            self.willow.repaint();
        }
    }

    /// Takes the device once the browser has finished handing one over.
    ///
    /// The web is the one platform with no thread to wait on: an adapter and a device arrive
    /// through promises, so acquisition is spawned at resume and the loop is woken by a paint
    /// request when it lands.
    #[cfg(target_family = "wasm")]
    fn receive(&mut self) {
        if self.ginkgo.is_some() {
            return;
        }
        let Some(ginkgo) = self
            .acquiring
            .as_ref()
            .and_then(|slot| slot.try_recv().ok())
        else {
            return;
        };
        self.acquiring = None;
        self.boot(ginkgo);
    }
}

impl ApplicationHandler for Foliage {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.willow.connected() {
            return;
        }
        self.willow.connect(event_loop);
        let area = self.willow.area();
        let scale = self.willow.scale();
        // The surface the platform actually gave is the first thing the tree is resolved against,
        // whatever was asked for at boot.
        self.grove.pending_resize = Some(area);
        #[cfg(not(target_family = "wasm"))]
        {
            let ginkgo = pollster::block_on(Ginkgo::acquire(self.willow.window(), area, scale));
            self.boot(ginkgo);
        }
        #[cfg(target_family = "wasm")]
        {
            let (sender, receiver) = std::sync::mpsc::channel();
            self.acquiring = Some(receiver);
            let window = self.willow.window();
            wasm_bindgen_futures::spawn_local(async move {
                let ginkgo = Ginkgo::acquire(window.clone(), area, scale).await;
                sender.send(ginkgo).ok();
                window.request_redraw();
            });
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => self.reconfigure(),
            WindowEvent::RedrawRequested => self.paint(),
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = self.at(position.x, position.y);
                // With no hover to report, a move with nothing held says nothing and owes no
                // frame. Where the pointer is still matters, because a press names no position.
                if self.held {
                    self.input(Input::Moved(self.cursor));
                }
            }
            WindowEvent::MouseInput { state, button, .. } if button == MouseButton::Left => {
                self.held = state == ElementState::Pressed;
                self.input(match self.held {
                    true => Input::Pressed(self.cursor),
                    false => Input::Released(self.cursor),
                });
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let delta = match delta {
                    // A notch is a distance, and the distance one notch stands for is the
                    // platform's convention rather than the engine's.
                    MouseScrollDelta::LineDelta(x, y) => Position::new(x * NOTCH, y * NOTCH),
                    MouseScrollDelta::PixelDelta(delta) => self.at(delta.x, delta.y),
                };
                self.input(Input::Wheeled {
                    at: self.cursor,
                    delta,
                });
            }
            WindowEvent::Touch(touch) => {
                let at = self.at(touch.location.x, touch.location.y);
                self.cursor = at;
                // One pointer. A second finger is not a second gesture, and until there is a
                // gesture that needs one it is not one at all.
                self.input(match touch.phase {
                    TouchPhase::Started => Input::Pressed(at),
                    TouchPhase::Moved => Input::Moved(at),
                    TouchPhase::Ended => Input::Released(at),
                    TouchPhase::Cancelled => Input::Cancelled,
                });
            }
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                for key in keys(&event) {
                    self.input(Input::Keyed(key));
                }
            }
            // Into the same queue as everything else, so what a key was held with is settled by
            // arrival order rather than by a flag read from the side.
            WindowEvent::ModifiersChanged(modifiers) => {
                self.input(Input::Modifiers(Modifiers {
                    shift: modifiers.state().shift_key(),
                    control: modifiers.state().control_key(),
                }));
            }
            // The gesture was taken away rather than finished, so it never becomes a tap.
            WindowEvent::CursorLeft { .. } | WindowEvent::Focused(false) => {
                if self.held {
                    self.held = false;
                    self.input(Input::Cancelled);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.owed() {
            self.willow.repaint();
        }
    }
}
