use crate::anim::animate;
use crate::ash::Ash;
use crate::ash::differential::{RenderQueue, RenderRemoveQueue, cached_differential};
use crate::asset::{Asset, AssetKey, AssetLoader, AssetSource, LoadAsset};
use crate::ginkgo::Ginkgo;
use crate::ginkgo::viewport::ViewportHandle;
use crate::remove::Remove;
use crate::time::Time;
use crate::tree::{IntoTargets, TargetedEvent};
use crate::virtual_keyboard::VirtualKeyboardAdapter;
use crate::willow::Willow;
use crate::{
    AndroidConnection, Animate, Animation, Area, Attachment, Color, Disable, EcsExtension,
    Elevation, Enable, Grid, Icon, Image, Interaction, Line, Location, Named, Opacity,
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

/// The engine instance: the ECS world, the frame schedules, the window, and the renderer.
///
/// Built with [`Foliage::new`], configured by attaching what the app needs and spawning
/// the initial tree, then handed control with
/// [`photosynthesize`](Foliage::photosynthesize), which does not return.
///
/// Before that call it doubles as a [`Tree`](crate::Tree)-like builder -- `leaf`, `branch`,
/// `animate`, `on_click` are all available here through the same
/// [`EcsExtension`](crate::EcsExtension) vocabulary systems use at runtime.
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
    /// This app's asset hosting convention, set once via [`asset_base`](Foliage::asset_base)
    /// and applied by [`asset_url`](Foliage::asset_url). Empty by default -- an app that
    /// never loads a `Url` asset never needs it.
    pub(crate) asset_base: String,
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
    /// Logical pixels one wheel notch scrolls, before a view's own
    /// [`ScrollInertia`](crate::grid::view::ScrollInertia) scaling.
    pub const SCROLL_SENSITIVITY: f32 = 40.0;
    /// Scroll direction multiplier where content follows the gesture -- push up, content
    /// goes up. The touch convention.
    pub const NATURAL_SCROLLING: f32 = -1.0;
    /// Scroll direction multiplier where the *viewport* follows the gesture -- push up,
    /// you move down the page. The wheel convention.
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
    /// A new engine instance. Attach what the app needs, build the tree, then hand
    /// control over with [`photosynthesize`](Self::photosynthesize).
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
            asset_base: String::new(),
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
    /// Installs an [`Attachment`]'s components, systems and resources. The built-in
    /// primitives are already attached by [`new`](Self::new); this is for an app's or a
    /// library's own.
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
    /// Requests an initial window size on desktop. Ignored where the platform owns the
    /// window's size.
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
    /// Registers a global observer -- one that watches an event across all entities,
    /// rather than being bound to one. Entity-scoped handlers go through
    /// [`subscribe`](Self::subscribe).
    pub fn define<M>(&mut self, obs: impl IntoObserver<M>) {
        self.world.add_observer(obs);
    }
    /// Fires a targeted event at one or more entities, delivered immediately to their
    /// observers.
    pub fn send_to<E>(&mut self, e: E, targets: impl IntoTargets)
    where
        E: TargetedEvent,
        for<'a> E::Trigger<'a>: Default,
    {
        self.world.send_to(e, targets);
    }
    /// Fires an untargeted event, delivered immediately to global observers.
    pub fn send<E>(&mut self, e: E)
    where
        E: Event,
        for<'a> E::Trigger<'a>: Default,
    {
        self.world.send(e);
    }
    /// Queues a message for whichever system reads it this frame -- deferred, unlike
    /// [`send`](Self::send). Requires [`enable_queued_event`](Self::enable_queued_event)
    /// for the type first.
    pub fn queue<E: Message>(&mut self, e: E) {
        self.world.queue(e);
    }
    /// Registers a message type so [`queue`](Self::queue) can carry it. Idempotent.
    pub fn enable_queued_event<E: Message + Clone + Send + Sync + 'static>(&mut self) {
        if self.world.get_resource::<Messages<E>>().is_none() {
            self.world.insert_resource(Messages::<E>::default());
            MessageRegistry::register_message::<E>(&mut self.world);
        }
    }
    /// Inserts components on an existing entity -- the way a live value is changed after
    /// spawn. Unchecked by entity ID, so a caller holding an id across a possible despawn
    /// should confirm the entity still exists.
    pub fn write_to<B: Bundle>(&mut self, entity: Entity, b: B) {
        self.world.write_to(entity, b);
    }
    /// Despawns entities and everything beneath them.
    pub fn remove(&mut self, targets: impl IntoTargets) {
        self.world.remove(targets);
    }
    /// Re-enables interaction on entities and their subtrees.
    pub fn enable(&mut self, targets: impl IntoTargets) {
        self.world.enable(targets);
    }
    /// Disables interaction on entities and their subtrees. They still draw; they stop
    /// competing for input.
    pub fn disable(&mut self, targets: impl IntoTargets) {
        self.world.disable(targets);
    }
    /// Registers the systems that tween `A`, letting `Animation<A>` run. Needed once per
    /// custom [`Animate`] type; the built-in ones do it themselves.
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
    /// Starts a sequence -- a group of animations sharing a timeline, whose completion
    /// fires one [`OnEnd`](crate::OnEnd). Pass it to
    /// [`Animation::during`](crate::Animation::during).
    pub fn sequence(&mut self) -> Entity {
        self.world.sequence()
    }
    /// Starts an animation, returning its entity.
    pub fn animate<A: Animate + Component>(&mut self, anim: Animation<A>) -> Entity {
        self.world.animate(anim)
    }
    /// Runs `end` once every animation in `seq` has finished -- how one stage of motion
    /// is chained onto the next.
    pub fn sequence_end<M>(&mut self, seq: Entity, end: impl IntoEntityObserver<M>) {
        self.world.sequence_end(seq, end);
    }
    /// Registers an observer scoped to one entity.
    pub fn subscribe<M>(&mut self, e: Entity, sub: impl IntoEntityObserver<M>) {
        self.world.subscribe(e, sub);
    }
    /// Runs `o` when `e` is clicked. Shorthand for subscribing to
    /// [`OnClick`](crate::OnClick).
    pub fn on_click<M>(&mut self, e: Entity, o: impl IntoEntityObserver<M>) {
        self.world.on_click(e, o);
    }
    /// Records `e` under `s` in [`Named`](crate::Named), so other code can find it
    /// without the id being threaded through.
    pub fn name<S: AsRef<str>>(&mut self, e: Entity, s: S) {
        self.world.name(e, s);
    }
    /// Records an asset key under `s` in [`Keyring`](crate::Keyring).
    pub fn store<S: AsRef<str>>(&mut self, key: AssetKey, s: S) {
        self.world.store(key, s);
    }
    /// Runs `tf` once after `t` milliseconds. Backed by a [`Timer`](crate::Timer) entity
    /// that despawns itself when it fires.
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
    /// Declares where this app's assets are served from -- the path segment between the
    /// page origin and an asset's own relative path. Set it once at startup and
    /// [`bundled_asset!`](crate::bundled_asset) resolves every wasm URL through it, so no
    /// call site repeats the convention and nothing has to define its own `fn(&str) ->
    /// String` to pass along. Leading and trailing slashes are optional.
    ///
    /// This crate still assumes nothing about hosting; it only stores what the app says.
    pub fn asset_base<S: AsRef<str>>(&mut self, base: S) {
        self.asset_base = base.as_ref().trim_matches('/').to_string();
    }
    /// `path` resolved against the page origin and [`asset_base`](Self::asset_base) --
    /// what [`bundled_asset!`](crate::bundled_asset) hands to `AssetSource::Url`. Public
    /// because the macro expands at the call site, but an app rarely calls it directly.
    #[cfg(target_family = "wasm")]
    pub fn asset_url(&self, path: &str) -> String {
        let path = path.trim_start_matches('/');
        if self.asset_base.is_empty() {
            format!("{}/{path}", Self::window_origin())
        } else {
            format!("{}/{}/{path}", Self::window_origin(), self.asset_base)
        }
    }
    /// Installs a tracing subscriber for `targets`, e.g.
    /// `"foliage_proper::grid=trace".parse().unwrap()`. Call before
    /// [`photosynthesize`](Self::photosynthesize); on wasm the logs go to the browser
    /// console.
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
