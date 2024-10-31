use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

use bevy_ecs::prelude::{Component, Entity, With};
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Query, ResMut, Resource};

use crate::ash::Render;
use crate::leaf::Visibility;

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

#[derive(Component, Clone)]
pub struct Differential<D: Component + PartialEq + Clone> {
    added: bool,
    _phantom: PhantomData<D>,
}
impl<D: Component + PartialEq + Clone> Default for Differential<D> {
    fn default() -> Self {
        Self::new()
    }
}
impl<D: Component + PartialEq + Clone> Differential<D> {
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
pub(crate) fn visibility_changed<D: Component + PartialEq + Clone + Send + Sync + 'static>(
    components: Query<
        (Entity, &RenderLink, &D, &Visibility),
        (Changed<Visibility>, With<Differential<D>>),
    >,
    mut render_queue: ResMut<RenderAddQueue<D>>,
) {
    for (entity, link, d, visibility) in components.iter() {
        if visibility.visible() {
            render_queue
                .cache
                .get_mut(link)
                .unwrap()
                .insert(entity, d.clone());
            render_queue
                .queue
                .get_mut(link)
                .expect("render-queue")
                .insert(entity, d.clone());
        }
    }
}
pub(crate) fn differential<D: Component + PartialEq + Clone + Send + Sync + 'static>(
    mut components: Query<(Entity, &RenderLink, &D, &mut Differential<D>, &Visibility), Changed<D>>,
    mut render_queue: ResMut<RenderAddQueue<D>>,
) {
    for (entity, link, d, mut local_cache, visibility) in components.iter_mut() {
        if visibility.visible() {
            let different = if render_queue.cache.get(link).unwrap().get(&entity).is_none()
                || local_cache.added
            {
                true
            } else {
                render_queue.cache.get(link).unwrap().get(&entity).unwrap() != d
            };
            local_cache.added = false;
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
