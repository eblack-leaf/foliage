use std::any::TypeId;
use std::marker::PhantomData;

use anymap::AnyMap;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::{event_update_system, Event, Events};
use bevy_ecs::prelude::{Component, IntoSystemConfigs, Resource};
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Command, Commands, Query, RunSystemOnce, StaticSystemParam, SystemParam};
use bytemuck::{Pod, Zeroable};
use compact_str::{CompactString, ToCompactString};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use leaf::Leaflet;

use crate::animate::{Animation, Interpolate};
use crate::ash::render_packet::RenderPacketForwarder;
use crate::ash::render_packet::RenderPacketPackage;
use crate::asset::{AssetContainer, AssetFetchFn, AssetKey, OnFetch};
use crate::conditional::{
    clean_scene, conditional_command, conditional_extension, conditional_scene_spawn,
    conditional_spawn,
};
use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::layer::Layer;
use crate::coordinate::position::{CReprPosition, Position};
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, InterfaceContext};
use crate::derivation::{component_derive_value, resource_derive_value};
use crate::elm::config::{CoreSet, ElmConfiguration, ExternalSet};
use crate::ginkgo::viewport::ViewportHandle;
use crate::interaction::InteractionListener;
use crate::job::{Container, Job, Task};
use crate::scene::Scene;
use crate::view::{
    Compositor, Navigate, PersistentView, View, ViewBuilder, ViewDescriptor, ViewHandle, Viewable,
};
use crate::window::ScaleFactor;
#[cfg(target_family = "wasm")]
use crate::Workflow;

pub mod config;
pub mod leaf;

pub struct Elm {
    initialized: bool,
    pub(crate) job: Job,
    limiters: AnyMap,
}

#[macro_export]
macro_rules! differential_enable {
    ($elm:ident $(,$typename:ty)+) => {
        $($elm.enable_differential::<$typename>();)+
    };
}
struct DifferentialLimiter<T>(PhantomData<T>);
impl<T> Default for DifferentialLimiter<T> {
    fn default() -> Self {
        DifferentialLimiter(PhantomData)
    }
}
struct ConditionalLimiter<C>(PhantomData<C>);
impl<C> Default for ConditionalLimiter<C> {
    fn default() -> Self {
        ConditionalLimiter(PhantomData)
    }
}
struct AnimationLimiter<C>(PhantomData<C>);
impl<C> Default for AnimationLimiter<C> {
    fn default() -> Self {
        AnimationLimiter(PhantomData)
    }
}
struct ComponentDerivationLimiter<C, D>(PhantomData<C>, PhantomData<D>);
impl<C, D> Default for ComponentDerivationLimiter<C, D> {
    fn default() -> Self {
        ComponentDerivationLimiter(PhantomData, PhantomData)
    }
}
struct ResourceDerivationLimiter<C, D>(PhantomData<C>, PhantomData<D>);
impl<C, D> Default for ResourceDerivationLimiter<C, D> {
    fn default() -> Self {
        ResourceDerivationLimiter(PhantomData, PhantomData)
    }
}
#[derive(Bundle, Clone)]
pub struct BundleExtension<T: Bundle + Clone, S: Bundle + Clone> {
    pub original: T,
    pub extension: S,
}

impl<T: Bundle + Clone, S: Bundle + Clone> BundleExtension<T, S> {
    pub fn new(t: T, s: S) -> Self {
        Self {
            original: t,
            extension: s,
        }
    }
}

pub trait BundleExtend
where
    Self: Bundle + Sized + Clone,
{
    fn extend<E: Bundle + Clone>(self, handle: E) -> BundleExtension<Self, E>;
}

impl<I: Bundle + Clone> BundleExtend for I {
    fn extend<E: Bundle + Clone>(self, handle: E) -> BundleExtension<I, E> {
        BundleExtension::new(self, handle)
    }
}
#[derive(Debug)]
pub enum EventStage {
    External,
    Process,
}
impl EventStage {
    pub fn set(&self) -> CoreSet {
        match self {
            EventStage::External => CoreSet::ExternalEvent,
            EventStage::Process => CoreSet::ProcessEvent,
        }
    }
}
pub trait Fetch {
    fn on_fetch(&mut self, key: AssetKey, func: AssetFetchFn);
}
impl<'w, 's> Fetch for Commands<'w, 's> {
    fn on_fetch(&mut self, key: AssetKey, func: AssetFetchFn) {
        self.spawn(OnFetch::new(key, func));
    }
}
impl Elm {
    pub fn main(&mut self) -> &mut Task {
        self.job.main()
    }
    pub fn startup(&mut self) -> &mut Task {
        self.job.startup()
    }
    pub fn teardown(&mut self) -> &mut Task {
        self.job.teardown()
    }
    pub fn container(&mut self) -> &mut Container {
        &mut self.job.container
    }
    pub fn on_fetch(&mut self, key: AssetKey, func: AssetFetchFn) {
        self.container().spawn(OnFetch::new(key, func));
    }
    pub(crate) fn new() -> Self {
        Self {
            initialized: false,
            job: Job::new(),
            limiters: AnyMap::new(),
        }
    }
    pub(crate) fn set_scale_factor(&mut self, factor: CoordinateUnit) {
        tracing::trace!("setting scale_factor: {:?}", factor);
        self.job.container.insert_resource(ScaleFactor::new(factor));
    }
    pub(crate) fn viewport_handle_changes(&mut self) -> Option<Position<InterfaceContext>> {
        tracing::trace!("pulling-viewport-handle-changes");
        self.job
            .container
            .get_resource_mut::<ViewportHandle>()
            .unwrap()
            .changes()
    }
    pub(crate) fn render_packet_package(&mut self) -> RenderPacketPackage {
        tracing::trace!("packaging-render-packet forwarded");
        self.job
            .container
            .get_resource_mut::<RenderPacketForwarder>()
            .unwrap()
            .package_for_transit()
    }
    pub(crate) fn initialized(&self) -> bool {
        self.initialized
    }
    pub(crate) fn attach_leafs(&mut self, leaflets: Vec<Leaflet>) {
        ElmConfiguration::configure_elm(self, &leaflets);
        self.enable_differential::<CReprPosition>();
        self.enable_differential::<CReprArea>();
        self.enable_differential::<Layer>();
        self.job
            .container
            .insert_resource(RenderPacketForwarder::default());
        self.main()
            .add_systems(remove_web_element.in_set(CoreSet::ExternalEvent));
        for leaf in leaflets {
            leaf.1(self)
        }
        self.initialized = true;
    }
    pub(crate) fn attach_viewport_handle(&mut self, area: Area<InterfaceContext>) {
        tracing::trace!("attaching viewport handle :{:?}", area);
        self.job
            .container
            .insert_resource(ViewportHandle::new(Section::default().with_area(area)));
    }
    pub(crate) fn set_viewport_handle_area(&mut self, area: Area<InterfaceContext>) {
        tracing::trace!("setting viewport handle :{:?}", area);
        self.job
            .container
            .get_resource_mut::<ViewportHandle>()
            .unwrap()
            .adjust_area(area);
    }
    pub fn add_event<E: Event>(&mut self, stage: EventStage) {
        tracing::trace!("add event to :{:?}", stage);
        self.job.container.insert_resource(Events::<E>::default());
        self.main()
            .add_systems((event_update_system::<E>.in_set(stage.set()),));
    }
    pub fn enable_differential<
        T: Component + Clone + PartialEq + Serialize + for<'a> Deserialize<'a>,
    >(
        &mut self,
    ) {
        if self.limiters.get::<DifferentialLimiter<T>>().is_none() {
            tracing::trace!("enabling differential :{:?}", ());
            self.job.main().add_systems((
                crate::differential::differential::<T>.in_set(CoreSet::Differential),
                crate::differential::send_on_differential_disable_changed::<T>
                    .in_set(CoreSet::Differential),
            ));
            self.limiters.insert(DifferentialLimiter::<T>::default());
        }
    }
    pub(crate) fn finish_initialization(&mut self) {
        tracing::trace!("finish initialization :{:?}", ());
        self.job.exec_startup();
        self.job.resume();
        self.initialized = true;
    }
    pub fn remove_web_element(&mut self, id: &'static str) {
        tracing::trace!("remove web element :{:?}", ());
        self.container().spawn(WebElementRemoval::new(id));
    }
    pub fn send_event<E: Event>(&mut self, e: E) {
        tracing::trace!("send event :{:?}", ());
        self.container().send_event(e);
    }
    #[cfg(target_family = "wasm")]
    pub fn load_remote_asset<W: Workflow + Default + Send + Sync + 'static, S: AsRef<str>>(
        &mut self,
        path: S,
    ) -> AssetKey {
        tracing::trace!("load-remote-asset :{:?}", path.as_ref());
        let id = self.generate_asset_key();
        let message = crate::system_message::SystemMessageAction::WasmAsset(
            id,
            format!(
                "{}{}",
                web_sys::window().unwrap().origin(),
                path.as_ref().to_string()
            ),
        );
        self.container()
            .get_non_send_resource::<crate::WorkflowBridge<W>>()
            .unwrap()
            .system_send(message);
        self.container()
            .get_resource_mut::<crate::AssetContainer>()
            .unwrap()
            .store(id, None);
        return id;
    }
    pub fn generate_asset_key(&self) -> AssetKey {
        let key = Uuid::new_v4().as_u128();
        tracing::trace!("generate asset-key :{:?}", key);
        key
    }
    pub fn store_local_asset(&mut self, id: AssetKey, bytes: Vec<u8>) {
        tracing::trace!("storing asset :{:?}", id);
        self.container()
            .get_resource_mut::<AssetContainer>()
            .unwrap()
            .store(id, Some(bytes));
    }
    pub fn add_interaction_handler<IH: Component + 'static, Ext: SystemParam + 'static>(
        &mut self,
        handler: InteractionHandlerFn<IH, Ext>,
    ) {
        let func = move |mut ext: StaticSystemParam<Ext>,
                         mut ihs: Query<
            (&InteractionListener, &mut IH),
            Changed<InteractionListener>,
        >| {
            for (trigger, mut ih) in ihs.iter_mut() {
                if trigger.active() {
                    handler(&mut ih, &mut ext);
                }
            }
        };
        tracing::trace!("adding interaction-handler :{:?}", ());
        self.main().add_systems(func.in_set(ExternalSet::Process));
    }
    pub fn enable_conditional<C: Bundle + Clone + Send + Sync + 'static>(&mut self) {
        if self
            .limiters
            .insert::<ConditionalLimiter<C>>(ConditionalLimiter::default())
            .is_none()
        {
            tracing::trace!("enabling-conditional:{:?}", ());
            self.main().add_systems((
                conditional_spawn::<C>.in_set(ExternalSet::ConditionalBind),
                conditional_extension::<C>.in_set(ExternalSet::ConditionalExt),
            ));
        }
    }
    pub fn enable_conditional_scene<S: Scene + Clone + Send + Sync + 'static>(&mut self) {
        if self
            .limiters
            .insert::<ConditionalLimiter<S>>(ConditionalLimiter::default())
            .is_none()
        {
            tracing::trace!("enabling-conditional-scene:{:?}", ());
            self.main().add_systems((
                conditional_scene_spawn::<S>.in_set(ExternalSet::ConditionalBind),
                clean_scene::<S>.in_set(CoreSet::ProcessEvent),
            ));
        }
    }
    pub fn enable_conditional_command<COMM: Command + Clone + Send + Sync + 'static>(&mut self) {
        if self
            .limiters
            .insert::<ConditionalLimiter<COMM>>(ConditionalLimiter::default())
            .is_none()
        {
            tracing::trace!("enabling-conditional-command:{:?}", ());
            self.main()
                .add_systems(conditional_command::<COMM>.in_set(CoreSet::ProcessEvent));
        }
    }
    pub fn enable_component_derivation<
        I: Component + Clone,
        D: Component + 'static + Send + Sync,
    >(
        &mut self,
    ) {
        if self
            .limiters
            .insert::<ComponentDerivationLimiter<I, D>>(ComponentDerivationLimiter::default())
            .is_none()
        {
            self.main()
                .add_systems(component_derive_value::<I, D>.in_set(CoreSet::ProcessEvent));
        }
    }
    pub fn enable_resource_derivation<R: Resource + Clone, V: Component + 'static + Send + Sync>(
        &mut self,
    ) {
        if self
            .limiters
            .insert::<ResourceDerivationLimiter<R, V>>(ResourceDerivationLimiter::default())
            .is_none()
        {
            self.main()
                .add_systems(resource_derive_value::<R, V>.in_set(CoreSet::ProcessEvent));
        }
    }
    pub fn navigate_to<VH: Into<ViewHandle>>(&mut self, vh: VH) {
        self.container().spawn(Navigate(vh.into()));
    }
    pub fn add_view<V: Viewable>(&mut self, vh: ViewHandle) {
        self.container()
            .get_resource_mut::<Compositor>()
            .unwrap()
            .views
            .insert(vh, View::new(V::view, V::GRID));
    }
    pub fn persistent_view<V: Viewable>(&mut self, vh: ViewHandle) {
        let desc = self
            .container()
            .run_system_once(move |mut cmd: Commands| -> ViewDescriptor {
                let d = V::view(ViewBuilder::new(&mut cmd));
                for a in d.pool().0.iter() {
                    cmd.entity(*a).insert(PersistentView::new(vh));
                }
                for b in d.branches().values() {
                    cmd.entity(b.target()).insert(PersistentView::new(vh));
                }
                d
            });

        self.container()
            .get_resource_mut::<Compositor>()
            .unwrap()
            .persistent
            .insert(vh, (V::GRID, desc));
    }
    pub fn enable_animation<I: Interpolate>(&mut self) {
        let limit = self
            .limiters
            .insert(AnimationLimiter::<I>::default())
            .is_none();
        if limit {
            tracing::trace!("enabling-animation:{:?}", ());
            self.enable_conditional::<Animation<I>>();
            self.main()
                .add_systems(crate::animate::apply::<I>.in_set(ExternalSet::Animation));
        }
    }
}
pub type InteractionHandlerFn<IH, Ext> = fn(&mut IH, &mut StaticSystemParam<Ext>);
pub(crate) fn compact_string_type_id<T: 'static>() -> CompactString {
    format!("{:?}", TypeId::of::<T>()).to_compact_string()
}
#[derive(Component, Copy, Clone, Default)]
pub struct Disabled(pub(crate) bool);
impl Disabled {
    pub fn is_disabled(&self) -> bool {
        self.0
    }
    pub fn disabled() -> Self {
        Self(true)
    }
    pub fn not_disabled() -> Self {
        Self(false)
    }
    pub fn disable(&mut self) {
        self.0 = true;
    }
    pub fn enable(&mut self) {
        self.0 = false;
    }
}
#[repr(C)]
#[derive(
    Component, Copy, Clone, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize, Default,
)]
pub struct Style(pub(crate) f32);
impl Style {
    pub fn fill() -> Self {
        Self(0.0)
    }
    pub fn ring() -> Self {
        Self(1.0)
    }
    pub fn is_fill(&self) -> bool {
        self.0 == 0.0
    }
}
#[derive(Component, Clone)]
pub struct WebElementRemoval(pub String);
impl WebElementRemoval {
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        Self(s.as_ref().to_string())
    }
}
pub(crate) fn remove_web_element(query: Query<(Entity, &WebElementRemoval)>, mut cmd: Commands) {
    for (entity, _removal) in query.iter() {
        #[cfg(target_family = "wasm")]
        {
            use wasm_bindgen::JsCast;
            let document = web_sys::window().unwrap().document().unwrap();
            if let Some(elem) = document.get_element_by_id(_removal.0.as_str()) {
                elem.dyn_into::<web_sys::HtmlElement>().unwrap().remove();
            }
        }
        cmd.entity(entity).despawn();
    }
}