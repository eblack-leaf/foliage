use crate::signal::TriggerTarget;
use bevy_ecs::component::Component;
use bevy_ecs::prelude::Entity;
use bevy_ecs::query::Changed;
use bevy_ecs::system::Query;
use std::collections::HashSet;

#[derive(Clone, Copy)]
pub struct ViewHandle(pub(crate) Entity);
pub struct Stage(pub(crate) i32);
#[derive(Component)]
pub struct View {
    pub(crate) stages: Vec<ViewStage>,
    current: Stage,
    pub(crate) targets: HashSet<TriggerTarget>,
}
impl View {
    pub(crate) fn new() -> Self {
        Self {
            stages: vec![],
            current: Stage(0),
            targets: Default::default(),
        }
    }
    pub(crate) fn awaiting_confirmation(&self) -> bool {
        self.stages
            .get(self.current.0 as usize)
            .expect("no-stage")
            .confirmed
            == self.targets
    }
}
pub struct ViewStage {
    pub(crate) signals: Vec<SignalHandle>,
    confirmed: HashSet<TriggerTarget>,
}
impl Default for ViewStage {
    fn default() -> Self {
        ViewStage {
            signals: vec![],
            confirmed: HashSet::new(),
        }
    }
}
pub struct SignalHandle {
    repr: Entity,
}
#[derive(Component, Eq, PartialEq, Copy, Clone)]
pub enum SignalConfirmation {
    Engaged,// switch to engaged on stage-change for each target
    Neutral,// will need to attempt to set confirmed
    Confirmed,// and if transition still running => set back to engaged
}
pub(crate) fn signal_confirmation(
    mut views: Query<(&mut View)>,
    mut targets: Query<(&mut SignalConfirmation), Changed<SignalConfirmation>>,
) {
    for view in views.iter_mut() {
        if view.awaiting_confirmation() {
            for target in view.targets.iter() {
                if let Ok(c) = targets.get_mut(target.0) {
                    if *c == SignalConfirmation::Confirmed {
                        view.stages
                            .get_mut(view.current.0 as usize)
                            .expect("no-stage")
                            .confirmed
                            .insert(*target);
                        *c = SignalConfirmation::Neutral;
                    }
                }
            }
        }
    }
}
impl SignalHandle {
    pub fn repr(&self) -> Entity {
        self.repr
    }
    pub(crate) fn new(entity: Entity) -> Self {
        Self { repr: entity }
    }
}
