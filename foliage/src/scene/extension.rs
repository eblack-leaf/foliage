use bevy_ecs::entity::Entity;

pub enum SceneExtensionTarget {
    This,
    Other(Entity),
}
