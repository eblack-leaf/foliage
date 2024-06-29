pub use bevy_ecs;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::prelude::Resource;
use bevy_ecs::system::Command;
use futures_channel::oneshot;
pub use wgpu;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

use reference::{ViewConfigBuilder, ViewReference};
use signal::{ActionHandle, TriggeredAction};
use willow::Willow;

use crate::ash::{Ash, Render};
use crate::asset::{Asset, AssetKey, AssetLoader};
use crate::coordinate::area::Area;
use crate::coordinate::{Coordinates, DeviceContext};
use crate::elm::Elm;
use crate::ginkgo::viewport::ViewportHandle;
use crate::ginkgo::{Ginkgo, ScaleFactor};
use crate::grid::{Grid, GridPlacement};
use crate::icon::Icon;
use crate::image::Image;
use crate::interaction::{ClickInteractionListener, KeyboardAdapter, MouseAdapter, TouchAdapter};
use crate::panel::Panel;
use crate::signal::Signal;
use crate::view::{ViewComponents, ViewHandle};

pub mod ash;
pub mod asset;
pub mod color;
pub mod coordinate;
mod differential;
pub mod elm;
pub mod ginkgo;
pub mod grid;
pub mod icon;
pub mod image;
pub mod instances;
pub mod interaction;
pub mod panel;
mod reference;
pub mod signal;
pub mod view;
pub mod willow;

pub struct Foliage {
    willow: Willow,
    ash: Ash,
    ginkgo: Ginkgo,
    elm: Elm,
    android_connection: AndroidConnection,
    leaf_fns: Vec<Box<fn(&mut Elm)>>,
    leaves_fns: Vec<Box<fn(&mut Foliage)>>,
    booted: bool,
    #[allow(unused)]
    queue: Vec<WindowEvent>,
    #[allow(unused)]
    sender: Option<oneshot::Sender<Ginkgo>>,
    #[allow(unused)]
    recv: Option<oneshot::Receiver<Ginkgo>>,
    base_url: String,
}

impl Foliage {
    pub fn new() -> Self {
        let mut this = Self {
            willow: Willow::default(),
            ash: Ash::default(),
            ginkgo: Ginkgo::default(),
            elm: Elm::default(),
            android_connection: AndroidConnection::default(),
            leaf_fns: vec![],
            leaves_fns: vec![],
            booted: false,
            queue: vec![],
            sender: None,
            recv: None,
            base_url: "".to_string(),
        };
        this.attach_leaves::<CoreLeaves>();
        this.elm.ecs.world.insert_resource(AssetLoader::default());
        this
    }
    pub fn set_base_url<S: AsRef<str>>(&mut self, s: S) {
        self.base_url = s.as_ref().to_string();
    }
    pub fn set_window_size<A: Into<Area<DeviceContext>>>(&mut self, a: A) {
        self.willow.requested_size.replace(a.into());
    }
    pub fn set_window_title<S: AsRef<str>>(&mut self, s: S) {
        self.willow.title.replace(s.as_ref().to_string());
    }
    pub fn set_android_connection(&mut self, ac: AndroidConnection) {
        self.android_connection = ac;
    }
    pub fn attach_leaf<L: Leaf>(&mut self) {
        self.leaf_fns.push(Box::new(|e| {
            L::attach(e);
        }));
    }
    pub fn attach_leaves<L: Leaves>(&mut self) {
        self.leaves_fns.push(Box::new(|f| {
            L::attach(f);
        }));
    }
    pub fn add_renderer<R: Render>(&mut self) {
        self.ash.add_renderer::<R>();
    }
    pub fn spawn<B: Bundle + 'static + Send + Sync>(&mut self, b: B) {
        self.elm.ecs.world.spawn(b);
    }
    pub fn create_view(&mut self, grid_placement: GridPlacement, grid: Grid) -> ViewConfigBuilder {
        let handle = self
            .elm
            .ecs
            .world
            .spawn(ViewComponents::new(grid_placement, grid))
            .id();
        ViewConfigBuilder {
            root: handle,
            reference: Some(&mut self.elm),
            targets: Default::default(),
            stages: Default::default(),
        }
    }
    pub fn view(&mut self, vh: ViewHandle) -> ViewReference {
        ViewReference {
            root: vh.0,
            reference: &mut self.elm,
        }
    }
    pub fn create_action<A: Command + Clone + Send + Sync + 'static>(
        &mut self,
        a: A,
    ) -> ActionHandle {
        self.elm.checked_add_action_fns::<A>();
        let handle = self
            .elm
            .ecs
            .world
            .spawn((TriggeredAction(a), Signal::default()))
            .id();
        ActionHandle(handle)
    }
    pub fn insert_resource<R: Resource>(&mut self, r: R) {
        self.elm.ecs.world.insert_resource(r);
    }
    pub fn run(mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Wait);
        let proxy = event_loop.create_proxy();
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                console_error_panic_hook::set_once();
                let (sender, recv) = oneshot::channel();
                self.sender.replace(sender);
                self.recv.replace(recv);
                use winit::platform::web::EventLoopExtWebSys;
                let event_loop_function = EventLoop::spawn_app;
                (event_loop_function)(event_loop, self);
            } else {
                let event_loop_function = EventLoop::run_app;
                (event_loop_function)(event_loop, &mut self).expect("event-loop-run-app");
            }
        }
    }
    pub fn enable_retrieve<B: Bundle + Send + Sync + 'static>(&mut self) {
        self.elm.enable_retrieve::<B>();
    }
    #[cfg(target_family = "wasm")]
    pub fn load_remote_asset(&mut self, path: &str) -> AssetKey {
        let key = AssetLoader::generate_key();
        let (fetch, sender) = crate::asset::AssetFetch::new(key);
        self.elm
            .ecs
            .world
            .get_resource_mut::<AssetLoader>()
            .expect("asset-loader")
            .queue_fetch(fetch);
        let path = format!(
            "{}/{}/{}",
            web_sys::window().expect("window").origin(),
            self.base_url,
            path
        );
        wasm_bindgen_futures::spawn_local(async move {
            let asset = reqwest::Client::new()
                .get(path)
                .header("Accept", "application/octet-stream")
                .send()
                .await
                .expect("asset-request")
                .bytes()
                .await
                .expect("asset-bytes")
                .to_vec();
            sender.send(Asset::new(asset)).ok();
        });
        key
    }
    #[cfg(not(target_family = "wasm"))]
    pub fn load_native_asset(&mut self, bytes: Vec<u8>) -> AssetKey {
        let key = AssetLoader::generate_key();
        self.elm
            .ecs
            .world
            .get_resource_mut::<AssetLoader>()
            .expect("asset-loader")
            .assets
            .insert(key.clone(), Asset::new(bytes));
        key
    }
    fn leaves_attach(&mut self) {
        for leaves_fn in self
            .leaves_fns
            .drain(..)
            .collect::<Vec<Box<fn(&mut Foliage)>>>()
        {
            (leaves_fn)(self);
        }
    }
    fn finish_boot(&mut self) {
        self.ginkgo.configure_view(&self.willow);
        self.ginkgo.create_viewport(&self.willow);
        self.elm.configure(
            self.willow.actual_area().to_numerical(),
            self.ginkgo.configuration().scale_factor,
        );
        self.leaves_attach();
        self.elm.initialize(self.leaf_fns.drain(..).collect());
        self.ash.initialize(&self.ginkgo);
        self.booted = true;
    }
    fn process_event(&mut self, event: WindowEvent, event_loop: &ActiveEventLoop) {
        match event {
            WindowEvent::ActivationTokenDone { .. } => {}
            WindowEvent::Resized(_) => {
                self.elm.adjust_viewport_handle(&self.willow);
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
                    .elm
                    .ecs
                    .world
                    .get_resource_mut::<KeyboardAdapter>()
                    .expect("keys")
                    .parse(event.logical_key, event.state)
                {
                    self.elm.ecs.world.send_event(event);
                }
            }
            WindowEvent::ModifiersChanged(new_mods) => {
                self.elm
                    .ecs
                    .world
                    .get_resource_mut::<KeyboardAdapter>()
                    .expect("keyboard-adapter")
                    .mods = new_mods.state();
            }
            WindowEvent::Ime(_) => {}
            WindowEvent::CursorMoved {
                device_id: _device_id,
                position,
            } => {
                let scale_factor = self
                    .elm
                    .ecs
                    .world
                    .get_resource::<ScaleFactor>()
                    .expect("scale")
                    .clone();
                let viewport_position = self
                    .elm
                    .ecs
                    .world
                    .get_resource::<ViewportHandle>()
                    .expect("vh")
                    .section()
                    .position
                    .as_logical();
                if let Some(event) = self
                    .elm
                    .ecs
                    .world
                    .get_resource_mut::<MouseAdapter>()
                    .expect("mouse-adapter")
                    .set_cursor(position, viewport_position, scale_factor)
                {
                    self.elm.ecs.world.send_event(event);
                }
            }
            WindowEvent::CursorEntered { .. } => {}
            WindowEvent::CursorLeft { .. } => {}
            WindowEvent::MouseWheel { .. } => {}
            WindowEvent::MouseInput {
                device_id: _device_id,
                state,
                button,
            } => {
                if let Some(event) = self
                    .elm
                    .ecs
                    .world
                    .get_resource_mut::<MouseAdapter>()
                    .expect("mouse-adapter")
                    .parse(button, state)
                {
                    self.elm.ecs.world.send_event(event);
                }
            }
            WindowEvent::PinchGesture { .. } => {}
            WindowEvent::PanGesture { .. } => {}
            WindowEvent::DoubleTapGesture { .. } => {}
            WindowEvent::RotationGesture { .. } => {}
            WindowEvent::TouchpadPressure { .. } => {}
            WindowEvent::AxisMotion { .. } => {}
            WindowEvent::Touch(t) => {
                let scale_factor = self
                    .elm
                    .ecs
                    .world
                    .get_resource::<ScaleFactor>()
                    .expect("scale-factor")
                    .clone();
                let viewport_position = self
                    .elm
                    .ecs
                    .world
                    .get_resource::<ViewportHandle>()
                    .expect("vh")
                    .section()
                    .position
                    .as_logical();
                if let Some(event) = self
                    .elm
                    .ecs
                    .world
                    .get_resource_mut::<TouchAdapter>()
                    .expect("touch-adapter")
                    .parse(t, viewport_position, scale_factor)
                {
                    self.elm.ecs.world.send_event(event);
                }
            }
            WindowEvent::ScaleFactorChanged {
                scale_factor: _scale_factor,
                ..
            } => {
                self.elm.adjust_viewport_handle(&self.willow);
                self.ginkgo.configure_view(&self.willow);
                self.ginkgo.size_viewport(&self.willow);
            }
            WindowEvent::ThemeChanged(_) => {}
            WindowEvent::Occluded(_) => {}
            WindowEvent::RedrawRequested => {
                if !self.ash.drawn {
                    if let Some(vc) = self.elm.viewport_handle_changes() {
                        self.ginkgo.position_viewport(vc);
                    }
                    self.ash.render(&self.ginkgo, &mut self.elm);
                    self.ash.drawn = true;
                }
            }
        }
    }
}

impl ApplicationHandler for Foliage {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[cfg(not(target_family = "wasm"))]
        if !self.ginkgo.acquired() {
            self.willow.connect(event_loop);
            pollster::block_on(self.ginkgo.acquire_context(&self.willow));
            self.finish_boot();
        } else {
            #[cfg(target_os = "android")]
            {
                self.ginkgo.recreate_surface(&self.willow);
                self.ginkgo.configure_view(&self.willow);
                self.ginkgo.resize_viewport(&self.willow);
            }
        }
        #[cfg(target_family = "wasm")]
        if !self.ginkgo.acquired() {
            self.willow.connect(event_loop);
            let handle = self.willow.clone();
            let sender = self.sender.take().expect("sender");
            wasm_bindgen_futures::spawn_local(async move {
                let mut ginkgo = Ginkgo::default();
                ginkgo.acquire_context(&handle).await;
                sender.send(ginkgo).ok();
            });
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
        if !self.booted && self.recv.is_some() {
            if let Some(m) = self.recv.as_mut().unwrap().try_recv().ok() {
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
        if self.ash.drawn && self.elm.initialized() && self.booted {
            self.elm.process();
            self.willow.window().request_redraw();
            self.ash.drawn = false;
        }
    }
}

#[cfg(not(target_os = "android"))]
#[derive(Default, Copy, Clone)]
pub struct AndroidConnection();

#[cfg(target_os = "android")]
pub struct AndroidConnection(pub AndroidApp);

#[cfg(target_os = "android")]
pub type AndroidApp = winit::platform::android::activity::AndroidApp;

pub trait Leaf {
    fn attach(elm: &mut Elm);
}
pub trait Leaves {
    fn attach(foliage: &mut Foliage);
}
pub struct CoreLeaves;
impl Leaves for CoreLeaves {
    fn attach(foliage: &mut Foliage) {
        foliage.attach_leaf::<Panel>();
        foliage.add_renderer::<Panel>();
        foliage.attach_leaf::<Coordinates>();
        foliage.attach_leaf::<Icon>();
        foliage.add_renderer::<Icon>();
        foliage.attach_leaf::<Image>();
        foliage.add_renderer::<Image>();
        foliage.attach_leaf::<ClickInteractionListener>();
    }
}
