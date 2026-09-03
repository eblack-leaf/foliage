# The frame law

The ordering contract. Everything else in foliage is written against this.

## The sequence

```
1  intake    window and input events → input state. The clock is sampled once, here
2  dispatch  hit-test against what was drawn last frame → Pollen
3  frame()   the app reads settled state and Pollen, and queues ops
4  drain     the single apply — every queued op, FIFO by arrival, whatever its origin
5  animate   tweens advance                                              ┐
6  resolve   grid → location → section → extent → scroll → clip → rank   │ Rowan
7  settle    the inherited products, the box stack, focus                ┘
8  extract   changed state → render instances (Elm)
9  draw      (Ash)
```

`Fern` runs this sequence, and is the only thing that does.

`Root::frame` is called exactly once per frame, at step 3, and nowhere else.

## Laws

### F1 — One queue, one drain

There is one op queue. An op's position in it is determined by when it was queued and by nothing
else. An op queued from a `Sprig` on another thread and an op queued inside `frame()` are
indistinguishable to the engine: same ordering, same semantics, same timing.

This is a correction of prior behavior, not a description of it. The previous engine drained
`Sprig` ops into a *different* apply pass than the frame's own, so identical code behaved
differently depending on which side of the boundary it ran on.

The only observable difference that remains is inherent to concurrency: an op from another thread
lands in this frame's drain if it arrived before step 4, and the next frame's otherwise. Nothing
about *how* it is applied differs.

**Proof obligation:** for any op sequence, the resulting state is identical whether issued through
`Grove` or `Sprig`, given the same arrival order.

### F2 — The drain is total and FIFO

Ops apply in arrival order. A grow followed by a write to the same `Leaf` behaves the way it reads.
An op naming a `Leaf` that has withered is **dropped silently** — not an error, not a panic. That
is what makes a stale handle inert rather than dangerous.

### F3 — Reads do not change inside `frame()`

Nothing an app queues can land while it is still running: the drain is at step 4, after `frame()`
has returned. The state read at the top of `frame()` is the state read at the bottom.

`frame()` is therefore a pure function of (settled state, `Pollen`) → ops. An app cannot observe
its own write, and does not need to guess whether it is looking at before or after.

This is a deliberate trade against same-frame read-back, and it is the better half: read-back would
have made every read's meaning depend on where in the function it appeared.

### F4 — An app reads what is on screen and writes what will be on screen

The state `frame()` reads was settled at step 7 of the previous frame — which is exactly what step
9 drew. The ops it writes are applied at step 4 and reach step 9 of the same frame.

So a command issued in `frame()` reaches the screen in that frame, and a value read in `frame()` is
what the viewer is currently looking at. Neither is stale; they are one frame apart because they
are about different instants, and each is the correct instant for its purpose.

### F5 — Hit-testing runs against what was drawn

Dispatch is at step 2, before the drain, and tests against geometry settled at the end of the
previous frame.

This is correctness, not a concession to ordering. A pointer event was produced by a person looking
at the screen, and the screen is the previous frame's render. Hit-testing against geometry that has
moved since would resolve the gesture against something the person never saw.

It also means input is handled in the frame it arrived, with no added latency.

### F6 — One clock

The instant is sampled once, at step 1, and every phase in the frame sees that same value.
Animation, timers, and `frame_time()` all read it. Two things scheduled for the same moment cannot
disagree about when that moment is.

### F7 — The collection window is "since your last frame"

`Pollen` handed to `frame()` holds everything emitted since the previous `frame()` returned: input
emissions from step 2 of this frame, and animation, timer, asset and wither emissions from steps
4–7 of the last one.

One continuous window, not two. This is a second and independent reason the set is unordered — the
window spans a frame boundary, so any ordering across it would describe the engine's phases rather
than the app's world. See `pollen.md`.

### F8 — One writer per property, per phase

Every derived value has exactly one writer, in exactly one phase. `resolve` owns sections, extents,
offsets and clips; `settle` owns the inherited products, the box stack and focus; `extract` reads
and writes nothing but its own instance buffers.

A view's extent is written in `resolve` rather than in `settle` because the offset clamped to it is
what the drawn boxes are then computed from: the order is `rowan.md`'s R3 → R4, and it is not a free
choice.

Declared values — the ones an app writes — have two potential writers, the drain and `animate`.
The drain runs first, and **a direct write cancels any in-flight tween on that property**. So by
the time `animate` runs, no property has both a pending write and a running tween. The rule that
was previously advisory ("if a property is animated anywhere, animate it everywhere") is now
structural, and is gone.

## Consequences worth stating

- A `Leaf` is usable the instant it is handed out, including as a parent, though the element does
  not exist until step 4. Sampling one before then reads absent (`Presence::Planted`).
- A timer set for `0` fires no earlier than the next frame: queued at 3, applied at 4, ticked at 5,
  delivered at step 3 of the following frame. Honest rather than special-cased.
- `prune` in `frame()` emits `Withered` that the app sees on its next call.
- Off-thread ops never interleave with a partially-applied frame; they join the same drain.

## Headless

The headless suite runs steps 1–8 and skips 9, by calling the same `Fern` the event loop calls.
Every law above holds identically, which is what makes a headless test's evidence worth anything —
it is not a simulation of the frame, it is the frame without a rasterizer.

The clock is advanced explicitly rather than read from the platform, so timing is exact rather than
approximate.

### F9 — A frame runs when there is something to do

The loop idles otherwise. A frame is owed when any of these is true:

- input arrived, or the window resized
- an op is queued, from either side
- a tween or timer is live
- an asset landed
- the app asked for one

The previous engine never idled, and it is worth recording that this was not a considered decision:
`about_to_wait` ran a tick and unconditionally called `request_redraw()`, which guaranteed the next
`about_to_wait`. The `tick_pending` flag exists to stop ticks *stacking* between paints — the
opposite concern. Idle was never implemented rather than tried and reverted.

The one thing that argued against it does not: skipping a frame cannot lose a change, because
extraction compares against a cached value rather than a dirty flag. Deferred, never dropped.

**The app asking is the clause that matters.** An app driving its own motion from `frame_time()` —
scroll coasting, a hand-rolled transition — has nothing the engine can detect, and would stall.
`Grove::again()` requests the next frame; called during `frame()`, it keeps the loop running for
exactly as long as the app is doing something. This is testable headlessly, which is the reason to
make it explicit rather than heuristic.

Whether a frame is owed is the loop's own question. It is answered inside `photosynthesize`, from
state the engine already holds, and is not a method on `Grove`, on `Foliage`, or on anything else —
nothing outside the loop has cause to ask. `again()` is the app's half of this law and the only
half an app can see.

## Open

Nothing. The remaining decisions in this area belong to `rowan.md` (how the resolve phases divide)
and `aspen.md` (which is written).
