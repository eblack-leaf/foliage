use crate::ash::differential::RenderRemoveQueue;
use crate::foliage::Foliage;
use crate::{Attachment, Branch, StackDeps, Tree};
use bevy_ecs::change_detection::ResMut;
use bevy_ecs::prelude::{Event, Query, Trigger};

impl Attachment for Remove {
    fn attach(foliage: &mut Foliage) {
        foliage.define(Remove::observer);
    }
}
#[derive(Event, Copy, Clone)]
pub struct Remove {}
impl Remove {
    pub fn new() -> Self {
        Self {}
    }
    pub(crate) fn push_remove_packet<R: Clone + Send + Sync + 'static>(
        trigger: Trigger<Self>,
        mut queue: ResMut<RenderRemoveQueue<R>>,
    ) {
        queue.queue.insert(trigger.entity());
    }
    fn observer(
        trigger: Trigger<Self>,
        mut tree: Tree,
        branches: Query<&Branch>,
        stack_deps: Query<&StackDeps>,
    ) {
        if tree.get_entity(trigger.entity()).is_none() {
            return;
        }
        tree.entity(trigger.entity()).despawn();
        let mut deps = branches.get(trigger.entity()).unwrap().ids.clone();
        if let Ok(sd) = stack_deps.get(trigger.entity()) {
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
