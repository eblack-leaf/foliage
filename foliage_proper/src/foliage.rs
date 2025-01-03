use crate::ash::differential::{cached_differential, RenderQueue, RenderRemoveQueue};
use crate::ash::Ash;
use crate::asset::{Asset, AssetKey, AssetLoader};
use crate::ginkgo::viewport::ViewportHandle;
use crate::ginkgo::Ginkgo;
use crate::remove::Remove;
use crate::time::Time;
use crate::willow::Willow;
use crate::{
    AndroidConnection, Area, Attachment, Disable, EcsExtension, Grid, Interaction, Physical,
    SystemSet, Text,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::{event_update_system, Event, EventRegistry, Events};
use bevy_ecs::observer::TriggerTargets;
use bevy_ecs::prelude::{apply_deferred, IntoSystemConfigs, IntoSystemSetConfigs, Schedule, World};
use bevy_ecs::system::IntoObserverSystem;
use futures_channel::oneshot;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;
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
        foliage.define(Disable::interactions);
        Grid::attach(&mut foliage);
        Interaction::attach(&mut foliage);
        Ash::attach(&mut foliage);
        Text::attach(&mut foliage);
        Asset::attach(&mut foliage);
        Time::attach(&mut foliage);
        Remove::attach(&mut foliage);
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
                self.receiver.replace(recv);
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
        let (fetch, sender) = crate::asset::AssetFetch::new(key);
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
