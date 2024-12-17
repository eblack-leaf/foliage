mod text;

pub use bevy_ecs;
use bevy_ecs::component::StorageType::Table;
use bevy_ecs::component::{ComponentHooks, ComponentId, StorageType};
use bevy_ecs::observer::TriggerTargets;
pub use bevy_ecs::prelude::*;
use bevy_ecs::system::IntoObserverSystem;
use bevy_ecs::world::DeferredWorld;
pub use nalgebra;
pub use nalgebra::*;
use std::collections::HashSet;
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
    pub fn evaluate<Targets: AsRef<[Entity]>>(&mut self, targets: Targets) {
        self.world.evaluate(targets);
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
    fn evaluate<Targets: AsRef<[Entity]>>(&mut self, targets: Targets);
    fn remove<Targets: AsRef<[Entity]>>(&mut self, targets: Targets);
    fn write_to<B: Bundle>(&mut self, entity: Entity, b: B);
    fn enable<Targets: AsRef<[Entity]>>(&mut self, targets: Targets);
    fn disable<Targets: AsRef<[Entity]>>(&mut self, targets: Targets);
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
    fn evaluate<Targets: AsRef<[Entity]>>(&mut self, targets: Targets) {
        // TODO replace with batch
        for t in targets.as_ref().iter() {
            self.write_to(*t, Evaluate::new());
        }
    }
    fn remove<Targets: AsRef<[Entity]>>(&mut self, targets: Targets) {
        // TODO replace with batch
        for t in targets.as_ref().iter() {
            self.write_to(*t, Remove::new());
        }
    }
    fn write_to<B: Bundle>(&mut self, entity: Entity, b: B) {
        self.entity(entity).insert(b);
    }
    fn enable<Targets: AsRef<[Entity]>>(&mut self, targets: Targets) {
        // TODO replace with batch
        for t in targets.as_ref().iter() {
            self.write_to(*t, Enable::new());
        }
    }
    fn disable<Targets: AsRef<[Entity]>>(&mut self, targets: Targets) {
        // TODO replace with batch
        for t in targets.as_ref().iter() {
            self.write_to(*t, Disable::new());
        }
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
    fn evaluate<Targets: AsRef<[Entity]>>(&mut self, targets: Targets) {
        self.commands().evaluate(targets);
    }
    fn remove<Targets: AsRef<[Entity]>>(&mut self, targets: Targets) {
        self.commands().remove(targets);
    }
    fn write_to<B: Bundle>(&mut self, entity: Entity, b: B) {
        self.commands().write_to(entity, b);
    }
    fn enable<Targets: AsRef<[Entity]>>(&mut self, targets: Targets) {
        self.commands().enable(targets);
    }
    fn disable<Targets: AsRef<[Entity]>>(&mut self, targets: Targets) {
        self.commands().disable(targets);
    }
}
#[derive(Copy, Clone)]
pub struct Evaluate {}
impl Evaluate {
    pub fn new() -> Self {
        Self {}
    }
}
impl Component for Evaluate {
    const STORAGE_TYPE: StorageType = Table;
    fn register_component_hooks(_hooks: &mut ComponentHooks) {
        todo!()
    }
}
#[derive(Copy, Clone)]
pub struct Remove {}
impl Remove {
    pub fn new() -> Self {
        Self {}
    }
}
impl Component for Remove {
    const STORAGE_TYPE: StorageType = Table;
    fn register_component_hooks(_hooks: &mut ComponentHooks) {
        todo!()
    }
}
#[derive(Copy, Clone)]
pub struct Enable {}
impl Enable {
    pub fn new() -> Enable {
        Enable {}
    }
}
impl Component for Enable {
    const STORAGE_TYPE: StorageType = Table;
    fn register_component_hooks(_hooks: &mut ComponentHooks) {
        todo!()
    }
}
#[derive(Copy, Clone)]
pub struct Disable {}
impl Disable {
    pub fn new() -> Disable {
        Disable {}
    }
}
impl Component for Disable {
    const STORAGE_TYPE: StorageType = Table;
    fn register_component_hooks(_hooks: &mut ComponentHooks) {
        todo!()
    }
}
#[derive(Copy, Clone)]
pub struct Stem {
    pub id: Option<Entity>,
}
impl Component for Stem {
    const STORAGE_TYPE: StorageType = Table;
    fn register_component_hooks(_hooks: &mut ComponentHooks) {
        _hooks.on_insert(Self::on_insert);
        _hooks.on_replace(Self::on_replace);
    }
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
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        let stem = world.get::<Stem>(this).copied().unwrap();
        if let Some(s) = stem.id {
            if let Some(mut deps) = world.get_mut::<Branch>(s) {
                deps.ids.insert(this);
            }
        }
    }
    fn on_replace(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        let stem = world.get::<Stem>(this).copied().unwrap();
        if let Some(s) = stem.id {
            if let Some(mut deps) = world.get_mut::<Branch>(s) {
                deps.ids.remove(&this);
            }
        }
    }
}
#[derive(Component, Clone)]
pub struct Branch {
    pub ids: HashSet<Entity>,
}
impl Default for Branch {
    fn default() -> Self {
        Self {
            ids: HashSet::new(),
        }
    }
}
