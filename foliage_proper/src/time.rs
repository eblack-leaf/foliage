use crate::Attachment;
use crate::EcsExtension;
use crate::foliage::{Foliage, MainMarkers};
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::event::Event;
use bevy_ecs::prelude::{ResMut, Resource};
use bevy_ecs::system::{Commands, Query, Res};

pub type Moment = web_time::Instant;
pub type TimeDelta = web_time::Duration;
#[allow(unused)]
pub struct TimeMarker(pub(crate) TimeDelta);
impl TimeMarker {
    #[allow(unused)]
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
    #[allow(unused)]
    pub fn mark(&self) -> TimeMarker {
        TimeMarker(self.total)
    }
    #[allow(unused)]
    pub fn time_since(&self, mark: TimeMarker) -> TimeDelta {
        self.total - mark.0
    }
    pub fn frame_diff(&self) -> TimeDelta {
        self.frame_diff
    }
}
pub(crate) fn update_time(mut time: ResMut<Time>) {
    time.update();
}
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub struct OnEnd {}
#[derive(Component)]
pub struct Timer {
    time_left: TimeDelta,
}
impl Timer {
    #[allow(unused)]
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
            cmd.trigger_targets(
                OnEnd {
                    entity: Entity::PLACEHOLDER,
                },
                entity,
            );
            cmd.entity(entity).despawn();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EcsExtension;
    use bevy_ecs::observer::On;

    #[test]
    fn frame_diff_after_a_large_gap_is_clamped_to_the_skip_resistance_factor() {
        // simulating a stall (a debugger pause, a dropped frame, a backgrounded tab) rather
        // than sleeping real wall-clock time in a test -- `last` is `pub(crate)`-reachable
        // from here specifically so this can be deterministic instead of flaky.
        let mut time = Time::new();
        time.last = Moment::now() - TimeDelta::from_secs(1);
        time.update();

        assert!(
            time.frame_diff() <= TimeDelta::from_millis(Time::TIME_SKIP_RESISTANCE_FACTOR),
            "a full second of real gap must be clamped down to the resistance factor, not \
             passed through and causing every animation to jump"
        );
    }

    #[test]
    fn total_accumulates_across_updates_by_the_clamped_frame_diff() {
        let mut time = Time::new();
        time.last = Moment::now() - TimeDelta::from_millis(5);
        time.update();
        let after_first = time.total;
        assert!(after_first > TimeDelta::ZERO);

        time.last = Moment::now() - TimeDelta::from_millis(5);
        time.update();

        assert!(time.total > after_first, "total should keep accumulating, not reset per update");
    }

    #[test]
    fn a_timer_with_zero_time_left_fires_on_end_and_despawns_on_the_first_tick() {
        let mut foliage = Foliage::new();
        let entity = foliage.world.spawn(Timer::new(TimeDelta::ZERO)).id();
        foliage.world.flush();

        foliage.main.run(&mut foliage.world);

        assert!(foliage.world.get_entity(entity).is_err(), "should have despawned itself");
    }

    #[test]
    fn a_timer_with_time_left_survives_a_single_quick_tick() {
        let mut foliage = Foliage::new();
        let entity = foliage
            .world
            .spawn(Timer::new(TimeDelta::from_secs(100)))
            .id();
        foliage.world.flush();

        foliage.main.run(&mut foliage.world);

        assert!(
            foliage.world.get_entity(entity).is_ok(),
            "a single headless tick's real elapsed time is nowhere near 100 seconds"
        );
    }

    #[test]
    fn a_timer_reaching_zero_triggers_on_end_before_despawning() {
        // the fired entity won't survive to be queried afterward, so the flag has to live
        // somewhere that outlives the despawn -- a Resource, updated by a *global* observer
        // (`world.add_observer`, not tied to any one entity), rather than a component on the
        // entity itself.
        #[derive(Resource, Default)]
        struct Fired(bool);

        fn mark(_trigger: On<OnEnd>, mut fired: ResMut<Fired>) {
            fired.0 = true;
        }

        let mut foliage = Foliage::new();
        foliage.world.insert_resource(Fired::default());
        foliage.world.add_observer(mark);
        let entity = foliage.world.spawn(Timer::new(TimeDelta::ZERO)).id();
        foliage.world.flush();

        foliage.main.run(&mut foliage.world);

        assert!(foliage.world.get_entity(entity).is_err(), "sanity: despawned");
        assert!(foliage.world.resource::<Fired>().0, "OnEnd should have fired before the despawn");
    }
}
