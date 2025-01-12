use crate::{ClipContext, Component, Logical, Position, Section, Tree};
use bevy_ecs::change_detection::DetectChanges;
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Ref;
use bevy_ecs::query::Changed;
use bevy_ecs::system::Query;
use bevy_ecs::world::DeferredWorld;
use std::collections::HashSet;

#[derive(Component, Copy, Clone, Debug, Default)]
pub(crate) struct ViewAdjustment(pub(crate) Position<Logical>);
#[derive(Component, Copy, Clone, Debug)]
#[require(ViewAdjustment)]
pub struct View {
    pub offset: Position<Logical>,
    pub extent: Section<Logical>,
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
pub(crate) fn extent_check_v2(
    adjustments: Query<(Entity, &ViewAdjustment), Changed<ViewAdjustment>>,
    mut views: Query<&mut View>,
    contexts: Query<(Entity, Ref<ViewContext>)>,
    sections: Query<(Entity, Ref<Section<Logical>>)>,
    mut tree: Tree,
) {
    let mut to_check = HashSet::new();
    for (entity, adjustment) in adjustments.iter() {
        to_check.insert(entity);
    }
    for (entity, context) in contexts.iter() {
        if context.is_changed() {
            if let Some(id) = context.id {
                to_check.insert(id);
            }
        }
    }
    for (entity, section) in sections.iter() {
        if section.is_changed() {
            if let Ok((_, context)) = contexts.get(entity) {
                if let Some(id) = context.id {
                    to_check.insert(id);
                }
            }
        }
    }
    if to_check.is_empty() {
        return;
    }
    for entity in to_check.iter() {
        let section = *sections.get(*entity).unwrap().1;
        views.get_mut(*entity).unwrap().extent = section; // TODO check semantics
    }
    for (entity, context) in contexts.iter() {
        if let Some(id) = context.id {
            if to_check.contains(&id) {
                if let Ok(mut view) = views.get_mut(id) {
                    if let Ok((_, section)) = sections.get(entity) {
                        let mut relative = *section;
                        relative.position += view.offset;
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
        }
    }
    for entity in to_check {
        let old_offset = views.get(entity).unwrap().offset;
        let mut view = views.get_mut(entity).unwrap();
        if let Ok((_, adjustment)) = adjustments.get(entity) {
            view.offset += adjustment.0;
        }
        let section = *sections.get(entity).unwrap().1;
        let over_right = section.right() + view.offset.left();
        if over_right > view.extent.width() {
            let val = view.extent.width() - section.right();
            view.offset.set_left(val);
        }
        let over_left = section.left() + view.offset.left();
        if over_left < view.extent.left() {
            let val = view.extent.left() - section.left();
            view.offset.set_left(val);
        }
        let over_bottom = section.bottom() + view.offset.top();
        if over_bottom > view.extent.height() {
            let val = view.extent.height() - section.bottom();
            view.offset.set_top(val);
        }
        let over_top = section.top() + view.offset.top();
        if over_top < view.extent.top() {
            let val = view.extent.top() - section.top();
            view.offset.set_top(val);
        }
        if old_offset != view.offset {
            tree.entity(entity).insert(section);
        }
    }
}
// #[derive(Resource, Default)]
// pub(crate) struct ExtentCheckIds(pub(crate) HashSet<Entity>);
// pub(crate) fn prepare_extent(
//     deps: Query<&ViewContext, Or<(Changed<Section<Logical>>, Changed<ViewContext>)>>,
//     views: Query<Entity, Changed<View>>,
//     mut to_check: ResMut<ExtentCheckIds>,
// ) {
//     if deps.is_empty() && views.is_empty() {
//         return;
//     }
//     to_check.0.clear();
//     for context in deps.iter() {
//         if let Some(id) = context.id {
//             to_check.0.insert(id);
//         }
//     }
//     for changed in views.iter() {
//         to_check.0.insert(changed);
//     }
// }
// pub(crate) fn extent_check(
//     deps: Query<(&ViewContext, &Section<Logical>)>,
//     mut views: Query<(Entity, &Section<Logical>, &mut View)>,
//     mut tree: Tree,
//     mut to_check: ResMut<ExtentCheckIds>,
// ) {
//     if to_check.0.is_empty() {
//         return;
//     }
//     for id in to_check.0.iter() {
//         let section = *views.get(*id).unwrap().1;
//         views.get_mut(*id).unwrap().2.extent = Section::default();
//     }
//     for (context, section) in deps.iter() {
//         if let Some(id) = context.id {
//             if to_check.0.contains(&id) {
//                 let mut relative = *section;
//                 let mut view = views.get_mut(id).unwrap().2;
//                 relative.position += view.offset;
//                 if relative.left() < view.extent.left() {
//                     view.extent.set_left(relative.left());
//                 }
//                 if relative.right() > view.extent.width() {
//                     view.extent.set_width(relative.right());
//                 }
//                 if relative.top() < view.extent.top() {
//                     view.extent.set_top(relative.top());
//                 }
//                 if relative.bottom() > view.extent.height() {
//                     view.extent.set_height(relative.bottom());
//                 }
//                 println!("view: {} {}", view.offset, view.extent);
//             }
//         }
//     }
//     to_check.0.clear();
//     for (e, section, mut view) in views.iter_mut() {
//         let mut changed = false;
//         let cached = view.offset;
//         if view.offset.left() + section.width() > view.extent.width() {
//             let value = view.extent.width() - section.width();
//             view.offset.set_left(value);
//             changed = true;
//         }
//         if view.offset.top() + section.height() > view.extent.height() {
//             let value = view.extent.height() - section.height();
//             view.offset.set_top(value);
//             changed = true;
//         }
//         if view.offset.left() < view.extent.left() {
//             let value = view.extent.left();
//             view.offset.set_left(value);
//             changed = true;
//         }
//         if view.offset.top() < view.extent.top() {
//             let value = view.extent.top();
//             view.offset.set_top(value);
//             changed = true;
//         }
//         if changed && cached != view.offset {
//             // NOTE: this is to trigger recursive locations w/ new view.offset
//             // it is the same section it had before
//             println!("view: {} {}", view.offset, view.extent);
//             tree.entity(e).insert(*section);
//         }
//     }
// }
