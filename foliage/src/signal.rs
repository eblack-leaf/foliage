use crate::differential::{RenderLink, RenderRemoveQueue};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Bundle, Changed, Commands, Component, Query, ResMut};
#[derive(Component, Default)]
pub struct Signal {
    message: i32,
}
impl Signal {
    pub fn neutral(&self) -> bool {
        self.message == 0
    }
    pub fn low(&self) -> bool {
        self.message == 1
    }
    pub fn high(&self) -> bool {
        self.message == 2
    }
}
#[derive(Component)]
pub struct TriggerTarget(pub Entity);
#[derive(Component)]
pub struct TriggeredBundle<B: Bundle + 'static + Send + Sync + Clone>(pub B);
pub(crate) fn signaled_clean(
    mut to_clean: Query<(&mut Signal, &TriggerTarget), Changed<Signal>>,
    mut cmd: Commands,
) {
    for (mut signal, target) in to_clean.iter_mut() {
        if signal.low() {
            cmd.entity(target.0).insert(Clean::should_clean());
        }
    }
}
pub(crate) fn clear_signal(mut signals: Query<(&mut Signal), Changed<Signal>>) {
    for mut signal in signals.iter_mut() {
        *signal = Signal::default();
    }
}
pub(crate) fn signaled_spawn<B: Bundle + 'static + Send + Sync + Clone>(
    to_spawn: Query<(&Signal, &TriggeredBundle<B>, &TriggerTarget), Changed<Signal>>,
    mut cmd: Commands,
) {
    for (signal, bundle, target) in to_spawn.iter() {
        if signal.high() {
            cmd.entity(target.0).insert(bundle.0.clone());
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
            cmd.entity(entity).retain::<Clean>();
        }
    }
}
