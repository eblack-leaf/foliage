use crate::coordinate::section::Section;
use crate::coordinate::{DeviceContext, LogicalContext};
use crate::ginkgo::ScaleFactor;
use crate::{Attachment, Component, Foliage, Resource};
use bevy_ecs::change_detection::Res;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Added, Or, RemovedComponents, ResMut, With};
use bevy_ecs::query::Changed;
use bevy_ecs::system::Query;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

pub struct Ash {}
impl Attachment for Ash {
    fn attach(foliage: &mut Foliage) {
        foliage.world.insert_resource(ClippingSectionQueue::default());
        foliage.diff.add_systems(pull_clipping_section);
    }
}
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
#[derive(Debug, Copy, Clone, PartialEq, Default, PartialOrd)]
pub enum ClippingContext {
    #[default]
    Screen,
    Entity(Entity),
}
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ClippingSection(pub(crate) Section<DeviceContext>);
#[derive(Component, Copy, Clone, Default)]
pub struct EnableClipping {}
pub(crate) fn pull_clipping_section(
    query: Query<
        (Entity, &Section<LogicalContext>),
        (
            Or<(Added<EnableClipping>, Changed<Section<LogicalContext>>)>,
            With<EnableClipping>,
        ),
    >,
    mut queue: ResMut<ClippingSectionQueue>,
    scale_factor: Res<ScaleFactor>,
    mut removed: RemovedComponents<EnableClipping>,
) {
    for (entity, section) in query.iter() {
        queue.update.insert(
            entity,
            ClippingSection(section.to_device(scale_factor.value())),
        );
    }
    for entity in removed.read() {
        queue.remove.insert(entity);
    }
}
#[derive(Resource, Default)]
pub(crate) struct ClippingSectionQueue {
    update: HashMap<Entity, ClippingSection>,
    remove: HashSet<Entity>,
}
