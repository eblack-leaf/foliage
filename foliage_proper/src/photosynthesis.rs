use crate::foliage::Foliage;
use crate::ginkgo::viewport::ViewportHandle;
use crate::ginkgo::ScaleFactor;
use crate::interaction::{
    Interaction, InteractionPhase, KeyboardAdapter, MouseAdapter, TouchAdapter,
};
use crate::Position;
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
            self.main.run(&mut self.world);
            self.user.run(&mut self.world);
            self.diff.run(&mut self.world);
            self.willow.window().request_redraw();
            self.ash.drawn = false;
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
                if let Some(event) = self
                    .world
                    .get_resource_mut::<KeyboardAdapter>()
                    .expect("keys")
                    .parse(event.logical_key, event.state)
                {
                    self.world.send_event(event);
                }
            }
            WindowEvent::ModifiersChanged(new_mods) => {
                self.world
                    .get_resource_mut::<KeyboardAdapter>()
                    .expect("keyboard-adapter")
                    .mods = new_mods.state();
            }
            WindowEvent::Ime(_) => {}
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
                    self.world.send_event(event);
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
                let event = Interaction::new(InteractionPhase::Start, vh + cursor, true);
                let end_event = Interaction::new(InteractionPhase::End, vh + cursor + px, true);
                self.world.send_event(event);
                self.world.send_event(end_event);
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
                    self.world.send_event(event);
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
                    self.world.send_event(event);
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
                    // TODO extract
                    self.ash.prepare(&mut self.world, &self.ginkgo);
                    self.ash.render(&self.ginkgo);
                    self.ash.drawn = true;
                    // self.ran_at_least_once = false;
                }
            }
        }
    }
}
