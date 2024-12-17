use crate::{Component, Resource};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::ResMut;
use bevy_ecs::query::Changed;
use bevy_ecs::system::Query;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

#[derive(Clone)]
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
#[derive(Component, Clone)]
pub struct RenderTokenCache<
    R: Clone + Send + Sync + 'static,
    RT: Clone + Send + Sync + 'static + PartialEq,
> {
    pub cache: Option<RT>,
    _phantom: PhantomData<R>,
}
impl<R: Clone + Send + Sync + 'static, RT: Clone + Send + Sync + 'static + PartialEq>
    RenderTokenCache<R, RT>
{
    pub fn new(cache: RT) -> Self {
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
    pub fn compare(&mut self, token: RT) -> bool {
        todo!()
    }
}
impl<R: Clone + Send + Sync + 'static, RT: Clone + Send + Sync + 'static + PartialEq> Default
    for RenderTokenCache<R, RT>
{
    fn default() -> Self {
        Self::blank()
    }
}
pub fn cached_differential<
    R: Clone + Send + Sync + 'static,
    RT: Clone + Send + Sync + 'static + Component + PartialEq,
>(
    values: Query<&RT, Changed<RT>>,
    mut caches: Query<&mut RenderTokenCache<R, RT>>,
    mut queue: ResMut<RenderQueue<R, RT>>,
) {
    todo!()
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
