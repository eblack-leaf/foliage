use crate::Resource;
use bevy_ecs::entity::Entity;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

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
