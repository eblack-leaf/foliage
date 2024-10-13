pub use bevy_ecs;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::prelude::{Entity, Event, Resource};
use futures_channel::oneshot;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;
pub use wgpu;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

use willow::Willow;

use crate::anim::{Animate, EnabledAnimations};
use crate::ash::{Ash, ClippingContext, Render};
use crate::asset::{Asset, AssetKey, AssetLoader};
use crate::coordinate::area::Area;
use crate::coordinate::{Coordinates, DeviceContext};
use crate::elm::{Ecs, Elm};
use crate::ginkgo::viewport::ViewportHandle;
use crate::ginkgo::{Ginkgo, ScaleFactor};
use crate::icon::{Icon, IconId, IconRequest};
use crate::image::Image;
use crate::interaction::{ClickInteractionListener, KeyboardAdapter, MouseAdapter, TouchAdapter};
use crate::leaf::{
    render_link_on_remove, resolve_elevation, resolve_visibility, trigger_interactions_enable,
    triggered_remove,
};
use crate::opacity::triggered_opacity;
use crate::panel::Panel;
use crate::shape::line::Line;
use crate::shape::Shape;
use crate::style::Style;
use crate::text::Text;
use crate::time::Time;
use crate::web_ext::HrefLink;

pub mod anim;
pub mod ash;
pub mod asset;
pub mod clipboard;
pub mod color;
pub mod coordinate;
pub mod differential;
pub mod elm;
pub mod ginkgo;
pub mod grid;
pub mod icon;
pub mod image;
pub mod instances;
pub mod interaction;
pub mod layout;
pub mod leaf;
pub mod opacity;
pub mod panel;
pub mod shape;
pub mod style;
pub mod text;
pub mod texture;
pub mod time;
pub mod tree;
pub mod twig;
mod virtual_keyboard;
pub mod web_ext;
pub mod willow;

pub struct Foliage {
    willow: Willow,
    ash: Ash,
    ginkgo: Ginkgo,
    elm: Elm,
    android_connection: AndroidConnection,
    root_fns: Vec<fn(&mut Elm)>,
    roots_fns: Vec<fn(&mut Foliage)>,
    booted: bool,
    #[allow(unused)]
    queue: Vec<WindowEvent>,
    #[allow(unused)]
    sender: Option<oneshot::Sender<Ginkgo>>,
    #[allow(unused)]
    recv: Option<oneshot::Receiver<Ginkgo>>,
    base_url: String,
}
impl Default for Foliage {
    fn default() -> Self {
        Self::new()
    }
}

impl Foliage {
    pub fn new() -> Self {
        let mut this = Self {
            willow: Willow::default(),
            ash: Ash::default(),
            ginkgo: Ginkgo::default(),
            elm: Elm::default(),
            android_connection: AndroidConnection::default(),
            root_fns: vec![],
            roots_fns: vec![],
            booted: false,
            queue: vec![],
            sender: None,
            recv: None,
            base_url: "".to_string(),
        };
        this.define_roots::<Trunk>();
        this.elm.ecs.insert_resource(AssetLoader::default());
        this.elm.ecs.observe(trigger_interactions_enable);
        this.elm.ecs.observe(triggered_opacity);
        this.elm.ecs.observe(triggered_remove);
        // this.elm.ecs.observe(triggered_resolve_grid_locations);
        // this.elm.ecs.observe(stem_remove);
        this.elm.ecs.observe(render_link_on_remove);
        this.elm.ecs.observe(resolve_visibility);
        this.elm.ecs.observe(resolve_elevation);
        // this.elm.ecs.observe(update_stem_trigger);
        this
    }
    pub fn enable_animation<A: Animate>(&mut self) {
        self.elm.enable_animation::<A>();
    }
    pub fn load_icon<ID: Into<IconId>, B: AsRef<[u8]>>(&mut self, id: ID, bytes: B) {
        self.spawn(IconRequest::new(id, bytes.as_ref().to_vec()));
    }
    pub fn set_base_url<S: AsRef<str>>(&mut self, s: S) {
        self.base_url = s.as_ref().to_string();
    }
    pub fn set_desktop_size<A: Into<Area<DeviceContext>>>(&mut self, a: A) {
        self.willow.requested_size.replace(a.into());
    }
    pub fn set_window_title<S: AsRef<str>>(&mut self, s: S) {
        self.willow.title.replace(s.as_ref().to_string());
    }
    pub fn set_android_connection(&mut self, ac: AndroidConnection) {
        self.android_connection = ac;
    }
    pub fn attach_root<L: Root>(&mut self) {
        self.root_fns.push(|e| {
            L::attach(e);
        });
    }
    pub fn define_roots<L: Roots>(&mut self) {
        self.roots_fns.push(|f| {
            L::attach(f);
        });
    }
    pub fn add_renderer<R: Render>(&mut self) {
        self.ash.add_renderer::<R>();
        self.elm.enable_differential::<R, ClippingContext>();
    }
    pub fn spawn<B: Bundle + 'static + Send + Sync>(&mut self, b: B) -> Entity {
        self.elm.ecs.spawn(b).id()
    }
    pub fn enable_event<E: Event + Clone + Send + Sync + 'static>(&mut self) {
        self.elm.enable_event::<E>();
    }
    pub fn send_event<E: Event>(&mut self, e: E) {
        self.elm.ecs.send_event(e);
    }
    pub fn insert_resource<R: Resource>(&mut self, r: R) {
        self.elm.ecs.insert_resource(r);
    }
    pub fn ecs(&mut self) -> &mut Ecs {
        &mut self.elm.ecs
    }
    pub fn photosynthesize(mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Wait);
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
    pub fn enable_tracing(&self, targets: Targets) {
        #[cfg(not(target_family = "wasm"))]
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .compact()
                    .with_filter(targets),
            )
            .init();
        #[cfg(target_family = "wasm")]
        {
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_writer(
                            tracing_subscriber_wasm::MakeConsoleWriter::default()
                                .map_trace_level_to(tracing::Level::TRACE),
                        )
                        .without_time()
                        .with_filter(targets),
                )
                .init();
        }
    }
    pub fn enable_retrieve<B: Bundle + Send + Sync + 'static>(&mut self) {
        self.elm.enable_retrieve::<B>();
    }
    #[cfg(target_family = "wasm")]
    pub fn load_remote_asset(&mut self, path: &str) -> AssetKey {
        let key = AssetLoader::generate_key();
        let (fetch, sender) = asset::AssetFetch::new(key);
        self.elm
            .ecs
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
            .get_resource_mut::<AssetLoader>()
            .expect("asset-loader")
            .assets
            .insert(key, Asset::new(bytes));
        key
    }
    fn grow_roots(&mut self) {
        for root_fn in self.roots_fns.drain(..).collect::<Vec<fn(&mut Foliage)>>() {
            (root_fn)(self);
        }
    }
    fn finish_boot(&mut self) {
        self.ginkgo.configure_view(&self.willow);
        self.ginkgo.create_viewport(&self.willow);
        self.elm.configure(
            self.willow.actual_area(),
            self.ginkgo.configuration().scale_factor,
        );
        self.grow_roots();
        self.elm.initialize(self.root_fns.drain(..).collect());
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
                    .get_resource_mut::<KeyboardAdapter>()
                    .expect("keys")
                    .parse(event.logical_key, event.state)
                {
                    self.elm.ecs.send_event(event);
                }
            }
            WindowEvent::ModifiersChanged(new_mods) => {
                self.elm
                    .ecs
                    .get_resource_mut::<KeyboardAdapter>()
                    .expect("keyboard-adapter")
                    .mods = new_mods.state();
            }
            WindowEvent::Ime(_) => {}
            WindowEvent::CursorMoved {
                device_id: _device_id,
                position,
            } => {
                let scale_factor = *self.elm.ecs.get_resource::<ScaleFactor>().expect("scale");
                let viewport_position = self
                    .elm
                    .ecs
                    .get_resource::<ViewportHandle>()
                    .expect("vh")
                    .section()
                    .position;
                if let Some(event) = self
                    .elm
                    .ecs
                    .get_resource_mut::<MouseAdapter>()
                    .expect("mouse-adapter")
                    .set_cursor(position, viewport_position, scale_factor)
                {
                    self.elm.ecs.send_event(event);
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
                    .get_resource_mut::<MouseAdapter>()
                    .expect("mouse-adapter")
                    .parse(button, state)
                {
                    self.elm.ecs.send_event(event);
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
                    .elm
                    .ecs
                    .get_resource::<ScaleFactor>()
                    .expect("scale-factor");
                let viewport_position = self
                    .elm
                    .ecs
                    .get_resource::<ViewportHandle>()
                    .expect("vh")
                    .section()
                    .position;
                if let Some(event) = self
                    .elm
                    .ecs
                    .get_resource_mut::<TouchAdapter>()
                    .expect("touch-adapter")
                    .parse(t, viewport_position, scale_factor)
                {
                    self.elm.ecs.send_event(event);
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
                        let pos = vc.to_device(self.ginkgo.configuration().scale_factor.value());
                        self.ginkgo.position_viewport(pos);
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

pub trait Root {
    fn attach(elm: &mut Elm);
}
pub trait Roots {
    fn attach(foliage: &mut Foliage);
}
pub struct Trunk;
impl Roots for Trunk {
    fn attach(foliage: &mut Foliage) {
        foliage.attach_root::<Panel>();
        foliage.add_renderer::<Panel>();
        foliage.attach_root::<Coordinates>();
        foliage.attach_root::<Icon>();
        foliage.add_renderer::<Icon>();
        foliage.attach_root::<Image>();
        foliage.add_renderer::<Image>();
        foliage.attach_root::<ClickInteractionListener>();
        foliage.attach_root::<Text>();
        foliage.add_renderer::<Text>();
        foliage.attach_root::<Style>();
        foliage.attach_root::<Time>();
        foliage.attach_root::<EnabledAnimations>();
        foliage.attach_root::<Shape>();
        foliage.add_renderer::<Shape>();
        foliage.attach_root::<Line>();
        foliage.enable_event::<HrefLink>();
    }
}
