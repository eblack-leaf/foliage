use crate::EcsExtension;
use crate::Trigger;
use crate::ash::differential::RenderRemoveQueue;
use crate::foliage::Foliage;
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
#[foliage_macros::targeted_event]
#[derive(Copy)]
/// Despawns an entity and everything beneath it, and tells each renderer to drop the
/// instances it held -- so nothing lingers on the GPU after the entity is gone.
pub struct Remove {}
impl Remove {
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
        tracing::trace!(entity = ?trigger.event_target(), deps_count = deps.len(), "remove: observer fired");
        if deps.is_empty() {
            return;
        }
        tree.trigger_targets(Remove::new(), deps.drain().collect::<Vec<_>>());
    }
}
