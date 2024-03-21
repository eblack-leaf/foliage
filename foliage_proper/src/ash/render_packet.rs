use std::collections::{HashMap, HashSet};

use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, Resource};
use serde::{Deserialize, Serialize};

use crate::ash::identification::{RenderId, RenderIdentification};
use crate::ash::render::Render;
use crate::differential::{DifferentialId, DifferentialIdentification};

pub type RenderPacketDifferential = Option<Vec<u8>>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RenderPacket(pub HashMap<DifferentialId, RenderPacketDifferential>);

impl Default for RenderPacket {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderPacket {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn get<T: DifferentialIdentification + for<'a> Deserialize<'a> + 'static>(
        &self,
    ) -> Option<T> {
        if let Some(Some(v)) = self.0.get(&T::diff_id()) {
            return rmp_serde::from_slice::<T>(v.as_slice()).ok();
        }
        None
    }
}

#[derive(Default, Component, Clone)]
pub struct RenderPacketStore {
    pub(crate) render_packet: Option<RenderPacket>,
}

impl RenderPacketStore {
    pub(crate) fn new() -> Self {
        Self {
            render_packet: None,
        }
    }
    pub(crate) fn retrieve(&mut self) -> RenderPacket {
        tracing::trace!("retrieving render-packet");
        let data = self.render_packet.take().unwrap();
        self.render_packet.replace(RenderPacket::new());
        data
    }
    pub(crate) fn put<T: DifferentialIdentification + Serialize + 'static>(&mut self, data: T) {
        if self.render_packet.is_none() {
            self.render_packet.replace(RenderPacket::new());
        }
        let serialized = rmp_serde::to_vec(&data).expect("serialization");
        self.render_packet
            .as_mut()
            .unwrap()
            .0
            .insert(T::diff_id(), Some(serialized));
    }
    pub fn get<T: DifferentialIdentification + for<'a> Deserialize<'a> + 'static>(
        &self,
    ) -> Option<T> {
        if let Some(packet) = self.render_packet.as_ref() {
            return packet.get::<T>();
        }
        None
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub(crate) struct RenderPacketSignature(pub(crate) RenderId, pub(crate) Entity);

#[derive(Resource, Default)]
pub(crate) struct RenderPacketForwarder {
    pub(crate) render_packets: HashMap<RenderPacketSignature, RenderPacket>,
    pub(crate) removals: HashMap<RenderId, HashSet<Entity>>,
    pub(crate) removal_queue: HashMap<RenderId, HashSet<Entity>>,
    pub(crate) orphaned: HashSet<Entity>,
}

impl RenderPacketForwarder {
    pub(crate) fn forward_packet(&mut self, id: &RenderId, entity: Entity, packet: RenderPacket) {
        tracing::trace!("forwarding-packet");
        self.render_packets
            .insert(RenderPacketSignature(id.clone(), entity), packet);
        if let Some(rems) = self.removals.get_mut(id) {
            rems.remove(&entity);
        }
    }
    pub(crate) fn remove(&mut self, id: &RenderId, entity: Entity) {
        tracing::trace!("removing-packet");
        if self.removal_queue.get(id).is_none() {
            self.removal_queue.insert(id.clone(), HashSet::new());
        }
        if let Some(set) = self.removal_queue.get_mut(id) {
            set.insert(entity);
        }
        self.render_packets
            .remove(&RenderPacketSignature(id.clone(), entity));
    }
    pub(crate) fn package_for_transit(&mut self) -> RenderPacketPackage {
        tracing::trace!("packaging render-packets");
        let mut package = RenderPacketPackage::default();
        for (signature, packet) in self.render_packets.drain() {
            if package.0.get(&signature.0).is_none() {
                package
                    .0
                    .insert(signature.0.clone(), RenderPacketQueue::new());
            }
            package
                .0
                .get_mut(&signature.0)
                .unwrap()
                .0
                .insert(signature.1, packet);
        }
        for (id, mut removal) in self.removal_queue.drain() {
            if !self.removals.contains_key(&id) {
                self.removals.insert(id.clone(), HashSet::new());
            }
            for rem in removal.drain() {
                if !self.removals.get(&id).unwrap().contains(&rem) {
                    self.removals.get_mut(&id).unwrap().insert(rem);
                    if package.0.get(&id).is_none() {
                        package.0.insert(id.clone(), RenderPacketQueue::new());
                    }
                    package.0.get_mut(&id).unwrap().1.push(rem);
                }
            }
        }
        package.1.extend(self.orphaned.drain());
        package
    }
}

#[derive(Serialize, Deserialize, Default)]
pub(crate) struct RenderPacketPackage(
    pub(crate) HashMap<RenderId, RenderPacketQueue>,
    pub(crate) HashSet<Entity>,
);

impl RenderPacketPackage {
    pub(crate) fn obtain<T: Render + 'static>(&mut self) -> Option<RenderPacketQueue> {
        self.establish::<T>()
    }
    pub(crate) fn establish<T: Render + 'static>(&mut self) -> Option<RenderPacketQueue> {
        self.0.insert(T::render_id(), RenderPacketQueue::new())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct RenderPacketQueue(pub HashMap<Entity, RenderPacket>, pub Vec<Entity>);

impl RenderPacketQueue {
    pub(crate) fn new() -> Self {
        Self(HashMap::new(), vec![])
    }
    pub(crate) fn retrieve_removals(&mut self) -> Vec<Entity> {
        self.1.drain(..).collect()
    }
    pub(crate) fn retrieve_packet(&mut self, entity: Entity) -> Option<RenderPacket> {
        self.0.remove(&entity)
    }
}