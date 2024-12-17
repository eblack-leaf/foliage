use crate::disable::Disable;
use crate::enable::Enable;
use crate::leaf::Leaf;
use crate::remove::Remove;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::observer::TriggerTargets;
use bevy_ecs::prelude::{Commands, World};

pub type Tree<'w, 's> = Commands<'w, 's>;

pub trait EcsExtension {
    fn leaf<B: Bundle>(&mut self, b: B) -> Entity;
    fn send_to<E: Event>(&mut self, e: E, targets: impl TriggerTargets + Send + Sync + 'static);
    fn send<E: Event>(&mut self, e: E);
    fn queue<E: Event>(&mut self, e: E);
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