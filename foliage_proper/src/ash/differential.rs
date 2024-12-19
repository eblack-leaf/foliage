use crate::ash::queue::RenderQueue;
use crate::{Component, ResolvedVisibility};
use bevy_ecs::change_detection::ResMut;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Changed, ParamSet, Query};
use std::marker::PhantomData;

#[derive(Component, Clone)]
pub struct Differential<
    R: Clone + Send + Sync + 'static,
    RT: Clone + Send + Sync + 'static + PartialEq,
> {
    pub cache: Option<RT>,
    _phantom: PhantomData<R>,
}

impl<R: Clone + Send + Sync + 'static, RT: Clone + Send + Sync + 'static + PartialEq>
    Differential<R, RT>
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
    for Differential<R, RT>
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
    mut caches: Query<&mut Differential<R, RT>>,
    visibility: ParamSet<(
        Query<&ResolvedVisibility>,
        Query<Entity, Changed<ResolvedVisibility>>,
    )>,
    mut queue: ResMut<RenderQueue<R, RT>>,
) {
    todo!()
}
