use crate::Position;
use crate::foliage::Foliage;
use crate::ginkgo::ScaleFactor;
use crate::ginkgo::viewport::ViewportHandle;
use crate::interaction::{
    Interaction, InteractionMethod, InteractionPhase, KeyboardAdapter, MouseAdapter, TouchAdapter,
};
use tracing::trace;
use winit::application::ApplicationHandler;
use winit::event::{MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

impl ApplicationHandler for Foliage {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        trace!("resuming");
        if !self.ginkgo.acquired() {
            #[cfg(not(target_family = "wasm"))]
            {
                self.willow.connect(event_loop);
                pollster::block_on(self.ginkgo.acquire_context(&self.willow));
                self.finish_boot();
            }
            #[cfg(target_family = "wasm")]
            {
                self.willow.connect(event_loop);
                let handle = self.willow.clone();
                let sender = self.sender.take().expect("sender");
                wasm_bindgen_futures::spawn_local(async move {
                    let mut ginkgo = crate::ginkgo::Ginkgo::default();
                    ginkgo.acquire_context(&handle).await;
                    sender.send(ginkgo).ok();
                });
            }
        } else {
            self.ginkgo.recreate_surface(&self.willow);
            self.ginkgo.configure_view(&self.willow);
            self.ginkgo.size_viewport(&self.willow);
            self.suspended = false;
            // if the redraw `about_to_wait` requested before suspending never got to paint
            // (`RedrawRequested` explicitly skips drawing while `self.suspended`), this would
            // otherwise stay stuck true forever -- gating out every tick permanently, since
            // it only ever clears on a successful paint.
            self.tick_pending = false;
        }
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        #[cfg(target_family = "wasm")]
        if !self.booted {
            self.queue.push(event);
            return;
        }
        self.process_event(event, event_loop);
    }
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        #[cfg(target_family = "wasm")]
        if !self.booted && self.receiver.is_some() {
            if let Some(m) = self.receiver.as_mut().unwrap().try_recv().ok() {
                if let Some(g) = m {
                    self.ginkgo = g;
                    self.finish_boot();
                    let queue = self.queue.drain(..).collect::<Vec<WindowEvent>>();
                    for event in queue {
                        self.process_event(event, _event_loop);
                    }
                }
            }
        }
        if self.booted {
            // `about_to_wait` isn't 1:1 with real paint frames, especially on web (see
            // `Foliage::tick_pending`'s own doc) -- only simulate a new tick if the last one
            // we requested a redraw for has actually painted. Otherwise a burst of
            // `about_to_wait` calls between two real paints would each re-run
            // `main`/`user`/`diff`, stacking up ECS churn that never individually renders.
            if !self.tick_pending {
                // Heartbeat: is the event loop ticking rapidly (busy-looping somewhere inside
                // main/user/diff, or in redraw) or is `about_to_wait` itself not being called
                // for seconds at a stretch (blocked further upstream, e.g. in winit/OS event
                // dispatch before we even get control back)? A large `since_last` here narrows
                // a multi-second stall to one side of that question.
                self.main.run(&mut self.world);
                self.frame();
                self.diff.run(&mut self.world);
                self.willow.window().request_redraw();
                self.ash.drawn = false;
                self.tick_pending = true;
            }
            self.ran_at_least_once = true;
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        if self.ginkgo.acquired() {
            self.ginkgo.suspend();
            self.suspended = true;
        }
    }
}
impl Foliage {
    fn process_event(&mut self, event: WindowEvent, event_loop: &ActiveEventLoop) {
        match event {
            WindowEvent::ActivationTokenDone { .. } => {}
            WindowEvent::Resized(_) => {
                self.world
                    .get_resource_mut::<ViewportHandle>()
                    .unwrap()
                    .resize(
                        self.willow
                            .actual_area()
                            .to_logical(self.ginkgo.configuration().scale_factor.value()),
                    );
                self.ginkgo.configure_view(&self.willow);
                self.ginkgo.size_viewport(&self.willow);
                self.willow.window().request_redraw();
            }
            WindowEvent::Moved(_) => {}
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Destroyed => {}
            WindowEvent::DroppedFile(_) => {}
            WindowEvent::HoveredFile(_) => {}
            WindowEvent::HoveredFileCancelled => {}
            WindowEvent::Focused(_) => {}
            WindowEvent::KeyboardInput {
                device_id: _device_id,
                event,
                ..
            } => {
                let (logical, physical) = {
                    let mut adapter = self
                        .world
                        .get_resource_mut::<KeyboardAdapter>()
                        .expect("keys");
                    tracing::trace!(
                        logical_key = ?event.logical_key,
                        physical_key = ?event.physical_key,
                        state = ?event.state,
                        repeat = event.repeat,
                        current_mods = ?adapter.mods,
                        "photosynthesis: KeyboardInput received"
                    );
                    let physical = adapter.parse_physical(event.physical_key, event.state);
                    let logical = adapter.parse(event.logical_key, event.state, event.repeat);
                    (logical, physical)
                };
                if let Some(event) = logical {
                    self.world.trigger(event);
                }
                if let Some(event) = physical {
                    self.world.trigger(event);
                }
            }
            WindowEvent::ModifiersChanged(new_mods) => {
                let converted: crate::Modifiers = new_mods.state().into();
                tracing::trace!(
                    winit_state = ?new_mods.state(),
                    converted = ?converted,
                    "photosynthesis: ModifiersChanged received"
                );
                self.world
                    .get_resource_mut::<KeyboardAdapter>()
                    .expect("keyboard-adapter")
                    .mods = converted;
            }
            WindowEvent::Ime(ime) => {
                // Composition UI (preedit rendering) is deliberately out of scope; committed text
                // reuses the logical-key path so focus routing to the text input comes for free.
                if let winit::event::Ime::Commit(text) = ime {
                    self.world.trigger(crate::InputSequence::new(
                        crate::Key::Character(text),
                        crate::Modifiers::default(),
                    ));
                }
            }
            WindowEvent::CursorMoved {
                device_id: _device_id,
                position,
            } => {
                let scale_factor = *self.world.get_resource::<ScaleFactor>().expect("scale");
                let viewport_position = self
                    .world
                    .get_resource::<ViewportHandle>()
                    .expect("vh")
                    .section()
                    .position;
                if let Some(event) = self
                    .world
                    .get_resource_mut::<MouseAdapter>()
                    .expect("mouse-adapter")
                    .set_cursor(position, viewport_position, scale_factor)
                {
                    self.world.write_message(event);
                }
            }
            WindowEvent::CursorEntered { .. } => {}
            WindowEvent::CursorLeft { .. } => {}
            WindowEvent::MouseWheel {
                device_id: _,
                delta,
                phase: _phase,
            } => {
                let px = match delta {
                    MouseScrollDelta::LineDelta(x, y) => Position::logical((
                        x * Self::SCROLL_SENSITIVITY,
                        y * Self::SCROLL_SENSITIVITY * Self::VIEW_SCROLLING,
                    )),
                    MouseScrollDelta::PixelDelta(px) => Position::physical((px.x, px.y))
                        .to_logical(
                            self.world
                                .get_resource::<ScaleFactor>()
                                .expect("scale-factor")
                                .value(),
                        ),
                };
                let cursor = self
                    .world
                    .get_resource::<MouseAdapter>()
                    .expect("mouse-adapter")
                    .cursor;
                let vh = self
                    .world
                    .get_resource_mut::<ViewportHandle>()
                    .expect("vh")
                    .section()
                    .position;
                let event = Interaction::new(
                    InteractionPhase::Start,
                    vh + cursor,
                    InteractionMethod::ScrollWheel,
                );
                let end_event = Interaction::new(
                    InteractionPhase::End,
                    vh + cursor + px,
                    InteractionMethod::ScrollWheel,
                );
                self.world.write_message(event);
                self.world.write_message(end_event);
            }
            WindowEvent::MouseInput {
                device_id: _device_id,
                state,
                button,
            } => {
                if let Some(event) = self
                    .world
                    .get_resource_mut::<MouseAdapter>()
                    .expect("mouse-adapter")
                    .parse(button, state)
                {
                    self.world.write_message(event);
                }
            }
            WindowEvent::PinchGesture { .. } => {}
            WindowEvent::PanGesture { .. } => {}
            WindowEvent::DoubleTapGesture { .. } => {}
            WindowEvent::RotationGesture { .. } => {}
            WindowEvent::TouchpadPressure { .. } => {}
            WindowEvent::AxisMotion { .. } => {}
            WindowEvent::Touch(t) => {
                let scale_factor = *self
                    .world
                    .get_resource::<ScaleFactor>()
                    .expect("scale-factor");
                let viewport_position = self
                    .world
                    .get_resource::<ViewportHandle>()
                    .expect("vh")
                    .section()
                    .position;
                if let Some(event) = self
                    .world
                    .get_resource_mut::<TouchAdapter>()
                    .expect("touch-adapter")
                    .parse(t, viewport_position, scale_factor)
                {
                    self.world.write_message(event);
                }
            }
            WindowEvent::ScaleFactorChanged {
                scale_factor: _scale_factor,
                ..
            } => {
                self.world
                    .get_resource_mut::<ViewportHandle>()
                    .unwrap()
                    .resize(
                        self.willow
                            .actual_area()
                            .to_logical(self.ginkgo.configuration().scale_factor.value()),
                    );
                self.ginkgo.configure_view(&self.willow);
                self.ginkgo.size_viewport(&self.willow);
            }
            WindowEvent::ThemeChanged(_) => {}
            WindowEvent::Occluded(_) => {}
            WindowEvent::RedrawRequested => {
                if !self.ash.drawn && self.ran_at_least_once && !self.suspended {
                    if let Some(vc) = self
                        .world
                        .get_resource_mut::<ViewportHandle>()
                        .unwrap()
                        .user_translations()
                    {
                        let pos = vc.to_physical(self.ginkgo.configuration().scale_factor.value());
                        self.ginkgo.position_viewport(pos);
                    }
                    self.ash.prepare(&mut self.world, &self.ginkgo);
                    let clear = *self.world.resource::<crate::ClearColor>();
                    self.ash.render(&self.ginkgo, clear);
                    self.ash.drawn = true;
                    self.tick_pending = false;
                    // self.ran_at_least_once = false;
                }
            }
        }
    }
}
