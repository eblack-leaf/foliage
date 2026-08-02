# Canopy: The Frame Surface

Once [`Foliage::photosynthesize`](./app.md) is called, an app lives entirely inside one
closure, called once per frame:

```rust
// foliage/examples/interaction.rs
let mut foliage = Foliage::new();
foliage.desktop_size((360, 220));

let mut demo: Option<Demo> = None;
foliage.photosynthesize(move |canopy: &mut Canopy| {
    let demo = demo.get_or_insert_with(|| grow(canopy));
    for bloom in canopy.take() {
        // ...
    }
});
```

There is no separate setup phase that builds an initial tree before the loop starts --
the opening screen is just the first frame's `get_or_insert_with`. Everything an app
does, from the very first element it grows to the last one it prunes, goes through the
`&mut Canopy` this closure is handed.

## What crosses the seam

Canopy is a read-only bundle of engine state plus a command queue, not a window onto the
`bevy_ecs` world itself:

```rust
// foliage_proper/src/boundary/canopy.rs
pub struct Canopy<'w, 's> {
    pub(crate) reads: Reads<'w, 's>,
    pub(crate) queue: &'w mut Vec<Op>,
    pub(crate) blooms: Vec<Bloom>,
    pub(crate) allocator: bevy_ecs::entity::RemoteAllocator,
}
```

Everything on it is plain data: commands queued in ([`Op`](./spawning.md), never named by
app code directly -- see [`Grows`](#grows-the-command-vocabulary) below), emissions taken
out ([`Bloom`](#bloom-what-happened)), and read-only samples taken at the callsite
([`Sap`](#sampling-sap-and-sample)). No engine type is handed out, and nothing an app
holds borrows from the world -- which is what makes it safe for an app to run its own
`bevy_ecs`, at whatever version it likes, without the two ever meeting.

## `Grows`: the command vocabulary

Every way an app can change the tree lives on one trait, implemented identically by
`Canopy` and [`Sprig`](#sprig-the-off-thread-half):

```rust
// foliage_proper/src/boundary/verbs.rs
pub trait Grows: Queues {
    fn leaf(&mut self, spec: impl Into<Spec>) -> Leaf;
    fn branch(&mut self, under: Leaf, spec: impl Into<Spec>) -> Leaf;
    fn prune(&mut self, leaf: Leaf);
    fn text(&mut self, leaf: Leaf, value: impl Into<String>);
    fn color(&mut self, leaf: Leaf, to: Color);
    fn location(&mut self, leaf: Leaf, to: Location);
    fn animate(&mut self, leaf: Leaf, to: Motion, timing: Timing) ;
    fn sequence(&mut self) -> Leaf;
    fn timer(&mut self, millis: u64) -> Leaf;
    // .. enable/disable/visible/elevation/scroll/name/tween/load_asset/and more
}
```

`leaf`/`branch` are covered in [Specs and Sprout](./spawning.md) -- they're how a
`Panel::new()...` builder actually becomes something on screen. The rest change or
animate an already-grown `Leaf`. `Grows` is sealed (its supertrait `Queues` is
`pub(crate)`), so this list is exactly what an app can ask the engine to do -- nothing
more, and nothing an app defines itself.

Commands are queued, not applied in place: everything pushed during the closure lands,
in the order it was written, immediately after the closure returns. A `leaf` followed by
a write to the `Leaf` it returned behaves exactly the way it reads, even though the
element doesn't exist as a real entity until that queue is drained.

## Bloom: what happened

```rust
// foliage_proper/src/boundary/bloom.rs
pub enum Bloom {
    Clicked(Leaf),
    Engaged(Leaf),
    Dragged(Leaf),
    Disengaged(Leaf),
    Focused(Leaf),
    Unfocused(Leaf),
    Key { key: Key, mods: Modifiers },
    TextChanged { leaf: Leaf, value: String },
    TextAction { leaf: Leaf, action: TextInputAction },
    Tween { tween: Tween, values: Vec<f32> },
    TimerFinished(Leaf),
    SequenceFinished(Leaf),
    AssetLoaded { key: AssetKey },
    Withered(Leaf),
    Resized { viewport: Area<Logical>, layout: Layout, short: bool },
    // ..
}
```

`canopy.take()` moves this frame's emissions out, ready for a `for bloom in canopy.take()`
loop that can itself call back into `canopy` without a borrow-checker dance. A single
physical click can produce several `Clicked` emissions in one frame -- one for the
element under the pointer, and one for every pass-through element the gesture crossed --
arriving in hit-test order. A `Leaf` reported here may already have withered by the time
you act on it; that's safe, since every command naming a withered `Leaf` is a no-op.

## Sampling: `Sap` and `Sample`

Reading current state -- as opposed to reacting to what changed -- goes through one
method:

```rust
// foliage_proper/src/boundary/canopy.rs
pub fn sample(&self, leaf: Leaf, what: Sap) -> Option<Sample<'_>>;
```

`Sap` is the exhaustive list of what a frame may observe about an element -- `Section`,
`Text`, `Color`, `ScrollOffset`, `Children`, and so on. If it isn't a `Sap` variant, app
code cannot see it: the render internals (glyph rectangles, caret position, resolved
clip) are deliberately absent, since they're the engine's own working state rather than
anything an app needs. `Sample` is what a given `Sap` reads back as; a handful of common
cases (`section`, `text_of`, `scroll_offset`, `is_visible`, ...) are typed convenience
wrappers over `sample` for the lookups every app ends up doing anyway.

`Canopy` also answers questions that aren't about a specific element: `layout()`, the
current breakpoint; `short()`, whether the viewport is vertically cramped; `frame_time()`,
for scaling per-frame motion so it runs at the same speed regardless of frame rate;
`viewport()`; `pointer()`/`pointer_velocity()`; `named(..)`/`asset_named(..)`, looking up
a `Leaf`/`AssetKey` registered earlier by name; and `asset(..)`, an asset's bytes once
loaded.

## Sprig: the off-thread half

```rust
// foliage_proper/src/boundary/sprig.rs
#[derive(Clone)]
pub struct Sprig {
    queue: Arc<Mutex<Vec<Op>>>,
    allocator: RemoteAllocator,
}
```

`Sprig` carries the exact same `Grows` vocabulary as `Canopy`, so code that changes the
tree reads identically whether it runs in the frame closure or on another thread. It's
`Send` and `Clone` -- obtained once via [`Foliage::sprig`](./app.md) -- and command-only:
there is no sampling half, because reads only make sense at the frame callsite, where the
engine is quiescent and a value read is a value that is actually true. Everything queued
through a `Sprig` is applied at the top of the next frame, ahead of that frame's own
commands, in the order it arrived -- which is what makes it safe for a background thread
to grow and change elements without ever touching the engine's world.
