use crate::time::{TimeDelta, TimeMarker};
use bevy_ecs::prelude::Component;

#[derive(Component, Copy, Clone, Default, Debug)]
pub struct Timer {
    pub start: Option<TimeMarker>,
    pub start_offset: Option<TimeDelta>,
    pub interval: TimeDelta,
    pub last_mark: Option<TimeMarker>,
}
impl Timer {
    pub fn with_start_offset<TD: Into<TimeDelta>>(mut self, start_offset: TD) -> Self {
        self.start_offset.replace(start_offset.into());
        self
    }
    pub fn new<TD: Into<TimeDelta>>(interval: TD) -> Self {
        let interval = interval.into();
        Self {
            start: None,
            start_offset: None,
            interval,
            last_mark: None,
        }
    }
    pub fn mark<TM: Into<TimeMarker>>(&mut self, mark: TM) {
        self.last_mark.replace(mark.into());
    }
    pub fn percent_elapsed<TD: Into<TimeDelta>>(&self, delta: TD) -> f32 {
        (delta.into() / self.interval).as_f32()
    }
    pub fn time_elapsed(&self) -> Option<TimeDelta> {
        if let Some(start) = self.start {
            if let Some(last_mark) = self.last_mark {
                let diff = last_mark - start;
                if diff.0.is_sign_positive() {
                    return Option::from(diff);
                }
            }
        }
        None
    }
    pub fn overage(&self) -> Option<TimeDelta> {
        if self.finished() {
            return Some(self.time_elapsed().unwrap() - self.interval);
        }
        None
    }
    pub fn finished(&self) -> bool {
        if let Some(elapsed) = self.time_elapsed() {
            if elapsed > self.interval {
                return true;
            }
        }
        false
    }
    pub fn set_interval<TD: Into<TimeDelta>>(&mut self, interval: TD) {
        self.interval = interval.into();
    }
    pub fn set_offset<TD: Into<TimeDelta>>(&mut self, offset: Option<TD>) {
        match offset {
            None => self.start_offset.take(),
            Some(off) => self.start_offset.replace(off.into()),
        };
    }
    pub fn start<TM: Into<TimeMarker>>(&mut self, now: TM) {
        self.start.replace(
            now.into()
                .offset(self.start_offset.take().unwrap_or_default()),
        );
    }
    pub fn start_with_offset<TM: Into<TimeMarker>>(&mut self, now: TM, offset: Option<TimeDelta>) {
        self.set_offset(offset);
        self.start(now);
    }
    pub fn reset(&mut self) {
        self.start.take();
        self.last_mark.take();
    }
    pub fn not_started(&self) -> bool {
        self.start.is_none()
    }
    pub fn started(&self) -> bool {
        self.start.is_some()
    }
}
