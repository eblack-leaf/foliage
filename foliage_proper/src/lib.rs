mod ash;
mod attachment;
mod color;
mod disable;
mod enable;
mod leaf;
mod opacity;
mod ops;
mod remove;
mod text;
mod tree;
mod texture;
mod time;
mod virtual_keyboard;
mod web_ext;
mod willow;
mod ginkgo;
mod coordinate;
mod asset;
mod platform;
mod interaction;

use crate::ash::cached_differential;
use crate::coordinate::Coordinates;
pub use ash::{Differential, RenderQueue, RenderRemoveQueue, RenderToken};
pub use attachment::Attachment;
pub use bevy_ecs;
use bevy_ecs::observer::TriggerTargets;
pub use bevy_ecs::prelude::*;
use bevy_ecs::system::IntoObserverSystem;
pub use color::Color;
pub use coordinate::elevation::{Elevation, Layer};
pub use leaf::{Branch, Leaf, Stem};
pub use opacity::Opacity;
pub use ops::{Update, Write};
#[cfg(target_os = "android")]
pub use platform::AndroidApp;
pub use platform::AndroidConnection;
pub use text::{FontSize, Text};
pub use tree::{EcsExtension, Tree};

pub struct Foliage {
    pub world: World,
    pub main: Schedule,
    pub user: Schedule,
    pub diff: Schedule,
}
impl Foliage {
    pub fn new() -> Foliage {
        Foliage {
            world: Default::default(),
            main: Default::default(),
            user: Default::default(),
            diff: Default::default(),
        }
    }
    pub fn photosynthesize(&self) {
        todo!()
    }
    pub fn desktop_size<V: Into<Coordinates>>(&self, v: V) {
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
    pub fn write_to<B: Bundle>(&mut self, entity: Entity, b: B) {
        self.world.write_to(entity, b);
    }
    pub fn remove(&mut self, targets: impl TriggerTargets + Send + Sync + 'static) {
        self.world.remove(targets);
    }
    pub fn enable(&mut self, targets: impl TriggerTargets + Send + Sync + 'static) {
        self.world.enable(targets);
    }
    pub fn disable(&mut self, targets: impl TriggerTargets + Send + Sync + 'static) {
        self.world.disable(targets);
    }
    pub fn remove_queue<R: Clone + Send + Sync + 'static>(&mut self) {
        debug_assert_eq!(
            self.world.get_resource::<RenderRemoveQueue<R>>().is_none(),
            true
        );
        self.world.insert_resource(RenderRemoveQueue::<R>::new());
    }
    pub fn differential<
        R: Clone + Send + Sync + 'static,
        RT: Clone + Send + Sync + 'static + Component + PartialEq,
    >(
        &mut self,
    ) {
        debug_assert_eq!(
            self.world.get_resource::<RenderQueue<R, RT>>().is_none(),
            true
        );
        self.world.insert_resource(RenderQueue::<R, RT>::new());
        self.diff.add_systems(cached_differential::<R, RT>);
    }
}
