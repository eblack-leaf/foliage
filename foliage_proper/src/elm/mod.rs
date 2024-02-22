pub mod config;
pub mod leaf;

use std::any::TypeId;
use std::marker::PhantomData;

use crate::animate::trigger::Trigger;
use crate::ash::render_packet::RenderPacketForwarder;
use crate::ash::render_packet::RenderPacketPackage;
use crate::asset::{AssetContainer, AssetFetchFn, AssetKey, OnFetch};
use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::layer::Layer;
use crate::coordinate::position::{CReprPosition, Position};
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, InterfaceContext};
use crate::elm::config::{CoreSet, ElmConfiguration, ExternalSet};
use crate::ginkgo::viewport::ViewportHandle;
use crate::job::{Container, Job, Task};
use crate::scene::{Binder, Scene};
use crate::tree::{
    conditional_extension, conditional_scene_spawn, conditional_spawn, sprout, Forest, Navigation,
    Seed,
};
use crate::window::ScaleFactor;
#[cfg(target_family = "wasm")]
use crate::Workflow;
use anymap::AnyMap;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::event::{event_update_system, Event, Events};
use bevy_ecs::prelude::{Component, DetectChanges, IntoSystemConfigs, Res};
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Commands, Query, ResMut, StaticSystemParam, SystemParam};
use bytemuck::{Pod, Zeroable};
use compact_str::{CompactString, ToCompactString};
use leaf::Leaflet;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
        self.job.container.insert_resource(ScaleFactor::new(factor));
    }
    pub(crate) fn viewport_handle_changes(&mut self) -> Option<Position<InterfaceContext>> {
        self.job
            .container
            .get_resource_mut::<ViewportHandle>()
            .unwrap()
            .changes()
    }
    pub(crate) fn render_packet_package(&mut self) -> RenderPacketPackage {
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
        for leaf in leaflets {
            leaf.1(self)
        }
        self.initialized = true;
    }
    pub(crate) fn attach_viewport_handle(&mut self, area: Area<InterfaceContext>) {
        self.job
            .container
            .insert_resource(ViewportHandle::new(Section::default().with_area(area)));
    }
    pub(crate) fn set_viewport_handle_area(&mut self, area: Area<InterfaceContext>) {
        self.job
            .container
            .get_resource_mut::<ViewportHandle>()
            .unwrap()
            .adjust_area(area);
    }
    pub fn add_event<E: Event>(&mut self, stage: EventStage) {
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
            self.job.main().add_systems((
                crate::differential::differential::<T>.in_set(CoreSet::Differential),
                crate::differential::send_on_differential_disable_changed::<T>
                    .in_set(CoreSet::Differential),
            ));
            self.limiters.insert(DifferentialLimiter::<T>::default());
        }
    }
    pub(crate) fn finish_initialization(&mut self) {
        self.job.exec_startup();
        self.job.resume();
        self.initialized = true;
    }
    pub fn remove_web_element(_id: &'static str) {
        #[cfg(target_family = "wasm")]
        {
            use wasm_bindgen::JsCast;
            let document = web_sys::window().unwrap().document().unwrap();
            if let Some(elem) = document.get_element_by_id(_id) {
                elem.dyn_into::<web_sys::HtmlElement>().unwrap().remove();
            }
        }
    }
    pub fn send_event<E: Event>(&mut self, e: E) {
        self.container().send_event(e);
    }
    #[cfg(target_family = "wasm")]
    pub fn load_remote_asset<W: Workflow + Default + Send + Sync + 'static, S: AsRef<str>>(
        &mut self,
        path: S,
    ) -> AssetKey {
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
            .get_non_send_resource::<crate::WorkflowConnectionBase<W>>()
            .unwrap()
            .system_send(message);
        self.container()
            .get_resource_mut::<crate::AssetContainer>()
            .unwrap()
            .store(id, None);
        return id;
    }
    pub fn generate_asset_key(&self) -> AssetKey {
        Uuid::new_v4().as_u128()
    }
    pub fn store_local_asset(&mut self, id: AssetKey, bytes: Vec<u8>) {
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
                         mut ihs: Query<(&Trigger, &mut IH), Changed<Trigger>>| {
            for (trigger, mut ih) in ihs.iter_mut() {
                if trigger.active() {
                    handler(&mut ih, &mut ext);
                }
            }
        };
        self.main().add_systems(func.in_set(ExternalSet::Process));
    }
    pub fn view_trigger<H: Component + Send + 'static, C: Seed + Send + Sync + 'static>(&mut self) {
        self.add_interaction_handler::<H, Commands>(|_ih, mut ext| {
            Forest::navigate::<C>(&mut ext);
        });
    }
    pub fn enable_conditional<C: Bundle + Clone + Send + Sync + 'static>(&mut self) {
        self.main().add_systems((
            conditional_spawn::<C>.in_set(ExternalSet::BranchBind),
            conditional_extension::<C>.in_set(ExternalSet::BranchExt),
        ));
    }
    pub fn enable_conditional_scene<S: Scene + Clone + Send + Sync + 'static>(&mut self) {
        self.main()
            .add_systems(conditional_scene_spawn::<S>.in_set(ExternalSet::BranchBind));
    }
    pub fn enable_seed<S: Seed + Send + Sync + 'static>(&mut self) {
        self.main()
            .add_systems(sprout::<S>.in_set(ExternalSet::Sprout));
    }
    pub fn navigate<S: Seed + Send + Sync + 'static>(&mut self) {
        self.container().spawn(Navigation::<S>::new());
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
}
#[repr(C)]
#[derive(
    Component, Copy, Clone, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize, Default,
)]
pub struct ElementStyle(pub(crate) f32);
impl ElementStyle {
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