# Animation

An app doesn't build an `Animation` directly -- it calls
[`Grows::animate`](./canopy.md) with a [`Motion`](./spawning.md) and a
[`Timing`](./spawning.md). What follows is how that request actually runs underneath,
and the two pieces of it (`Ease`, `Repeat`) an app configures directly through `Timing`.

## `Ease` and `Repeat`: the author-facing knobs

```rust
// foliage_proper/src/anim/mod.rs
pub enum Repeat {
    Once,
    Times(u32),
    Forever,
}
```

`Ease` is a curve function over the `0.0..=1.0` progress fraction -- `Ease::DECELERATE`
is `Timing`'s default. `Repeat` is how many times a tween replays after its first pass;
paired with `Timing::backtrack()`, a replay can run back the way it came (`A -> B -> A ->
B`) instead of snapping to the start and repeating forward. Which one reads right depends
on the values, not on taste: a `0`-to-`360` rotation wants the snap, since the ends look
identical; a `0.4`-to-`1.0` opacity pulse wants `backtrack`, since snapping back is a jump
the eye catches every cycle.

## `Animate`: engine-internal, one contract per animatable type

```rust
// foliage_proper/src/anim/mod.rs
pub trait Animate
where
    Self: Sized + Send + Sync + 'static + Clone,
{
    fn interpolations(start: &Self, end: &Self) -> Interpolations;
    fn apply(&mut self, interpolations: &mut Interpolations);
}
```

`interpolations` decomposes a start/end pair into a flat list of `f32` lerp channels
(`Color` uses four, one per r/g/b/a; `Outline` uses one, its width); `apply` reassembles
a live-interpolated value from wherever those channels currently sit. Registration is one
call per type, `Foliage::enable_animation::<Outline>()` (`pub(crate)`; the built-in
animatable types do this themselves), which adds one generic `animate::<A>` system rather
than a per-type one.

`Motion`'s variants are exactly the `Animate` types the engine exposes across the
boundary. An app never implements `Animate` for its own component, because it has no
component of its own for the engine to animate -- everything an app can grow and change
already goes through [`Grows`](./canopy.md).

## `Animation<A>`: the internal builder

```rust
// foliage_proper/src/anim/mod.rs
pub struct Animation<A: Animate> {
    anim_target: Option<Entity>,
    a: A,
    sequence_time_range: SequenceTimeRange,
    ease: Ease,
    seq: Entity,
}
impl<A: Animate> Animation<A> {
    pub fn new(a: A) -> Self { .. }
    pub fn targeting(mut self, lh: Entity) -> Self { .. }
    pub fn start(mut self, s: u64) -> Self { .. }   // ms
    pub fn finish(mut self, e: u64) -> Self { .. }  // ms
    pub fn eased(mut self, ease: Ease) -> Self { .. }
    pub fn during(mut self, seq: Entity) -> Self { .. }
}
```

`a` is the **end** value -- `Animation::new(Color::gray(900))` means "animate to this
color," starting from whatever the target's current value is. This is what
[`Grows::animate`](./canopy.md) builds and runs on an app's behalf, through
[`Tree::animate`](./tree.md), which spawns an `AnimationRunner<A>` entity that drives the
interpolation tick by tick against [`Time`](./time.md)'s frame delta, writing the
interpolated value onto the target every frame until it finishes.

## Sequencing: `during(seq)`

`seq` above is a sequence entity, opened with `Tree::spawn_sequence`/`Tree::sequence_at`
-- animations sharing one still freely overlap in their own `start`/`finish` windows; a
sequence groups them only for the purpose of knowing when *all* of them are done. Full
mechanism (the `SequenceMarker` counter, why the sequence entity self-despawns) is in
[Inside the Engine](./tree.md); across the boundary this is
[`Grows::sequence`/`Grows::animate_during`](./canopy.md) and
[`Bloom::SequenceFinished`](./canopy.md).
