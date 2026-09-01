# Rowan — resolution

How declared state becomes resolved geometry. Steps 5–7 of the frame (`frame.md`).

## Thesis

> **Rowan recomputes. Elm decides what changed.**

Resolution runs over everything, every frame. There is no dirty tracking, no invalidation, and no
mechanism by which a value can be stale because something forgot to mark it.

This is the whole of the redesign. The previous engine tried to be incremental — component
lifecycle hooks and per-entity observers firing on write, with `react`/`react_any`/`refire` to make
the first fire and every later fire share a path. Every ordering bug, every stale read, and the
entire `Refire` / `TargetedEvent` / `IntoTargets` apparatus existed to service that ambition. It
was never load-bearing: it was there so *app* code could register reactions, and app code has not
been able to since the boundary landed.

The cost of dropping it is arithmetic over a few numbers per element per frame. For thousands of
elements that is microseconds. The saving is an entire class of bug.

Nothing is re-uploaded to the GPU on account of this: `Elm` compares resolved values against a
cached copy and extracts only genuine differences. Recomputation is CPU-side and invisible
downstream.

**The one exception is shaping.** Turning a string into positioned glyphs is genuinely expensive
and is memoized on `(value, font, size)`. Geometry is recomputed totally; shaping is recomputed
when its inputs differ. That is a measured exception, not a hedge — everything else recomputes.

## The resolver

The core is a pure function:

```rust
fn resolve(config: &Config, ctx: &Context) -> Span

struct Context {
    axis:        Axis,      // which axis is being resolved
    parent:      Section,   // the parent's layout box — what a grid divides
    anchor:      Section,   // the anchored element's geometry; zero when there is none
    intrinsic:   Area,      // this element's own measured size, for content()
    tracks:      Tracks,    // the parent's grid, at the breakpoint in force
    cell:        Area,      // this element's own character cell, for letters()
    parent_cell: Area,      // the parent's, for a letter-pitched track
}
```

One axis at a time, because that is how R2a and R2b call it. The breakpoint is resolved *before*
this — picking a `Config` out of a `Location` and a `Tracks` out of a `Grid` is a lookup, not
arithmetic, and keeping it out leaves the resolver reading only numbers.

There are two character cells because there are two fonts. An app registers as many as it likes and
each element chooses, so `8.letters()` is eight cells of the element's own, while a letter-pitched
grid track is in the font of the element the grid is *on*. Collapsing them into one is what would
stop a letter-pitched grid being addressable by children of a different size.

No world access, no entity ids, no mutation. Given the same inputs it gives the same answer, and it
may be called as many times per entity per frame as anything needs.

Two consequences, both large:

**Aspen's requirement is satisfied by construction.** An animating element resolves both its
endpoints in the same context and interpolates the results (`aspen.md`). This is the requirement
that made the pure-function shape non-negotiable — the old design could not resolve an entity twice
because resolution *was* the mutation.

**The layout algebra is testable as arithmetic.** Every placement rule, unit, breakpoint fallback
and anchor case is a test over a struct in, a struct out — no ECS, no engine, no frame. This is
where the bulk of `placement.md`'s proof obligations get discharged, and it is why they can be
exhaustive.

## The passes

In order. Each writes exactly one thing, and nothing writes what another pass owns (F8).

| | Pass | Direction | Produces |
|---|---|---|---|
| R1 | `measure` | — | intrinsic max-content widths, icon and image natural extents |
| R2a | `horizontal` | top-down, dependency order | the horizontal axis |
| R2m | `wrap` | — | each text run wraps at its resolved width → intrinsic height |
| R2b | `vertical` | top-down, dependency order | the vertical axis → `LayoutSection` |
| R3 | `extent` | **bottom-up** | each view's scrollable extent, from its children's `LayoutSection`s — never from clip or culling (`views.md`) |
| R4 | `scroll` | top-down | offsets clamped to extent, accumulated down; `Section` = `LayoutSection` − accumulated offset |
| R5 | `clip` | top-down | each element's clip rect: the intersection of its ancestors'. A rect only — whether something is *culled* is Elm's decision at extract, never a state on the element |
| R6 | `rank` | — | `ResolvedElevation`, with a stable tie-break |
| R7 | `inherit` | top-down | visibility, opacity and disabled products (`lifecycle.md`) |
| R8 | `regions` | — | hit regions: section, shape, enabled, and opacity |

### Why R3 goes the other way

R2 through R5 look top-down, but R3 is not, and the reason is a genuine cycle in the problem:

- a child resolves against its parent's box
- a scrolling parent's *extent* is derived from where its children ended up
- the parent's scroll offset is clamped to that extent
- that offset moves the children

The cycle is real but not vicious, because extent affects only **scroll clamping**, never the
parent's own box. Splitting it into a top-down layout sweep, a bottom-up extent sweep, and a
top-down scroll application resolves it in one pass each, with no iteration to convergence.

This is also the root of the scrolling problems in `views.md`: "extent comes from children" is a
back-edge in the dependency graph, and a child parked outside its parent's box feeds that back-edge
a scrollbar nobody asked for. The fix belongs in `views.md`; the structure it has to live inside
is here.

### Why R2 is three passes

Width flows down and height flows up, which is what makes text that wraps sizable at all. The
split is bounded at exactly two resolution passes with no iteration, because a monospace run's
max-content width is free — see `placement.md`, which owns this.

Both sub-passes call the same pure resolver. That is only safe because it is pure: there is no
accumulated state for a second call to corrupt. This is the second requirement purity buys, after
Aspen's.

### Anchors make R2 a DAG

An anchor may point anywhere — a later sibling, a cousin, an element in another subtree. So R2 is
ordered by dependency, not by tree depth: an element resolves after its parent *and* after whatever
it anchors to.

An animating element's two endpoints resolve at its own position in that order, against the same
settled dependencies, so the two answers are consistent with each other. An element anchored to an
animating element sees the interpolated box, which is correct — it should track the motion, not the
destination.

## What is gone

`Author`, `LeafSprout`, `Sprout`, `Spec`, `WithExtras`, `Refire`, `IntoTargets`, `TargetedEvent`,
`Tree::react`, `react_any`, `refire`, `forward`, and every component lifecycle hook that existed to
trigger resolution.

Spawning inserts components. It triggers nothing. The next frame's R1–R8 resolve the new element
along with everything else, because a fresh element is simply part of "everything".

This is what makes the spawn path uniform: there is no two-phase spawn-then-patch, no ordering
requirement between "trimmings" and the real bundle, and no first-resolve-against-placeholder
hazard. Those all existed to get reactive hooks to fire against real data. Nothing fires.

## Where events legitimately remain

Observers and messages are kept for things that genuinely *are* events, arriving from outside the
frame's control:

- window and input events (step 1)
- asset arrival, which is asynchronous by nature

Never for recomputing derived state. That is the line.

## Proof obligations

Pure, against the resolver alone:

- every unit, designator and breakpoint fallback, exhaustively
- anchor resolution against a given target box
- `content()` against a given measure
- aspect-ratio constraint

Headless, against the passes:

- a child parked outside its parent does *or* does not grow the extent, per `views.md`
- nested scrolling regions accumulate offsets correctly
- clip rects intersect through several levels
- equal-elevation siblings rank in the stable order F-defined by `lifecycle.md`
- opacity and visibility products through several levels
- an element resolves correctly on the frame it is spawned, with no extra frame of settling
- resolution is idempotent: two ticks with no input produce identical output

## Open

Anchor cycles are settled: a **hard error, refused at the op that would create one**, so the tree
is never cyclic and R2's ordering is a valid DAG by construction. See `placement.md`.
R6's tie-break is settled: **allocation order**, which is total, monotonic and unaffected by
prunes because names are never reused. See `lifecycle.md`.
