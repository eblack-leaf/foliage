use crate::ash::{
    RenderPacket, RenderPacketManager, RenderPacketStorage, RenderPackets, RenderTag,
};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Bundle, Component, Query, Without};
use bevy_ecs::query::Changed;
use bevy_ecs::system::ResMut;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Component, Clone)]
pub struct Differential<T: Component + Clone + PartialEq + Send + Sync + 'static> {
    cache: T,
    differential: Option<T>,
}
impl<T: Component + Clone + PartialEq + Send + Sync + 'static> Differential<T> {
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
pub struct DifferentialDisable {}
#[derive(Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct DifferentialTag(pub(crate) Uuid);
pub(crate) trait Differentiable {
    fn id() -> DifferentialTag;
}
impl<T: Component> Differentiable for T {
    fn id() -> DifferentialTag {
        DifferentialTag(Uuid::new_v4())
    }
}
pub(crate) fn differential<
    T: Component
        + Clone
        + PartialEq
        + Send
        + Sync
        + 'static
        + Differentiable
        + Serialize
        + for<'a> Deserialize<'a>,
>(
    mut query: Query<
        (&T, &mut Differential<T>, &mut RenderPacket),
        (Changed<T>, Without<DifferentialDisable>),
    >,
) {
    for (t, mut diff, mut render_packet) in query.iter_mut() {
        if diff.updated(t) {
            render_packet.insert(diff.differential().take().unwrap());
        }
    }
}
pub(crate) fn send_render_packet(
    mut query: Query<
        (Entity, &mut RenderPacket, &RenderTag),
        (Without<DifferentialDisable>, Changed<RenderPacket>),
    >,
    mut render_packet_manager: ResMut<RenderPacketManager>,
) {
    for (entity, mut packet, tag) in query.iter_mut() {
        if let Some(valid_entity_render_packet) = packet.0.take() {
            if let Some(render_packet_slot) = render_packet_manager.packets.get_mut(tag) {
                if let Some(render_packets) = render_packet_slot {
                    render_packets.insert(entity, RenderPacket::new(valid_entity_render_packet));
                } else {
                    render_packet_slot.replace(RenderPackets::new());
                    render_packet_slot
                        .as_mut()
                        .unwrap()
                        .insert(entity, RenderPacket::new(valid_entity_render_packet));
                }
            }
        }
        packet.0.replace(RenderPacketStorage::new());
    }
}
#[derive(Bundle, Clone)]
pub struct DifferentialBundle<T: Component + Clone + PartialEq + Send + Sync + 'static> {
    pub component: T,
    pub differential: Differential<T>,
}
impl<T: Component + Clone + PartialEq + Send + Sync + 'static> DifferentialBundle<T> {
    pub fn new(t: T) -> Self {
        Self {
            component: t.clone(),
            differential: Differential::new(t),
        }
    }
}
