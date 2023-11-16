use crate::job::Job;
use crate::Leaflet;
use anymap::AnyMap;
use bevy_ecs::prelude::{Component, IntoSystemConfigs, SystemSet};
use bevy_ecs::schedule::IntoSystemSetConfigs;
use compact_str::{CompactString, ToCompactString};
use serde::{Deserialize, Serialize};
use std::any::TypeId;
use std::marker::PhantomData;
use crate::r_ash::render_packet::RenderPacketForwarder;

pub struct Elm {
    initialized: bool,
    pub job: Job,
    differential_limiter: AnyMap,
}
struct DifferentialLimiter<T>(bool, PhantomData<T>);
impl<T> Default for DifferentialLimiter<T> {
    fn default() -> Self {
        DifferentialLimiter(false, PhantomData)
    }
}
impl Elm {
    pub(crate) fn new() -> Self {
        Self {
            initialized: false,
            job: Job::new(),
            differential_limiter: AnyMap::new(),
        }
    }
    pub(crate) fn initialized(&self) -> bool {
        self.initialized
    }
    pub(crate) fn attach_leafs(&mut self, leaflets: Vec<Leaflet>) {
        self.job
            .main()
            .configure_sets((SystemSets::Differential, SystemSets::RenderPacket).chain());
        self.job.main().add_systems((
            crate::differential::send_render_packet.in_set(SystemSets::RenderPacket),
        ));
        self.job.container.insert_resource(RenderPacketForwarder::default());
        for leaf in leaflets {
            leaf.0(self)
        }
        self.initialized = true;
    }
    pub fn enable_differential<
        T: Component + Clone + PartialEq + Serialize + for<'a> Deserialize<'a>,
    >(
        &mut self,
    ) {
        if self
            .differential_limiter
            .get::<DifferentialLimiter<T>>()
            .is_none()
        {
            self.differential_limiter
                .insert(DifferentialLimiter::<T>::default())
        }
        if !self
            .differential_limiter
            .get::<DifferentialLimiter<T>>()
            .unwrap()
            .0
        {
            self.job.main().add_systems((
                crate::differential::differential::<T>.in_set(SystemSets::Differential),
            ));
            self.differential_limiter
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
    Differential,
    RenderPacket,
}

pub(crate) fn compact_string_type_id<T: 'static>() -> CompactString {
    format!("{:?}", TypeId::of::<T>()).to_compact_string()
}
