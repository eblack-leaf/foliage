use crate::Attachment;
use crate::foliage::{Foliage, MainMarkers};
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{ResMut, Resource};
use bevy_ecs::system::{Query, Res};

/// A point in time. `web_time`'s instant rather than `std`'s, so the same code keeps
/// working on wasm, where `std::time::Instant` panics.
pub type Moment = web_time::Instant;
/// A span of time, on the same `web_time` basis as [`Moment`].
pub type TimeDelta = web_time::Duration;
/// A stamp of [`Time`]'s accumulated total, for measuring a span across frames with
/// [`Time::time_since`]. Based on accumulated time rather than wall clock, so it inherits
/// the same stall clamping.
#[allow(unused)]
pub struct TimeMarker(pub(crate) TimeDelta);
impl TimeMarker {
    /// Accumulated time when this marker was taken.
    #[allow(unused)]
    pub fn since_beginning(&self) -> TimeDelta {
        self.0
    }
}
/// The frame clock: how long the last frame took, and how much time has accumulated.
///
/// Advanced once per frame. Read [`frame_diff`](Time::frame_diff) to make motion
/// frame-rate independent rather than per-tick.
#[derive(Resource)]
pub struct Time {
    total: TimeDelta,
    last: Moment,
    frame_diff: TimeDelta,
    fps_time: TimeDelta,
    fps_count: i32,
}
impl Time {
    /// Ceiling on a single frame's measured length, in milliseconds. A real gap -- a
    /// debugger pause, a backgrounded tab, a dropped frame -- would otherwise arrive as
    /// one enormous `frame_diff` and teleport everything mid-animation. Clamping trades
    /// exact elapsed time for continuity.
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
            self.fps_count = 0;
            self.fps_time = TimeDelta::default();
        }
        self.last = now;
    }
    /// Stamps the current accumulated total, to measure against later.
    #[allow(unused)]
    pub fn mark(&self) -> TimeMarker {
        TimeMarker(self.total)
    }
    /// Accumulated time since `mark` was taken.
    #[allow(unused)]
    pub fn time_since(&self, mark: TimeMarker) -> TimeDelta {
        self.total - mark.0
    }
    /// How long the last frame took, clamped by
    /// [`TIME_SKIP_RESISTANCE_FACTOR`](Self::TIME_SKIP_RESISTANCE_FACTOR). Scale
    /// per-frame motion by this so it runs at the same speed at any frame rate.
    pub fn frame_diff(&self) -> TimeDelta {
        self.frame_diff
    }
}
/// Advances the frame clock, once per frame.
pub(crate) fn update_time(mut time: ResMut<Time>) {
    time.update();
}
/// Fired at a [`Timer`] entity when it runs out, and at a
/// [`Sequence`](crate::Sequence) when its last animation finishes -- the hook for
/// chaining one stage of motion onto the next.
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub struct OnEnd {}
/// A countdown that triggers [`OnEnd`] and then despawns its own entity.
///
/// One-shot by construction: the entity is gone once it fires, so a repeating timer means
/// spawning a fresh one from the handler.
#[derive(Component)]
pub struct Timer {
    time_left: TimeDelta,
}
impl Timer {
    /// A countdown of `time_left`, ticked down by each frame's own
    /// [`frame_diff`](Time::frame_diff).
    #[allow(unused)]
    pub fn new(time_left: TimeDelta) -> Self {
        Self { time_left }
    }
}
/// Ticks every [`Timer`] down, firing [`OnEnd`] and despawning the ones that reach zero.
pub(crate) fn timers(
    time: Res<Time>,
    mut timers: Query<(Entity, &mut Timer)>,
    mut cmd: crate::Tree,
) {
    for (entity, mut timer) in timers.iter_mut() {
        timer.time_left = timer
            .time_left
            .checked_sub(time.frame_diff())
            .unwrap_or_default();
        if timer.time_left.is_zero() {
            cmd.send_to(
                OnEnd {
                    entity: Entity::PLACEHOLDER,
                },
                entity,
            );
            cmd.despawn(entity);
        }
    }
}
impl Attachment for Time {
    fn attach(foliage: &mut Foliage) {
        use bevy_ecs::prelude::IntoScheduleConfigs;
        let mut time = Time::new();
        time.start();
        foliage.world.insert_resource(time);
        foliage
            .main
            .add_systems((update_time, timers).chain().in_set(MainMarkers::External));
    }
}
