use crate::{Component, Logical, Resource, Section};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Query, ResMut};
use bevy_ecs::world::DeferredWorld;
use std::collections::{HashMap, HashSet};

#[derive(Component, Debug, Copy, Clone, PartialEq, Default, PartialOrd, Eq, Ord, Hash)]
#[component(on_insert = ClipContext::on_insert)]
#[component(on_replace = ClipContext::on_replace)]
#[require(ClipListeners, ClipSection)]
pub enum ClipContext {
    #[default]
    Screen,
    Entity(Entity),
}
impl ClipContext {
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        let value = world.get::<ClipContext>(this).unwrap();
        match value {
            ClipContext::Screen => {
                world.commands().entity(this).insert(ClipSection(None));
            }
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
#[derive(Resource, Default)]
pub(crate) struct ClipQueue {
    pub(crate) queue: HashMap<Entity, ClipSection>,
}
pub(crate) fn prepare_clip_section(
    sections: Query<(Entity, &Section<Logical>), Changed<Section<Logical>>>,
    clip_contexts: Query<&ClipContext>,
    mut clip_sections: Query<&mut ClipSection>,
    clip_listeners: Query<&ClipListeners>,
    mut clip_queue: ResMut<ClipQueue>,
) {
    for (entity, section) in sections.iter() {
        if let Ok(listeners) = clip_listeners.get(entity) {
            for listener in listeners.listeners.iter() {
                let value = clip_contexts.get(*listener).unwrap();
                match value {
                    ClipContext::Screen => {
                        println!("screen for {:?}", listener);
                    }
                    ClipContext::Entity(_e) => {
                        println!("clip-section {} for {:?}", section, listener);
                        clip_sections
                            .get_mut(*listener)
                            .unwrap()
                            .0
                            .replace(*section);
                        clip_queue.queue.insert(entity, ClipSection(Some(*section)));
                        break;
                    }
                }
            }
        }
    }
}
#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct ClipSection(pub(crate) Option<Section<Logical>>);
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
