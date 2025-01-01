mod ash;
mod asset;
mod attachment;
mod color;
mod coordinate;
mod disable;
mod enable;
mod ginkgo;
mod grid;
mod interaction;
mod leaf;
mod opacity;
mod ops;
mod photosynthesis;
mod platform;
mod remove;
mod text;
mod texture;
mod time;
mod tree;
mod virtual_keyboard;
mod visibility;
mod web_ext;
mod willow;

pub(crate) use self::ash::differential::Differential;
use self::ash::differential::{cached_differential, RenderQueue, RenderRemoveQueue};
pub use crate::ash::clip::ClipContext;
pub(crate) use crate::ash::clip::ClipSection;
use crate::ash::Ash;
use crate::asset::{Asset, AssetKey, AssetLoader};
pub use crate::coordinate::{
    area::{Area, CReprArea},
    position::{CReprPosition, Position},
    section::{CReprSection, Section},
    CoordinateContext, CoordinateUnit, Coordinates, Logical, Numerical, Physical,
};
use crate::ginkgo::viewport::ViewportHandle;
use crate::ginkgo::Ginkgo;
use crate::interaction::Interaction;
use crate::remove::Remove;
use crate::time::Time;
use crate::willow::Willow;
pub use attachment::Attachment;
pub use bevy_ecs;
use bevy_ecs::event::{event_update_system, EventRegistry};
use bevy_ecs::observer::TriggerTargets;
pub use bevy_ecs::prelude::*;
use bevy_ecs::system::IntoObserverSystem;
pub use color::Luminance;
pub use color::{CReprColor, Color};
pub use coordinate::elevation::{Elevation, ResolvedElevation};
use futures_channel::oneshot;
pub use grid::{
    auto, stack, Grid, GridUnit, Layout, Location, LocationAxisDescriptor, LocationAxisType,
};
pub use grid::{GridExt, Justify, Stack, StackDeps};
pub use leaf::{Branch, Leaf, Stem};
pub use opacity::Opacity;
pub use ops::{Update, Write};
#[cfg(target_os = "android")]
pub use platform::AndroidApp;
pub use platform::AndroidConnection;
pub use text::{AutoHeight, FontSize, GlyphColors, HorizontalAlignment, Text, VerticalAlignment};
use tracing_subscriber::filter::Targets;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer as TSL;
pub use tree::{EcsExtension, Tree};
pub use visibility::{InheritedVisibility, ResolvedVisibility, Visibility};
use winit::event::WindowEvent;
use winit::event_loop::{ControlFlow, EventLoop};
pub struct Foliage {
    pub world: World,
    pub(crate) main: Schedule,
    pub user: Schedule,
    pub(crate) diff: Schedule,
    pub base_url: String,
    pub(crate) willow: Willow,
    pub(crate) ginkgo: Ginkgo,
    pub(crate) ash: Ash,
    pub(crate) android_connection: AndroidConnection,
    pub(crate) booted: bool,
    pub(crate) queue: Vec<WindowEvent>,
    pub(crate) sender: Option<oneshot::Sender<Ginkgo>>,
    pub(crate) receiver: Option<oneshot::Receiver<Ginkgo>>,
    pub(crate) user_attachments: Vec<fn(&mut Foliage)>,
}
impl Foliage {
    pub const SCROLL_SENSITIVITY: f32 = 40.0;
    pub const NATURAL_SCROLLING: f32 = -1.0;
    pub const VIEW_SCROLLING: f32 = 1.0;
    pub fn new() -> Foliage {
        let mut foliage = Foliage {
            world: Default::default(),
            main: Default::default(),
            user: Default::default(),
            diff: Default::default(),
            willow: Default::default(),
            ginkgo: Default::default(),
            ash: Default::default(),
            base_url: "".to_string(),
            android_connection: Default::default(),
            booted: false,
            queue: vec![],
            sender: None,
            receiver: None,
            user_attachments: vec![],
        };
        foliage.diff.configure_sets(
            (
                DiffMarkers::Prepare,
                DiffMarkers::Finalize,
                DiffMarkers::Extract,
            )
                .chain(),
        );
        foliage.diff.add_systems((
            apply_deferred
                .after(DiffMarkers::Prepare)
                .before(DiffMarkers::Finalize),
            apply_deferred
                .after(DiffMarkers::Finalize)
                .before(DiffMarkers::Extract),
        ));
        foliage.main.add_systems(event_update_system);
        Interaction::attach(&mut foliage);
        Ash::attach(&mut foliage);
        Text::attach(&mut foliage);
        Asset::attach(&mut foliage);
        Time::attach(&mut foliage);
        Remove::attach(&mut foliage);
        Grid::attach(&mut foliage);
        foliage
    }
    pub fn attach<A: Attachment>(&mut self) {
        self.user_attachments.push(A::attach);
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
                event_loop_function(event_loop, self);
            } else {
                let event_loop_function = EventLoop::run_app;
                event_loop_function(event_loop, &mut self).expect("event-loop-run-app");
            }
        }
    }
    pub fn desktop_size<V: Into<Area<Physical>>>(&mut self, v: V) {
        self.willow.requested_size.replace(v.into());
    }
    pub fn url<S: AsRef<str>>(&mut self, path: S) {
        self.base_url = path.as_ref().to_string();
    }
    pub fn define<E: Event + 'static, B: Bundle, M, D: IntoObserverSystem<E, B, M>>(
        &mut self,
        obs: D,
    ) {
        self.world.add_observer(obs);
    }
    pub fn leaf<B: Bundle>(&mut self, b: B) -> Entity {
        self.world.leaf(b)
    }
    pub fn send_to<E: Event>(
        &mut self,
        e: E,
        targets: impl TriggerTargets + Send + Sync + 'static,
    ) {
        self.world.send_to(e, targets);
    }
    pub fn send<E: Event>(&mut self, e: E) {
        self.world.send(e);
    }
    pub fn queue<E: Event>(&mut self, e: E) {
        self.world.queue(e);
    }
    pub fn enable_queued_event<E: Event + Clone + Send + Sync + 'static>(&mut self) {
        if self.world.get_resource::<Events<E>>().is_none() {
            self.world.insert_resource(Events::<E>::default());
            EventRegistry::register_event::<E>(&mut self.world);
        }
    }
    pub fn write_to<B: Bundle>(&mut self, entity: Entity, b: B) {
        self.world.write_to(entity, b);
    }
    pub fn remove(&mut self, targets: impl TriggerTargets + Send + Sync + 'static) {
        self.world.remove(targets);
    }
    pub fn enable(&mut self, targets: impl TriggerTargets + Send + Sync + 'static) {
        self.world.enable(targets);
    }
    pub fn disable(&mut self, targets: impl TriggerTargets + Send + Sync + 'static) {
        self.world.disable(targets);
    }
    pub(crate) fn remove_queue<R: Clone + Send + Sync + 'static>(&mut self) {
        debug_assert_eq!(
            self.world.get_resource::<RenderRemoveQueue<R>>().is_none(),
            true
        );
        self.world.insert_resource(RenderRemoveQueue::<R>::new());
    }
    pub(crate) fn differential<
        R: Clone + Send + Sync + 'static,
        RT: Clone + Send + Sync + 'static + Component + PartialEq,
    >(
        &mut self,
    ) {
        debug_assert_eq!(
            self.world.get_resource::<RenderQueue<R, RT>>().is_none(),
            true
        );
        self.world.insert_resource(RenderQueue::<R, RT>::new());
        self.diff
            .add_systems(cached_differential::<R, RT>.in_set(DiffMarkers::Extract));
    }
    #[cfg(target_family = "wasm")]
    pub fn load_remote_asset(&mut self, path: &str) -> AssetKey {
        let key = AssetLoader::generate_key();
        let (fetch, sender) = asset::AssetFetch::new(key);
        self.world
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
        self.world
            .get_resource_mut::<AssetLoader>()
            .expect("asset-loader")
            .assets
            .insert(key, Asset::new(bytes));
        key
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
    pub(crate) fn finish_boot(&mut self) {
        self.ginkgo.configure_view(&self.willow);
        self.ginkgo.create_viewport(&self.willow);
        let scale_factor = self.ginkgo.configuration().scale_factor;
        self.world.insert_resource(ViewportHandle::new(
            self.willow.actual_area().to_logical(scale_factor.value()),
        ));
        self.world.insert_resource(scale_factor);
        for a_fn in self.user_attachments.drain(..).collect::<Vec<_>>() {
            a_fn(self);
        }
        self.ash.initialize(&self.ginkgo);
        self.booted = true;
    }
}
#[derive(SystemSet, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy, Debug)]
pub(crate) enum DiffMarkers {
    Prepare,
    Finalize,
    Extract,
}
