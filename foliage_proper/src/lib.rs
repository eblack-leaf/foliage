mod text;
mod tree;
mod enable;
mod disable;
mod remove;
mod leaf;
mod ops;
mod location;
mod attachment;
mod elevation;

pub use attachment::Attachment;
pub use bevy_ecs;
use bevy_ecs::observer::TriggerTargets;
pub use bevy_ecs::prelude::*;
use bevy_ecs::system::IntoObserverSystem;
pub use elevation::{Elevation, Layer};
pub use leaf::{Branch, Leaf, Stem};
pub use location::Location;
pub use nalgebra;
pub use nalgebra::*;
pub use ops::{Update, Write};
pub use text::{FontSize, Text};
pub use tree::{EcsExtension, Tree};
pub struct Foliage {
    pub world: World,
}
impl Foliage {
    pub fn new() -> Foliage {
        Foliage {
            world: Default::default(),
        }
    }
    pub fn photosynthesize(&self) {
        todo!()
    }
    pub fn desktop_size<V: Into<Vector2<u32>>>(&self, v: V) {
        todo!()
    }
    pub fn url<S: AsRef<str>>(&self, path: S) {
        todo!()
    }
    pub fn define<E: Event + 'static, B: Bundle, M, D: IntoObserverSystem<E, B, M>>(
        &mut self,
        obs: D,
    ) {
        self.world.add_observer(obs);
    }
    pub fn leaf<B: Bundle>(&mut self, b: B) -> Entity {
        self.world.leaf(b)
    }
    pub fn send_to<E: Event>(
        &mut self,
        e: E,
        targets: impl TriggerTargets + Send + Sync + 'static,
    ) {
        self.world.send_to(e, targets);
    }
    pub fn send<E: Event>(&mut self, e: E) {
        self.world.send(e);
    }
    pub fn queue<E: Event>(&mut self, e: E) {
        self.world.queue(e);
    }
    pub fn remove<Targets: AsRef<[Entity]>>(&mut self, targets: Targets) {
        self.world.remove(targets);
    }
    pub fn write_to<B: Bundle>(&mut self, entity: Entity, b: B) {
        self.world.write_to(entity, b);
    }
    pub fn enable<Targets: AsRef<[Entity]>>(&mut self, targets: Targets) {
        self.world.enable(targets);
    }
    pub fn disable<Targets: AsRef<[Entity]>>(&mut self, targets: Targets) {
        self.world.disable(targets);
    }
}
