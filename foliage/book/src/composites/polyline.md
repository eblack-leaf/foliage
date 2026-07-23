# Polyline

`Polyline` has no rendering pipeline of its own -- it's built entirely from two existing
primitives, [`Line`](../line.md) and [`Polygon`](../polygon.md):

```rust
// foliage_proper/src/composite/polyline.rs
pub struct Polyline {}
pub struct PolylinePoints(pub Vec<Position<Logical>>);  // the vertex chain, rewritten as one unit
```

A `Line` segment already draws an antialiased quad; a full-`rounding` `Polygon` already
draws a true circle regardless of side count (see [Polygon](../polygon.md)). A polyline
is just the composition: one `Line` per pair of consecutive points, plus one round
`Polygon` "joint" at every interior vertex.

## Why joints exist

The doc comment is precise about the actual geometric problem: `Line`'s end caps are
square (each segment is an independent quad), so two segments meeting at an angle leave
a wedge-shaped gap on the outer side of the turn. A circle of diameter equal to the line
weight, centered exactly on the shared vertex, covers that gap exactly -- the standard
round-join technique, correct because each segment's near corners sit precisely
`weight / 2` from the vertex, which is exactly the circle's own radius.

## Draw-in, without spawning or despawning anything

```rust
// foliage_proper/src/composite/polyline.rs
pub struct PolylineDrawProgress(pub f32); // 0.0..=1.0 of the path, by arc length
```

The doc comment spells out the actual design constraint this satisfies: the full point
list always determines segment/joint *count* -- and therefore the whole entity pool --
completely independent of `PolylineDrawProgress`. So animating progress from 0 to 1 (a
"draw the line in" effect) never spawns or despawns a single entity after the first
frame; it only ever changes how much of each already-existing segment is visible.
Defaults to `1.0` (fully drawn), so ignoring this entirely reproduces the plain,
already-drawn polyline.

## A sliding window's own bookkeeping

```rust
pub struct PolylineDroppedPoints(pub usize);
```

For a caller trimming old points off the *front* of a live-updating line (a scrolling
chart, a trail that only keeps the last N points) -- `PolylineDroppedPoints` is the
running total ever dropped, written alongside the shrunk `PolylinePoints` on the same
tick. It's cumulative, not a per-write delta: `Polyline` tracks its own last-seen total
internally and diffs against it, so re-sending the same total, or never touching this at
all (it defaults to `0`), is always a safe no-op. It's only meaningful without a `dash`
pattern active -- a dash's segment boundaries don't correspond 1:1 with point count, so
with one active, this is read but not acted on.
