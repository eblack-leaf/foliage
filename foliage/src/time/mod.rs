mod timer;

use crate::elm::config::{CoreSet, ElmConfiguration};
use crate::elm::leaf::{EmptySetDescriptor, Leaf};
use crate::elm::Elm;
use bevy_ecs::prelude::{IntoSystemConfigs, Resource};
use bevy_ecs::system::ResMut;
use std::ops::{Add, AddAssign, Div, Sub, SubAssign};

#[derive(Resource)]
pub struct Time {
    #[cfg(not(target_family = "wasm"))]
    pub(crate) beginning: std::time::Instant,
    #[cfg(target_family = "wasm")]
    pub(crate) beginning: f64,
    pub(crate) current: f64,
    pub(crate) last: f64,
}
impl Time {
    pub fn milli_to_sec(ms: f64) -> f64 {
        ms / 1000.0
    }
    pub(crate) fn new() -> Self {
        #[cfg(target_family = "wasm")]
        let wasm_beginning = match web_sys::window().unwrap().performance() {
            Some(perf) => perf.now(),
            None => 0.0,
        };
        Self {
            #[cfg(not(target_family = "wasm"))]
            beginning: std::time::Instant::now(),
            #[cfg(target_family = "wasm")]
            beginning: Self::milli_to_sec(wasm_beginning),
            current: 0.0,
            last: 0.0,
        }
    }
    /// get the current time as a marker to now
    pub fn mark(&self) -> TimeMarker {
        TimeMarker(self.current)
    }
    /// return the time since a marker
    pub fn time_since(&self, marker: TimeMarker) -> TimeDelta {
        TimeDelta(self.current - marker.0)
    }
    /// how long it has been since the last frame
    pub fn frame_diff(&self) -> TimeDelta {
        TimeDelta(self.current - self.last)
    }

    pub(crate) fn read(&mut self) -> TimeDelta {
        self.last = self.current;
        self.set_to_now();
        self.frame_diff()
    }

    pub(crate) fn set_to_now(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.current = std::time::Instant::now()
                .duration_since(self.beginning)
                .as_secs_f64();
        }
        #[cfg(target_arch = "wasm32")]
        {
            let now = match web_sys::window().expect("no window").performance() {
                Some(perf) => perf.now(),
                None => self.last,
            };
            self.current = Self::milli_to_sec(now) - self.beginning;
        }
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
        elm.main()
            .add_systems((read_time.in_set(CoreSet::ExternalEvent),));
    }
}
/// signifies a point in time
#[derive(PartialOrd, PartialEq, Copy, Clone, Debug)]
pub struct TimeMarker(pub f64);

impl TimeMarker {
    pub fn offset<TD: Into<TimeDelta>>(self, delta: TD) -> Self {
        Self(self.0 + delta.into().0)
    }
}
impl Sub for TimeMarker {
    type Output = TimeDelta;

    fn sub(self, rhs: Self) -> Self::Output {
        TimeDelta::from(self.0 - rhs.0)
    }
}
/// signifies a change in time
#[derive(PartialOrd, PartialEq, Copy, Clone, Default, Debug)]
pub struct TimeDelta(pub f64);

impl TimeDelta {
    pub fn as_f32(&self) -> f32 {
        self.0 as f32
    }
}
impl From<f64> for TimeDelta {
    fn from(value: f64) -> Self {
        TimeDelta(value)
    }
}
impl SubAssign for TimeDelta {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0
    }
}

impl Sub for TimeDelta {
    type Output = TimeDelta;
    fn sub(self, rhs: Self) -> Self::Output {
        TimeDelta(self.0 - rhs.0)
    }
}

impl AddAssign for TimeDelta {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

impl Add for TimeDelta {
    type Output = TimeDelta;
    fn add(self, rhs: Self) -> Self::Output {
        TimeDelta(self.0 + rhs.0)
    }
}

impl Div for TimeDelta {
    type Output = TimeDelta;
    fn div(self, rhs: Self) -> Self::Output {
        TimeDelta(self.0 / rhs.0)
    }
}

impl From<f32> for TimeDelta {
    fn from(value: f32) -> Self {
        Self(value as f64)
    }
}
impl From<i32> for TimeDelta {
    fn from(value: i32) -> Self {
        Self(value as f64)
    }
}