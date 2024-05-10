use crate::ash::Render;
use crate::elm::Elm;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::prelude::{Component, Entity};
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Query, ResMut, Resource};
use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

#[derive(Component, Clone, Eq, PartialEq, Hash, Copy)]
pub struct RenderLink(TypeId);
impl RenderLink {
    pub fn new<R: Render>() -> Self {
        Self(TypeId::of::<R>())
    }
}
#[derive(Resource)]
pub struct RenderAddQueue<D: Component> {
    pub queue: HashMap<RenderLink, HashMap<Entity, D>>,
}
impl<D: Component> Default for RenderAddQueue<D> {
    fn default() -> Self {
        Self {
            queue: HashMap::new(),
        }
    }
}
#[derive(Resource, Default)]
pub struct RenderRemoveQueue {
    pub queue: HashMap<RenderLink, HashSet<Entity>>,
}
#[derive(Bundle)]
pub struct Differentiable<D: Component + PartialEq + Clone> {
    pub component: D,
    pub diff: CachedDifferential<D>,
}
#[derive(Component)]
pub struct CachedDifferential<D: Component + PartialEq + Clone> {
    pub(crate) last: Option<D>,
}
pub struct RenderPacket<D: Component + PartialEq + Clone> {
    pub entity: Entity,
    pub value: D,
}
impl<D: Component + PartialEq + Clone> From<(Entity, D)> for RenderPacket<D> {
    fn from(value: (Entity, D)) -> Self {
        Self {
            entity: value.0,
            value: value.1,
        }
    }
}
pub(crate) fn differential<D: Component + PartialEq + Clone + Send + Sync + 'static>(
    mut components: Query<(Entity, &RenderLink, &D, &mut CachedDifferential<D>), Changed<D>>,
    mut render_queue: ResMut<RenderAddQueue<D>>,
) {
    for (entity, link, d, mut cache) in components.iter_mut() {
        let different = if cache.last.is_none() {
            true
        } else if cache.last.as_ref().unwrap() != d {
            true
        } else {
            false
        };
        if different {
            render_queue
                .queue
                .get_mut(link)
                .expect("render-queue")
                .insert(entity, d.clone());
        }
    }
}
#[derive(Resource)]
pub(crate) struct DifferentialScheduleLimiter<D>(PhantomData<D>);
impl<D> Default for DifferentialScheduleLimiter<D> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
pub(crate) fn enable_differential<R: Render, D: Component + PartialEq + Clone>(elm: &mut Elm) {
    if !elm
        .ecs
        .world
        .contains_resource::<DifferentialScheduleLimiter<D>>()
    {
        elm.scheduler.main.add_systems((differential::<D>,));
        elm.ecs
            .world
            .insert_resource(DifferentialScheduleLimiter::<D>::default())
    }
    if !elm.ecs.world.contains_resource::<RenderAddQueue<D>>() {
        elm.ecs
            .world
            .insert_resource(RenderAddQueue::<D>::default());
    }
    let link = RenderLink::new::<R>();
    elm.ecs
        .world
        .get_resource_mut::<RenderAddQueue<D>>()
        .unwrap()
        .queue
        .insert(link, HashMap::new());
    elm.ecs
        .world
        .get_resource_mut::<RenderRemoveQueue>()
        .unwrap()
        .queue
        .insert(link, HashSet::new());
}
