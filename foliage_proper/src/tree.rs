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
    fn write_to<B: Bundle>(&mut self, entity: Entity, b: B);
    fn remove(&mut self, targets: impl TriggerTargets + Send + Sync + 'static);
    fn enable(&mut self, targets: impl TriggerTargets + Send + Sync + 'static);
    fn disable(&mut self, targets: impl TriggerTargets + Send + Sync + 'static);
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
    fn write_to<B: Bundle>(&mut self, entity: Entity, b: B) {
        self.entity(entity).insert(b);
    }
    fn remove(&mut self, targets: impl TriggerTargets + Send + Sync + 'static) {
        self.send_to(Remove::new(), targets);
    }
    fn enable(&mut self, targets: impl TriggerTargets + Send + Sync + 'static) {
        self.send_to(Enable::new(), targets);
    }
    fn disable(&mut self, targets: impl TriggerTargets + Send + Sync + 'static) {
        self.send_to(Disable::new(), targets);
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
    fn write_to<B: Bundle>(&mut self, entity: Entity, b: B) {
        self.commands().write_to(entity, b);
    }
    fn remove(&mut self, targets: impl TriggerTargets + Send + Sync + 'static) {
        self.commands().remove(targets);
    }
    fn enable(&mut self, targets: impl TriggerTargets + Send + Sync + 'static) {
        self.commands().enable(targets);
    }
    fn disable(&mut self, targets: impl TriggerTargets + Send + Sync + 'static) {
        self.commands().disable(targets);
    }
}
