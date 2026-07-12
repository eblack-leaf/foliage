use crate::ash::differential::RenderRemoveQueue;
use crate::foliage::Foliage;
use crate::EcsExtension;
use crate::Trigger;
use crate::{AnchorDeps, Attachment, Branch, Tree};
use bevy_ecs::change_detection::ResMut;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::prelude::{Event, Query};

impl Attachment for Remove {
    fn attach(foliage: &mut Foliage) {
        foliage.define(Remove::observer);
    }
}
#[derive(EntityEvent, Copy, Clone)]
pub struct Remove {
    entity: Entity,
}
impl Default for Remove {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
        }
    }
}
crate::targeted_event!(Remove);
impl Remove {
    pub fn new() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
        }
    }
    pub(crate) fn push_remove_packet<R: Clone + Send + Sync + 'static>(
        trigger: Trigger<Self>,
        mut queue: ResMut<RenderRemoveQueue<R>>,
    ) {
        queue.queue.insert(trigger.event_target());
    }
    fn observer(
        trigger: Trigger<Self>,
        mut tree: Tree,
        branches: Query<&Branch>,
        stack_deps: Query<&AnchorDeps>,
    ) {
        if tree.get_entity(trigger.event_target()).is_err() {
            return;
        }
        tree.entity(trigger.event_target()).despawn();
        let mut deps = branches.get(trigger.event_target()).unwrap().ids.clone();
        if let Ok(sd) = stack_deps.get(trigger.event_target()) {
            for e in sd.ids.iter() {
                deps.insert(*e);
            }
        }
        if deps.is_empty() {
            return;
        }
        tree.trigger_targets(Remove::new(), deps.drain().collect::<Vec<_>>());
    }
}
