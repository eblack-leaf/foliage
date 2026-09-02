//! The loop, and the law that governs it.
//!
//! [`Fern::run`](crate::fern::run) is steps 1 through 8 and is the whole of what the headless suite
//! reaches. This is the other side: the platform's events on the way in, step 9 on the way out, and
//! F9's question -- whether a frame is owed at all.

use core::time::Duration;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

use crate::ash::Ash;
use crate::fern;
use crate::foliage::Foliage;
use crate::ginkgo::Ginkgo;
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
    fn reconfigure(&mut self) {
        let area = self.willow.area();
        let scale = self.willow.scale();
        if let Some(ginkgo) = &mut self.ginkgo {
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
            ash.absorb(&self.grove.elm, ginkgo);
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
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.owed() {
            self.willow.repaint();
        }
    }
}
