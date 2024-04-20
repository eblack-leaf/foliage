use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Component;
use bevy_ecs::system::{Commands, Query, Res};

use crate::animate::trigger::Trigger;
use crate::time::{Time, TimeDelta};

#[derive(Copy, Clone)]
pub enum TimerState {
    Running,
    Paused,
}
#[derive(Component, Clone)]
pub struct Timer {
    pub state: TimerState,
    pub duration: TimeDelta,
    pub target: Option<Entity>,
    pub trigger: Option<Trigger>,
}
impl Timer {
    pub fn new(duration: TimeDelta) -> Self {
        Self {
            state: TimerState::Paused,
            duration,
            target: None,
            trigger: None,
        }
    }
    pub fn on_end(mut self, target: Entity, trigger: Trigger) -> Self {
        self.target.replace(target);
        self.trigger.replace(trigger);
        self
    }
}
pub(crate) fn update(mut query: Query<(Entity, &mut Timer)>, mut cmd: Commands, time: Res<Time>) {
    for (entity, mut timer) in query.iter_mut() {
        let elapsed = time.frame_diff().min(TIME_SKIP_RESISTANCE);
        timer.duration = timer.duration.checked_sub(elapsed).unwrap_or_default();
        if timer.duration.is_zero() {
            // trigger on-end
            if let Some(e) = timer.target {
                cmd.entity(e)
                    .insert(timer.trigger.unwrap_or(Trigger::active()));
            }
            cmd.entity(entity).remove::<Timer>();
        }
    }
}

pub const TIME_SKIP_RESISTANCE: TimeDelta = TimeDelta::from_millis(42);
