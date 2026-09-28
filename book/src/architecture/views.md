# Scrolling

A scrolling region is an ordinary element that declared `scrolls`. Everything particular to it lives
in [`view.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/view.rs), and the passes
that compute it are three of resolution's: R3, R4 and R5.

## An element scrolls because it said so

```rust,ignore
// view.rs
pub struct Scroll {
    axes: Axes,              // which axes it scrolls
    contains: Option<Axes>,  // which of those absorb a gesture at their end
}
```

A grid divides a box and says nothing about scrolling, and content overflowing a box does not make it
scroll. An axis that was not declared does not scroll, has no extent and cannot be moved. It is not a
scrolling axis with a range of zero, which is a different and harder thing to reason about.

The axes and what each does at its end are one value rather than two flags, because a policy for an
axis the region does not scroll would describe nothing, and two declarations that could disagree
about which axes are in play is exactly the failure the per-axis split exists to avoid.

## Three values, three passes

| Value | Written by | Holds |
|---|---|---|
| `Extent` | R3, bottom-up | how far the region's content reaches from its own near edges |
| `Offset` | R4, top-down (and dispatch, under a drag) | how far the region has been moved, clamped to what it can reach |
| clip | R5, top-down | the box the regions above an element leave visible |

The **extent** is measured from where the children were *placed*, never from what is drawn: content
scrolled out of sight is exactly what an extent describes and has to stay reachable. It counts only
declared axes, is measured outward from the region's near edge (so content at a negative offset
creates no room to scroll back into), and is never smaller than the region's own box. A hidden child,
a pinned one and a floating one are not content. [Resolution](resolution.md#r3-extent) has the pass.

The **range** a region can move is its extent less its own box, per axis, and never negative.

The **offset** is in logical pixels, read and written, and positive means content moved toward the
near edge, which is what a drag away from that edge produces. R4 clamps it against the extent R3 just
measured, so a region whose content shrank under it comes back into range in the same frame.

**Culling** is not a state. R5 computes a rectangle, and extraction decides from it whether to draw
an element. Nothing records "currently scrolled away", so nothing (the extent least of all) can
read it.

## Four ways to move a region

| Cause | Where it is decided | Where it lands |
|---|---|---|
| a drag, or a wheel | dispatch, against last frame's extent | written to the offset at once |
| `scroll(region, ScrollTo)` | the drain records it | answered in R4 |
| `Motion::Scroll` | the drain starts it | blended in R4 |
| a coast from a release | dispatch starts it | advanced in R4 |

The three that land in R4 are answered there rather than where they were written, in one function,
because all three need the extent, and the extent is R3's, one pass earlier in the same frame:

```rust,ignore
// view.rs
pub(crate) fn asked(grove: &mut Grove, elements: &Elements) {
    coast(grove, elements);
    sought(grove, elements);
    animated(grove, elements);
}
```

So scrolling to the end of a list that grew this frame lands at the end of the list as it is *now*,
with no settling frame.

They never contend, because each ends the others where it starts: a `scroll` cancels any scroll motion
on the region and stops its coast; starting a scroll motion stops the coast; and a drag halts the
coast and cancels the motion. The person's hand always wins, and the last thing written wins among the
rest. That is F8, one writer per property, applied to a region's position.

## Destinations

A `ScrollTo` names its unit at the call site:

| | Lands at |
|---|---|
| `ScrollTo::px(n)` | `n` logical pixels from the content's origin |
| `ScrollTo::fraction(f)` | `f` of the range, where 1.0 is as far as it goes |
| `ScrollTo::start()`, `end()` | either end of the range |
| `ScrollTo::show(leaf)` | the least movement that brings `leaf` into view, and none if it is already in view |

Every form but `px` is stated **relative to the region's own range**, so it means the same thing on
either axis and needs no axis named: the end is the end of each, and bringing an element into view is
one place in two dimensions. `px` is the one absolute distance, and on a region that scrolls both ways
two hundred pixels down and two hundred across are unrelated distances that happen to share a number,
so it has to name an axis with `.on(..)`. The drain checks all of this when the op is applied (the
region scrolls, the destination moves an axis it scrolls, the element `show` names is grown under the
region) and drops the op with a reason if not.

`show` is a subtraction: the element's placed box against the region's, on each axis the destination
covers, moved by the least that brings its near edge into view or its far edge in, clamped to the
range.

## Coasting

A release hands the region that held the drag its velocity: the mean over the last 100 milliseconds
of the gesture ([Interaction](interaction.md#a-gestures-life)). If it is at least `Momentum::minimum`
(40 logical pixels a second by default) and the half-life is not zero, the region **coasts**.

A coast is deliberately not a motion. A motion interpolates between two known ends over a known
duration; a coast has no target and no duration, and runs until it settles. It decays continuously:

```rust,ignore
// view.rs
pub(crate) fn coasted(velocity: f32, half_life: f32, elapsed: f32) -> (f32, f32) {
    if half_life <= 0.0 {
        return (0.0, 0.0);
    }
    if elapsed <= 0.0 {
        return (0.0, velocity);
    }
    let remaining = 0.5f32.powf(elapsed / half_life);
    let travelled = velocity * half_life * (1.0 - remaining) / core::f32::consts::LN_2;
    (travelled, velocity * remaining)
}
```

The distance is the integral of the speed, not one frame's speed times its duration, so a fling reaches
the same place at thirty frames a second as at a hundred and twenty. Left to run out, a coast travels
its release speed times the half-life divided by ln 2, about 1.44 times their product, less what the
minimum speed cuts off at the tail.

Like a motion, a coast's first frame is not charged: it begins at the dispatch that released it, and
that frame's delta is the interval the drag was still being made in, which the drag already paid for.

When a coast reaches the end of its range, it does what a drag would do at the same place: hands the
remaining speed outward to the next region in the chain that scrolls on that axis, or, if the region
contains that axis, stops there. Below the minimum speed it settles. A new press on a coasting region
catches it: the coast stops, and the press is spent on the catch.

Coasts are held beside the tree, keyed by region and axis, because the two axes reach their ends
independently and each hands outward on its own. The loop owes frames for as long as any coast is
running, since nothing on the app's side is asking for them.

## Chaining, and the one knob

By default every axis **chains**: a region that can move no further in the direction asked hands the
gesture outward, so a drag inside a list keeps moving the page once the list is done. `contain` on
an axis makes the region absorb it instead, for a region that owns its gesture outright (a map, an
editor, a pane in a fixed shell) where reaching the bottom and having the whole page lurch is a bug
every time. The same question is asked for a drag, a wheel and a coast, of the same region at the
same place in its range, because there is one answer to it:

```rust,ignore
// interaction/mod.rs
pub(crate) fn outward(grove: &Grove, chain: &[Leaf], index: usize, axis: Axis) -> Option<usize> {
    if grove.tree.scrolls(chain[index]).is_some_and(|scroll| scroll.absorbs(axis)) {
        return None;
    }
    scrolling(grove, chain, index + 1, axis)
}
```

Whether a region "can no longer consume" needs no flag either: a region at its end takes none of a
delta going further out and all of one coming back, so the answer is about where the region sits,
not about anything declared.

## Out of the flow: pinned and floating

Two declarations take an element partly out of its region, and each is **one declaration with two
consequences**, because the two are one question asked twice and two flags could drift out of
agreement:

| | Moves with the content | Counts toward the extent | Clipped by the region |
|---|---|---|---|
| ordinary | yes | yes | yes |
| `pinned` | **no** | **no** | yes |
| `floats(escape)` | yes | **no** | **no** |

A **pinned** header stays at the top while content slides under it: it receives every scrolling
offset except its own region's, and it keeps its place in the tree, so the clip, the opacity product
and the disable cascade still apply. A **floating** menu sits over its region rather than in it: it
travels with the row that opened it, is not cut off at the region's edge, and invents no room to
scroll to.

How far a floating element escapes is stated, because both answers are right somewhere. A menu on a
row of a table inside a scrolling panel wants out of both; a tooltip on a file in a list inside a
sidebar wants out of the list and not out of the sidebar, or it paints across the page beside it.
`Escape::Region` leaves the region the element is in, `Escape::Surface` leaves every region, and
`Escape::Within(leaf)` leaves every region up to a named one, which still holds it. Naming the
element rather than counting regions is what survives a wrapper being added in between. A name that
is not an ancestor falls back to `Region`, rather than escaping further than it was told to.

## Reading a region

`tap` reads a region's `Offset` and `Extent` directly, and `Progress` is derived on the spot: the
offset over the range, per axis, in `0.0..=1.0`, which is what a scrollbar reads. A region with
nowhere to go reads zero, which is the honest answer: there is no progress through a range of
nothing. All three answer `None` on an element that does not scroll, which keeps "an axis that was not
declared has no extent" true of the whole element as well as of one of its axes.

## What scrolling costs

A scroll moves the drawn box of everything inside the region, so every descendant is `moved` for
every frame of a drag or a coast, and extraction revisits each of them. Most of them are usually
outside the clip and culled. [What a frame costs](performance.md) has the numbers and the candidate
improvements.
