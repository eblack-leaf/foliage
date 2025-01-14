use crate::{Component, Logical, Position, Section, Stem, Tree};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Changed, DetectChanges, Query, Ref};
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
}
impl Default for View {
    fn default() -> Self {
        Self::new()
    }
}
fn ovrscrl(
    entity: Entity,
    ovr: Position<Logical>,
    views: &mut Query<&mut View>,
    contexts: &Query<(Entity, Ref<Stem>)>,
    sections: &Query<(Entity, Ref<Section<Logical>>)>,
    to_trigger: &mut HashSet<Entity>,
) -> (Option<Entity>, Position<Logical>) {
    let old_offset = views.get(entity).unwrap().offset;
    let mut view = views.get_mut(entity).unwrap();
    view.offset += ovr;
    let section = *sections.get(entity).unwrap().1;
    let mut over = Position::default();
    let over_right = section.right() + view.offset.left();
    if over_right > view.extent.width() {
        let val = view.extent.width() - section.right();
        over.set_left(view.offset.left() - val);
        view.offset.set_left(val);
    }
    let over_bottom = section.bottom() + view.offset.top();
    if over_bottom > view.extent.height() {
        let val = view.extent.height() - section.bottom();
        over.set_top(view.offset.top() - val);
        view.offset.set_top(val);
    }
    let over_left = section.left() + view.offset.left();
    if over_left < view.extent.left() {
        let val = view.extent.left() - section.left();
        over.set_left(view.offset.left() - val);
        view.offset.set_left(val);
    }
    let over_top = section.top() + view.offset.top();
    if over_top < view.extent.top() {
        let val = view.extent.top() - section.top();
        over.set_top(view.offset.top() - val);
        view.offset.set_top(val);
    }
    if old_offset != view.offset {
        to_trigger.insert(entity);
    }
    (contexts.get(entity).unwrap().1.id, over)
}
pub(crate) fn extent_check_v2(
    adjustments: Query<(Entity, &ViewAdjustment), Changed<ViewAdjustment>>,
    mut views: Query<&mut View>,
    contexts: Query<(Entity, Ref<Stem>)>,
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
        views.get_mut(*entity).unwrap().extent =
            Section::new(section.position, (section.right(), section.bottom()));
        // TODO check semantics
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
    let mut to_trigger = HashSet::new();
    for entity in to_check.iter() {
        let _ovr = ovrscrl(
            *entity,
            Position::default(),
            &mut views,
            &contexts,
            &sections,
            &mut to_trigger,
        );
    }
    for entity in to_check.iter() {
        let mut view = views.get_mut(*entity).unwrap();
        if let Ok((_, adjustment)) = adjustments.get(*entity) {
            view.offset += adjustment.0;
            to_trigger.insert(*entity);
        }
    }

    for entity in to_check {
        let mut overscroll = ovrscrl(
            entity,
            Position::default(),
            &mut views,
            &contexts,
            &sections,
            &mut to_trigger,
        );
        while overscroll.0.is_some() && overscroll.1 != Position::default() {
            let id = overscroll.0.unwrap();
            overscroll = ovrscrl(
                id,
                overscroll.1,
                &mut views,
                &contexts,
                &sections,
                &mut to_trigger,
            );
        }
    }
    let mut in_chain = HashSet::new();
    for entity in to_trigger.iter() {
        let mut stem = *contexts.get(*entity).unwrap().1;
        while stem.id.is_some() {
            let id = stem.id.unwrap();
            if to_trigger.contains(&id) {
                in_chain.insert(*entity);
                break;
            }
            stem = *contexts.get(id).unwrap().1;
        }
    }
    for entity in to_trigger.difference(&in_chain) {
        let section = *sections.get(*entity).unwrap().1;
        tree.entity(*entity).insert(section);
    }
}
