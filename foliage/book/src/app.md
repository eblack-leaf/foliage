# The App: Foliage and Its Schedules

Something has to own the pieces that don't belong to any one widget: the ECS `World`
itself, the window, the GPU device, and the ordering that ties a tick together. That's
`Foliage`:

```rust
// foliage_proper/src/foliage.rs
pub struct Foliage {
    pub(crate) world: World,
    pub(crate) main: Schedule,
    pub(crate) diff: Schedule,
    pub(crate) frame: Option<Box<dyn FnMut(&mut Forest<'_, '_>)>>,
    pub(crate) ops: Vec<Op>,
    pub(crate) sprig: Sprig,
    pub(crate) willow: Willow,
    pub(crate) ginkgo: Ginkgo,
    pub(crate) ash: Ash,
    ...
}
```

Every field is `pub(crate)` -- an app never reaches the world, the schedules, or the
render backend directly. What it gets instead is the setup calls below, and the
[`Forest`](./forest.md) handed to the closure passed to
[`photosynthesize`](#running-the-app).

## Two schedules, plus the frame closure, run in order every tick

`main` is the library's own systems (input translation, animation stepping); `diff` is
where [differential](./differential.md) extraction happens. There is no schedule an app
adds its own systems to -- app code is not an ECS system at all, it's the plain closure
given to `photosynthesize`, run once per tick between the other two:

```rust
// foliage_proper/src/photosynthesis.rs
self.main.run(&mut self.world);
self.frame();   // drains Sprig, applies queued ops, runs the app's Forest closure, applies its ops
self.diff.run(&mut self.world);
```

Ordering it this way means: library input-handling runs first, so the frame closure sees
a settled world to react to; the closure's own commands are applied as soon as it
returns, so they take effect the same tick; and differential extraction runs last, seeing
*everything* that changed that tick -- library and app writes alike -- in one pass.

`main` and `diff` are each internally ordered by a `SystemSet` enum, chained so each
phase fully completes (with an `ApplyDeferred` flush between phases) before the next
starts:

```rust
// foliage_proper/src/foliage.rs
pub(crate) enum MainMarkers { External, Animation, Process }
pub(crate) enum DiffMarkers { Prepare, Finalize, Extract }
```

## `Foliage::new()`: every subsystem attaches itself

Nothing in `Foliage::new()` special-cases any one widget type. Every subsystem
implements `Attachment` (`pub(crate)` -- engine-internal setup, not something an app or
an external library can add to) and registers itself:

```rust
// foliage_proper/src/foliage.rs
crate::boundary::Boundary::attach(&mut foliage);
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
`ResolvedElevation`, and `ClipContext` all happen -- see [Panel](./panel.md). This is why
adding a new renderable field to an existing widget never requires touching `foliage.rs`
itself: `attach` is the widget's own responsibility.

## Running the app

`Foliage::new()` (or `Foliage::android(app)` on that platform) builds the instance. What
an app calls before handing off control is all `pub`, on `Foliage` directly:
`desktop_size(..)` requests an initial window size; `font(..)`/`icon(..)` register
artwork and faces that have to exist before anything drawn with them is grown; `tune(..)`
installs a [`Tuning`](./attachment.md) value; `asset_base(..)` declares the app's asset
hosting convention.

`foliage.photosynthesize(closure)` hands control to a `winit::EventLoop` and never
returns (on native) -- see [Photosynthesis](./photosynthesis.md) for what happens inside
that loop. `foliage.sprig()` -- callable any time before or after `photosynthesize` --
hands back a `Send`, cloneable [`Sprig`](./forest.md) for issuing commands from another
thread.
