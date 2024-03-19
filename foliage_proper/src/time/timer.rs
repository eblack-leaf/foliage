use crate::animate::ANIMATE_SKIP_RESISTANCE;
use crate::time::{Time, TimeDelta};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Component;
use bevy_ecs::system::{Commands, Query, Res};
#[derive(Copy, Clone)]
pub enum TimerState {
    Running,
    Paused,
}
#[derive(Component, Clone)]
pub struct Timer {
    pub state: TimerState,
    pub duration: TimeDelta,
}
impl Timer {
    pub fn new(duration: TimeDelta) -> Self {
        Self {
            state: TimerState::Paused,
            duration,
        }
    }
}
fn update(mut query: Query<(Entity, &mut Timer)>, mut cmd: Commands, time: Res<Time>) {
    for (entity, mut timer) in query.iter_mut() {
        let elapsed = time
            .frame_diff()
            .min(TimeDelta::from_millis(ANIMATE_SKIP_RESISTANCE));
        timer.duration = timer.duration.checked_sub(elapsed).unwrap_or_default();
        if timer.duration.is_zero() {
            // trigger on-end
            cmd.entity(entity).remove::<Timer>();
        }
    }
}
