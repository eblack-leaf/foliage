use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, Resource};
use bevy_ecs::system::{Commands, Query, ResMut};
use std::collections::{HashMap, HashSet};
pub struct ViewBuilder<'a, 'w, 's> {
    cmd: &'a mut Commands<'w, 's>,
}
impl<'a, 'w, 's> ViewBuilder<'a, 'w, 's> {
    pub fn new(cmd: &mut Commands) -> Self {
        Self { cmd }
    }
    pub fn finish(self) -> ViewDescriptor {
        todo!()
    }
}
#[derive(Default)]
pub struct ViewDescriptor {
    pool: EntityPool,
    // branch + specific entity collection
}
pub type Create = fn(ViewBuilder) -> ViewDescriptor;
pub struct View {
    create: Box<Create>,
    desc: ViewDescriptor,
}
impl View {
    pub fn new(create: Create) -> Self {
        Self {
            create: Box::new(create),
            desc: ViewDescriptor::default(),
        }
    }
}
#[derive(Component, Copy, Clone)]
pub struct Navigate(pub ViewHandle);
fn navigation(
    query: Query<(Entity, Navigate)>,
    mut cmd: Commands,
    mut compositor: ResMut<Compositor>,
) {
    if let Some(l) = query.iter().last() {
        // despawn old
        // call .create(cmd) ...
        // set view
    }
    for (e, _) in query.iter() {
        cmd.entity(e).despawn();
    }
}
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Default)]
pub struct ViewHandle(pub i32);
impl From<i32> for ViewHandle {
    fn from(value: i32) -> Self {
        Self(value)
    }
}
#[derive(Clone, Default)]
pub struct EntityPool(pub HashSet<Entity>);
#[derive(Resource, Default)]
pub struct Compositor {
    views: HashMap<ViewHandle, View>,
    current: Option<ViewHandle>,
}