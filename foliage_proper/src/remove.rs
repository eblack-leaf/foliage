use crate::RenderRemoveQueue;
use bevy_ecs::change_detection::ResMut;
use bevy_ecs::prelude::{Event, Trigger};

#[derive(Event, Copy, Clone)]
pub struct Remove {}

impl Remove {
    pub fn new() -> Self {
        Self {}
    }
    pub fn token_push<R: Clone + Send + Sync + 'static>(
        trigger: Trigger<Self>,
        mut queue: ResMut<RenderRemoveQueue<R>>,
    ) {
        queue.queue.insert(trigger.entity());
    }
}
