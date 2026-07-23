# Animation

Any component that implements `Animate` can be animated the same way, through the same
call:

```rust
// foliage_proper/src/anim/mod.rs
pub trait Animate where Self: Sized + Send + Sync + 'static + Clone {
    fn interpolations(start: &Self, end: &Self) -> Interpolations;
    fn apply(&mut self, interpolations: &mut Interpolations);
}
```

`interpolations` decomposes a start/end pair into a flat list of `f32` lerp channels
(`Color` uses four, one per r/g/b/a; `Outline` uses one, its width; `Elevation` uses
one, its amount); `apply` reassembles a live-interpolated value from wherever those
channels currently sit. This is the entire contract -- a new animatable component only
ever needs these two functions, not a bespoke tweening system of its own. Registration is
one call per type: `foliage.enable_animation::<Outline>()` (see [Panel](./panel.md)'s
`Attachment`), which adds one generic `animate::<A>` system rather than a
per-type system.

## `Animation<A>`: the builder

```rust
// foliage_proper/src/anim/mod.rs
pub struct Animation<A: Animate> { anim_target: Option<Entity>, a: A, sequence_time_range: SequenceTimeRange, ease: Ease, seq: Entity }
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
color," starting from whatever the target's current value is. `tree.animate(anim)` (via
[`EcsExtension`](./tree.md)) spawns an `AnimationRunner<A>` entity that drives the
interpolation tick by tick against [`Time`](./time.md)'s frame delta, writing the
interpolated value onto the target every frame until it finishes.

## `Sequence`: grouping without recomputing timing

`Sequence::new(tree).animate(a1).animate(a2).end(on_finish)` (covered in
[Tree and Graft](./tree.md)) is the ergonomic layer over `during(seq)` -- animations in
a sequence can still freely overlap in their own `start`/`finish` windows; grouping them
only removes the repeated `tree.animate(...).during(seq)` boilerplate per line, it
doesn't compute or infer relative timing for you.

## Ease

`Ease::DECELERATE` is the default (`Animation::new`'s own default). Eases are pure
`t -> t'` curve functions over the `0.0..=1.0` progress fraction, applied uniformly
across every interpolation channel `interpolations()` produced -- so an eased color
animation and an eased position animation share the exact same curve math, just fed
different channel counts.
