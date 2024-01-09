pub mod config;
pub mod leaf;

use std::any::TypeId;
use std::marker::PhantomData;

use anymap::AnyMap;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::event::{event_update_system, Event, Events};
use bevy_ecs::prelude::{Component, DetectChanges, IntoSystemConfigs, Res};
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Commands, Query, ResMut, StaticSystemParam, SystemParam};
use compact_str::{CompactString, ToCompactString};
use leaf::Leaflet;
use serde::{Deserialize, Serialize};

use crate::ash::render_packet::RenderPacketForwarder;
use crate::ash::render_packet::RenderPacketPackage;
use crate::compositor::segment::ResponsiveSegment;
use crate::compositor::{Compositor, CurrentView, Segmental, ViewHandle};
use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::layer::Layer;
use crate::coordinate::position::{CReprPosition, Position};
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, InterfaceContext};
use crate::elm::config::{CoreSet, ElmConfiguration, ExternalSet};
use crate::ginkgo::viewport::ViewportHandle;
use crate::interaction::InteractionListener;
use crate::job::{Job, Task};
use crate::scene::{Anchor, Scene, SceneCoordinator};
use crate::window::ScaleFactor;

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
#[derive(Bundle)]
pub struct BundleExtension<T: Bundle, S: Bundle> {
    pub original: T,
    pub extension: S,
}

impl<T: Bundle, S: Bundle> BundleExtension<T, S> {
    pub fn new(t: T, s: S) -> Self {
        Self {
            original: t,
            extension: s,
        }
    }
}

pub trait BundleExtend
where
    Self: Bundle + Sized,
{
    fn extend<E: Bundle>(self, handle: E) -> BundleExtension<Self, E>;
}

impl<I: Bundle> BundleExtend for I {
    fn extend<E: Bundle>(self, handle: E) -> BundleExtension<I, E> {
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
            .insert_resource(SceneCoordinator::default());
        self.job
            .container
            .insert_resource(RenderPacketForwarder::default());
        for leaf in leaflets {
            leaf.1(self)
        }
        self.job.exec_startup();
        self.initialized = true;
    }
    pub(crate) fn attach_viewport_handle(&mut self, area: Area<InterfaceContext>) {
        self.job
            .container
            .insert_resource(ViewportHandle::new(Section::default().with_area(area)));
        self.job.container.insert_resource(Compositor::new(area));
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
        self.job.resume();
        self.initialized = true;
    }
    pub fn add_view_binding<
        VH: Into<ViewHandle>,
        B: Bundle + Clone,
        Ext: Bundle + Clone,
        RS: Into<ResponsiveSegment>,
    >(
        &mut self,
        vh: VH,
        b: B,
        rs: RS,
        ext: Ext,
    ) {
        let view_handle = vh.into();
        let responsive_segment = rs.into().at_view(view_handle);
        let func = move |current: Res<CurrentView>,
                         mut cmd: Commands,
                         mut compositor: ResMut<Compositor>| {
            {
                if current.0 == view_handle {
                    let entity = cmd
                        .spawn(b.clone())
                        .insert(ext.clone())
                        .insert(Segmental::new(responsive_segment.clone()))
                        .id();
                    compositor.add_to_view(view_handle, entity);
                }
            }
        };
        self.main().add_systems((func
            .in_set(ExternalSet::ViewBindings)
            .run_if(|cv: Res<CurrentView>| -> bool { cv.is_changed() }),));
    }
    pub fn add_view_scene_binding<
        VH: Into<ViewHandle>,
        S: Scene,
        Ext: Bundle + Clone,
        RS: Into<ResponsiveSegment>,
    >(
        &mut self,
        vh: VH,
        args: S::Args<'static>,
        rs: RS,
        ext: Ext,
    ) {
        let view_handle = vh.into();
        let responsive_segment = rs.into().at_view(view_handle);
        let func = move |current: Res<CurrentView>,
                         mut cmd: Commands,
                         mut compositor: ResMut<Compositor>,
                         external_args: StaticSystemParam<S::ExternalArgs>,
                         mut coordinator: ResMut<SceneCoordinator>| {
            {
                if current.0 == view_handle {
                    let (_handle, entity) = coordinator.spawn_scene::<S>(
                        Anchor::default(),
                        &args,
                        &external_args,
                        &mut cmd,
                    );
                    cmd.entity(entity)
                        .insert(ext.clone())
                        .insert(Segmental::new(responsive_segment.clone()));
                    compositor.add_to_view(view_handle, entity);
                }
            }
        };
        self.main().add_systems((func
            .in_set(ExternalSet::ViewBindings)
            .run_if(|cv: Res<CurrentView>| -> bool { cv.is_changed() }),));
    }
    pub fn add_interaction_handler<IH: Component + 'static, Ext: SystemParam + 'static>(
        &mut self,
        trigger: InteractionHandlerTrigger,
        handler: fn(&StaticSystemParam<Ext>, &mut IH),
    ) {
        let func = move |ext: StaticSystemParam<Ext>,
                         mut ihs: Query<
            (&InteractionListener, &mut IH),
            Changed<InteractionListener>,
        >| {
            for (listener, mut ih) in ihs.iter_mut() {
                let should_run = match trigger {
                    InteractionHandlerTrigger::Active => listener.active(),
                    InteractionHandlerTrigger::EngagedStart => listener.engaged_start(),
                    InteractionHandlerTrigger::EngagedEnd => listener.engaged_end(),
                    InteractionHandlerTrigger::Engaged => listener.engaged(),
                };
                if should_run {
                    handler(&ext, &mut ih);
                }
            }
        };
        self.main().add_systems(func.in_set(ExternalSet::Process));
    }
}
pub enum InteractionHandlerTrigger {
    Active,
    Engaged,
    EngagedStart,
    EngagedEnd,
}
pub(crate) fn compact_string_type_id<T: 'static>() -> CompactString {
    format!("{:?}", TypeId::of::<T>()).to_compact_string()
}
#[derive(Component, Copy, Clone, Default)]
pub struct Disabled(pub(crate) bool);
impl Disabled {
    pub fn disabled(&self) -> bool {
        self.0
    }
    pub fn active() -> Self {
        Self(true)
    }
    pub fn inactive() -> Self {
        Self(false)
    }
}
#[derive(Component, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum ElementStyle {
    Ring,
    Fill,
}
