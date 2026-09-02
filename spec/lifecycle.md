# Lifecycle

What a `Leaf` names over time, and the three separate ways an element can be "off".

## States

```
Planted ──▶ Live ──▶ Withered
```

| State | Means |
|---|---|
| `Planted` | Named, but the op that grows it has not been applied. The normal state for the rest of the frame that planted it |
| `Live` | In the tree |
| `Withered` | Pruned, or taken down with an ancestor. **Terminal** |

**A name is never reused.** Allocation is monotonic, so a `Leaf` that has withered can never come
to mean something else — which is what makes holding one indefinitely safe.

Sampling a `Planted` leaf reads absent, because there is nothing there yet. That is not an error
state to check for; it is the honest answer for one frame.

## Stale handles are inert

Every op naming a withered `Leaf` is dropped silently (F2). Not an error, not a panic, not a
result to unwrap.

This is what makes teardown races a non-problem: an app may hold a `Leaf` across frames, act on it
after the element is gone, and nothing happens. The alternative — checking presence before every
call — would be noise at every call site to guard against something the engine can answer for
itself.

`prune` takes the element **and everything beneath it**. Each one emits `Withered`, so an app
holding handles into a pruned subtree learns about all of them, not just the one it pruned.

## Three ways to be off

They are genuinely different, and the previous engine's documentation had to keep warning about the
gaps between them. Stated together, the distinctions are the point.

| | Draws | In the box stack | Receives input | In a view's extent | Children follow |
|---|---|---|---|---|---|
| `visible(false)` | no | no | no | **no** | yes |
| `disable()` | **yes** | **yes** | no — and **swallows** | yes | yes |
| `opacity(0.0)` | nothing to see | no | no | yes | yes |

**`visible(false)`** is the real hide. Skipped by drawing, by hit-testing, and by a containing
view's extent, while keeping its state and its `Leaf`.

It means **app intent only**. Being scrolled out of view is not a kind of hidden: culling is a
decision `Elm` makes at extract time and is never recorded on the element, so scrolled-out content
still counts toward extent and can be scrolled back to. `views.md` explains why conflating the two
was what made hiding-by-parking necessary in the first place.

**`disable()`** is inert but present. It still draws — a greyed control is still a control — and it
still occupies the box stack, so it **blocks**: a press on a disabled button is eaten, not passed
through to whatever is behind it. That is the difference between disabled and decoration, and it is
what makes `disable(page_root)` sufficient on its own when a drawer opens: the page goes inert and
stops swallowing nothing. No separate scrim.

Swallowing covers every kind of input uniformly — taps, drags, and scroll. A disabled scrolling
region does not scroll, and the gesture does not pass outward to one that would. Half-alive is not
a state anything should be in.

**`opacity(0.0)`** leaves the box stack, closing the footgun the previous engine documented at
length: a zero-opacity element that still took clicks, worked around with `disable()` plus a timer
to re-enable when a fade landed. Fully transparent is not there. Any opacity above zero is.

## Inheritance is computed, not commanded

Disabled, visible and opacity are **inherited products**, recomputed every frame in Rowan's
`inherit` pass (R7) — not cascades pushed down at the moment of the call.

`disable(leaf)` writes one flag on one element. The pass derives the product for the subtree. Three
consequences worth having:

- **Nothing has to remember the children.** There is no cascade to write and none to get wrong.
- **A child grown under a disabled parent is disabled on its first frame**, because the pass does
  not care when it arrived.
- **Re-enabling is symmetric.** `enable(leaf)` clears the flag; the subtree recomputes. Elements
  disabled in their own right stay disabled, because the product is over the whole ancestry, not a
  single inherited bit that was overwritten on the way down.

This is the recompute-totally thesis (`rowan.md`) paying for itself: the hardest part of cascading
state is keeping it correct as the tree changes, and a pass that recomputes from scratch has
nothing to keep.

## Focus follows

Disabling or hiding the subtree that holds focus **moves focus out of it**, and the change is
reported through `Pollen` like any other.

Keyboard input arriving at an element that cannot act on it is a dead app, and it is invisible in
testing that uses a pointer. Disabled and hidden elements are skipped by focus order for the same
reason.

## Elevation tie-break

Two siblings at equal elevation covering the same pixels rank by **allocation order** — the order
`plant`/`branch` were called.

This closes `rowan.md`'s open R6 question. Allocation order is total, monotonic, stable across
frames, and unaffected by prunes, since names are never reused. It also matches what a reader
expects: what you grew later sits on top.

It remains better to separate the elevations deliberately. A defined tie-break means the result is
not arbitrary; it does not mean the result is what you meant.

## Elevation is relative, and only relative

`up(n)` and `down(n)`, accumulated down the tree. There is no form that states a layer outright.

An absolute layer has to be chosen against every other one in the program, so two composites that
reach for the same round number collide — and what settles the collision is a tie-break neither of
them can see. That is the failure this document's tie-break makes *defined* rather than *correct*,
and a global number namespace is the one way to hit it constantly. It is CSS's `z-index: 9999`, and
the reason that number keeps growing.

Relative elevation accumulates through the tree, so two elements can only tie if they are
structurally related — which is where the tie-break means something ("what you grew later sits on
top") and where the code that grew both can separate them.

**An element that has to clear the stack it was grown in is grown somewhere else.** The trunk
decides what takes an element down and what it stacks among; `anchored` decides where it sits. A
dropdown planted at top level still tracks the control that opened it, and still addresses that
control's grid (`placement.md`), so leaving costs it nothing. It is also the right answer for
clipping, since a dropdown inside a scrolling view should not be clipped by it (`views.md`).

This is why no stacking tier or reset layer is needed: that machinery exists to arbitrate between
absolute values, and there are none.

## Proof obligations

Headless:

- every op against a withered `Leaf` is a no-op, one per verb
- `prune` emits `Withered` for the whole subtree, not just the named element
- a `Planted` leaf samples absent, then resolves on the frame its grow applies
- a disabled subtree receives no taps, no drags, and does not scroll
- a press on a disabled element does not reach what is behind it
- a child grown under a disabled parent is disabled on its first frame
- `enable` on an ancestor does not enable a descendant disabled in its own right
- a zero-opacity element receives nothing; a `0.01`-opacity element receives normally
- disabling the subtree holding focus moves focus out and reports it
- equal-elevation siblings rank by allocation order, and that survives a prune of an earlier
  sibling
- the tie-break is the order names were allocated, not the order the drain grew them
- elevation accumulates down the tree, so elevating a trunk moves its whole subtree and nothing
  inside it is rewritten
- an element that draws nothing still elevates what is grown under it
