use crate::anim::animate;
use crate::ash::Ash;
use crate::ash::differential::{RenderQueue, RenderRemoveQueue, cached_differential};
use crate::asset::{Asset, AssetKey, AssetLoader, AssetSource, LoadAsset};
use crate::ginkgo::Ginkgo;
use crate::ginkgo::viewport::ViewportHandle;
use crate::remove::Remove;
use crate::time::{OnEnd, Time};
use crate::tree::{IntoTargets, TargetedEvent};
use crate::virtual_keyboard::VirtualKeyboardAdapter;
use crate::willow::Willow;
use crate::{
    AndroidConnection, Animate, Animation, Area, Attachment, Color, Disable, EcsExtension,
    Elevation, Enable, Grid, Icon, Image, Interaction, Line, Location, Named, OnClick, Opacity,
    Panel, Physical, Polygon, Resource, SystemSet, Text, TextInput, Visibility,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::message::{Message, MessageRegistry, Messages, message_update_system};
use bevy_ecs::observer::{IntoEntityObserver, IntoObserver};
use bevy_ecs::prelude::{ApplyDeferred, IntoScheduleConfigs, Schedule, World};
use futures_channel::oneshot;
use std::marker::PhantomData;
use tracing_subscriber::Layer;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use winit::event::WindowEvent;
use winit::event_loop::{ControlFlow, EventLoop};

pub struct Foliage {
    pub world: World,
    pub(crate) main: Schedule,
    pub user: Schedule,
    pub(crate) diff: Schedule,
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
    /// True from the moment `about_to_wait` requests a redraw until that redraw actually
    /// paints. Winit's `about_to_wait` isn't 1:1 with real paint frames -- high-frequency
    /// input (mouse move, wheel/scroll especially on web, where each DOM event tends to
    /// pump its own cycle rather than batching like native OS event queues do) can fire it
    /// many times before the next `RedrawRequested`. Without this gate, each of those ticks
    /// re-runs `main`/`user`/`diff` and requests another redraw, stacking up several
    /// generations of ECS churn (entity spawns/despawns from reactive rebuilds, e.g.
    /// `Polyline`) that never individually get painted -- only the last of which should
    /// exist by the time a paint finally happens. See the winit `ApplicationHandler` docs:
    /// "high frequency event sources... could potentially lead to lots of wake ups and
    /// also lots of corresponding `AboutToWait` events."
    pub(crate) tick_pending: bool,
}

#[cfg(not(target_os = "android"))]
impl Default for Foliage {
    fn default() -> Self {
        Self::new()
    }
}

impl Foliage {
    pub const SCROLL_SENSITIVITY: f32 = 40.0;
    pub const NATURAL_SCROLLING: f32 = -1.0;
    pub const VIEW_SCROLLING: f32 = 1.0;
    /// Every other platform's entry point owns nothing external -- this is the one place
    /// there's a real handle (the `AndroidApp` Android hands you at process start) that has
    /// to exist *before* a `Foliage` is meaningful, so unlike every other platform there's no
    /// default to fall back on here: without it, `photosynthesize` has nothing to build the
    /// event loop against.
    #[cfg(target_os = "android")]
    pub fn android(app: crate::AndroidApp) -> Foliage {
        Self::build(AndroidConnection(app))
    }
    #[cfg(not(target_os = "android"))]
    pub fn new() -> Foliage {
        Self::build(AndroidConnection::default())
    }
    fn build(android_connection: AndroidConnection) -> Foliage {
        let mut foliage = Foliage {
            world: Default::default(),
            main: Default::default(),
            user: Default::default(),
            diff: Default::default(),
            willow: Default::default(),
            ginkgo: Default::default(),
            ash: Default::default(),
            android_connection,
            booted: false,
            queue: vec![],
            sender: None,
            receiver: None,
            ran_at_least_once: false,
            suspended: false,
            tick_pending: false,
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
        Line::attach(&mut foliage);
        Polygon::attach(&mut foliage);
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
        // winit's android backend has nothing to poll events from without the `AndroidApp`
        // handle threaded through at event-loop construction -- `EventLoop::new()` alone
        // panics there. Every other platform has no such handle to give it.
        #[cfg(target_os = "android")]
        let event_loop = {
            use winit::platform::android::EventLoopBuilderExtAndroid;
            EventLoop::builder()
                .with_android_app(self.android_connection.0.clone())
                .build()
                .unwrap()
        };
        #[cfg(not(target_os = "android"))]
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
    /// The browser's own origin (e.g. `https://example.com`) -- a raw environment fact, not a
    /// hosting convention. Callers building a full URL for `AssetSource::Url` compose whatever
    /// path structure their own deployment uses on top of this themselves; the crate assumes
    /// nothing about where an app's assets live.
    #[cfg(target_family = "wasm")]
    pub fn window_origin() -> String {
        web_sys::window().expect("window").origin()
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
    /// Load an asset at runtime -- `Bytes` for data already in hand, `Url` for a full,
    /// already-resolved location to fetch from (native: blocking; wasm: async, resolves once
    /// `Image`/whatever holds `key` next reacts to `OnRetrieval`). `key` is usable immediately
    /// regardless of which variant is given or how long it takes to resolve.
    pub fn load_asset(&mut self, source: AssetSource) -> AssetKey {
        let key = AssetLoader::generate_key();
        self.send(LoadAsset { key, source });
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
