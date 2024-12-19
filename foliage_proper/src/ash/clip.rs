use crate::{Component, LogicalContext, Section};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::query::Changed;
use bevy_ecs::system::Query;
use bevy_ecs::world::DeferredWorld;
use std::collections::HashSet;

#[derive(Component, Debug, Copy, Clone, PartialEq, Default, PartialOrd)]
#[component(on_insert = ClipContext::on_insert)]
#[component(on_replace = ClipContext::on_replace)]
#[require(ClipSection)]
#[require(ClipListeners)]
pub enum ClipContext {
    #[default]
    Screen,
    Entity(Entity),
}
impl ClipContext {
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        let value = world.get::<ClipContext>(this).unwrap();
        match value {
            ClipContext::Screen => {}
            ClipContext::Entity(e) => {
                if let Some(mut listeners) = world.get_mut::<ClipListeners>(*e) {
                    listeners.listeners.insert(this);
                }
            }
        }
    }
    fn on_replace(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        let value = world.get::<ClipContext>(this).unwrap();
        match value {
            ClipContext::Screen => {}
            ClipContext::Entity(e) => {
                if let Some(mut listeners) = world.get_mut::<ClipListeners>(*e) {
                    listeners.listeners.remove(&this);
                }
            }
        }
    }
}
pub(crate) fn prepare_clip_section(
    sections: Query<(Entity, &Section<LogicalContext>), Changed<Section<LogicalContext>>>,
    mut clip_sections: Query<&mut ClipSection>,
    clip_contexts: Query<&ClipContext>,
    clip_listeners: Query<&ClipListeners>,
) {
    for (entity, section) in sections.iter() {
        if let Ok(listeners) = clip_listeners.get(entity) {
            for listener in listeners.listeners.iter() {
                let value = clip_contexts.get(*listener).unwrap();
                match value {
                    ClipContext::Screen => {}
                    ClipContext::Entity(_e) => {
                        if let Ok(mut clip_section) = clip_sections.get_mut(*listener) {
                            clip_section.0 = *section;
                        }
                    }
                }
            }
        }
    }
}
#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
pub struct ClipSection(pub(crate) Section<LogicalContext>);
#[derive(Component)]
pub(crate) struct ClipListeners {
    pub(crate) listeners: HashSet<Entity>,
}
impl ClipListeners {
    pub(crate) fn new() -> Self {
        Self {
            listeners: Default::default(),
        }
    }
}
impl Default for ClipListeners {
    fn default() -> Self {
        Self::new()
    }
}
