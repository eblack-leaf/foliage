use crate::{ClipContext, Layer};
use bevy_ecs::entity::Entity;

pub struct Node {
    pub layer: Layer,
    pub clip_context: ClipContext,
    pub entity: Entity,
}
