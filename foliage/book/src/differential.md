# From Change to Pixels: Differential

Widget state lives in ordinary ECS components -- a `Panel`'s `Color`, its `Section`, its
`Outline`. Every tick, some of these change; most don't. If every renderable attribute on
every entity were re-uploaded to the GPU on every tick regardless, cost would scale with
total UI size instead of with how much actually changed. `Differential` is the layer that
prevents that.

## The cache

```rust
// foliage_proper/src/ash/differential.rs
pub(crate) struct Differential<R, RP> {
    pub(crate) cache: Option<RP>,
    _phantom: PhantomData<R>,
}
impl<R, RP: PartialEq> Differential<R, RP> {
    pub(crate) fn different(&mut self, packet: RP) -> bool {
        let mut different = false;
        if let Some(cached) = self.cache.as_ref() {
            if cached != &packet { different = true; }
        } else {
            different = true;
        }
        self.cache.replace(packet);
        different
    }
}
```

`R` is the renderer/pipeline the value belongs to (`Panel`, `Text`, ...); `RP` is the
specific attribute type (`Color`, `Section<Logical>`, ...). Each renderable component
`#[require]`s one `Differential<Self, X>` per attribute it cares about -- e.g. `Panel`
requires `Differential<Self, Color>`, `Differential<Self, ResolvedElevation>`,
`Differential<Self, ClipContext>`, and so on (`panel/mod.rs:26-32`).

## The system: `cached_differential`

One generic system, instantiated per `(R, RP)` pair, does the actual comparison:

```rust
// foliage_proper/src/ash/differential.rs (abridged)
pub(crate) fn cached_differential<R, RP>(
    mut values: ParamSet<(Query<(Entity, &RP), (Changed<RP>, With<Differential<R, RP>>)>, Query<&RP>)>,
    mut caches: Query<&mut Differential<R, RP>>,
    mut visibility: ParamSet<(Query<&ResolvedVisibility>, Query<Entity, (Changed<ResolvedVisibility>, With<Differential<R, RP>>)>)>,
    mut queue: ResMut<RenderQueue<R, RP>>,
) {
    // visibility just turned on: re-send the current value, even if it didn't itself change
    // (an entity mid-teardown can be missing its attribute or cache in the same frame its
    // visibility changes -- skipped, not treated as an error)
    ...
    // value actually changed *and* is visible: compare against cache, queue if different
    ...
}
```

Two cases matter here, and the doc comment on the function spells out why both exist:
a value that changed while invisible shouldn't be sent (nothing to draw), but a value
that *didn't* change while visibility flips back on still needs to be re-sent once,
because the renderer may have dropped it while it was hidden.

## Registration

`Foliage::differential::<R, RT>()` is what wires a type pair into this system:

```rust
// foliage_proper/src/foliage.rs
pub(crate) fn differential<R, RT>(&mut self) {
    self.world.insert_resource(RenderQueue::<R, RT>::new());
    self.diff.add_systems(cached_differential::<R, RT>.in_set(DiffMarkers::Extract));
}
```

Every widget type that has render-relevant fields calls this once per field during its
own `Attachment::attach` (see `Panel::attach` in [Panel](./panel.md) for a concrete
example) -- there's no central registry of "everything renderable"; each type declares
its own differentials.

## The drain: `RenderQueueHandle`

The `Extract` phase (see [The App](./app.md) for where that fits among the schedules)
populates a `RenderQueue<R, RP> { queue: HashMap<Entity, RP> }` resource per pair. On the
render side, [`Ash::prepare`](./ash.md) drains each one exactly once per frame via
`RenderQueueHandle::attribute::<R, RP>()`, which takes the whole map and clears it --
whatever wasn't queued this frame simply isn't touched, which is the entire performance
argument: cost is proportional to what changed, not to what exists.

This is the seam between ECS state (this chapter) and the actual rendering backend --
[Ash](./ash.md), [Ginkgo](./ginkgo.md), and [Photosynthesis](./photosynthesis.md) cover
what happens on the other side of that drain.
