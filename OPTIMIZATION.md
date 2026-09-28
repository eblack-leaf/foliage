# Optimisation

Open candidates only. Each says what is done today, where it lives, and what to measure before
changing it. Numbers are from one machine (Radeon RX 7900 XTX, Vulkan) at 4096 elements; a quiet
frame there is 1.49ms.

## Measuring one

`cargo run -p foliage --release --example stress -- <load> <cells>` saturates one phase of the frame
at a chosen tree size and writes what each phase cost, taken from the engine's own spans. `idle`
writes nothing and is the floor for a tree of that size; every other load is that floor plus one kind
of write.

`draw` opens by acquiring the surface, so what it reports is the wait for the display rather than
work. What a frame costs on the processor is `frame` plus `absorb`. A share is of all three, so on a
run held to the display's cadence a phase made cheaper moves time into `draw` rather than shrinking
the total. **A change shows in the millisecond column and not in the share.**

Loads today: `idle`, `layout`, `color`, `text`, `animate`, `churn`, `repaint`. There is **no scroll
load**, and scrolling is the most common continuous interaction — see below.

## Writes that do not touch geometry still resolve geometry

Every write ends in `Tree::declared` (`tree.rs`), and `rowan::read` closes that set over trunks and
anchors (`rowan.rs`, `dirty = all || declared || trunk dirty || anchor dirty`). So a write resolves
the element's whole subtree through measure, shape and both axes, whatever it wrote.

- **Opacity, visible and disabled need no mark.** The gated passes never read them. The passes that
  walk the whole order every frame (extent, scroll, clip, rank, inherit, regions) read them fresh,
  and `inherit` already sets `moved` when an element's product changes (`rowan.rs`, R7).
- **Fill needs `moved` and not `dirty`.** Nothing in resolution reads it; extraction does, and
  extraction skips what is not `moved`. `Elements` already keeps the two as separate arrays. What is
  missing is a set on `Tree` beside `touched` for paint-only writes (fill, tint, mark, plate, fit,
  rounding, a polygon's shape) that `read` feeds into `moved` alone. Rounding is safe there: the hit
  test reads `Gestures.shape` (`Box`, or the inscribed ellipse from `round_hit_area`) and never the
  corners.
- **Every running motion calls `tree.declared` every frame** (`aspen/mod.rs`, the tick). That
  includes `Motion::Opacity`, `Motion::Color` and `Motion::Palette`, so a fade or a colour motion on
  a container re-resolves the container's subtree every frame it runs. Only `Location` and `Trace`
  motions need it. `Color`, `Palette` and `Shape` belong in the paint set (a polygon's shape reaches
  neither layout nor the hit test), and `Opacity` in neither.
- **Setters write unconditionally.** `set_visible`, `set_opacity` and `set_location` go through
  `entity.insert` and mark without comparing. `Tree::overwrite` already requires `PartialEq`, writes
  in place and returns whether the value changed. Routing these setters through it and marking only
  on change is the no-op check for free. The visible case: `text_input::settled` sets every field's
  caret visibility, and a focused selection's three locations, on every frame that runs.

The site pays for this today: the showcase switches pages with `visible`, and the internals rail
fades in with `Motion::Opacity`. Measure with `animate` and `color` before and after.

## Scrolling moves every descendant through extraction and the renderers

R4 subtracts the offset from every descendant's box (`rowan.rs`, `scroll`), so everything in a
scrolled region is `moved` every frame of a scroll or a coast. What that costs downstream, unmeasured:

- **Every run in the region is rebuilt glyph by glyph.** `Texts::write` (`ash/text.rs`) does an atlas
  lookup and a device-pixel snap per glyph, even though the run only translated.
- **An element crossing the clip edge churns the stack.** `painted` (`elm.rs`) culls against the
  clip, so crossing it withdraws or inserts an instance, which resorts that renderer and restacks
  every renderer (`Ash::restack`).
- **A region nested inside a scrolled one restacks every frame.** Its clip moves with the scroll, a
  changed clip sets `disturbed`, and `disturbed` restacks the whole shared stack.

Candidates, after a `scroll` load exists: cull against the clip plus a margin, so crossing the edge
happens less often and the scissor still hides what is outside; and keep each glyph's placement
with its run so a translated run re-snaps without looking up the atlas.

What does not fit: a per-draw scroll offset. A span is a run of the rank-merged stack sharing a
renderer, a clip and a binding, not a region — a region's content is many spans interleaved with
everything else — and lines and glyphs are snapped to device pixels at absorb, which a fractional
offset applied later would undo.

## The regions stack is rebuilt every frame that runs

`regions` (R8) walks the whole order, collects every present, unclipped element and sorts them by
rank on every frame, even when one hover fade is the only thing moving. The shared draw walk is
already gated on the stack moving; regions can be gated the same way on anything `moved`, a rank
change or a gesture change.

Extraction reads the same inherited product, drawn box, clip and rank one step later over the same
order. Once regions is gated, what is left is that duplication.

## The accumulation passes walk the whole order

Extent 0.25ms, scroll 0.16, clip 0.11, rank 0.13, inherit 0.16 — about 0.8ms a frame whatever
changed. Each is a product carried down (or up) the trunks. The consistent form is the per-element
skip the gated passes already use: a clean element whose trunk's product did not change keeps what
it held. Not contiguous ranges — the order is by dependency, anchors included, so a subtree is not
guaranteed to sit together in it.

## Hidden subtrees are still resolved

Extraction skips what is invisible or clipped away (`painted`), but resolution does not. A resize or
a breakpoint invalidates everything, and a page parked with `visible(false)` is resolved with the
rest. Leaving a hidden subtree dirty and resolving it when it is shown makes a resize cost what is
on screen. The cost to decide: `Vein::Drawn` on a hidden element would read a stale box, which has
to be stated as the contract rather than discovered.

## A changed run is rebuilt entire

The scratch is kept between frames, so a frame that changes nothing allocates nothing there. A run
that changed by one character is still rebuilt whole, and what a run costs is its length — which is
what typing into a long `TextArea` pays per keystroke.

The diff is deliberately of the run and not of its glyphs: a run is one entry in the one stack, and
the renderer holds its glyphs under its own numbering. A finer diff has to stay on the renderer's
side of that line or it reopens R6's tie-break.

## The shared walk is the stack's size

It runs only when the stack moved, and when it runs it walks the whole stack: merging every
renderer's slots by rank, taking each instance's depth from its position, and cutting spans on
renderer, clip and binding. Depth is a position, so an insertion rewrites every depth after it. The
scroll churn above is what opens this gate most often, so measure scroll first.

## R1 and `shaped` read the world element by element

Both reach for a typeface and a run per element per pass, and a `Location` is read four times a
frame (once per axis, by `measurable`, and by `reach`). Gathering them into columns means `Tree`
hands out a read of the whole tree rather than an accessor per property. Weak: the two passes are
`measure` 0.44ms and `shaped` 0.29, and much of that is in the font and the shaping cache rather
than in the reach, against 0.45 for the read that would carry it.

## Fill rate, on a weaker GPU

Untested off the desktop. Everything draws back to front with alpha blending, and depth writes are
on (`ginkgo/mod.rs`, `LessEqual`) only to hold the order, so the depth test never rejects anything:
a full-screen page panel under layered cards shades most pixels several times, each through an SDF.
On an integrated GPU at 2× this is more likely the limit than anything above. Measure `draw` at the
laptop's native resolution first. If it is fill-bound, the candidate is a front-to-back pass for
opaque interiors, so covered pixels are rejected before shading, with only edges and translucent
elements blended.
