# Polyline

What a path element would be if foliage draws one, and what it costs.

## What a chain answers, and what it does not

A path is written today as a chain of `Line`s meeting end to end. Drawn opaque with `Cap::Round` that
is correct at every weight and every turn angle: a round cap is a disc of half the weight centred on
the point, which is exactly the wedge two rectangles leave open on the outside of a turn, so the
chain is indistinguishable from a path the engine knows about.

Two things are outside it.

- **Partial alpha.** Two strokes are two elements and therefore two blends, so wherever they overlap
  the ground is painted twice. Under a fill stated at partial alpha, or one passing through it on a
  `Motion::Opacity` in or out, every vertex reads heavier than the stroke it joins -- visibly from
  around eight logical pixels of weight. Caps cannot answer it, because the overlap is what covers
  the wedge.
- **Anything measured along the path.** A dash pattern and `Motion::DrawProgress` are both positions
  in arc length, and a chain of separate strokes has no arc length: each segment knows its own two
  ends and nothing about where it sits in the whole.

## The unit of evaluation is the pixel, not the path

One quad over the path's bounding box, with the fragment stage measuring against every segment, is
the expensive reading of "one coverage evaluation". A path is mostly empty space inside its bounds
and the per-pixel cost would grow with the number of segments, so both the area drawn and the work
per pixel would be paid where there is no ink.

Nothing about the geometry needs to change. The line renderer already places an *oriented* quad
along its segment and measures an exact field inside it, so a stroke costs its own footprint rather
than its bounding box, and a path built out of those quads costs the sum of them.

What has to change is which quad answers a pixel. At a shared endpoint the two segments meeting
there both cover the pixels around it, and the bisector of the two directions is the locus where
they are equidistant. So each segment carries its neighbours' bisector normals and **discards
fragments on the neighbour's side**. Every pixel is then answered exactly once, by the segment
nearest to it -- and on its own side of the bisector a segment's own field *is* the field of the
union, so the coverage it computes is the coverage the path has.

That is what makes it correct under partial alpha, and it is also why the answer is not simply to
abut the quads: two feathered edges meeting sum to about three quarters of full coverage and draw a
faint seam along every join. A claimed pixel is measured against the real shape rather than against
a truncated one.

## The shape in the stack

**One element, one entry, many quads** -- the shape a text run already has. The path takes one rank,
one clip and one hit-testable box, and the renderer holds its segments under its own numbering, so
the stack never learns what a segment is. That is what makes a path one opacity, one fill and one
diff; it is not what makes the joins composite, which is the local question above.

Extraction states the path entire and compares it entire, as it does a run: a point that moves
rewrites the path's quads. What a path costs is its length.

## Joins fall out of the field, not out of a join table

Past its own endpoint a segment's field is already a disc of half the weight, because the projection
onto the segment is clamped -- that is what draws a round cap. Under the bisector claim each of the
two segments draws its own half of that disc and neither draws the other's, so a **round join is
exact and free**: the same disc as today's round caps, drawn once instead of twice.

A **miter** is that field allowed to reach as far as the intersection of the two outer edges, bounded
by a limit past which it becomes a **bevel** -- the disc cut off by the line between the two outer
corners. All three are a bound on how far past the bisector the field may reach, rather than three
pieces of geometry.

The two ends of the path keep `Cap`, which means what it means today.

## Arc length carries dashes and draw progress

Each segment carries where it starts along the path and how long it is. The fragment stage already
computes the projection along the segment, so the arc position of a pixel is one multiply-add on a
value it has.

- A **dash pattern** is a modulo on that position. Because the start accumulates along the chain, the
  pattern is continuous through the joins, which is the one thing a chain of separate strokes can
  never be however it is written.
- **`Motion::DrawProgress`** is the same gate with one bound: the path is drawn where its arc
  position is below the reveal. Revealing by arc length needs the resolved positions of every point,
  which the same walk that assigns the arc starts already has.

Both are one mechanism, which is why they belong to the same slice as the element.

## What it costs

- **Per instance**, beside what a `LineQuad` already carries: two bisector normals, the segment's
  start along the path, and the path's length. The dash pattern is the element's rather than the
  segment's.
- **Per pixel**: a dot product and a discard, over a field that is evaluated already. A pixel near a
  join costs what a pixel anywhere else does.
- **Per frame**: a walk of the path's resolved points to accumulate arc length and the bisectors,
  which is the pass `DrawProgress` needs anyway.

## What it does not answer

A path that crosses **itself**. Two segments that do not share an endpoint have no bisector between
them, so at partial alpha a crossing is still painted twice. Only drawing the path into a coverage
target and compositing it once answers that, which is a target and a pass per path -- not worth it
for the paths a page draws.

## Snapping is a stroke's, not a path's

`LineQuad::new` puts an axis-aligned stroke on whole device pixels, moving both of its ends and its
weight. That is right for a rule, which is a stroke on its own and wants to be crisp. It is wrong
inside a path, where it moves the endpoint a neighbour is meeting -- so a segment of a path is
placed where it was told and the feather answers for it, whatever angle it runs at.

## When it is worth building

For opaque paths, it is not: the chain is already right, and what a `Polyline` would add is a dash
pattern, a draw progress and a pointed join. It becomes worth building the moment a path can be
translucent or can fade, because from that point the chain is wrong at every vertex and nothing an
app can write makes it right.
