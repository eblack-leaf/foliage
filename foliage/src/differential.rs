use crate::r_ash::render::RenderId;
use crate::r_ash::render_packet::RenderPacketForwarder;
use crate::r_ash::render_packet::RenderPacketStore;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Bundle, Component, Or, Query};
use bevy_ecs::query::Changed;
use bevy_ecs::system::ResMut;
use compact_str::{CompactString, ToCompactString};
use serde::{Deserialize, Serialize};
use std::any::TypeId;

#[derive(Component, Clone)]
pub struct Differential<T: Component + Clone + PartialEq + Send + Sync + 'static> {
    cache: T,
    differential: Option<T>,
}
impl<T: Component + Clone + PartialEq + Send + Sync + 'static> Differential<T> {
    #[allow(unused)]
    pub fn new(t: T) -> Self {
        Self {
            cache: t,
            differential: None,
        }
    }
    pub(crate) fn updated(&mut self, t: &T) -> bool {
        if t != &self.cache {
            self.differential.replace(t.clone());
            self.cache = t.clone();
            return true;
        }
        false
    }
    pub(crate) fn differential(&mut self) -> Option<T> {
        self.differential.take()
    }
}
#[derive(Component, Default, Copy, Clone)]
pub struct DifferentialDisable(bool);
impl DifferentialDisable {
    pub(crate) fn disable(&mut self) {
        self.0 = true;
    }
    pub fn is_disabled(&self) -> bool {
        self.0
    }
    pub(crate) fn clear(&mut self) {
        self.0 = false;
    }
}
#[derive(Hash, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct DifferentialId(pub(crate) CompactString);
pub trait DifferentialIdentification {
    fn id() -> DifferentialId;
}
impl<T: Component> DifferentialIdentification for T {
    fn id() -> DifferentialId {
        DifferentialId(format!("{:?}", TypeId::of::<T>()).to_compact_string())
    }
}
pub(crate) fn differential<
    T: Component
        + Clone
        + PartialEq
        + Send
        + Sync
        + 'static
        + DifferentialIdentification
        + Serialize
        + for<'a> Deserialize<'a>,
>(
    mut query: Query<(&T, &mut Differential<T>, &mut RenderPacketStore), Changed<T>>,
) {
    for (t, mut diff, mut render_packet_store) in query.iter_mut() {
        if diff.updated(t) {
            render_packet_store.put(diff.differential().take().unwrap());
        }
    }
}
pub(crate) fn send_render_packet(
    mut query: Query<
        (
            Entity,
            &mut RenderPacketStore,
            &RenderId,
            &DifferentialDisable,
            &Despawn,
        ),
        Or<(
            Changed<DifferentialDisable>,
            Changed<RenderPacketStore>,
            Changed<Despawn>,
        )>,
    >,
    mut render_packet_forwarder: ResMut<RenderPacketForwarder>,
) {
    for (entity, mut packet, id, disable, despawn) in query.iter_mut() {
        if disable.is_disabled() || despawn.should_despawn() {
            render_packet_forwarder.remove(id, entity);
        } else {
            // need to forward Layer.z when enable sorting by z in renderer packages
            render_packet_forwarder.forward_packet(id, entity, packet.retrieve());
        }
    }
}
#[derive(Bundle, Clone)]
pub struct DifferentialBundle<T: Component + Clone + PartialEq + Send + Sync + 'static> {
    pub component: T,
    pub differential: Differential<T>,
}
impl<T: Component + Clone + PartialEq + Send + Sync + 'static> DifferentialBundle<T> {
    #[allow(unused)]
    pub fn new(t: T) -> Self {
        Self {
            component: t.clone(),
            differential: Differential::new(t),
        }
    }
}
#[derive(Component, Copy, Clone, Serialize, Deserialize, Default)]
pub struct Despawn(bool);
impl Despawn {
    pub fn despawn(&mut self) {
        self.0 = true;
    }
    pub fn should_despawn(&self) -> bool {
        self.0
    }
}
