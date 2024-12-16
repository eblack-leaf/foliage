pub use bevy_ecs;
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
    pub fn branch<B: Event>(&mut self, branch: Branch<B>) -> Entity {
        let root = self.world.spawn(Stem::new(branch.stem)).id();
        // size to full of stem()
        self.world.trigger_targets(branch.event, root);
        root
    }
}
pub type Tree<'w, 's> = Commands<'w, 's>;
pub struct Branch<B: Event> {
    pub stem: Option<Entity>,
    pub event: B,
}
impl<B: Event> Branch<B> {
    pub fn new(event: B) -> Branch<B> {
        Self { stem: None, event }
    }
    pub fn stem(mut self, stem: Entity) -> Branch<B> {
        self.stem = Some(stem);
        self
    }
}
#[derive(Component)]
#[require(Group)]
pub struct Stem {
    pub id: Option<Entity>,
}
impl Stem {
    pub fn new(id: Option<Entity>) -> Self {
        Self { id }
    }
}
#[derive(Component)]
pub struct Group {
    pub ids: Vec<Entity>,
}
impl Default for Group {
    fn default() -> Self {
        Self { ids: vec![] }
    }
}
