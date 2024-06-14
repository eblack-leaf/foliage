use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

use bevy_ecs::bundle::Bundle;
use bevy_ecs::prelude::{Component, Entity, With};
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Query, ResMut, Resource};

use crate::ash::Render;

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
    pub cache: HashMap<RenderLink, HashMap<Entity, D>>,
}
impl<D: Component> Default for RenderAddQueue<D> {
    fn default() -> Self {
        Self {
            queue: HashMap::new(),
            cache: Default::default(),
        }
    }
}

#[derive(Resource, Default)]
pub struct RenderRemoveQueue {
    pub queue: HashMap<RenderLink, HashSet<Entity>>,
}

#[derive(Bundle, Clone)]
pub struct Differential<D: Component + PartialEq + Clone> {
    pub component: D,
    pub cache: DifferentialCache<D>,
}

impl<D: Component + PartialEq + Clone> Differential<D> {
    pub fn new(d: D) -> Self {
        Self {
            component: d,
            cache: DifferentialCache::new(),
        }
    }
}

#[derive(Component, Clone)]
pub struct DifferentialCache<D: Component + PartialEq + Clone> {
    _phantom: PhantomData<D>
}

impl<D: Component + PartialEq + Clone> DifferentialCache<D> {
    pub(crate) fn new() -> Self {
        Self {
            _phantom: PhantomData
        }
    }
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
    mut components: Query<(Entity, &RenderLink, &D), (Changed<D>, With<DifferentialCache<D>>)>,
    mut render_queue: ResMut<RenderAddQueue<D>>,
) {
    for (entity, link, d) in components.iter_mut() {
        let different = if render_queue.cache.get(link).unwrap().get(&entity).is_none() {
            true
        } else if render_queue.cache.get(link).unwrap().get(&entity).unwrap() != d {
            true
        } else {
            false
        };
        render_queue.cache.get_mut(link).unwrap().insert(entity, d.clone());
        if different {
            render_queue
                .queue
                .get_mut(link)
                .expect("render-queue")
                .insert(entity, d.clone());
        }
    }
}
