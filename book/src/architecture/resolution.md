# Resolution

Resolution is steps 6 and 7 of the frame: every element's declarations go in, and its box, its clip,
its place in the stack and its inherited off-states come out. It lives in
[`rowan.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/rowan.rs), `Rowan`, and
it calls one pure function, [the resolver](resolver.md), for the arithmetic of a single axis of a
single element.

```rust,ignore
// rowan.rs, shortened
pub(crate) fn run(grove: &mut Grove) {
    let mut elements = core::mem::take(&mut grove.elements);
    read(&mut elements, grove);          // what to resolve, and the order to resolve it in
    grove.tree.taken();                  // writes after this belong to the next frame
    measure(grove, &mut elements);       // R1
    axes(grove, &mut elements);          // R2a, R2m, R2b
    extent(grove, &elements);            // R3
    scroll(grove, &mut elements);        // R4
    clip(&mut elements);                 // R5
    rank(&mut elements);                 // R6
    inherit(&mut elements);              // R7
    regions(grove, &mut elements);       // R8
    grove.elements = elements;
}
```

## The passes

| | Pass | Direction | Writes |
|---|---|---|---|
| R1 | measure | any | the character cell, the shaped run, and the max-content width |
| R2a | horizontal | dependency order | every left edge and width |
| R2m | wrap | bottom-up | how tall each element's content turned out: the other half of `content()` |
| R2b | vertical | dependency order | every top edge and height, and with them the placed box |
| R3 | extent | bottom-up | how far each scrolling region's content reaches |
| R4 | scroll | top-down | the drawn box: the placed box less every scrolling ancestor's offset |
| R5 | clip | top-down | what the scrolling regions above an element leave visible of it |
| R6 | rank | dependency order | where each element sits in the stack |
| R7 | inherit | top-down | the visible, opacity and disabled products over the ancestry |
| R8 | regions | rank order | the box stack the next frame's dispatch reads |

Each pass writes exactly one thing, and nothing writes what another pass owns (F8). Each runs exactly
once per frame: **nothing iterates to a fixed point.**

## Held between frames

Everything resolution produces lives in one struct, `Elements`, kept on the grove between frames
and never written back into the world:

```rust,ignore
// rowan.rs, shortened
pub(crate) struct Elements {
    pub(crate) order: Vec<Leaf>,      // every live element, in dependency order
    trunk: Vec<Option<usize>>,        // what each hangs off, as a position in `order`
    anchor: Vec<Option<usize>>,       // what each is anchored to, likewise
    branches: Vec<u32>,               // children, as one flat array:
    children: Vec<u32>,               //   children[branches[at]..branches[at + 1]]
    pub(crate) standing: Vec<Standing>, // what each declared that the later passes read
    section: Vec<Section>,            // the placed box
    intrinsic: Vec<Area>,             // what content() reads
    reached: Vec<Option<f32>>,        // how far each reaches toward its trunk's measure
    cell: Vec<Area>,                  // the character cell
    pub(crate) composed: Vec<Option<Arc<Shaped>>>, // the shaped run
    pub(crate) drawn: Vec<Section>,   // the placed box, moved by scrolling
    pub(crate) clip: Vec<Section>,
    pub(crate) rank: Vec<ResolvedElevation>,
    pub(crate) inherited: Vec<Inherited>,
    dirty: Vec<bool>,                 // resolve this one again
    pub(crate) moved: Vec<bool>,      // something extraction reads changed
    // ...
}
```

The shape is the point. Every pass asks the same questions of the same elements (what does it hang
off, what is it anchored to, what is under it, what does each of those offer a placement), and asked
of the world each one is a lookup by entity. The arithmetic a pass does is small enough that the
lookups, not the arithmetic, were what a frame cost. So the tree is read once into this order,
with every dependency stored as a **position** rather than a name, and from then on every pass
indexes arrays. A trunk's contribution to what its child inherits is `inherited[trunk]`, not a hash
lookup. Moving to this shape took an idle frame at eight thousand elements from 2.9 milliseconds to
0.25 ([What a frame costs](performance.md)).

## The order

An element resolves after its trunk and after its anchor, and an anchor can point anywhere: at a
later sibling, a cousin, an element in another subtree. So the order is not tree depth but a
topological sort of the graph whose edges are "hangs off" and "is anchored to", built with Kahn's
algorithm (`structure` in `rowan.rs`). Each element has at most two dependencies, so the whole graph
fits in one flat array rather than a vector per element.

The order only changes when an element is grown or pruned or an anchor is written, which the tree
records as `restructured`. On any other frame the order is not touched. When it is rebuilt, every
column is carried from each element's old position to its new one, so an element keeps what it
resolved to however the order moved around it, and the elements that went are listed in `gone` for
extraction to let go of.

Anchor cycles are refused when they are written, but a trunk and an anchor can still form one
between them: an element anchored to something grown under it waits on a box that is waiting on
its own. That is not a contradiction (both boxes resolve, just not against each other's settled
value), so whatever Kahn's leaves over is appended in allocation order and resolves against what its
dependency last was. **Every live element is in the order.** An element left out would have no box,
no rank and no place in the stack, and nothing would say so.

## What resolves again

`read` works out, before any pass runs, what this frame has to do. It starts from
[what the tree recorded](tree.md#recording-what-was-written):

- an element whose declarations were written (`touched`) is **dirty**;
- one that was only restyled has its `Standing` read again and is marked **moved**, and resolves
  again only if something it is placed by was also written;
- a repaint marks everything moved and nothing dirty;
- an invalidation (a resize, a breakpoint, a font arriving) marks everything dirty.

Then one walk down the order closes it over dependencies: an element is dirty if it was written, or
if its trunk or its anchor is dirty. Because the order puts every dependency first, that is a single
pass rather than a search.

```rust,ignore
// rowan.rs, in `read`
for at in 0..count {
    let decided = |on: Option<usize>| on.is_some_and(|on| on >= at || elements.dirty[on]);
    let dirty =
        elements.dirty[at] || decided(elements.trunk[at]) || decided(elements.anchor[at]);
    elements.dirty[at] = dirty;
    elements.moved[at] |= dirty;
    elements.measures[at] |= dirty;
    elements.resolved[at] = !dirty;
    // ...
}
```

A clean element is skipped by every pass that computes a per-element answer, and its column already
holds what it resolved to last frame, which is exactly what resolving it again would produce. The
module comment puts it this way: **a value here can be old. It cannot be wrong.** Nothing is ever
marked stale, and there is no invalidation to get wrong, because what decides is the record of what
was written, made where it was written.

The passes that carry an accumulation (R3 to R7) run over every element anyway, because what they
carry is a running product with no position to start from. They record, as they go, whether what
they wrote differs from what was held, which is how an element that only moved under a scroll comes
to be marked `moved`.

## R1: measure

R1 visits only elements that say something new: grown this frame, text rewritten, or everything
after a breakpoint or a font change. For each, it reads the element's font and size at the
breakpoint in force, takes its character cell from the font, and shapes its run through the
[shaping cache](text.md). The cell is what `letters` counts in. The run's **max-content width** is
its longest line's character count times the cell width, which in a monospaced face is exact and
needs no layout at all. That is written as the width half of the element's intrinsic size.

## R2: the two axes, and the measure between them

Text wrapping makes a height depend on a width, and the width comes from layout, so a box sized to
what it holds looks circular. It is not, because **width flows down and height flows up**:

1. **R2a** resolves the horizontal axis of every dirty element, in dependency order. Every input is
   available: widths come from trunks, and a run's max-content width came from R1 without any
   layout.
2. **R2m** walks bottom-up and measures. Each run wraps at the width R2a just gave it, and its line
   count times its cell height is its measured height. An element with children also reaches over
   them: the furthest any of them extends below its top edge. The element takes the greater of the
   two as the height half of its intrinsic size.
3. **R2b** resolves the vertical axis, and `height(content())` reads what R2m measured.

R2m is the one moment in the frame when every width is known and no height is, which is exactly what
measuring needs.

The reach over children counts only children whose vertical placement **describes its own extent**:
pixels, letters, its own content, any reading of the already-settled horizontal axis. A child placed
by a percentage of its trunk's height, a row of its trunk's grid, or an anchor's edge is asking how
tall something else is, so it cannot also be what decides that height. It is left out of the measure
and given its real height by R2b like anything else. This is `Config::measurable`
([`placement/role.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/placement/role.rs)),
and it is the reason [Placing elements](../usage/placement.md#sizing-a-box-to-what-it-holds) states
the rule it does.

R2m also marks what has to measure again, and it runs *up* the order rather than down it: a measure
travels toward the trunk and stops there, where a box travels away from it. An element that measured
to something new is marked dirty, and R2b's own closure (`dirty[at] |= dirty[trunk] || dirty[anchor]`
at the top of each axis) spreads that to everything placed against it. That is why R2m sits between
the axes and not after them.

An element measuring again because one child moved does not ask the others. How far each child
reaches is a column of its own, `reached`, written when the child is resolved in its trunk's reach
and read back otherwise:

```rust,ignore
// rowan.rs
fn reach(grove: &Grove, elements: &mut Elements, at: usize) -> f32 {
    let mut reach: f32 = 0.0;
    for index in elements.branches[at] as usize..elements.branches[at + 1] as usize {
        let child_at = elements.children[index] as usize;
        if elements.dirty[child_at] {
            let child = elements.order[child_at];
            let far = measurable(grove, child).then(|| {
                let context = raised(elements, at, child_at);
                geometry(grove, child, &context, Axis::Vertical).0.far
            });
            elements.reached[child_at] = far;
        }
        if let Some(far) = elements.reached[child_at] {
            reach = reach.max(far);
        }
    }
    reach
}
```

It is the rule every other column keeps, applied to one more answer. What a child's reach reads is
its own placement, the horizontal half of what it and its trunk and anchor resolved to, and its own
measure; each of those that moves makes the child dirty, the last one here in R2m itself, before
the walk reaches the trunk. So a container of thousands with one child written resolves one child
to measure itself, and the column is never behind.

**A motion in progress is resolved inside the same axis.** For an element with a `Location` motion
running, R2 resolves the target *and* the placement it departed from, in the same context, and
blends the two. Both ends are answered against this frame's trunks and anchors, so a resize or a
moved anchor mid-motion reaches both at once. That only works because the resolver is pure: it can
be called twice for one element on one axis with nothing to go stale. [Motion](motion.md) has the
rest.

## R3: extent

Bottom-up, each scrolling region takes the far corner of everything under it, as placed by R2:

- a hidden element is not content, and neither is anything under it;
- a `pinned` child does not move with the content, and a `floats` child sits over the region rather
  than in it, so neither counts;
- a child that scrolls in its own right contributes its box, not its content;
- only a declared axis has an extent; an axis the region does not scroll reads its own box;
- the extent is measured from the region's near edge and is never smaller than its own box.

Two things are deliberately not consulted: what is drawn, because content scrolled out of sight is
exactly what an extent describes; and the offset, because the offset is clamped to the extent, and
measuring against it would make the extent depend on itself.

## R4: scroll

R4 first answers the three ways a region is asked to move, because all three need the extent R3
just measured: a coast still running from a release, a `scroll` written this frame, and a scroll
motion part way through ([Scrolling](views.md)). Then it walks down the order, accumulating offsets:
an element's drawn box is its placed box less the sum of every scrolling ancestor's offset.

A region's own box does not move under its own offset; what moves is everything grown inside it. A
pinned child receives every offset except its nearest region's. And each offset is clamped against
the extent as it is applied, so a region whose content shrank comes back into range in the same
frame rather than sitting somewhere it can no longer reach.

The placed box and the drawn box are kept apart on purpose. A change to the placed box means the
layout moved something and its children must follow. A change to the drawn box means only that it
moved under a scroll, and nothing needs resolving for that.

## R5: clip

A clip is a rectangle and nothing else: the intersection of the drawn boxes of every scrolling
region above the element. A region does not clip itself, only what is inside it, and an element with
no scrolling ancestor is not clipped at all. A floating element escapes as far as its `Escape` says:
the region it is in, every region, or every region up to a named element.

Whether an element is *culled* is not decided here. That is extraction's decision, made from this
rectangle, and never recorded on the element, so nothing downstream (least of all the extent) can
read "currently clipped away" as a state.

## R6: rank

Elevation accumulates down the tree: an element's rank is its trunk's rank plus its own declared
offset. Ties are broken by allocation order, the `Growth` taken when the name was handed out, so the
order is total and two runs of the same script stack identically. Nothing here reads a box: where
an element sits in the stack has nothing to do with where it sits on the surface.

## R7: inherit

The three off-states are declared per element and resolved as a product over the ancestry: visible
if the trunk is visible and it is; opacity multiplied through; disabled if the trunk is disabled or it
is. There is no cascade to write and none to get wrong. An element grown under a disabled trunk is
disabled on its first frame because the pass does not care when it arrived, and enabling the trunk
leaves anything disabled in its own right disabled because nothing was overwritten on the way down.

## R8: regions

The box stack is every element that is present (visible, and not fully transparent) and not clipped
away entirely, sorted front-most first, each carrying its drawn box, its clip, its hit shape, and
whether it is tangible, receives, and is disabled. It is what the next frame's dispatch reads
([Interaction](interaction.md)).

It is rebuilt only on a frame where something it is built from changed, which R4 to R7 recorded as
`restack`, and re-sorted only when a rank changed. A frame where one element's color fades, or
nothing happens at all, keeps the stack the last one built.

## Two cycles that are not vicious

Resolution has two places that look circular, and each is resolved in single passes rather than by
iterating:

- **Width and height.** A wrapped height depends on a width. Widths never depend on heights (the
  [type system](resolver.md#the-order-is-in-the-types) refuses to let a horizontal role read one),
  so R2a, R2m and R2b in that order are enough.
- **Extent and offset.** A child resolves against its region's box, the region's extent comes from
  where its children landed, and the offset is clamped to the extent. But the extent affects only the
  clamp, never the region's own box, so a top-down layout, a bottom-up extent and a top-down scroll
  settle it.

## What resolution hands on

- **`moved`**, per element: whether anything extraction reads changed. Extraction visits only these.
- **`gone`**: every element that withered since the order was last built, and what it drew.
- **The box stack**, for the next frame's dispatch.
- **Every column**, for `tap` to read and for the next frame to start from.

And one housekeeping step: after R1, if any run stopped being stated this frame, the shaping cache is
swept of runs no element still holds.
