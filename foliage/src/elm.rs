use std::any::TypeId;
use std::marker::PhantomData;

use anymap::AnyMap;
use bevy_ecs::prelude::{apply_deferred, Bundle, Component, IntoSystemConfigs, SystemSet};
use bevy_ecs::schedule::IntoSystemSetConfigs;
use compact_str::{CompactString, ToCompactString};
use serde::{Deserialize, Serialize};

use crate::ash::render_packet::RenderPacketForwarder;
use crate::ash::render_packet::RenderPacketPackage;
use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, InterfaceContext};
use crate::ginkgo::viewport::ViewportHandle;
use crate::job::Job;
use crate::scene::bind::SceneNode;
use crate::window::ScaleFactor;

pub struct Elm {
    initialized: bool,
    pub job: Job,
    limiters: AnyMap,
}

#[macro_export]
macro_rules! differential_enable {
    ($elm:ident $(,$typename:ty)+) => {
        $($elm.enable_differential::<$typename>();)+
    };
}
struct DifferentialLimiter<T>(bool, PhantomData<T>);
struct SceneBindLimiter<T>(bool, PhantomData<T>);
impl<T> Default for DifferentialLimiter<T> {
    fn default() -> Self {
        DifferentialLimiter(false, PhantomData)
    }
}
impl<T> Default for SceneBindLimiter<T> {
    fn default() -> Self {
        SceneBindLimiter(false, PhantomData)
    }
}

impl Elm {
    pub(crate) fn new() -> Self {
        Self {
            initialized: false,
            job: Job::new(),
            limiters: AnyMap::new(),
        }
    }
    pub(crate) fn set_scale_factor(&mut self, factor: CoordinateUnit) {
        self.job.container.insert_resource(ScaleFactor(factor));
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
        self.job.main().configure_sets(
            (
                SystemSets::Spawn,
                SystemSets::SceneBinding,
                SystemSets::Resolve,
                SystemSets::ScenePlacement,
                SystemSets::FinalizeCoordinate,
                SystemSets::Differential,
                SystemSets::RenderPacket,
            )
                .chain(),
        );
        self.job.main().add_systems((
            crate::scene::align::place.in_set(SystemSets::ScenePlacement),
            crate::scene::align::place_layer.in_set(SystemSets::ScenePlacement),
            crate::differential::send_render_packet.in_set(SystemSets::RenderPacket),
            crate::differential::despawn
                .in_set(SystemSets::RenderPacket)
                .after(crate::differential::send_render_packet),
            apply_deferred
                .after(SystemSets::Spawn)
                .before(SystemSets::Resolve),
        ));
        self.enable_differential::<Layer>();
        self.job
            .container
            .insert_resource(RenderPacketForwarder::default());
        for leaf in leaflets {
            leaf.0(self)
        }
        self.initialized = true;
    }
    pub fn enable_scene_bind<T: SceneNode>(&mut self) {
        if self.limiters.get::<SceneBindLimiter<T>>().is_none() {
            self.limiters.insert(SceneBindLimiter::<T>::default());
        }
        if !self.limiters.get::<SceneBindLimiter<T>>().unwrap().0 {
            let set = if T::IS_SCENE { SystemSets::SubSceneBinding } else { SystemSets::SceneBinding };
            self.job
                .main()
                .add_systems((crate::scene::bind::Binder::<T, T::IS_SCENE>::bind.in_set(set),));
            self.limiters
                .get_mut::<SceneBindLimiter<T>>()
                .as_mut()
                .unwrap()
                .0 = true;
        }
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
            .section
            .area = area;
    }
    pub fn enable_differential<
        T: Component + Clone + PartialEq + Serialize + for<'a> Deserialize<'a>,
    >(
        &mut self,
    ) {
        if self.limiters.get::<DifferentialLimiter<T>>().is_none() {
            self.limiters.insert(DifferentialLimiter::<T>::default());
        }
        if !self.limiters.get::<DifferentialLimiter<T>>().unwrap().0 {
            self.job.main().add_systems((
                crate::differential::differential::<T>.in_set(SystemSets::Differential),
                crate::differential::send_on_differential_disable_changed::<T>
                    .in_set(SystemSets::Differential),
            ));
            self.limiters
                .get_mut::<DifferentialLimiter<T>>()
                .as_mut()
                .unwrap()
                .0 = true;
        }
    }
    pub(crate) fn finish_initialization(&mut self) {
        self.job.resume();
        self.initialized = true;
    }
}

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum SystemSets {
    Spawn,
    SceneBinding,
    Resolve,
    ScenePlacement,
    FinalizeCoordinate,
    Differential,
    RenderPacket,
    SubSceneBinding,
}

pub(crate) fn compact_string_type_id<T: 'static>() -> CompactString {
    format!("{:?}", TypeId::of::<T>()).to_compact_string()
}

pub trait Leaf {
    fn attach(elm: &mut Elm);
}

pub(crate) struct Leaflet(pub(crate) Box<fn(&mut Elm)>);

impl Leaflet {
    pub(crate) fn leaf_fn<T: Leaf>() -> Self {
        Self(Box::new(T::attach))
    }
}