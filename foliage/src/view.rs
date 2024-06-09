use crate::signal::{Signal, TriggerTarget};
use bevy_ecs::component::Component;
use bevy_ecs::prelude::Entity;
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Commands, Query};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy)]
pub struct ViewHandle(pub(crate) Entity);
#[derive(Copy, Clone, Default)]
pub struct Stage(pub(crate) i32);
#[derive(Component)]
pub struct View {
    pub(crate) stages: Vec<ViewStage>,
    pub(crate) targets: HashSet<TriggerTarget>,
}
#[derive(Component, Copy, Clone, Default)]
pub struct CurrentViewStage {
    pub(crate) stage: Stage,
}
impl View {
    pub(crate) fn new() -> Self {
        Self {
            stages: vec![],
            targets: Default::default(),
        }
    }
    pub(crate) fn awaiting_confirmation(&self, stage: Stage) -> bool {
        self.stages
            .get(stage.0 as usize)
            .expect("no-stage")
            .confirmed
            == self.targets
    }
}
pub(crate) struct StagedSignal {
    pub(crate) handle: SignalHandle,
    pub(crate) state_on_stage_start: Signal,
}
pub struct ViewStage {
    pub(crate) signals: HashMap<Entity, StagedSignal>,
    confirmed: HashSet<TriggerTarget>,
}
impl Default for ViewStage {
    fn default() -> Self {
        ViewStage {
            signals: HashMap::new(),
            confirmed: HashSet::new(),
        }
    }
}
pub struct SignalHandle {
    repr: Entity,
}
pub(crate) fn signal_stage(
    mut views: Query<(&mut View, &CurrentViewStage), Changed<CurrentViewStage>>,
    mut cmd: Commands,
) {
    for (mut view, current) in views.iter_mut() {
        for target in view.targets.iter() {
            cmd.entity(target.0).insert(SignalConfirmation::Engaged);
        }
        view.stages
            .get_mut(current.stage.0 as usize)
            .expect("no-stage")
            .confirmed
            .clear();
        for signal in view
            .stages
            .get(current.stage.0 as usize)
            .expect("no-stage")
            .signals
            .iter()
        {
            cmd.entity(*signal.0).insert(signal.1.state_on_stage_start);
        }
    }
}
// TODO transitions will need to set to ::Engaged if ::Confirmed && has transition after this
pub(crate) fn attempt_to_confirm(mut confirmees: Query<&mut SignalConfirmation>) {
    for mut confirm in confirmees.iter_mut() {
        if *confirm == SignalConfirmation::Engaged {
            *confirm = SignalConfirmation::Confirmed;
        }
    }
}
#[derive(Component, Eq, PartialEq, Copy, Clone, Default)]
pub enum SignalConfirmation {
    Engaged, // switch to engaged on stage-change for each target
    #[default]
    Neutral, // will need to attempt to set confirmed
    Confirmed, // and if transition still running => set back to engaged
}
pub(crate) fn signal_confirmation(
    mut views: Query<(&mut View, &CurrentViewStage)>,
    mut targets: Query<&mut SignalConfirmation, Changed<SignalConfirmation>>,
) {
    for (mut view, current) in views.iter_mut() {
        if view.awaiting_confirmation(current.stage) {
            let mut confirmed_targets = HashSet::new();
            for target in view.targets.iter() {
                if let Ok(mut c) = targets.get_mut(target.0) {
                    if *c == SignalConfirmation::Confirmed {
                        confirmed_targets.insert(*target);
                        *c = SignalConfirmation::Neutral;
                    }
                }
            }
            for target in confirmed_targets {
                let index = current.stage.0 as usize;
                view.stages
                    .get_mut(index)
                    .expect("no-stage")
                    .confirmed
                    .insert(target);
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
