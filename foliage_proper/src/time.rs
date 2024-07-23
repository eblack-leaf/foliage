use bevy_ecs::prelude::{IntoSystemConfigs, ResMut, Resource};

use crate::elm::{Elm, ScheduleMarkers};
use crate::Leaf;

pub type Moment = web_time::Instant;
pub type TimeDelta = web_time::Duration;
pub struct TimeMarker(pub(crate) TimeDelta);
impl TimeMarker {
    pub fn since_beginning(&self) -> TimeDelta {
        self.0
    }
}
#[derive(Resource)]
pub struct Time {
    total: TimeDelta,
    last: Moment,
    frame_diff: TimeDelta,
}
impl Time {
    pub(crate) const TIME_SKIP_RESISTANCE_FACTOR: u64 = 33;
    pub(crate) fn new() -> Self {
        Self {
            total: Default::default(),
            last: Moment::now(),
            frame_diff: Default::default(),
        }
    }
    pub(crate) fn start(&mut self) {
        self.last = Moment::now();
    }
    pub(crate) fn update(&mut self) {
        let now = Moment::now();
        self.frame_diff =
            (now - self.last).min(TimeDelta::from_millis(Self::TIME_SKIP_RESISTANCE_FACTOR));
        self.total += self.frame_diff;
        self.last = now;
    }
    pub fn mark(&self) -> TimeMarker {
        TimeMarker(self.total)
    }
    pub fn time_since(&self, mark: TimeMarker) -> TimeDelta {
        self.total - mark.0
    }
    pub fn frame_diff(&self) -> TimeDelta {
        self.frame_diff
    }
}
pub(crate) fn start(mut time: ResMut<Time>) {
    time.start();
}
pub(crate) fn update_time(mut time: ResMut<Time>) {
    time.update();
}
impl Leaf for Time {
    fn attach(elm: &mut Elm) {
        elm.ecs.world.insert_resource(Time::new());
        elm.scheduler.startup.add_systems(start);
        elm.scheduler
            .main
            .add_systems(update_time.in_set(ScheduleMarkers::External));
    }
}
