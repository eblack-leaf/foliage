use bevy_ecs::prelude::{IntoSystemConfigs, Resource};
use bevy_ecs::system::ResMut;

use crate::elm::config::{CoreSet, ElmConfiguration, ExternalSet};
use crate::elm::leaf::{EmptySetDescriptor, Leaf};
use crate::elm::Elm;

#[allow(unused)]
pub mod timer;

pub type TimeMarker = web_time::Instant;
pub type TimeDelta = web_time::Duration;
#[derive(Resource)]
pub struct Time {
    pub(crate) beginning: TimeMarker,
    pub(crate) current: TimeMarker,
    pub(crate) last: TimeMarker,
}
impl Time {
    pub fn milli_to_sec(ms: f64) -> f64 {
        ms / 1000.0
    }
    pub(crate) fn new() -> Self {
        Self {
            beginning: TimeMarker::now(),
            current: TimeMarker::now(),
            last: TimeMarker::now(),
        }
    }
    /// get the current time as a marker to now
    pub fn mark(&self) -> TimeMarker {
        self.current
    }
    /// return the time since a marker
    pub fn time_since(&self, marker: TimeMarker) -> TimeDelta {
        TimeDelta::from(self.current - marker)
    }
    pub fn total_time(&self) -> TimeDelta {
        TimeDelta::from(TimeMarker::now() - self.beginning)
    }
    /// how long it has been since the last frame
    pub fn frame_diff(&self) -> TimeDelta {
        let val = TimeDelta::from(self.current - self.last);
        tracing::trace!("frame-diff: {:?}", val);
        val
    }

    pub(crate) fn read(&mut self) -> TimeDelta {
        self.last = self.current;
        self.set_to_now();
        self.frame_diff()
    }

    pub(crate) fn set_to_now(&mut self) {
        self.current = TimeMarker::now();
    }
}
fn start_time(mut time: ResMut<Time>) {
    time.set_to_now();
}
fn read_time(mut time: ResMut<Time>) {
    let _ = time.read();
}
impl Leaf for Time {
    type SetDescriptor = EmptySetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        elm.job.container.insert_resource(Time::new());
        elm.startup().add_systems(start_time);
        elm.main().add_systems((
            read_time.in_set(CoreSet::ExternalEvent),
            timer::update.in_set(ExternalSet::Animation),
        ));
    }
}
