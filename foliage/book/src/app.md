# The App: Foliage and Its Schedules

Something has to own the pieces that don't belong to any one widget: the ECS `World`
itself, the window, the GPU device, and the ordering that ties a tick together. That's
`Foliage`:

```rust
// foliage_proper/src/foliage.rs
pub struct Foliage {
    pub world: World,
    pub(crate) main: Schedule,
    pub user: Schedule,
    pub(crate) diff: Schedule,
    pub(crate) willow: Willow,
    pub(crate) ginkgo: Ginkgo,
    pub(crate) ash: Ash,
    ...
}
```

## Three schedules, run in order, every tick

`main` is the library's own systems (input translation, animation stepping); `user` is
app code, added via `foliage.user.add_systems(...)`; `diff` is where
[differential](./differential.md) extraction happens. [Photosynthesis](./photosynthesis.md)
runs all three, in that fixed order, once per tick:

```rust
// foliage_proper/src/photosynthesis.rs
self.main.run(&mut self.world);
self.user.run(&mut self.world);
self.diff.run(&mut self.world);
```

`main` and `diff` are each internally ordered by a `SystemSet` enum, chained so each
phase fully completes (with an `ApplyDeferred` flush between phases) before the next
starts:

```rust
// foliage_proper/src/foliage.rs
pub(crate) enum MainMarkers { External, Animation, Process }
pub(crate) enum DiffMarkers { Prepare, Finalize, Extract }
```

Ordering `user` strictly between `main` and `diff` means: library input-handling runs
first, then app code sees a settled world to react to, then differential extraction sees
*everything* that changed that tick -- library and app writes alike -- in one pass.

## `Foliage::new()`: every subsystem attaches itself

Nothing in `Foliage::new()` special-cases any one widget type. Every subsystem
implements `Attachment` and registers itself:

```rust
// foliage_proper/src/foliage.rs
Disable::attach(&mut foliage);
Enable::attach(&mut foliage);
Panel::attach(&mut foliage);
Line::attach(&mut foliage);
...
Ash::attach(&mut foliage);
Text::attach(&mut foliage);
Asset::attach(&mut foliage);
...
```

`Panel::attach`, for instance, is exactly where `Panel`'s [`differential`](./differential.md)
registrations for `Section<Logical>`, `BlendedOpacity`, `Color`, `Outline`,
`ResolvedElevation`, and `ClipContext` all happen (`panel/mod.rs:159-173`) -- see
[Panel](./panel.md). This is why adding a new renderable field to an existing widget
never requires touching `foliage.rs` itself: `attach` is the widget's own responsibility.

## Running the app

`foliage.photosynthesize()` hands control to a `winit::EventLoop` and never returns
(on native) -- see [Photosynthesis](./photosynthesis.md) for what happens inside that
loop. `foliage.desktop_size(..)` and asset-loading calls (`store`, `load_asset`) are set
up before that call; `foliage.user.add_systems(..)` is how app code adds its own
per-tick systems.
