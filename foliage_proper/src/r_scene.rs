use crate::coordinate::{Coordinate, InterfaceContext};
use bevy_ecs::prelude::{Commands, Entity, Query};

struct Anchor(Coordinate<InterfaceContext>);
struct Alignment {
    // placement markers (grid or custom)
}
struct SceneBinding(i32);
struct SceneNode {
    entity: Entity,
    is_scene: bool,
}
struct ScenePtr(Entity);
struct Bindings(Vec<SceneNode>);
trait Scene {
    type ConfigParams;
    fn config(query: Query<(), ()>);
    // self is the Args to the scene
    // only create bindings; will be configured above
    fn bind(self, cmd: &mut Commands) -> Entity;
}
fn resolve_anchor() {
    // queue resolution requests
}
fn finalize_anchor() {
    // take resolution requests and finalize
}
fn despawn_bindings() {
    // loop entity-pool (bindings) +
    //      if is_scene => loop that ones entity-pool
    //          despawn.signal_despawn()
}
fn scene_bind() {
    // call Scene::bind(cmd)
}