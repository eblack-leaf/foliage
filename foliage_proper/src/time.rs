use crate::foliage::Foliage;
use crate::Attachment;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::prelude::{ResMut, Resource};
use bevy_ecs::system::{Commands, Query, Res};

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
    fps_time: TimeDelta,
    fps_count: i32,
}
impl Time {
    pub(crate) const TIME_SKIP_RESISTANCE_FACTOR: u64 = 33;
    pub(crate) fn new() -> Self {
        Self {
            total: Default::default(),
            last: Moment::now(),
            frame_diff: Default::default(),
            fps_time: Default::default(),
            fps_count: 0,
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
        self.fps_time += self.frame_diff;
        self.fps_count += 1;
        if self.fps_time >= TimeDelta::from_secs(1) {
            // println!("fps: {} @ {:?}", self.fps_count, self.fps_time);
            self.fps_count = 0;
            self.fps_time = TimeDelta::default();
        }
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
#[derive(Event, Copy, Clone, Default)]
pub struct OnEnd {}
#[derive(Component)]
pub struct Timer {
    time_left: TimeDelta,
}
impl Timer {
    pub fn new(time_left: TimeDelta) -> Self {
        Self { time_left }
    }
}
pub(crate) fn timers(time: Res<Time>, mut timers: Query<(Entity, &mut Timer)>, mut cmd: Commands) {
    for (entity, mut timer) in timers.iter_mut() {
        timer.time_left = timer
            .time_left
            .checked_sub(time.frame_diff())
            .unwrap_or_default();
        if timer.time_left.is_zero() {
            cmd.trigger_targets(OnEnd {}, entity);
            cmd.entity(entity).despawn();
        }
    }
}
impl Attachment for Time {
    fn attach(foliage: &mut Foliage) {
        use bevy_ecs::prelude::IntoSystemConfigs;
        let mut time = Time::new();
        time.start();
        foliage.world.insert_resource(time);
        foliage.main.add_systems((update_time, timers).chain());
    }
}
