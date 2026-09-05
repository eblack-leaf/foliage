# Optimisation

Candidates, not defects. Each is deliberate as built and none is measured, so the first work here is
a measurement rather than a change — the shapes below are what to look at when a frame is too
expensive, and what each one would cost to move.

## Rowan recomputes the whole tree every frame

The frame's cost is the tree's size rather than the change's. Nothing is skipped and nothing is
cached, which is what makes a resolution reproducible and every ordering question answerable in one
place.

Moving it means a dirty set, and the constraint is R2: resolution is dependency-ordered and an anchor
may point anywhere — a later sibling, a cousin, another subtree — so a dirty set has to close over
everything anchored into it before it is safe to resolve only that.

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

## The shared walk

It runs only when the stack moved, and when it runs it walks the whole stack — merging every
renderer's slots by rank, taking each instance's depth from its position, and cutting the walk into
spans on renderer, clip and binding. The gate is already there; the cost when it opens is the stack's
size.

## The sheet never reclaims

A failed pack draws blank and traces once. Nothing fills the sheet today — see the eviction note in
`TODO.md` — so this is a candidate only once something does.
