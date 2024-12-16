pub use bevy_ecs;
use bevy_ecs::observer::TriggerTargets;
pub use bevy_ecs::prelude::*;
use bevy_ecs::system::IntoObserverSystem;
pub use nalgebra;
pub use nalgebra::*;
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
        let entity = self.world.spawn((Leaf::new(), b)).id();
        entity
    }
    pub fn send_to<E: Event>(&mut self, e: E, targets: impl TriggerTargets) {
        self.world.trigger_targets(e, targets);
    }
    pub fn send<E: Event>(&mut self, e: E) {
        self.world.trigger(e);
    }
    pub fn queue<E: Event>(&mut self, e: E) {
        self.world.send_event(e);
    }
    pub fn evaluate(&mut self, targets: impl TriggerTargets) {
        // self.world.trigger_targets(Evaluate::recursive(), targets);
    }
    pub fn remove(&mut self, targets: impl TriggerTargets) {
        // self.world.trigger_targets(Remove::new(), targets);
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
