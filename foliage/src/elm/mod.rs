pub mod config;
pub mod leaf;

use std::any::TypeId;
use std::marker::PhantomData;

use anymap::AnyMap;
use bevy_ecs::prelude::{Component, IntoSystemConfigs};
use compact_str::{CompactString, ToCompactString};
use leaf::Leaflet;
use serde::{Deserialize, Serialize};

use crate::ash::render_packet::RenderPacketForwarder;
use crate::ash::render_packet::RenderPacketPackage;
use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::layer::Layer;
use crate::coordinate::position::{CReprPosition, Position};
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, InterfaceContext};
use crate::elm::config::{CoreSet, ElmConfiguration};
use crate::ginkgo::viewport::ViewportHandle;
use crate::job::{Job, Task};
use crate::scene::SceneCompositor;
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
struct DifferentialLimiter<T>(bool, PhantomData<T>);
impl<T> Default for DifferentialLimiter<T> {
    fn default() -> Self {
        DifferentialLimiter(false, PhantomData)
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
        ElmConfiguration::configure_elm(self, &leaflets);
        self.enable_differential::<CReprPosition>();
        self.enable_differential::<CReprArea>();
        self.enable_differential::<Layer>();
        self.job
            .container
            .insert_resource(SceneCompositor::default());
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
                crate::differential::differential::<T>.in_set(CoreSet::Differential),
                crate::differential::send_on_differential_disable_changed::<T>
                    .in_set(CoreSet::Differential),
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

pub(crate) fn compact_string_type_id<T: 'static>() -> CompactString {
    format!("{:?}", TypeId::of::<T>()).to_compact_string()
}
