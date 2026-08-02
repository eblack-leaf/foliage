# Time

Every animation ([Animation](./anim.md)) and every `Timer` needs a per-tick delta that's
resilient to real-world scheduling hiccups -- a debugger pause, a backgrounded browser
tab, a dropped frame -- without every consumer having to defend against a giant delta
individually.

```rust
// foliage_proper/src/time.rs
pub struct Time { total: TimeDelta, last: Moment, frame_diff: TimeDelta, ... }
impl Time {
    pub(crate) const TIME_SKIP_RESISTANCE_FACTOR: u64 = 33; // ms
    pub(crate) fn update(&mut self) {
        let now = Moment::now();
        self.frame_diff = (now - self.last).min(TimeDelta::from_millis(Self::TIME_SKIP_RESISTANCE_FACTOR));
        self.total += self.frame_diff;
        self.last = now;
    }
}
```

`frame_diff` is clamped to 33ms regardless of how much real wall-clock time actually
passed -- a full second of stall clamps down to one frame's worth, rather than being
passed straight through and causing every running animation to jump instantly to wherever
that much real elapsed time would have carried it. The crate's own test
(`frame_diff_after_a_large_gap_is_clamped_to_the_skip_resistance_factor`) simulates
exactly this: fast-forwarding `last` by a full second and asserting the resulting
`frame_diff` still respects the clamp, deterministically -- not by actually sleeping
real time in the test.

## `Timer`: a self-despawning entity, not a callback

```rust
// foliage_proper/src/time.rs
pub(crate) fn timers(time: Res<Time>, mut timers: Query<(Entity, &mut Timer)>, mut cmd: Commands) {
    for (entity, mut timer) in timers.iter_mut() {
        timer.time_left = timer.time_left.checked_sub(time.frame_diff()).unwrap_or_default();
        if timer.time_left.is_zero() {
            cmd.trigger_targets(OnEnd { entity: Entity::PLACEHOLDER }, entity);
            cmd.entity(entity).despawn();
        }
    }
}
```

`Tree::timer(ms, observer)` (engine-internal; see [Inside the Engine](./tree.md)) spawns
a bare `Timer` entity and observes `OnEnd` on it -- when the countdown reaches zero,
`OnEnd` fires *before* the entity despawns (verified by its own test,
`a_timer_reaching_zero_triggers_on_end_before_despawning`), then the entity is gone. No
handle to cancel or a separate scheduler resource to manage -- the timer entity *is* the
whole lifecycle, and pruning it before it fires is cancellation, the same
[Remove](./lifecycle.md) mechanism every other teardown uses. Across the boundary this is
[`Grows::timer`](./canopy.md)/[`Bloom::TimerFinished`](./canopy.md).

## `Moment`/`TimeDelta`: not `std::time`

Both are `web_time` types, not `std::time::Instant`/`Duration` -- `std::time::Instant`
doesn't exist on `wasm32-unknown-unknown` (there's no OS clock syscall to back it), so
`web_time` provides the same API backed by the browser's `performance.now()` there and
plain `std::time` natively, letting the rest of the crate use one type across every
platform without its own `cfg` branch per call site.
