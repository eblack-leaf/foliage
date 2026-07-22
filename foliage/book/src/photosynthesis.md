# The Event Loop: Photosynthesis

Everything so far -- [Leaf](./leaf.md), [Sprout](./spawning.md), [Differential](./differential.md),
[Ash](./ash.md), [Ginkgo](./ginkgo.md), [Willow](./willow.md) -- is inert without
something driving it, tick after tick, in response to real input. `photosynthesis.rs`
implements `winit::ApplicationHandler` for `Foliage`; it's the crate's actual entry
point once `foliage.photosynthesize()` (see [The App](./app.md)) hands control to
`winit`.

## Booting: `resumed`

The first `resumed` call is where the window and GPU device actually get created --
deferred this late (rather than in `Foliage::new()`) because `winit` doesn't hand out an
`ActiveEventLoop` to create a window from until then:

```rust
// foliage_proper/src/photosynthesis.rs
fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    if !self.ginkgo.acquired() {
        self.willow.connect(event_loop);
        pollster::block_on(self.ginkgo.acquire_context(&self.willow));
        self.finish_boot();
    } else {
        // returning from suspend (mobile/web): reconnect the surface, don't recreate the device
        self.ginkgo.recreate_surface(&self.willow);
        ...
    }
}
```

On wasm, device acquisition is async for real (no `pollster::block_on` available), so it
spawns a local future and boots once a channel receives the finished `Ginkgo` --
`about_to_wait` drains a queued backlog of window events once that boot completes, so
nothing that arrived during the async wait is lost.

## Every tick: `main` → `user` → `diff`

`about_to_wait` is where a tick actually runs, gated by `tick_pending` -- a flag that's
true from the moment a redraw is requested until it actually paints:

```rust
// foliage_proper/src/photosynthesis.rs
if !self.tick_pending {
    self.main.run(&mut self.world);
    self.user.run(&mut self.world);
    self.diff.run(&mut self.world);
    self.willow.window().request_redraw();
    self.ash.drawn = false;
    self.tick_pending = true;
}
```

The gate exists because `about_to_wait` isn't 1:1 with real paint frames -- high-frequency
input (mouse move, and especially web, where each DOM event tends to pump its own cycle
rather than batching like native OS queues do) can fire it many times before the next
`RedrawRequested`. Without the gate, each of those would re-run all three schedules and
request another redraw, stacking up generations of ECS churn (entity spawns/despawns from
reactive rebuilds) that never individually get painted -- only the last should exist by
the time a paint actually happens.

## Input: `process_event`

Every `winit::WindowEvent` (keyboard, mouse, touch, scroll, resize, IME) is translated
here into foliage's own event types and written into the ECS -- `KeyboardInput` goes
through a `KeyboardAdapter` resource and gets `world.trigger(..)`'d; `CursorMoved`/
`MouseInput`/`Touch` go through their own adapters and become `Interaction` messages.
This is the boundary between "OS event" and "ECS event" -- nothing past this function
knows `winit` exists.

## Painting: `RedrawRequested`

```rust
// foliage_proper/src/photosynthesis.rs
WindowEvent::RedrawRequested => {
    if !self.ash.drawn && self.ran_at_least_once && !self.suspended {
        self.ash.prepare(&mut self.world, &self.ginkgo);
        self.ash.render(&self.ginkgo);
        self.ash.drawn = true;
        self.tick_pending = false;
    }
}
```

`ash.prepare` is where [differential](./differential.md) queues actually get drained
into GPU-ready node lists; `ash.render` is where those get submitted through `ginkgo`.
Both are covered in [Ash](./ash.md). `tick_pending` clears here, closing the loop the
previous section describes.
