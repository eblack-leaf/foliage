use crate::ash::differential::RenderRemoveQueue;
use crate::{Attachment, Branch, Foliage, Tree};
use bevy_ecs::change_detection::ResMut;
use bevy_ecs::entity::Entity;
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
    pub fn push_remove_packet<R: Clone + Send + Sync + 'static>(
        trigger: Trigger<Self>,
        mut queue: ResMut<RenderRemoveQueue<R>>,
    ) {
        queue.queue.insert(trigger.entity());
    }
    fn observer(trigger: Trigger<Self>, mut tree: Tree, branches: Query<&Branch>) {
        if tree.get_entity(trigger.entity()).is_none() {
            return;
        }
        tree.entity(trigger.entity()).despawn();
        let deps = branches.get(trigger.entity()).unwrap();
        let d = deps.ids.iter().map(|e| *e).collect::<Vec<Entity>>();
        if d.is_empty() {
            return;
        }
        tree.trigger_targets(Remove::new(), d);
    }
}
