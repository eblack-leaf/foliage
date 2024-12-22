use crate::{Component, ResolvedVisibility, Resource};
use bevy_ecs::change_detection::ResMut;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Changed, ParamSet, Query};
use std::marker::PhantomData;
use std::collections::{HashMap, HashSet};

#[derive(Component, Clone)]
pub(crate) struct Differential<
    R: Clone + Send + Sync + 'static,
    RT: Clone + Send + Sync + 'static + PartialEq,
> {
    pub(crate) cache: Option<RT>,
    _phantom: PhantomData<R>,
}

impl<R: Clone + Send + Sync + 'static, RT: Clone + Send + Sync + 'static + PartialEq>
Differential<R, RT>
{
    pub(crate) fn new(cache: RT) -> Self {
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
    pub(crate) fn compare(&mut self, token: RT) -> bool {
        todo!()
    }
}

impl<R: Clone + Send + Sync + 'static, RT: Clone + Send + Sync + 'static + PartialEq> Default
for Differential<R, RT>
{
    fn default() -> Self {
        Self::blank()
    }
}

pub(crate) fn cached_differential<
    R: Clone + Send + Sync + 'static,
    RT: Clone + Send + Sync + 'static + Component + PartialEq,
>(
    values: Query<&RT, Changed<RT>>,
    mut caches: Query<&mut Differential<R, RT>>,
    visibility: ParamSet<(
        Query<&ResolvedVisibility>,
        Query<Entity, Changed<ResolvedVisibility>>,
    )>,
    mut queue: ResMut<RenderQueue<R, RT>>,
) {
    todo!()
}

#[derive(Clone)]
pub(crate) struct RenderToken<R: Clone + Send + Sync + 'static, RT: Clone + Send + Sync + 'static> {
    pub(crate) token: RT,
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
pub(crate) struct RenderQueue<R: Clone + Send + Sync + 'static, RT: Clone + Send + Sync + 'static> {
    pub(crate) queue: HashMap<Entity, RenderToken<R, RT>>,
}

impl<R: Clone + Send + Sync + 'static, RT: Clone + Send + Sync + 'static> RenderQueue<R, RT> {
    pub(crate) fn new() -> Self {
        Self {
            queue: HashMap::new(),
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