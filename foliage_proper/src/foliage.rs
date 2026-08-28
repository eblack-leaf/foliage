use crate::anim::animate;
use crate::ash::Ash;
use crate::ash::differential::{RenderQueue, RenderRemoveQueue, cached_differential};
use crate::asset::{Asset, AssetKey, AssetLoader, AssetSource, LoadAsset};
use crate::boundary::moss::Emissions;
use crate::boundary::root::{Root, Rooted};
use crate::ginkgo::Ginkgo;
use crate::ginkgo::viewport::ViewportHandle;
use crate::remove::Remove;
use crate::text::monospaced::{FontId, MonospacedFont};
use crate::time::Time;
use crate::virtual_keyboard::VirtualKeyboardAdapter;
use crate::willow::Willow;
use crate::{
    AndroidConnection, Animate, Area, Attachment, Color, Disable, Elevation, Enable, Grid,
    Icon, Image, Interaction, Line, Location, Named, Opacity, Panel, Physical, Polygon, Resource,
    SystemSet, Text, TextInput, Visibility,
};
use crate::{Forest, Sprig};
use bevy_ecs::component::Component;
use bevy_ecs::message::{Message, MessageRegistry, Messages, message_update_system};
use bevy_ecs::observer::IntoObserver;
use bevy_ecs::prelude::{ApplyDeferred, IntoScheduleConfigs, Schedule, World};
use bevy_ecs::system::SystemState;
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
/// Built with [`Foliage::new`], configured with the setup calls below -- artwork and font
/// registration, desktop sizing, tuning -- then handed control with
/// [`photosynthesize`](Foliage::photosynthesize), which does not return. The tree itself
/// is never touched here: an app grows and changes it from inside its [`Root`], through the
/// [`Forest`] that root is handed each frame.
pub struct Foliage {
    pub(crate) world: World,
    pub(crate) main: Schedule,
    pub(crate) diff: Schedule,
    /// The app itself, once [`root`](Foliage::root) has named its type. Held here rather than
    /// in the world because `Foliage` owns the world -- so handing it a `Forest` borrowed from
    /// that same world is a plain reborrow, with no interior-mutability dance to arrange it.
    pub(crate) root: Option<Box<dyn Rooted>>,
    /// Read state for the root, kept across frames so its query caches persist.
    pub(crate) reads: Option<SystemState<crate::boundary::forest::Reads<'static, 'static>>>,
    /// This frame's commands, drained into the world after the root returns.
    pub(crate) ops: Vec<crate::boundary::op::Op>,
    /// The off-thread command channel, drained ahead of the root's own commands.
    pub(crate) sprig: Sprig,
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
    /// and applied by `asset_url` (wasm-only). Empty by default -- an app that never loads
    /// a `Url` asset never needs it.
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
    /// Logical pixels one wheel notch scrolls.
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
        let world = World::default();
        // Every name an app is ever handed comes from this one allocator, whichever side of
        // the boundary asks for it -- which is what makes two of them impossible to collide.
        let allocator = world.entity_allocator().build_remote_allocator();
        let mut foliage = Foliage {
            world,
            main: Default::default(),
            diff: Default::default(),
            root: None,
            reads: None,
            ops: Vec::new(),
            sprig: Sprig::new(allocator),
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
        crate::boundary::Boundary::attach(&mut foliage);
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
    /// A [`Tree`](crate::Tree) over this instance's world. Internal: an app builds its tree
    /// inside its [`Root`], through [`Forest`], and never touches the world.
    pub(crate) fn tree(&mut self) -> crate::Tree<'_, '_> {
        crate::AsTree::tree(&mut self.world)
    }
    /// Runs one frame of app code: hand it what happened and what things are, take back what
    /// it wants done, and do it.
    ///
    /// Ordering here is the whole contract. Off-thread commands drain first, so they are
    /// never interleaved with the frame's own; the root's commands then apply in the order it
    /// wrote them; and all of it lands before `diff` runs, so a command issued this frame
    /// reaches the screen this frame.
    pub(crate) fn frame(&mut self) {
        let Some(mut root) = self.root.take() else {
            return;
        };
        self.sprig.drain_into(&mut self.ops);
        crate::boundary::op::apply(&mut self.world, &mut self.ops);
        let mosses = core::mem::take(&mut self.world.resource_mut::<Emissions>().0);
        let mut reads = self
            .reads
            .take()
            .unwrap_or_else(|| SystemState::new(&mut self.world));
        {
            // Every param is read-only, so this borrow cannot fail on conflicting access --
            // the only way `get` errors is a param that is invalid outright, which for a set
            // of plain queries and resources means a resource that does not exist yet.
            let reads = reads
                .get(&self.world)
                .expect("frame reads are all read-only and always available");
            // Published from the same reads the root is about to be handed, so a worker's
            // ambient state is the frame's own rather than a second answer assembled elsewhere.
            self.sprig.publish(crate::Conditions {
                viewport: reads.viewport.section(),
                layout: *reads.layout,
                short: *reads.short == crate::Short::Yes,
                scale_factor: reads.scale_factor.value(),
                frame_time: reads.time.frame_diff(),
            });
            // Off-thread listeners get the same emissions the root does, from the same collection,
            // before the root has had a chance to act on them and change what they describe.
            self.sprig.deliver(&mosses);
            let mut forest = Forest {
                reads,
                queue: &mut self.ops,
                allocator: self.sprig.allocator(),
            };
            root.frame(&mut forest, mosses);
        }
        self.reads = Some(reads);
        self.root = Some(root);
        crate::boundary::op::apply(&mut self.world, &mut self.ops);
    }
    /// A cloneable, `Send` handle for issuing commands from your own thread.
    ///
    /// Command-only by design: reads happen at the frame callsite, where the engine is
    /// quiescent and a value read is a value that is actually true. Everything queued through
    /// a `Sprig` is applied at the top of the next frame, ahead of that frame's own commands.
    pub fn sprig(&self) -> Sprig {
        self.sprig.clone()
    }
    /// Runs the app, calling the [`Root`]'s `frame` once per frame -- after the engine has
    /// settled and before anything is drawn. Does not return.
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
    /// Names the [`Root`] this app is. Everything after it is that type's business: it grows
    /// the tree on the first frame and is called once per frame from then on.
    ///
    /// The root stays on this thread deliberately -- that is what lets it hold a [`Forest`]
    /// and read live state directly instead of asking and waiting. For work that belongs on
    /// another thread, take a [`Sprig`](Self::sprig) before calling this.
    pub fn root<R: Root>(&mut self) {
        self.root = Some(Box::new(crate::boundary::root::Planted::<R>::new()));
    }
    /// Requests an initial window size on desktop. Ignored where the platform owns the
    /// window's size.
    pub fn desktop_size<V: Into<Area<Physical>>>(&mut self, v: V) {
        self.willow.requested_size.replace(v.into());
    }
    /// How the desktop identifies this application. Ignored where the platform has no such notion.
    ///
    /// Not a title. A title labels a window for a person to read and may change while the app
    /// runs; this identifies the *application* so a shell can match the running window back to the
    /// `.desktop` entry that launched it, and it should stay fixed for the life of the program.
    ///
    /// Set it to the same string as the entry's `StartupWMClass` -- conventionally the binary's
    /// name, or a reverse-DNS id if the app ships one. **Unset, nothing is published**, and a
    /// desktop that cannot recognise the window shows a second, generic icon next to the one the
    /// user clicked to start it.
    ///
    /// On Linux and the BSDs this becomes the `xdg_toplevel` app_id under Wayland and the
    /// `WM_CLASS` pair under X11, from the one value.
    pub fn app_id<S: Into<String>>(&mut self, v: S) {
        self.willow.app_id.replace(v.into());
    }
    /// The browser's own origin (e.g. `https://example.com`) -- a raw environment fact, not a
    /// hosting convention. Callers building a full URL for `AssetSource::Url` compose whatever
    /// path structure their own deployment uses on top of this themselves; the crate assumes
    /// nothing about where an app's assets live.
    #[cfg(target_family = "wasm")]
    pub fn window_origin() -> String {
        web_sys::window().expect("window").origin()
    }
    /// Registers a global observer -- one that watches an event across all entities, rather
    /// than being bound to one. Internal: the argument is a bevy observer over bevy events,
    /// which is the whole vocabulary the boundary exists to keep on this side.
    pub(crate) fn define<M>(&mut self, obs: impl IntoObserver<M>) {
        self.world.add_observer(obs);
    }
    /// Registers a message type so a message can be carried. Idempotent.
    pub(crate) fn enable_queued_event<E: Message + Clone + Send + Sync + 'static>(&mut self) {
        if self.world.get_resource::<Messages<E>>().is_none() {
            self.world.insert_resource(Messages::<E>::default());
            MessageRegistry::register_message::<E>(&mut self.world);
        }
    }
    /// Registers the systems that tween `A`, letting `Animation<A>` run. Needed once per
    /// [`Animate`] type; the built-in ones do it themselves.
    pub(crate) fn enable_animation<
        A: Animate + Component<Mutability = bevy_ecs::component::Mutable>,
    >(
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
        self.tree().send(LoadAsset { key, source });
        key
    }
    /// Registers an icon's artwork, so every [`Icon`] drawn with that id has a field to draw
    /// from. Startup-only, like [`font`](Self::font) -- the generated module `foliage_icons`
    /// emits calls a `register(&mut Foliage)` makes in one go.
    pub fn icon(&mut self, memory: crate::IconMemory) {
        self.world.spawn(memory);
    }
    /// Sets a startup resource an app is allowed to tune: scroll momentum, the text input's
    /// key bindings, and anything else the engine reads once and never writes.
    pub fn tune<R: crate::Tuning>(&mut self, value: R) {
        value.install(self);
    }
    /// Registers a monospace font and hands back the [`FontId`] naming it. Put that id on a
    /// [`Text`] entity (composites forward it like [`FontSize`](crate::FontSize)) to draw
    /// with it; anything that never sets one uses the bundled JetBrains Mono.
    ///
    /// **Panics if `bytes` is not monospaced.** foliage sizes and addresses text by a fixed
    /// character cell -- `.letters()`, the caret's columns -- so a proportional font does
    /// not merely look different, it mispositions. Checked here, where the offending font
    /// can still be identified.
    ///
    /// Registration is startup-only: this takes `&mut Foliage`, which no longer exists once
    /// [`photosynthesize`](Self::photosynthesize) has been called. That keeps a `Text` from
    /// ever naming a font that has not been registered yet, but it also means a font fetched
    /// at runtime cannot be registered -- bundle it with `include_bytes!` instead.
    pub fn font(&mut self, bytes: &[u8]) -> FontId {
        let font = MonospacedFont::parse(bytes, Text::OPT_SCALE);
        self.world
            .get_resource_mut::<MonospacedFont>()
            .expect("fonts")
            .add(font)
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
    /// console, and on Android to logcat (`adb logcat -s foliage`).
    pub fn enable_tracing(&self, targets: Targets) {
        #[cfg(all(not(target_family = "wasm"), not(target_os = "android")))]
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .compact()
                    .with_filter(targets),
            )
            .init();
        // Android drops stdout/stderr, so the default `fmt` writer reaches nothing at all --
        // logs have to go through `__android_log_write` to show up in logcat. Same shape as
        // the wasm arm below: only the writer changes.
        #[cfg(target_os = "android")]
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(paranoid_android::AndroidLogMakeWriter::new(
                        "foliage".to_string(),
                    ))
                    .without_time()
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
