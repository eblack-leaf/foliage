use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, IntoSystemConfigs};
use bevy_ecs::query::Changed;
use bevy_ecs::system::Query;

use crate::elm::config::{CoreSet, ElmConfiguration};
use crate::elm::leaf::{EmptySetDescriptor, Leaf};
use crate::elm::Elm;

#[derive(Default, Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum TriggerState {
    Active,
    #[default]
    Neutral,
    Inverse,
}
#[derive(Default, Copy, Clone, Debug, Hash, Eq, PartialEq, Component)]
pub struct Trigger(pub(crate) TriggerState);
impl Trigger {
    pub fn active() -> Self {
        Self(TriggerState::Active)
    }
    pub fn inverse() -> Self {
        Self(TriggerState::Inverse)
    }
    pub fn neutral() -> Self {
        Self(TriggerState::Neutral)
    }
    pub fn is_active(&self) -> bool {
        self.0 == TriggerState::Active
    }
    pub fn is_inverse(&self) -> bool {
        self.0 == TriggerState::Inverse
    }
    pub fn is_neutral(&self) -> bool {
        self.0 == TriggerState::Neutral
    }
    pub fn set(&mut self, trigger_state: TriggerState) {
        self.0 = trigger_state;
    }
}
fn clear_triggered(mut triggers: Query<&mut Trigger, Changed<Trigger>>) {
    for mut trigger in triggers.iter_mut() {
        if trigger.0 != TriggerState::Neutral {
            trigger.0 = TriggerState::Neutral;
        }
    }
}
impl Leaf for Trigger {
    type SetDescriptor = EmptySetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        elm.main()
            .add_systems((clear_triggered.after(CoreSet::Differential),));
    }
}
#[derive(Copy, Clone)]
pub struct TriggerEntity {
    pub trigger: Trigger,
    pub entity: Entity,
}
impl TriggerEntity {
    pub fn new(entity: Entity, trigger: Trigger) -> Self {
        Self { trigger, entity }
    }
    pub fn trigger(&self) -> Trigger {
        self.trigger
    }
    pub fn entity(&self) -> Entity {
        self.entity
    }
}