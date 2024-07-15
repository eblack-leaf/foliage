use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

use bevy_ecs::bundle::Bundle;
use bevy_ecs::prelude::{Component, Entity, With};
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Commands, Query, ResMut, Resource};

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
    added: bool,
    _phantom: PhantomData<D>,
}

impl<D: Component + PartialEq + Clone> DifferentialCache<D> {
    pub(crate) fn new() -> Self {
        Self {
            added: true,
            _phantom: PhantomData,
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
pub(crate) fn added_invalidate<D: Component + PartialEq + Clone + Send + Sync + 'static>(
    mut added: Query<
        (Entity, &RenderLink, &D, &mut DifferentialCache<D>, &Remove),
        (Changed<D>, With<DifferentialCache<D>>),
    >,
    mut render_queue: ResMut<RenderAddQueue<D>>,
) {
    for (entity, link, d, mut local_cache, remove) in added.iter_mut() {
        if local_cache.added && remove.should_keep() {
            render_queue
                .cache
                .get_mut(link)
                .unwrap()
                .insert(entity, d.clone());
            render_queue
                .queue
                .get_mut(link)
                .unwrap()
                .insert(entity, d.clone());
            local_cache.added = false;
        }
    }
}
pub(crate) fn differential<D: Component + PartialEq + Clone + Send + Sync + 'static>(
    components: Query<(Entity, &RenderLink, &D, &Remove), (Changed<D>, With<DifferentialCache<D>>)>,
    mut render_queue: ResMut<RenderAddQueue<D>>,
) {
    for (entity, link, d, remove) in components.iter() {
        if remove.should_keep() {
            let different = if render_queue.cache.get(link).unwrap().get(&entity).is_none() {
                true
            } else if render_queue.cache.get(link).unwrap().get(&entity).unwrap() != d {
                true
            } else {
                false
            };
            render_queue
                .cache
                .get_mut(link)
                .unwrap()
                .insert(entity, d.clone());
            if different {
                render_queue
                    .queue
                    .get_mut(link)
                    .expect("render-queue")
                    .insert(entity, d.clone());
            }
        }
    }
}
#[derive(Component, Copy, Clone)]
pub struct Remove {
    should_remove: bool,
}
impl Remove {
    pub fn should_keep(&self) -> bool {
        !self.should_remove
    }
    pub fn should_remove(&self) -> bool {
        self.should_remove
    }
    pub fn remove() -> Self {
        Self {
            should_remove: true,
        }
    }
    pub fn keep() -> Self {
        Self {
            should_remove: false,
        }
    }
}
// TODO include root + dependent clean up here or let read if remove-true
pub(crate) fn remove(
    removals: Query<(Entity, &Remove, Option<&RenderLink>), Changed<Remove>>,
    mut cmd: Commands,
    mut remove_queue: ResMut<RenderRemoveQueue>,
) {
    for (entity, remove, opt_link) in removals.iter() {
        if remove.should_remove() {
            if let Some(link) = opt_link {
                remove_queue.queue.get_mut(&link).unwrap().insert(entity);
            }
            cmd.entity(entity).despawn();
        }
    }
}
