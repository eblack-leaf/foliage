use crate::ash::tag::RenderTag;
use crate::differential::{Differentiable, DifferentialTag};
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Resource)]
pub(crate) struct RenderPacketManager {
    pub(crate) packets: HashMap<RenderTag, Option<RenderPackets>>,
}

pub type RenderPackets = HashMap<Entity, RenderPacket>;

impl RenderPacketManager {
    pub(crate) fn new() -> Self {
        Self {
            packets: HashMap::new(),
        }
    }
    pub(crate) fn get(&mut self, tag: RenderTag) -> Option<RenderPackets> {
        if let Some(h) = self.packets.get_mut(&tag) {
            return h.take();
        }
        None
    }
}

#[derive(Component, Serialize, Deserialize)]
pub struct RenderPacket(pub Option<RenderPacketStorage>);

pub(crate) type RenderPacketStorage = HashMap<DifferentialTag, Option<Vec<u8>>>;

impl RenderPacket {
    pub(crate) fn new(packets: RenderPacketStorage) -> Self {
        Self(Some(packets))
    }
    pub(crate) fn insert<T: Component + Differentiable + Serialize>(&mut self, t: T) {
        let serialized = rmp_serde::to_vec(&t).expect("serialization");
        self.0.as_mut().unwrap().insert(T::id(), Some(serialized));
    }
    #[allow(unused)]
    pub(crate) fn get<T: Component + Differentiable + for<'a> Deserialize<'a>>(&self) -> Option<T> {
        if let Some(Some(v)) = self.0.as_ref().unwrap().get(&T::id()) {
            return rmp_serde::from_slice::<T>(v.as_slice()).ok();
        }
        None
    }
}
