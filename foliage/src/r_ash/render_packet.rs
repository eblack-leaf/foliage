use crate::differential::{DifferentialId, DifferentialIdentification};
use crate::r_ash::render::{RenderId, RenderPacketPackage};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, Resource};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    pub(crate) fn put<T: DifferentialIdentification + Serialize + 'static>(&mut self, data: T) {
        let serialized = rmp_serde::to_vec(&data).expect("serialization");
        self.render_packet
            .as_mut()
            .unwrap()
            .insert(T::id(), Some(serialized));
    }
    pub fn get<T: DifferentialIdentification + for<'a> Deserialize<'a> + 'static>(
        &self,
    ) -> Option<T> {
        if let Some(Some(v)) = self.render_packet.as_ref().unwrap().get(&T::id()) {
            return rmp_serde::from_slice::<T>(v.as_slice()).ok();
        }
        None
    }
}
#[derive(Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub(crate) struct RenderPacketSignature(pub(crate) RenderId, pub(crate) Entity);
#[derive(Resource, Default)]
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
    pub(crate) fn package_for_transit(&mut self) -> RenderPacketPackage {
        let mut package = RenderPacketPackage::default();

        package
    }
}
