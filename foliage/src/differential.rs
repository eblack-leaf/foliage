use std::any::TypeId;
use std::fmt::Debug;

use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Bundle, Commands, Component, Or, Query};
use bevy_ecs::query::Changed;
use bevy_ecs::system::ResMut;
use compact_str::{CompactString, ToCompactString};
use serde::{Deserialize, Serialize};

use crate::ash::identification::{RenderId, RenderIdentification};
use crate::ash::render::Render;
use crate::ash::render_packet::RenderPacketForwarder;
use crate::ash::render_packet::RenderPacketStore;
use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::layer::Layer;
use crate::coordinate::position::{CReprPosition, Position};
use crate::coordinate::InterfaceContext;
use crate::ginkgo::viewport::ViewportHandle;

#[derive(Bundle, Clone)]
pub struct Differentiable {
    position: Position<InterfaceContext>,
    area: Area<InterfaceContext>,
    layer: DifferentialBundle<Layer>,
    disable: DifferentialDisable,
    despawn: Despawn,
    store: RenderPacketStore,
    render_id: RenderId,
    c_pos: DifferentialBundle<CReprPosition>,
    c_area: DifferentialBundle<CReprArea>,
}

impl Differentiable {
    pub fn new<T: Render + 'static>(
        position: Position<InterfaceContext>,
        area: Area<InterfaceContext>,
        layer: Layer,
    ) -> Self {
        Self {
            position,
            c_pos: DifferentialBundle::new(CReprPosition::default()),
            c_area: DifferentialBundle::new(CReprArea::default()),
            layer: DifferentialBundle::new(layer),
            despawn: Despawn::default(),
            disable: DifferentialDisable::default(),
            store: RenderPacketStore::default(),
            render_id: T::render_id(),
            area,
        }
    }
}

#[derive(Component, Clone)]
pub struct Differential<T: Component + Clone + PartialEq + Send + Sync + 'static> {
    cache: T,
    differential: Option<T>,
    set_from_cache: bool,
}

impl<T: Component + Clone + PartialEq + Send + Sync + 'static> Differential<T> {
    #[allow(unused)]
    pub(crate) fn new(t: T) -> Self {
        Self {
            cache: t,
            differential: None,
            set_from_cache: false,
        }
    }
    pub(crate) fn updated(&mut self, t: &T) -> bool {
        if t != &self.cache {
            self.differential.replace(t.clone());
            self.cache = t.clone();
            return true;
        } else if self.set_from_cache {
            self.differential.replace(t.clone());
            self.cache = t.clone();
            self.set_from_cache = false;
            return true;
        }
        false
    }
    pub fn push_cached(&mut self) {
        self.set_from_cache = true;
    }
    pub(crate) fn differential(&mut self) -> Option<T> {
        self.differential.take()
    }
}

#[derive(Component, Default, Copy, Clone)]
pub struct DifferentialDisable(bool);

impl DifferentialDisable {
    #[allow(unused)]
    pub(crate) fn disable(&mut self) {
        self.0 = true;
    }
    pub fn is_disabled(&self) -> bool {
        self.0
    }
    #[allow(unused)]
    pub(crate) fn clear(&mut self) {
        self.0 = false;
    }
}

#[derive(Hash, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct DifferentialId(pub(crate) CompactString);

pub trait DifferentialIdentification {
    fn diff_id() -> DifferentialId;
}

impl<T: 'static> DifferentialIdentification for T {
    fn diff_id() -> DifferentialId {
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
    mut query: Query<
        (Entity, &T, &mut Differential<T>, &mut RenderPacketStore),
        Or<(Changed<T>, Changed<Differential<T>>)>,
    >,
) {
    for (entity, t, mut diff, mut render_packet_store) in query.iter_mut() {
        if diff.updated(t) {
            tracing::trace!("differential-updated: {:?}", entity);
            render_packet_store.put(diff.differential().take().unwrap());
        }
    }
}

pub(crate) fn send_on_differential_disable_changed<
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
    mut query: Query<
        (
            &T,
            &mut Differential<T>,
            &mut RenderPacketStore,
            &DifferentialDisable,
        ),
        Changed<DifferentialDisable>,
    >,
) {
    for (t, mut diff, mut render_packet_store, disable) in query.iter_mut() {
        if !disable.is_disabled() {
            diff.cache = t.clone();
            render_packet_store.put(t.clone());
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
            tracing::trace!("removing render-packet: {:?}", entity);
            render_packet_forwarder.remove(id, entity);
        } else {
            tracing::trace!("forwarding render-packet: {:?}", entity);
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
    pub fn signal_despawn() -> Self {
        Self(true)
    }
}
pub(crate) fn despawn(
    despawned: Query<(Entity, &Despawn), Changed<Despawn>>,
    mut viewport_handle: ResMut<ViewportHandle>,
    mut cmd: Commands,
) {
    if viewport_handle.area_updated() {
        viewport_handle.area_updated = false;
    }
    for (entity, despawn) in despawned.iter() {
        if despawn.should_despawn() {
            cmd.entity(entity).despawn();
        }
    }
}
