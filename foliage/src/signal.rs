use bevy_ecs::change_detection::Res;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Bundle, Changed, Commands, Component, Query, ResMut};
use bevy_ecs::system::Command;

use crate::differential::{RenderLink, RenderRemoveQueue};
use crate::grid::{Layout, LayoutFilter};
use crate::view::{SignalConfirmation, ViewHandle};

#[derive(Component, Default, Copy, Clone)]
pub struct Signal {
    message: i32,
}
impl Signal {
    pub fn is_neutral(&self) -> bool {
        self.message == 0
    }
    pub fn should_clean(&self) -> bool {
        self.message == 1
    }
    pub fn should_spawn(&self) -> bool {
        self.message == 2
    }
    pub fn neutral() -> Self {
        Self { message: 0 }
    }
    pub fn clean() -> Self {
        Self { message: 1 }
    }
    pub fn spawn() -> Self {
        Self { message: 2 }
    }
}
#[derive(Bundle)]
pub(crate) struct Signaler {
    signal: Signal,
    target: TriggerTarget,
}
impl Signaler {
    pub(crate) fn new(target: TriggerTarget) -> Self {
        Self {
            signal: Default::default(),
            target,
        }
    }
}
#[derive(Component, Hash, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct TriggerTarget(pub(crate) Entity);
impl TriggerTarget {
    pub fn value(&self) -> Entity {
        self.0
    }
}
#[derive(Component)]
pub(crate) struct TriggeredAttribute<A: Bundle + 'static + Send + Sync + Clone>(pub(crate) A);
#[derive(Component)]
pub(crate) struct FilteredTriggeredAttribute<A: Bundle + 'static + Send + Sync + Clone>(
    pub(crate) A,
    pub(crate) LayoutFilter,
);
// TODO run after the regular signaled-spawn to clear base + filtered bundle dependencies
pub(crate) fn filtered_signaled_spawn<A: Bundle + 'static + Send + Sync + Clone>(
    to_spawn: Query<(&Signal, &FilteredTriggeredAttribute<A>, &TriggerTarget), Changed<Signal>>,
    mut cmd: Commands,
    layout_config: Res<Layout>,
) {
    for (signal, attribute, target) in to_spawn.iter() {
        if signal.should_spawn() && attribute.1.accepts(*layout_config) {
            cmd.entity(target.0).insert(attribute.0.clone());
        }
    }
}
pub(crate) fn signaled_clean(
    mut to_clean: Query<(&mut Signal, &TriggerTarget), Changed<Signal>>,
    mut cmd: Commands,
) {
    for (mut signal, target) in to_clean.iter_mut() {
        if signal.should_clean() {
            cmd.entity(target.0).insert(Clean::should_clean());
        }
    }
}
pub(crate) fn clear_signal(mut signals: Query<&mut Signal, Changed<Signal>>) {
    for mut signal in signals.iter_mut() {
        *signal = Signal::default();
    }
}
pub(crate) fn signaled_spawn<A: Bundle + 'static + Send + Sync + Clone>(
    to_spawn: Query<(&Signal, &TriggeredAttribute<A>, &TriggerTarget), Changed<Signal>>,
    mut cmd: Commands,
) {
    for (signal, attribute, target) in to_spawn.iter() {
        if signal.should_spawn() {
            cmd.entity(target.0).insert(attribute.0.clone());
        }
    }
}
#[derive(Component, Copy, Clone, Default)]
pub struct Clean {
    should_clean: bool,
}
impl Clean {
    pub fn clean_entity(&mut self) {
        self.should_clean = true;
    }
    pub fn should_clean() -> Self {
        Self { should_clean: true }
    }
}
pub(crate) fn clean(
    mut to_clean: Query<(Entity, &mut Clean, Option<&RenderLink>)>,
    mut remove_queue: ResMut<RenderRemoveQueue>,
    mut cmd: Commands,
) {
    for (entity, mut clean, opt_link) in to_clean.iter_mut() {
        if clean.should_clean {
            if let Some(link) = opt_link {
                remove_queue
                    .queue
                    .get_mut(&link)
                    .expect("invalid render link")
                    .insert(entity);
            }
            clean.should_clean = false;
            cmd.entity(entity).retain::<TargetComponents>();
        }
    }
}
#[derive(Bundle)]
pub(crate) struct TargetComponents {
    clean: Clean,
    confirm: SignalConfirmation,
    handle: ViewHandle,
}
impl TargetComponents {
    pub(crate) fn new(handle: ViewHandle) -> Self {
        Self {
            clean: Default::default(),
            confirm: Default::default(),
            handle,
        }
    }
}

pub(crate) fn filter_signal(
    mut signals: Query<(&mut Signal, &LayoutFilter), Changed<Signal>>,
    layout_config: Res<Layout>,
) {
    for (mut signal, filter) in signals.iter_mut() {
        if filter.accepts(*layout_config) {
            *signal = Signal::default();
        }
    }
}
#[derive(Copy, Clone)]
pub enum FilterMode {
    None,
    Any,
}

#[derive(Copy, Clone)]
pub struct ActionHandle(pub(crate) Entity);
impl ActionHandle {
    pub fn value(&self) -> Entity {
        self.0
    }
}
#[derive(Bundle)]
pub(crate) struct ActionSignal<A: Command + Send + Sync + 'static + Clone> {
    signal: Signal,
    action: TriggeredAction<A>,
}
impl<A: Command + Send + Sync + 'static + Clone> ActionSignal<A> {
    pub(crate) fn new(a: A) -> Self {
        Self {
            signal: Default::default(),
            action: TriggeredAction(a),
        }
    }
}
#[derive(Component)]
pub(crate) struct TriggeredAction<A: Command + Send + Sync + 'static + Clone>(pub(crate) A);

pub(crate) fn engage_action<A: Command + Send + Sync + 'static + Clone>(
    mut actions: Query<(&mut Signal, &TriggeredAction<A>), Changed<Signal>>,
    mut cmd: Commands,
) {
    for (mut signal, action) in actions.iter_mut() {
        if signal.should_spawn() {
            cmd.add(action.0.clone());
            *signal = Signal::default();
        }
    }
}
