use crate::differential::{DifferentialIdentification, DifferentialId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use bevy_ecs::prelude::{Component, Resource};
use bevy_ecs::entity::Entity;
use crate::r_ash::render::RenderId;
pub type RenderPacketDifferential = Option<Vec<u8>>;
pub type RenderPacket = HashMap<DifferentialId, RenderPacketDifferential>;
#[derive(Default, Component)]
pub struct RenderPacketStore {
    pub(crate) render_packet: Option<RenderPacket>,
}
impl RenderPacketStore {
    pub(crate) fn retrieve(&mut self) -> RenderPacket {
        let data = self.render_packet.take().unwrap();
        self.render_packet.replace(HashMap::new());
        data
    }
    pub(crate) fn put<T: DifferentialIdentification>(&mut self, data: T) {
        let serialized = rmp_serde::to_vec(&data).expect("serialization");
        self.render_packet.unwrap().insert(T::id(), Some(serialized));
    }
    pub fn get<T: DifferentialIdentification + for<'a> Deserialize<'a>>(&self) -> Option<T> {
        if let Some(Some(v)) = self.render_packet.unwrap().get(&T::id()) {
            return rmp_serde::from_slice::<T>(v.as_slice()).ok();
        }
        None
    }
}
#[derive(Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub(crate) struct RenderPacketSignature(pub(crate) RenderId, pub(crate) Entity);
#[derive(Resource)]
pub(crate) struct RenderPacketForwarder {
    pub(crate) render_packets: HashMap<RenderPacketSignature, RenderPacket>,
    pub(crate) removals: HashMap<RenderId, Vec<Entity>>,
}
impl RenderPacketForwarder {
    pub(crate) fn forward_packet(&mut self, id: &RenderId, entity: Entity, packet: RenderPacket) {
        todo!()
    }
    pub(crate) fn remove(&mut self, id: &RenderId, entity: Entity) {
        todo!()
    }
    pub(crate) fn exchange(&mut self) -> RenderPacketPackage {
        todo!()
    }
}
pub(crate) type PackagedRenderPacket = (RenderPacketSignature, RenderPacket);
pub(crate) type PackagedRemoval = (RenderId, Vec<Entity>);
#[derive(Serialize, Deserialize)]
pub(crate) struct RenderPacketPackage {
    pub(crate) packets: Vec<PackagedRenderPacket>,
    pub(crate) removals: Vec<PackagedRemoval>,
}
impl RenderPacketPackage {
    pub(crate) fn new(packets: Vec<PackagedRenderPacket>, removals: Vec<PackagedRemoval>) -> Self {
        Self {
            packets,
            removals,
        }
    }
}
