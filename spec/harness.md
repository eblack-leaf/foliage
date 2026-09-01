# The harness

How foliage is proven.

## The contract

> **It is the frame, minus the rasteriser. Not a simulation of it.**

This is the whole value of the thing. A harness that reimplements the frame produces evidence about
the harness; a harness that *is* the frame produces evidence about foliage.

Mechanically: there is **one `Fern::run`**. The event loop calls it and so does the suite. Neither
owns a copy. Everything in `frame.md` — F1 through F9 — holds inside the suite identically, because
it is the same code answering.

If a behaviour cannot be reached headlessly, that is a finding about the engine's structure, not a
reason to give the harness a second path.

## The harness is not a type

There is no harness object, no headless constructor, and no driving verb on anything an app can
reach. **The suite lives in-crate, under `#[cfg(test)]`, and calls `Fern` the way the platform loop
will.**

This is the rule it exists to keep:

> **Testing adds nothing to the public API.**

Every item the suite touches is there because the engine needs it. `Fern::run` is what the loop
calls. `Grove::new` is how a grove is built. The clock and the pending resize are the fields intake
reads. A test that reads `tap` or calls `plant` is calling exactly what an app calls, because those
are the only things an app has.

Driving is the platform's role. winit owns the loop, and nothing else — not an app, not a test —
holds a verb that advances a frame. A test does not need a loop; it needs one frame, which is one
call.

An external `tests/` directory is what breaks this. It sees only `pub` items, so proving the frame
from outside means publishing a driver, and the API grows a headless constructor, a tick, a clock
verb and a resize verb that no real run ever calls. The in-crate suite is what makes that pressure
disappear.

## What is swapped

Three things, all at the edges:

| | Windowed | Headless |
|---|---|---|
| **clock** | sampled from the platform at step 1 | advanced explicitly by the test |
| **input** | winit events | written directly as input state |
| **surface** | wgpu swapchain | none — step 9 does not run |

Nothing else. The op queue, the drain, Rowan, Aspen, interaction, focus, and Elm up to the point of
extraction all run exactly as they do in an app.

## Determinism

The clock is the only source of time, it is advanced explicitly, and `F6` already requires the
whole frame to see one instant. So:

> The same ops against the same clock produce the same state, every run.

No frame-rate dependence, no wall-clock flake, no sleeping in tests. An animation is checked by
advancing exactly half its duration and asserting the value, not by waiting and hoping.

## Shape

```rust
let mut grove = Grove::new(Area::new(400.0, 300.0));

let panel = grove.plant(Panel::new().at(..).elevate(..));
fern::run(&mut grove, None);
assert_eq!(grove.tap(panel, Vein::Section), ..);

grove.clock.advance(250);       // ms — half of a 500ms tween
fern::run(&mut grove, None);
assert_eq!(grove.tap(panel, Vein::Opacity), ..);

grove.pending_resize = Some(Area::new(800.0, 600.0));
fern::run(&mut grove, None);
```

Input is written as gestures rather than as synthetic winit events, because the winit translation
layer is platform integration — the thing the harness explicitly cannot cover. Pressing at a point
enters the pipeline where a translated event would.

`Pollen` is reached the way an app reaches it: as the argument `Fern` hands the root. A test that
needs it passes a `Rooted` of its own and reads it back afterwards.

```rust
let mut app = Observer::default();
fern::run(&mut grove, Some(&mut app));
assert!(app.last().clicked(panel));
```

The test owns that value either side of the frame and lends it for the duration, so there is no
accumulator to reach around and no ordered view a real app would not have — if the ordered view
existed, the rule in `pollen.md` would not hold.

## What it cannot prove

Stated plainly, so that "the tests pass" is never mistaken for more than it is:

- **Rasterisation.** Whether a rounded corner is right, whether a glyph lands on the pixel grid,
  whether a shader is correct. Covered later by golden images — wgpu renders to a texture with no
  surface and no window, so this runs in CI without a display, but it is a second tier and not this
  one.
- **Platform integration.** winit event translation, wgpu surface acquisition, the wasm boot
  handshake, Android lifecycle. Real code, and the CI build matrix is what covers it.
- **The loop.** Whether a frame is owed, and whether the engine idles when none is. F9 is a law
  about the loop, and the loop is `photosynthesize`.
- **Timing and performance.** An explicit clock says nothing about whether a frame fits in 16ms.

Everything else — layout, wrapping, breakpoints, scroll extents and handoff, elevation ranking,
clipping, hit-testing, gesture claiming, focus order and trapping, animation timing, op ordering,
withering, the disable cascade — is provable here, and is where the bulk of the suite lives.

## Why it comes first

Built before B1, because it is what every slice is checked with. A slice's proof obligations are
written into its spec section and discharged here; a slice with no way to be checked is a slice
whose spec has not been tested against reality yet.

It is also the forcing function for the `Fern` split. Making the suite share the event loop's frame
is only possible if intake and draw are genuinely at the edges — so building it early is what keeps
them there.

## Proof obligations

The harness is itself load-bearing, so a few tests are about it rather than through it:

- two identical scripts produce byte-identical resolved state
- a script produces the same state whether run in one batch or interleaved with idle frames
- `advance` is exact: a tween at `t = 0.5` reads its midpoint, not a value near it
- the frame laws hold: an op naming a withered leaf is dropped, reads do not change inside
  `frame()`, `Sprig` ops and in-frame ops are indistinguishable (F1–F3)
- a headless run and a windowed one driven by the same scripted input reach the same resolved
  state, checked once for a representative tree — the direct evidence that the harness is the frame
