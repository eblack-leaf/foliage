# Optimisation

Open candidates, and how to measure one. Each says what is done today, where it lives, and what to
measure before changing it.

## Measuring one

Three readings, because no one of them sees the whole frame.

**Headless, by aspect.** The suite carries a harness that runs the frame exactly as the platform
loop does, against a grove with no surface and a clock moved by hand, so what it times is the frame
on the processor and nothing else:

```sh
cargo test -p foliage --release --lib bench -- --ignored --nocapture --test-threads 1
FOLIAGE_BENCH=layout,scroll FOLIAGE_BENCH_CELLS=4096 FOLIAGE_BENCH_PHASES=1 cargo test ...
```

Every load runs at two sizes (512 and 4096 cells, two elements each), because what is worth checking
about most of them is how cost grows with the tree. `FOLIAGE_BENCH_PHASES` follows each reading
with the engine's own spans, as a tree. The loads (`foliage/src/tests/bench.rs`):

| load | what it writes each frame |
| --- | --- |
| `idle` | nothing: the floor at this size |
| `layout`, `layout/64` | every cell's placement, or a sixty-fourth of them |
| `color`, `color/64` | every cell's fill, or a sixty-fourth |
| `text`, `text/64` | every label, with a value nothing has shaped, or a sixty-fourth |
| `move` | a location motion live on every cell |
| `fade` | one opacity motion on the container of every cell |
| `hover` | one colour motion on one cell |
| `churn` | 32 cells pruned and grown again |
| `repaint` | the scheme restated |
| `toggle` | the field hidden and shown on alternate frames |
| `resize` | the viewport, which is every element written |
| `scroll` | a region four pages tall holding every cell, wheeled |
| `deep` | a chain of stems nested cells deep, its root moved |
| `type` | a character typed and taken back in an area holding cells words |

**Windowed, with the backend.** `cargo run -p foliage --release --example stress -- <load> <cells>`
saturates one phase under a real surface and writes what each phase cost. It is the only reading
that sees `absorb` -- what the backend does with a batch -- and the only one that sees `draw`.
`draw` opens by acquiring the surface, so what it reports is the wait for the display rather than
work, and a share is of `frame + absorb + draw`: **a change shows in the millisecond column and not in
the share.** Loads: `idle`, `layout`, `color`, `text`, `animate`, `churn`, `repaint`, `scroll`.

With no GPU, Mesa's lavapipe runs it on the processor: `apt install mesa-vulkan-drivers`, start
`Xvfb :99`, then `DISPLAY=:99 WGPU_BACKEND=vulkan` in front of the command. `draw` is then a
software rasteriser's and means nothing, and lavapipe's threads share the cores with the frame, so
`frame` reads high -- but `absorb` is the backend's own work and is comparable between two builds.

**Web.** The site runs as wasm, which is a different compiler target, pointer width and allocator.
The headless harness runs there too, under `wasm-bindgen-test-runner` in Node, from a copy of the
tree in which `tests/mod.rs` holds only `bench` (the rest of the suite asserts things that are true
only natively), `bench` is `#[wasm_bindgen_test]`, reads `web_time::Instant`, writes with
`console_log!`, and takes its knobs from `option_env!`. It reads the same shape as native, 30 to
60 per cent slower.

Numbers below are one machine: a 4-core cloud VM, no GPU. Wall-clock there jitters by a third on
the smallest loads, so a comparison is the best of several runs, alternated between the two builds.

## Where a frame goes today

At 4096 cells (8195 elements), best of three:

| load | native | wasm |
| --- | --- | --- |
| `idle` | 0.25ms | 0.39ms |
| `color` | 0.81ms | 1.31ms |
| `hover` | 0.27ms | 0.37ms |
| `repaint` | 0.77ms | 1.27ms |
| `scroll` | 0.48ms | 0.87ms |
| `layout` | 3.14ms | 4.11ms |
| `move` | 3.45ms | 4.46ms |
| `text` | 4.34ms | 4.89ms |
| `churn` | 1.84ms | 2.25ms |
| `deep` | 2.96ms | 3.94ms |

What was structural, and is now settled:

- **What resolution produces is held in resolution's own columns** (`Elements`, `rowan.rs`), kept
  between frames and read by position. Nothing is written back into the world per element per
  frame, and the passes that run over every element (R3 to R8) index arrays instead of looking
  each element up. That is the whole of the idle floor falling from 2.9ms to 0.25ms.
- **A write that moves nothing places nothing.** Fill, rounding, tint, shape, mark, plate, fit,
  visibility, opacity, disabled and elevation are `restyled` (`tree.rs`): read again and drawn
  again, and nothing re-resolved. A repaint or an asset arriving is every element drawn again and
  none placed. A running motion marks what it actually moves. Setters that write what is already
  held mark nothing.
- **R1 measures only what says something new**, and everything after it reads the run R1 left the
  element holding. The shaping cache is swept by what elements still hold, so nothing is shaped
  again to keep it alive.
- **R2m asks only the children that moved.** How far each child reaches toward its trunk's measure
  is held beside it (`Elements::reached`, `rowan.rs`) and found again only where the child is dirty,
  so a container of thousands with one child written resolves one child to measure itself, not all
  of them. At 4096 cells `reach` fell from 0.19ms a frame to 0.02ms under `layout/64`, which fell
  from 0.66ms to 0.46ms (`text/64` 0.61ms to 0.42ms, `churn` 2.05ms to 1.77ms; medians, the two
  builds alternated on one machine). `resolving_only_what_was_written_lands_where_resolving_everything_does`
  (`tests/rowan.rs`) holds it, and incremental resolution as a whole, to what resolving every element
  would have given.
- **Extraction visits only what moved**, and lets go of what went explicitly: an element that moved
  and is no longer painted, and one that withered (reported when the order is built again). There is
  no per-frame sweep of everything held.
- **The box stack is built only when something it reads moved**, and sorted only when a rank did.
- **The backend uploads runs, not slots**: neighbouring changed slots go up in one write, and a
  text renderer's runs are staged straight from where they are held.
- **What travels is small.** An op is 152 bytes rather than 3008 (a whole element and a scheme are
  behind pointers), a placement at every breakpoint is one pointer, and an expression of one term
  -- nearly all of them -- allocates nothing.

## Typing into a long run is linear in the run, several times over

The `type` load is the one this pass did not move: 0.74ms per keystroke in an area holding about
25,000 characters. A run that changed is shaped again whole (`Shaping::shape`), wrapped whole, and
drawn whole (`elm.rs`, `Shaped::place`), which is the diff "of the run and not of its glyphs" that
R6's tie-break asks for. On top of that, the caret and the three selection marks are placed with
`anchor().character(n)`, and `Wrap::cell_of` (`text/shape.rs`) walks the run from its start on every
read. Each mark reads a character on both axes, and the marks are resolved again whenever the run is
-- hidden or not -- so one focused field is several whole walks a frame, which is most of this
load's axis time (0.5ms of 0.74).

The cheap half is the walks: a wrap computed once per run and column count, and `cell_of` answered
from it. The diff of the run itself stays on the renderer's side of the line, as before.

## A structure change rebuilds the order and re-sorts the stack

`churn` (32 cells regrown a frame) is 1.84ms, and half of it is `structure` (0.86ms: the live set,
two lookups per element for trunk and anchor, and Kahn's) and the stack sorted again (0.38ms).
What resolved is already carried across the rebuild, so what is left is the order itself. A trunk
never changes after growth and an anchor only through `set_anchor`, so both could be carried rather
than asked again, and the ranks of what was carried did not move -- the new elements could be
merged into the sorted stack rather than the whole of it sorted. Measure with `churn` before and
after; neither changes what any other load pays.

## Scrolling moves every descendant through extraction and the renderers

R4 subtracts the offset from every descendant's box (`rowan.rs`, `scroll`), so everything in a
scrolled region is `moved` every frame of a scroll or a coast. `scroll` reads 0.48ms headless and
0.68ms of `absorb` under a surface at 4096 cells, most of which are outside the region and culled.
What that costs downstream:

- **Every run in the region is rebuilt glyph by glyph.** `Texts::write` (`ash/text.rs`) does an atlas
  lookup and a device-pixel snap per glyph, even though the run only translated.
- **An element crossing the clip edge churns the stack.** `painted` (`elm.rs`) culls against the
  clip, so crossing it withdraws or inserts an instance, which resorts that renderer and restacks
  every renderer (`Ash::restack`) -- which the `scroll` load does every frame.
- **A region nested inside a scrolled one restacks every frame.** Its clip moves with the scroll, a
  changed clip sets `disturbed`, and `disturbed` restacks the whole shared stack.

Candidates: cull against the clip plus a margin, so crossing the edge happens less often and the
scissor still hides what is outside; and keep each glyph's placement with its run so a translated
run re-snaps without looking up the atlas.

What does not fit: a per-draw scroll offset. A span is a run of the rank-merged stack sharing a
renderer, a clip and a binding, not a region -- a region's content is many spans interleaved with
everything else -- and lines and glyphs are snapped to device pixels at absorb, which a fractional
offset applied later would undo.

## Resolving an element still asks the world for its declaration

What is left of `layout`, `move` and `deep` is mostly the axes (`rowan.rs`, `axis`, about 1.2ms
over 8195 elements on two axes): one lookup of the element's placement per axis, and inside it,
bevy's `get::<T>()` finding the component's id by `TypeId` before it finds the component. A query
state kept on `Tree`, or component ids resolved once, would take the second part away; the first is
the declaration itself and stays. `measure`, `reach` (for the children it asks), and extraction's
pigment reads are the same shape.

## The shared walk is the stack's size

It runs only when the stack moved, and when it runs it walks the whole stack: merging every
renderer's slots by rank, taking each instance's depth from its position, and cutting spans on
renderer, clip and binding. Depth is a position, so an insertion rewrites every depth after it. The
scroll churn above is what opens this gate most often (`restack`, 0.13ms a frame under `scroll`).

## Hidden subtrees are still resolved

Extraction skips what is invisible or clipped away (`painted`), but resolution does not. A resize or
a breakpoint invalidates everything, and a page parked with `visible(false)` is resolved with the
rest. Leaving a hidden subtree dirty and resolving it when it is shown makes a resize cost what is
on screen. The cost to decide: `Vein::Drawn` on a hidden element would read a stale box, which has
to be stated as the contract rather than discovered.

## Fill rate, on a weaker GPU

Untested off the desktop. Everything draws back to front with alpha blending, and depth writes are
on (`ginkgo/mod.rs`, `LessEqual`) only to hold the order, so the depth test never rejects anything:
a full-screen page panel under layered cards shades most pixels several times, each through an SDF.
On an integrated GPU at 2× this is more likely the limit than anything above. Measure `draw` at the
laptop's native resolution first. If it is fill-bound, the candidate is a front-to-back pass for
opaque interiors, so covered pixels are rejected before shading, with only edges and translucent
elements blended.
