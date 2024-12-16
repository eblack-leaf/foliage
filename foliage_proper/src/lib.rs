mod text;
pub use bevy_ecs;
use bevy_ecs::observer::TriggerTargets;
pub use bevy_ecs::prelude::*;
use bevy_ecs::system::IntoObserverSystem;
pub use nalgebra;
pub use nalgebra::*;
pub use text::{FontSize, Text};
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
    pub fn evaluate(&mut self, targets: impl TriggerTargets) {
        self.world.evaluate(targets);
    }
    pub fn remove(&mut self, targets: impl TriggerTargets) {
        self.world.remove(targets);
    }
}
#[derive(Component)]
#[require(Stem, Branch)]
pub struct Leaf {}
impl Leaf {
    pub fn new() -> Leaf {
        Leaf {}
    }
}
pub type Tree<'w, 's> = Commands<'w, 's>;
pub trait EcsExtension {
    fn leaf<B: Bundle>(&mut self, b: B) -> Entity;
    fn send_to<E: Event>(&mut self, e: E, targets: impl TriggerTargets + Send + Sync + 'static);
    fn send<E: Event>(&mut self, e: E);
    fn queue<E: Event>(&mut self, e: E);
    fn evaluate(&mut self, targets: impl TriggerTargets);
    fn remove(&mut self, targets: impl TriggerTargets);
}
impl<'w, 's> EcsExtension for Tree<'w, 's> {
    fn leaf<B: Bundle>(&mut self, b: B) -> Entity {
        let entity = self.spawn((Leaf::new(), b)).id();
        entity
    }
    fn send_to<E: Event>(&mut self, e: E, targets: impl TriggerTargets + Send + Sync + 'static) {
        self.trigger_targets(e, targets);
    }
    fn send<E: Event>(&mut self, e: E) {
        self.trigger(e);
    }
    fn queue<E: Event>(&mut self, e: E) {
        self.send_event(e);
    }
    fn evaluate(&mut self, targets: impl TriggerTargets) {
        // self.trigger_targets(Evaluate::recursive(), targets);
    }
    fn remove(&mut self, targets: impl TriggerTargets) {
        // self.trigger_targets(Remove::new(), targets);
    }
}
impl EcsExtension for World {
    fn leaf<B: Bundle>(&mut self, b: B) -> Entity {
        self.commands().leaf(b)
    }
    fn send_to<E: Event>(&mut self, e: E, targets: impl TriggerTargets + Send + Sync + 'static) {
        self.commands().send_to(e, targets);
    }
    fn send<E: Event>(&mut self, e: E) {
        self.commands().send(e);
    }
    fn queue<E: Event>(&mut self, e: E) {
        EcsExtension::queue(&mut self.commands(), e);
    }
    fn evaluate(&mut self, targets: impl TriggerTargets) {
        // self.trigger_targets(Evaluate::recursive(), targets);
    }
    fn remove(&mut self, targets: impl TriggerTargets) {
        // self.trigger_targets(Remove::new(), targets);
    }
}
#[derive(Event)]
pub struct Evaluate {}
impl Evaluate {
    pub fn new() -> Self {
        Self {}
    }
}
#[derive(Event)]
pub struct Remove {}
impl Remove {
    pub fn new() -> Self {
        Self {}
    }
}
#[derive(Event)]
pub struct Enable {}
impl Enable {
    pub fn new() -> Enable {
        Enable {}
    }
}
#[derive(Event)]
pub struct Disable {}
impl Disable {
    pub fn new() -> Disable {
        Disable {}
    }
}
#[derive(Component)]
pub struct Stem {
    pub id: Option<Entity>,
}
impl Default for Stem {
    fn default() -> Self {
        Stem::none()
    }
}
impl Stem {
    pub fn new(id: Option<Entity>) -> Self {
        Self { id }
    }
    pub fn some(entity: Entity) -> Self {
        Self { id: Some(entity) }
    }
    pub fn none() -> Self {
        Self { id: None }
    }
}
#[derive(Component)]
pub struct Branch {
    pub ids: Vec<Entity>,
}
impl Default for Branch {
    fn default() -> Self {
        Self { ids: vec![] }
    }
}
