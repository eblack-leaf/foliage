use crate::{Event, Resource};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Trigger;
use bevy_ecs::system::ResMut;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

#[derive(Event, Clone)]
pub struct RenderToken<R: Clone + Send + Sync + 'static, RT: Clone + Send + Sync + 'static> {
    pub token: RT,
    _phantom: PhantomData<R>,
}

impl<R: Clone + Send + Sync + 'static, RT: Clone + Send + Sync + 'static> RenderToken<R, RT> {
    pub(crate) fn new(token: RT) -> Self {
        Self {
            token,
            _phantom: Default::default(),
        }
    }
}
#[derive(Resource)]
pub struct RenderQueue<R: Clone + Send + Sync + 'static, RT: Clone + Send + Sync + 'static> {
    pub queue: HashMap<Entity, RenderToken<R, RT>>,
}
impl<R: Clone + Send + Sync + 'static, RT: Clone + Send + Sync + 'static> RenderQueue<R, RT> {
    pub fn new() -> Self {
        Self {
            queue: HashMap::new(),
        }
    }
    pub fn token_fetch(
        trigger: Trigger<RenderToken<R, RT>>,
        mut queue: ResMut<RenderQueue<R, RT>>,
    ) {
        queue
            .queue
            .insert(trigger.entity(), trigger.event().clone());
    }
}
#[derive(Resource)]
pub struct RenderRemoveQueue<R: Clone + Send + Sync + 'static> {
    pub queue: HashSet<Entity>,
    _phantom: PhantomData<R>,
}
impl<R: Clone + Send + Sync + 'static> RenderRemoveQueue<R> {
    pub fn new() -> Self {
        Self {
            queue: HashSet::new(),
            _phantom: Default::default(),
        }
    }
}
