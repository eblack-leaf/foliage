use crate::anim::animate;
use crate::ash::differential::{cached_differential, RenderQueue, RenderRemoveQueue};
use crate::ash::Ash;
use crate::asset::{Asset, AssetKey, AssetLoader};
use crate::ginkgo::viewport::ViewportHandle;
use crate::ginkgo::Ginkgo;
use crate::remove::Remove;
use crate::time::{OnEnd, Time};
use crate::tree::{IntoTargets, TargetedEvent};
use crate::virtual_keyboard::VirtualKeyboardAdapter;
use crate::willow::Willow;
use crate::{
    AndroidConnection, Animate, Animation, Area, Attachment, Color, Disable, EcsExtension,
    Elevation, Enable, Grid, Icon, Image, Interaction, Location, Named, OnClick, Opacity, Panel,
    Physical, Resource, Shape, SystemSet, Text, TextInput, Visibility,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::message::{message_update_system, Message, MessageRegistry, Messages};
use bevy_ecs::observer::{IntoEntityObserver, IntoObserver};
use bevy_ecs::prelude::{ApplyDeferred, IntoScheduleConfigs, Schedule, World};
use futures_channel::oneshot;
use std::marker::PhantomData;
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
    #[allow(unused)]
    pub(crate) android_connection: AndroidConnection,
    pub(crate) booted: bool,
    #[allow(unused)]
    pub(crate) queue: Vec<WindowEvent>,
    #[allow(unused)]
    pub(crate) sender: Option<oneshot::Sender<Ginkgo>>,
    #[allow(unused)]
    pub(crate) receiver: Option<oneshot::Receiver<Ginkgo>>,
    pub(crate) ran_at_least_once: bool,
    pub(crate) suspended: bool,
}

impl Default for Foliage {
    fn default() -> Self {
        Self::new()
    }
}

impl Foliage {
    pub const SCROLL_SENSITIVITY: f32 = 100.0;
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
            ran_at_least_once: false,
            suspended: false,
        };
        foliage.main.configure_sets(
            (
                MainMarkers::External,
                MainMarkers::Animation,
                MainMarkers::Process,
            )
                .chain(),
        );
        foliage.diff.configure_sets(
            (
                DiffMarkers::Prepare,
                DiffMarkers::Finalize,
                DiffMarkers::Extract,
            )
                .chain(),
        );
        foliage.diff.add_systems((
            ApplyDeferred
                .after(DiffMarkers::Prepare)
                .before(DiffMarkers::Finalize),
            ApplyDeferred
                .after(DiffMarkers::Finalize)
                .before(DiffMarkers::Extract),
        ));
        foliage
            .main
            .add_systems(message_update_system.in_set(MainMarkers::External));
        Disable::attach(&mut foliage);
        Enable::attach(&mut foliage);
        Panel::attach(&mut foliage);
        Shape::attach(&mut foliage);
        Grid::attach(&mut foliage);
        Interaction::attach(&mut foliage);
        Icon::attach(&mut foliage);
        Ash::attach(&mut foliage);
        Text::attach(&mut foliage);
        Asset::attach(&mut foliage);
        Time::attach(&mut foliage);
        Remove::attach(&mut foliage);
        Opacity::attach(&mut foliage);
        Elevation::attach(&mut foliage);
        Color::attach(&mut foliage);
        Image::attach(&mut foliage);
        Visibility::attach(&mut foliage);
        Location::attach(&mut foliage);
        Named::attach(&mut foliage);
        TextInput::attach(&mut foliage);
        VirtualKeyboardAdapter::attach(&mut foliage);
        crate::Clipboard::attach(&mut foliage);
        foliage
    }
    pub fn attach<A: Attachment>(&mut self) {
        A::attach(self);
    }
    /// Runs the app -- the whole-organism, keeps-going process the event loop actually is,
    /// as distinct from [`Sow::grow`](crate::tree::Sow::grow) spawning one entity.
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
    pub fn define<M>(&mut self, obs: impl IntoObserver<M>) {
        self.world.add_observer(obs);
    }
    pub fn send_to<E>(&mut self, e: E, targets: impl IntoTargets)
    where
        E: TargetedEvent,
        for<'a> E::Trigger<'a>: Default,
    {
        self.world.send_to(e, targets);
    }
    pub fn send<E>(&mut self, e: E)
    where
        E: Event,
        for<'a> E::Trigger<'a>: Default,
    {
        self.world.send(e);
    }
    pub fn queue<E: Message>(&mut self, e: E) {
        self.world.queue(e);
    }
    pub fn enable_queued_event<E: Message + Clone + Send + Sync + 'static>(&mut self) {
        if self.world.get_resource::<Messages<E>>().is_none() {
            self.world.insert_resource(Messages::<E>::default());
            MessageRegistry::register_message::<E>(&mut self.world);
        }
    }
    pub fn write_to<B: Bundle>(&mut self, entity: Entity, b: B) {
        self.world.write_to(entity, b);
    }
    pub fn remove(&mut self, targets: impl IntoTargets) {
        self.world.remove(targets);
    }
    pub fn enable(&mut self, targets: impl IntoTargets) {
        self.world.enable(targets);
    }
    pub fn disable(&mut self, targets: impl IntoTargets) {
        self.world.disable(targets);
    }
    pub fn enable_animation<A: Animate + Component<Mutability = bevy_ecs::component::Mutable>>(
        &mut self,
    ) {
        debug_assert_eq!(
            self.world.get_resource::<AnimationLimiter<A>>().is_none(),
            true
        );
        self.main
            .add_systems(animate::<A>.in_set(MainMarkers::Animation));
        self.world.insert_resource(AnimationLimiter::<A>::new());
    }
    pub fn sequence(&mut self) -> Entity {
        self.world.sequence()
    }
    pub fn animate<A: Animate + Component>(&mut self, anim: Animation<A>) -> Entity {
        self.world.animate(anim)
    }
    pub fn sequence_end<M>(&mut self, seq: Entity, end: impl IntoEntityObserver<M>) {
        self.world.sequence_end(seq, end);
    }
    pub fn subscribe<M>(&mut self, e: Entity, sub: impl IntoEntityObserver<M>) {
        self.world.subscribe(e, sub);
    }
    pub fn on_click<M>(&mut self, e: Entity, o: impl IntoEntityObserver<M>) {
        self.world.on_click(e, o);
    }
    pub fn name<S: AsRef<str>>(&mut self, e: Entity, s: S) {
        self.world.name(e, s);
    }
    pub fn store<S: AsRef<str>>(&mut self, key: AssetKey, s: S) {
        self.world.store(key, s);
    }
    pub fn timer<M>(&mut self, t: u64, tf: impl IntoEntityObserver<M>) {
        self.world.timer(t, tf);
    }
    pub(crate) fn remove_queue<R: Clone + Send + Sync + 'static>(&mut self) {
        debug_assert!(self.world.get_resource::<RenderRemoveQueue<R>>().is_none());
        self.world.insert_resource(RenderRemoveQueue::<R>::new());
    }
    pub(crate) fn differential<
        R: Clone + Send + Sync + 'static,
        RT: Clone + Send + Sync + 'static + Component + PartialEq,
    >(
        &mut self,
    ) {
        debug_assert!(self.world.get_resource::<RenderQueue<R, RT>>().is_none());
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
        self.willow.window().set_ime_allowed(true);
        self.ginkgo.configure_view(&self.willow);
        self.ginkgo.create_viewport(&self.willow);
        let scale_factor = self.ginkgo.configuration().scale_factor;
        self.world.insert_resource(ViewportHandle::new(
            self.willow.actual_area().to_logical(scale_factor.value()),
        ));
        self.world.insert_resource(scale_factor);
        self.ash.initialize(&self.ginkgo);
        self.booted = true;
    }
}
#[derive(Resource)]
struct AnimationLimiter<A: Animate> {
    _phantom: PhantomData<A>,
}
impl<A: Animate> AnimationLimiter<A> {
    fn new() -> AnimationLimiter<A> {
        Self {
            _phantom: Default::default(),
        }
    }
}
#[derive(SystemSet, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy, Debug)]
pub(crate) enum MainMarkers {
    External,
    Animation,
    Process,
}
#[derive(SystemSet, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy, Debug)]
pub(crate) enum DiffMarkers {
    Prepare,
    Finalize,
    Extract,
}
