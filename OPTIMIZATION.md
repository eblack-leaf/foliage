# Optimisation

Candidates, not defects. Each is deliberate as built — the shapes below are what to look at when a
frame is too expensive, and what each one would cost to move. A candidate that has been measured
carries its measurement; for the rest, the first work is a measurement rather than a change.

## Measuring one

`cargo run -p foliage --release --example stress -- <load> <cells>` saturates one phase of the frame
at a chosen tree size and writes what each phase cost, taken from the engine's own spans. `idle`
writes nothing and is the floor for a tree of that size; every other load is that floor plus one kind
of write.

`draw` opens by acquiring the surface, so what it reports is the wait for the display rather than
work. What a frame costs on the processor is `frame` plus `absorb`.

## Rowan recomputes the whole tree every frame

The frame's cost is the tree's size rather than the change's. Nothing is skipped and nothing is
cached, which is what makes a resolution reproducible and every ordering question answerable in one
place.

**Measured.** `layout`, which rewrites every element's placement, against `idle`, which writes none.
Resolution costs the same in both at every size, so all of it is the tree's and none of it is the
change's:

| elements | 512 | 1024 | 2048 | 4096 | 8192 | 16384 |
| --- | --- | --- | --- | --- | --- | --- |
| nothing written | 0.35 | 0.63 | 1.30 | 2.95 | 6.31 | 14.25 |
| every placement rewritten | 0.33 | 0.64 | 1.33 | 3.06 | 6.30 | 13.79 |

Milliseconds a frame, on a Radeon RX 7900 XTX under Vulkan. The cost is linear at 0.65µs an element,
reaching 0.87µs at the largest size. It is 58% of what a frame costs when nothing at all has
changed, and a 60Hz budget runs out at around 13000 elements that are not moving.

Moving it means a dirty set, and the constraint is R2: resolution is dependency-ordered and an anchor
may point anywhere — a later sibling, a cousin, another subtree — so a dirty set has to close over
everything anchored into it before it is safe to resolve only that. What it would save is the figure
above, which reading by position has already taken 42% off: the closure over anchors costs what it
always did, against a smaller prize.

### What the passes spend it on

Not arithmetic — reaching for one element at a time. Two costs of about the same size, each around
ten nanoseconds against a few for the sums between them: a component read, which was a lookup by
entity; and a step of an accumulation, which was a hash on a map keyed by element.

Both are now read by position. `Ordering` holds each element's trunk and anchor as positions rather
than names, `gather` reads what every element offers a placement before the axes run, and the
accumulations R3 through R7 carry are arrays the length of the order. A write
that used to go through `insert` — the bundle machinery, which is what a component arriving for the
first time needs and not what one being overwritten every frame does — is now made in place, since
`grow` puts every one of them on the element when it is planted.

Together those took 42% off resolution. At 4096 elements: `clip` 0.49ms to 0.10, `scroll` 0.60 to
0.17, `rank` 0.39 to 0.13, `inherit` 0.44 to 0.16, `axis` 1.54 to 0.76 across its two passes, and
`extent` 0.40 to 0.25 — against 0.17 for the gather itself.

What is left is R1 and R2m — `wrap` 0.86ms and `measure` 0.48 at 4096 elements, together 46% of
resolution and the two the rewrite did not reach.

### R1 and R2m read and write in the same walk

Both reach for a typeface, a run and a placement, and the last two are handed out by reference rather
than copied. A reference into the world cannot be held across a write to it, and both passes write a
measure for each element as they go — so neither can be gathered the way the passes around them are.

Splitting each into three would make them able to be: one shared-borrow gather holding what the pass
reads, the arithmetic against arrays holding what it produces, and one exclusive scatter writing
`Cell` and `Intrinsic` back at the end. The shaping cache is reached mutably from the middle of
that, which is a different field of the grove than the tree and so already a separate borrow — R1
destructures for it as it stands.

**What it is worth.** Unmeasured. On what reading by position was worth to the passes it did reach,
between a third and a half of the pair — 0.5 to 0.8ms at 4096 elements, or a sixth to a quarter of
what resolution now costs. An estimate rather than a measurement, and the first work on it is the
measurement.

**What it costs.** `Tree` hands out one element's value at a time, and owning the world is what lets
it do that. A deferred pass asks instead for a read of the whole tree, computes away from it, and
hands back a batch to be written — so for those two passes the boundary stops being an accessor per
property and becomes a gather and a scatter. That is a decision about what `Tree` is for rather than
a change inside a pass, which is what separates it from the rewrite above.

## Extraction rewrites a changed run entire

The scratch is kept between frames, so a frame that changes nothing allocates nothing there. A run
that changed by one character is still rebuilt whole, and what a run costs is its length.

The diff is deliberately of the run and not of its glyphs: a run is one entry in the one stack, and
the renderer holds its glyphs under its own numbering. A finer diff has to stay on the renderer's
side of that line or it reopens R6's tie-break.

## `Pollen` builds its sets each frame

The sets are built every frame and handed out behind an `Arc`, including on frames where nothing was
reported. The candidate is reusing the buffers across frames, or building only what a frame actually
has to report.

**Measured, and not a cost.** Step 3 — which seals the drift, delivers it, and hands the app its
`Pollen` — takes 0.004ms a frame with nothing reported, and reads the same at 4096 elements as at
8192, so it is not the tree's size either. There is nothing here to reclaim.

## The shared walk

It runs only when the stack moved, and when it runs it walks the whole stack — merging every
renderer's slots by rank, taking each instance's depth from its position, and cutting the walk into
spans on renderer, clip and binding. The gate is already there; the cost when it opens is the stack's
size.

## The sheet never reclaims

A failed pack draws blank and traces once, and nothing is ever evicted. Nothing fills the shared
sheet either: marks are a bounded set packed once, and pictures have a texture each.

Should one ever fill, the packer already implies the shape — **shelves are the reclaim unit**, and
reclaiming one orphans the runs still pointing at its texels, so it needs the character kept per
glyph to re-cut them.
