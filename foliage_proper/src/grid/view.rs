use crate::{Area, ClipContext, Component, LogicalContext, Position, Section};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Or;
use bevy_ecs::query::Changed;
use bevy_ecs::system::{ParamSet, Query};
use bevy_ecs::world::DeferredWorld;
use std::collections::HashSet;

#[derive(Component, Copy, Clone)]
pub struct View {
    pub offset: Position<LogicalContext>,
    pub extent: Area<LogicalContext>,
}
impl View {
    pub fn new() -> View {
        View {
            offset: Default::default(),
            extent: Default::default(),
        }
    }
    pub fn context(entity: Entity) -> ViewContext {
        ViewContext::new(entity)
    }
}
impl Default for View {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Copy, Clone, Default, Component)]
#[component(on_insert = ViewContext::on_insert)]
pub struct ViewContext {
    pub id: Option<Entity>,
}
impl ViewContext {
    pub fn new(entity: Entity) -> ViewContext {
        ViewContext { id: Some(entity) }
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        let clip_context = if let Some(id) = world.get::<Self>(this).unwrap().id {
            ClipContext::Entity(id)
        } else {
            ClipContext::Screen
        };
        world.commands().entity(this).insert(clip_context);
    }
}
pub(crate) fn extent_check(
    mut changed: ParamSet<(
        Query<&ViewContext, Or<(Changed<Section<LogicalContext>>, Changed<ViewContext>)>>,
        Query<(&ViewContext, &Section<LogicalContext>)>,
    )>,
    mut views: ParamSet<(Query<Entity, Changed<View>>, Query<&mut View>)>,
) {
    let mut to_check = HashSet::new();
    for context in changed.p0().iter() {
        if let Some(id) = context.id {
            to_check.insert(id);
        }
    }
    for changed in views.p0().iter() {
        to_check.insert(changed);
    }
    if to_check.is_empty() {
        return;
    }
    for id in to_check.iter() {
        views.p1().get_mut(*id).unwrap().extent = Area::default();
    }
    for (context, section) in changed.p1().iter() {
        if let Some(id) = context.id {
            if to_check.contains(&id) {
                let mut view = views.p1().get_mut(id).unwrap();
                // TODO extend extent using section
            }
        }
    }
    // TODO overscroll push-back + trigger section::on_insert on view if so (give current section?)
}
