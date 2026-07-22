# The Render Backend: Ash

[Differential](./differential.md) queues changed attribute values per `(pipeline,
attribute)` pair, once per frame. Something has to drain all of those, resolve draw
order across entities from potentially different pipelines, and turn the result into
actual `wgpu` draw calls. That's `Ash`.

```rust
// foliage_proper/src/ash/mod.rs
pub(crate) struct Ash {
    pub(crate) nodes: Vec<Node>,
    pub(crate) contiguous: Vec<ContiguousSpan>,
    pub(crate) text: Option<Renderer<Text>>,
    pub(crate) panel: Option<Renderer<Panel>>,
    pub(crate) image: Option<Renderer<Image>>,
    pub(crate) icon: Option<Renderer<Icon>>,
    pub(crate) line: Option<Renderer<LineQuad>>,
    pub(crate) polygon: Option<Renderer<Polygon>>,
    pub(crate) elevation_order: Vec<Entity>,
    pub(crate) stack_key_cache: HashMap<Entity, StackKey>,
    ...
}
```

Six renderers, one per primitive pipeline -- every rendering primitive in the crate goes
through exactly one of these.

## `assign_elevations`: symbolic order to real depth

Authors write `Elevation::up(1)`/`Elevation::abs(0)` -- relative or absolute, symbolic
ordering, never a raw depth number. Something has to turn that into an actual
`ResolvedElevation` f32 the GPU can sort by, and it has to do it incrementally: a UI with
hundreds of entities can't afford to re-derive everyone's depth every time one entity's
`Elevation` changes. `assign_elevations` is a gapped/fractional-index scheme -- a changed
or newly-inserted entity is placed *between* its `StackKey`-sorted neighbors' already-assigned
values, touching only itself:

```rust
// foliage_proper/src/ash/mod.rs (abridged)
let new_value = match (left_v, right_v) {
    (Some(l), Some(r)) => (l + r) / 2.0,
    (Some(l), None) => l - initial_gap,
    (None, Some(r)) => r + initial_gap,
    (None, None) => (nf.near.value() + nf.far.value()) / 2.0,
};
```

Two edge cases force a full `renormalize` instead: a gap bisected so many times the
midpoint loses meaningful float precision, or a value landing exactly on the
[near/far](./willow.md) boundary. The boundary case has a real bug behind it, documented
in the source: repeatedly inserting new frontmost content (an overlay opening in front of
existing forward chrome, repeatedly) marches the frontmost value down toward `near`; once
it *reaches* `near`, the next insertion's `near - gap` used to clamp straight back to
`near` -- an identical depth to whatever was already there, resolved by non-deterministic
draw order. That's documented as the literal cause of "popover text sometimes renders
behind its own panel." Re-spacing on boundary contact keeps every depth distinct instead.

## `prepare`: draining the queues into nodes

```rust
// foliage_proper/src/ash/mod.rs (abridged)
pub(crate) fn prepare(&mut self, world: &mut World, ginkgo: &Ginkgo) {
    self.assign_elevations(world); // must run first -- prepare below reads ResolvedElevation
    let mut queues = RenderQueueHandle::new(world);
    let text_nodes = Render::prepare(self.text.as_mut().unwrap(), &mut queues, ginkgo);
    let panel_nodes = Render::prepare(self.panel.as_mut().unwrap(), &mut queues, ginkgo);
    // ... image, icon, line, polygon, same shape
}
```

Each pipeline's `Render::prepare` drains its own [`Differential`](./differential.md)
queues via `RenderQueueHandle::attribute::<R, RP>()` and returns the `Node`s that need
updating. `Ash` merges those into its own `nodes` list (replacing by `(pipeline, group,
instance_id)` identity, not blind appending), sorts by elevation then pipeline/group/clip
context/order, and groups adjacent same-pipeline/same-clip nodes into `ContiguousSpan`s --
the actual draw-call granularity.

## `render`: one pass, one scissor rect per span

```rust
// foliage_proper/src/ash/mod.rs (abridged)
for span in self.contiguous.iter() {
    let mut section = ginkgo.viewport().section();
    if let Some(clip) = self.clip.get(&span.clip_context) { section = section.intersection(...); }
    rpass.set_scissor_rect(...);
    match span.pipeline {
        PipelineId::Text => Render::render(self.text.as_mut().unwrap(), &mut rpass, parameters),
        PipelineId::Panel => Render::render(self.panel.as_mut().unwrap(), &mut rpass, parameters),
        // ...
    }
}
```

Clip is resolved per span from a `HashMap<Stem, ClipSection>` built earlier in `prepare`
from the `((), ResolvedClip)` differential -- `Ash::attach` registers that one directly
(`foliage.differential::<(), ResolvedClip>()`), since clip isn't owned by any one
pipeline the way `Color`/`Section` are owned by their widget type.

This is the last stop before pixels -- [Ginkgo](./ginkgo.md) is what `Render::prepare`/
`render` actually call into for buffers, pipelines, and the render pass itself.
