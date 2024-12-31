use crate::{ClipContext, Component, LogicalContext, Position, Section, Tree};
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
    pub extent: Section<LogicalContext>,
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
    mut deps: ParamSet<(
        Query<&ViewContext, Or<(Changed<Section<LogicalContext>>, Changed<ViewContext>)>>,
        Query<(&ViewContext, &Section<LogicalContext>)>,
    )>,
    changed_views: Query<Entity, Changed<View>>,
    mut views: Query<(Entity, &Section<LogicalContext>, &mut View)>,
    mut tree: Tree,
) {
    let mut to_check = HashSet::new();
    for context in deps.p0().iter() {
        if let Some(id) = context.id {
            to_check.insert(id);
        }
    }
    for changed in changed_views.iter() {
        to_check.insert(changed);
    }
    if to_check.is_empty() {
        return;
    }
    for id in to_check.iter() {
        views.get_mut(*id).unwrap().2.extent = Section::default();
    }
    for (context, section) in deps.p1().iter() {
        if let Some(id) = context.id {
            if to_check.contains(&id) {
                let mut relative = *section;
                let mut view = views.get_mut(id).unwrap().2;
                relative.position -= view.offset;
                if relative.left() < view.extent.left() {
                    view.extent.set_left(relative.left());
                }
                if relative.right() > view.extent.width() {
                    view.extent.set_width(relative.right());
                }
                if relative.top() < view.extent.top() {
                    view.extent.set_top(relative.top());
                }
                if relative.bottom() > view.extent.height() {
                    view.extent.set_height(relative.bottom());
                }
            }
        }
    }
    for (e, section, mut view) in views.iter_mut() {
        let mut changed = false;
        if view.offset.left() + section.width() > view.extent.width() {
            let value = view.extent.width() - section.width();
            view.offset.set_left(value);
            changed = true;
        }
        if view.offset.top() + section.height() > view.extent.height() {
            let value = view.extent.height() - section.height();
            view.offset.set_top(value);
            changed = true;
        }
        if view.offset.left() < view.extent.left() {
            let value = view.extent.left();
            view.offset.set_left(value);
            changed = true;
        }
        if view.offset.top() < view.extent.top() {
            let value = view.extent.top();
            view.offset.set_top(value);
            changed = true;
        }
        if changed {
            // NOTE: this is to trigger recursive locations w/ new view.offset
            // it is the same section it had before
            tree.entity(e).insert(*section);
        }
    }
}
