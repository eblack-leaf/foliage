use crate::{Component, ResolvedVisibility, Resource};
use bevy_ecs::change_detection::ResMut;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Changed, ParamSet, Query};
use bevy_ecs::world::World;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

#[derive(Component, Clone)]
pub(crate) struct Differential<
    R: Clone + Send + Sync + 'static,
    RP: Clone + Send + Sync + 'static + PartialEq,
> {
    pub(crate) cache: Option<RP>,
    _phantom: PhantomData<R>,
}

impl<R: Clone + Send + Sync + 'static, RP: Clone + Send + Sync + 'static + PartialEq>
    Differential<R, RP>
{
    pub(crate) fn new(cache: RP) -> Self {
        Self {
            cache: Some(cache),
            _phantom: Default::default(),
        }
    }
    fn blank() -> Self {
        Self {
            cache: None,
            _phantom: Default::default(),
        }
    }
    pub(crate) fn compare(&mut self, packet: RP) -> bool {
        todo!()
    }
}

impl<R: Clone + Send + Sync + 'static, RP: Clone + Send + Sync + 'static + PartialEq> Default
    for Differential<R, RP>
{
    fn default() -> Self {
        Self::blank()
    }
}

pub(crate) fn cached_differential<
    R: Clone + Send + Sync + 'static,
    RP: Clone + Send + Sync + 'static + Component + PartialEq,
>(
    values: Query<&RP, Changed<RP>>,
    mut caches: Query<&mut Differential<R, RP>>,
    visibility: ParamSet<(
        Query<&ResolvedVisibility>,
        Query<Entity, Changed<ResolvedVisibility>>,
    )>,
    mut queue: ResMut<RenderQueue<R, RP>>,
) {
    todo!()
}

#[derive(Resource)]
pub(crate) struct RenderQueue<R: Clone + Send + Sync + 'static, RP: Clone + Send + Sync + 'static> {
    pub(crate) queue: HashMap<Entity, RP>,
    _phantom: PhantomData<R>,
}

impl<R: Clone + Send + Sync + 'static, RP: Clone + Send + Sync + 'static> RenderQueue<R, RP> {
    pub(crate) fn new() -> Self {
        Self {
            queue: HashMap::new(),
            _phantom: Default::default(),
        }
    }
}

#[derive(Resource)]
pub(crate) struct RenderRemoveQueue<R: Clone + Send + Sync + 'static> {
    pub(crate) queue: HashSet<Entity>,
    _phantom: PhantomData<R>,
}

impl<R: Clone + Send + Sync + 'static> RenderRemoveQueue<R> {
    pub(crate) fn new() -> Self {
        Self {
            queue: HashSet::new(),
            _phantom: Default::default(),
        }
    }
}
pub(crate) struct Elm<'a> {
    pub(crate) world: &'a mut World,
}
impl<'a> Elm<'a> {
    pub(crate) fn new(world: &'a mut World) -> Self {
        Self { world }
    }
    pub(crate) fn removes<R: Clone + Send + Sync + 'static>(&mut self) -> Vec<Entity> {
        self.world
            .get_resource_mut::<RenderRemoveQueue<R>>()
            .unwrap()
            .queue
            .drain()
            .collect()
    }

    pub(crate) fn attribute<R: Clone + Send + Sync + 'static, RP: Clone + Send + Sync + 'static>(
        &mut self,
    ) -> Vec<(Entity, RP)> {
        self.world
            .get_resource_mut::<RenderQueue<R, RP>>()
            .unwrap()
            .queue
            .drain()
            .collect()
    }
}
