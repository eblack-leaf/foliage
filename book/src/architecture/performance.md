# What a frame costs

This chapter is about reading the engine's cost: how to measure a frame, what one costs today, where
the time goes, and what that means for an app. The open optimisation candidates, and the history
behind the numbers, are in
[`OPTIMIZATION.md`](https://github.com/eblack-leaf/foliage/blob/main/OPTIMIZATION.md).

## The cheapest frame is the one that does not run

Before any number: the engine idles. A frame runs only when something is owed
([The loop](loop.md#whether-a-frame-is-owed)), so an app that is not being touched, not animating and
not loading anything costs nothing at all between events. Everything below is the cost of a frame
that *does* run.

## Three readings

No single measurement sees the whole frame, so there are three.

**Headless, by aspect.** The suite carries a benchmark that runs the frame exactly as the loop does,
against a grove with no surface and a clock moved by hand, so what it times is the frame on the
processor and nothing else:

```sh
cargo test -p foliage --release --lib bench -- --ignored --nocapture --test-threads 1
FOLIAGE_BENCH=layout,scroll FOLIAGE_BENCH_CELLS=4096 FOLIAGE_BENCH_PHASES=1 cargo test ...
```

It grows a field of cells (each a shape with a label on it, so two elements per cell) and runs one
load at a time, each writing one kind of thing every frame. `FOLIAGE_BENCH_PHASES` follows each
reading with where the frame went, from the engine's own trace spans.

**Windowed, with the backend.** `cargo run -p foliage --release --example stress -- <load> <cells>`
saturates one phase under a real surface and prints a phase table once a second. It is the only
reading that sees `absorb` (what the backend does with a batch) and `draw`. Raise the cell count until
the frame rate falls below the display's; below that point, the profile is mostly the wait to
present.

**On the web.** The same headless harness runs under `wasm-bindgen-test-runner`; it reads the same
shape as native, 30 to 60 per cent slower.

## Where a frame goes today

Headless, release build, on a 4-core 2.1 GHz cloud VM with no GPU. Each reading is the median of
120 frames, and the table holds the best of three runs, because wall-clock on a shared VM jitters:

| Load | Writes each frame | 512 cells | 4096 cells |
|---|---|---|---|
| `idle` | nothing | 0.034 ms | 0.285 ms |
| `hover` | one color motion on one cell | 0.034 ms | 0.270 ms |
| `scroll` | a region holding every cell, wheeled | 0.034 ms | 0.543 ms |
| `color/64` | a sixty-fourth of the fills | 0.035 ms | 0.281 ms |
| `color` | every fill | 0.115 ms | 0.951 ms |
| `repaint` | the scheme | 0.092 ms | 0.825 ms |
| `fade` | one opacity motion on the container of every cell | 0.107 ms | 0.992 ms |
| `toggle` | the whole field hidden and shown on alternate frames | 0.119 ms | 1.208 ms |
| `layout/64` | a sixty-fourth of the placements | 0.053 ms | 0.486 ms |
| `text/64` | a sixty-fourth of the labels | 0.045 ms | 0.445 ms |
| `churn` | 32 cells pruned and grown again | 0.342 ms | 1.772 ms |
| `layout` | every placement | 0.331 ms | 3.304 ms |
| `move` | a location motion on every cell | 0.316 ms | 3.145 ms |
| `deep` | the root of a chain of stems nested as deep as there are cells | 0.294 ms | 3.189 ms |
| `resize` | the viewport | 0.376 ms | 3.757 ms |
| `text` | every label, with strings never shaped before | 0.414 ms | 4.483 ms |
| `type` | a character typed and taken back, in an area of that many words | 0.108 ms | 0.845 ms |

These agree with the table in `OPTIMIZATION.md`, measured separately on a similar machine, to within
the jitter a shared VM has. For scale: a 60 Hz display allows 16.7 milliseconds a frame, and the
4096-cell column is over eight thousand elements.

## Reading the table

Each load writes one kind of thing, so the difference between two rows is where those writes land.
The phase breakdowns (4096 cells) say which part of the frame paid.

**The floor is the accumulations.** An idle frame at eight thousand elements is about a quarter of a
millisecond, and almost all of it is resolution's passes that carry a running product (extent,
scroll, clip, inherit), each about 0.05 ms. They run over every element because what they carry has no
position to start from; they skip nothing but also do nothing expensive per element. Extraction on an
idle frame visits nothing that moved and costs a hundredth of a millisecond.

**A write that moves nothing places nothing.** `color`, `repaint`, `fade` and `hover` rewrite what
elements look like, and none of them runs the axes: `color` at 0.95 ms is re-reading each restyled
element's standing (0.34 ms), extraction resolving and comparing fills (0.23 ms), and the drain
applying eight thousand ops (0.19 ms). One motion on one cell (`hover`) is indistinguishable from
idle.

**A write that moves a box costs the box, and what depends on it.** `layout` at 3.3 ms is mostly the
two axes (1.3 ms) and the measure pass (0.50 ms), then the drain and extraction (about 0.6 ms each).
`layout/64` shows the incremental resolution working: a sixty-fourth of the writes costs a seventh of
the frame. What is left of it is mostly the accumulations every frame pays, and the box stack built
again (0.10 ms) because the cells that moved moved in it. The measure is hardly in it: every element
written still marks its ancestors to measure again, and the container over the field still measures,
but it asks only the sixty-four children that moved and reads what the other four thousand reached
from when they last did ([Resolution](resolution.md#r2-the-two-axes-and-the-measure-between-them)).
Before that, asking all of them was 0.19 ms of this load on its own, and `layout/64` was 0.66 ms.

**New text pays for shaping.** `text` at 4.5 ms is `measure` shaping four thousand strings it has
never seen (1.7 ms) and `wrap` wrapping them (0.76 ms). A string that has been shaped before, at that
font and size, is a cache lookup. The same labels rewritten to strings already on screen would cost
little more than `layout`.

**Changing the tree's shape rebuilds its order.** `churn` regrows only 32 cells, yet costs 1.8 ms, and
0.83 ms of it is `structure`: the dependency order rebuilt over all eight thousand elements, which any
grow or prune triggers. Another 0.36 ms is the box stack sorted again. Hiding and showing the entire
field (`toggle`, 1.2 ms) is cheaper than regrowing 32 cells of it.

**A resize is everything.** The viewport, the breakpoint and the short reading are read by every
placement, so `resize` is every element resolved and every run measured again (3.8 ms). Dragging a
window's edge does this every frame of the drag, which is why it is the most expensive thing that
routinely happens.

**Scrolling moves what is inside.** `scroll` resolves no more than idle does, but R4 moves the drawn
box of everything in the region, so every descendant is `moved` and extraction revisits all of it
(0.23 ms), most of it only to find it culled.

**Typing into a long run is linear in the run.** The `type` load is a character typed into an area
holding four thousand words, and it costs 0.85 ms per keystroke: the run is shaped, wrapped and drawn
whole, and placing the caret and selection reads character positions that each walk the run from its
start.

## What an app can do with this

Most apps are nowhere near these numbers: a few hundred elements, a handful of writes a frame. Where
one is, the table suggests where to look:

- **Prefer writes that do not move boxes.** Color, opacity, visibility, elevation and rounding changes
  are drawn again without being placed again.
- **Write the child, not the container.** A container sized to what it holds measures again when
  anything under it moves, and asks only what moved, so rewriting one row of a long list costs the
  row. Rewriting the container itself resolves everything under it.
- **Hide rather than regrow** things that come and go every frame: a grow or a prune rebuilds the
  order of the whole tree. (The reverse holds for something parked for good: a hidden subtree is still
  resolved whenever everything is, on a resize for instance.)
- **Reuse strings where it is natural.** Shaping is cached by string, font and size; a clock that
  rewrites its label every frame shapes a new string every frame.
- **Split very long text.** A text area's costs grow with the length of its run.
- **Expect a resize to cost the whole tree**, and keep the tree to what is on screen or near it.

## Following a frame in the trace

Every phase is a span. With a subscriber installed ([Debugging](../usage/debugging.md)) and
`RUST_LOG=foliage=trace`, each frame is a `frame` span containing `intake`, `dispatch`, `gestured`,
`root`, `drain`, `animate`, `resolve` (with `elements`, `measure`, `axis`, `wrap`, `extent`,
`scroll`, `clip` and `rank` under it), `settle` (with `inherit` and `regions`), `publish` and
`extract`, then the backend's `absorb`, `restack` and `draw`. The spans carry counts: how many
inputs dispatch took, how many ops the drain applied, how many elements resolution held and how many
were dirty, how many instances extraction wrote, withdrew and kept.

A profiler that understands spans, or the `stress` example's phase table, turns that into the
breakdowns above without any instrumentation of an app's own.
