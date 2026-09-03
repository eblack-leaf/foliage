# Views and scrolling

## 1. A grid is not a view

Today `Grid` carries `#[require(View)]`, so laying children out on a grid makes an element
scrollable whether or not anything wanted that.

The reason is plumbing, and its own comment says so: children resolve against their parent's scroll
offset, so every grid parent needed a `View` to carry one — *"most carry an offset of zero forever,
and that is fine."*

That need is already met elsewhere. Rowan's R4 accumulates offsets down the tree for every element,
scrolling or not, so nothing has to hold a zero to be a well-behaved parent.

> **`Grid` decides how children are laid out. Scrolling is declared separately.**

An element scrolls because it said so, and for no other reason.

## 2. Scrolling is declared per axis

Following the `overflow-x` / `overflow-y` split, which is right for the same reason here: the two
axes almost never want the same answer, and a single flag forces one.

```
.scrolls(Axes::Vertical)     // down only
.scrolls(Axes::Horizontal)
.scrolls(Axes::Both)
```

An axis that was not declared does not scroll, has no extent computed, and cannot be moved by drag,
wheel, or coast. It is not "scrollable with a range of zero" — it is not a scrolling axis, which is
a different and simpler thing to reason about.

`ScrollAxes` and the axis half of the old flag set collapse into this.

## 3. Boundary policy, per axis

What happens when a region reaches its end is a real choice, and `interaction.md` was too quick to
delete it. Two behaviours, per axis:

| | Means |
|---|---|
| **chain** (default) | Reaching the end hands the gesture outward to the next scrolling ancestor |
| **contain** | The region absorbs it. Nothing outside moves |

Chaining is what lets a drag inside a list keep moving the page once the list is done, which is
what a reader expects of ordinary content. `contain` is for a region that owns its gesture
outright — a map, an editor, a scrolling pane inside a fixed shell — where reaching the bottom and
having the whole page lurch is a bug every time.

The difference from the old `overscroll(bool)` is that chaining is now a **runtime handoff**
(`interaction.md` §7) rather than a prediction: the region yields when it actually cannot consume,
not because it was told at spawn that it might not be able to. `contain` is the declared exception
to that, and it is the only knob.

## 4. One unit: logical pixels

Today you **write** a fraction (`ScrollTo::y(percent)`, clamped 0..1) and **read** pixels
(`Sap::ScrollOffset`, `Sap::ScrollExtent`). Everything else in the system — extent, a drag delta,
a nudge of 50 — is pixels, so the one unit you cannot write in is the one everything is in.
"Scroll down 50 more" becomes: read pixels, read extent, divide, write a fraction.

> **Offset is logical pixels, read and written.**

`ScrollTo` still accepts other framings, but each one **names its unit at the call site**, which is
what was actually missing:

```
ScrollTo::px(240.0)        ScrollTo::start()
ScrollTo::fraction(0.5)    ScrollTo::end()
ScrollTo::show(leaf)       // scroll the minimum distance to bring a descendant into view
```

`ScrollTo::show` is here because jumping to a section is common enough that every app otherwise
computes it by hand, and it composes with `Motion::Scroll` (`aspen.md`) to animate.

`ScrollProgress` stays, read-only, as the derived 0..1 a scrollbar needs. Deriving it is fine;
being the only way to write was not.

## 5. Extent

Extent is the back-edge in Rowan's dependency graph (R3, bottom-up): children resolve against the
parent's box, and the parent's scrollable extent comes from where its children landed. That is why
it is the part most able to surprise — a child positioned outside its parent's box grows what the
parent can scroll to, producing *a scrollbar you did not order*.

### Hidden is not clipped

The old behaviour has a cause worth naming, because it was not carelessness: `Visibility` was doing
two unrelated jobs.

- **app intent** — "hide this"
- **engine culling** — "this scrolled out of the clip rect, do not send it to Ash"

Culling wrote the same flag. So excluding invisible children from extent would have excluded
*scrolled-out* children — the very content extent exists to describe, since a list you have scrolled
past must still be scrollable back to. Invisible children therefore had to count, and genuinely
hidden ones came with them. The parking footgun is a consequence of the overload, not a bug beside
it.

> **Culling is not a state. It is a decision Elm makes at extract time.**

Nothing on an element ever records "I am currently clipped away". Rowan produces a clip rect (R5);
`Elm` skips instances whose box does not meet it; hit-testing intersects box with clip directly.
There is no flag for extent to read, so the overload cannot recur.

There is no circularity: extent is R3 and reads `LayoutSection`, clip is R5. Extent is computed
before a clip rect exists and never consults one.

|  | Drawn | In the box stack | In extent |
|---|---|---|---|
| visible | yes | yes | **yes** |
| clipped out (derived) | no | no | **yes** |
| `visible(false)` (intent) | no | no | **no** |

### The rules

1. **Only along declared scroll axes.** A `Y` view computes no `X` extent, so a child extending
   sideways is simply out of frame. Most accidental extent came from the axis nobody was scrolling.
2. **Only children the app has not hidden contribute** — app intent, never culling. Scrolled-out
   content still counts, so scrolling back to it works; `visible(false)` genuinely removes a
   subtree, which makes it the whole answer for parking rather than half of one.
3. **Measured outward from the content origin, clamped at the near side.** A child at a negative
   offset does not create range to scroll *back* into. Content above the origin is a layout
   mistake, and the previous behaviour turned it into a feature.
4. **Never smaller than the view's own box.** A near-empty region has zero range, never negative.

What these deliberately do *not* do is exclude a visible child that is simply far away. If it is
visible and out there, it is content and it is reachable — that is coherent, where the old
behaviour was a surprise. The fix for parking is rule 2, not a guess about intent.

### Pinned children

Some children sit inside a view visually but should not travel with it — a header that stays at the
top while content slides under it, a button pinned to a corner.

These are not two properties. *Moving with the content* and *counting toward extent* are the same
question, so they get one answer:

```
.pinned()
```

A pinned element does not receive its nearest scrolling ancestor's offset in R4, and contributes
nothing to extent in R3. It cannot drift out of agreement with itself, which two separate flags
could.

This replaces the current workaround — parent it outside the view and anchor back in — which cost
the element its correct place in the tree, and with it clipping, opacity inheritance and the
disable cascade.

### Floating children

The other half of the same question, and the one `lifecycle.md` defers here: a child grown inside a
region but *positioned outside it* — a menu opening below the row that owns it, a tooltip beside a
list item.

The trunk decides what takes an element down, what it stacks among, and what clips it; `anchored`
decides where it sits. That split is what lets an element be placed against one element while living
under another. What it leaves is an element positioned outside its trunk and then **cut off at the
trunk's edge**, which is the whole point of having put it out there — and feeding the trunk's extent
a stretch of scroll range leading to content that is drawn over the region rather than in it.

Those are one question, so they get one answer:

```
.floats(Escape::Region)
```

A floating element is **not clipped** by the region it is grown in, and **contributes nothing** to
that region's extent. What it keeps is the region's offset: it travels with the content it is
anchored to, which is what holds a menu against the row that opened it. That is the whole difference
from `pinned`, which escapes the movement and keeps the clip.

> **`pinned` escapes the movement. `floats` escapes the clip. Both leave the extent, because
> neither is content.**

An element that says nothing is content: it is clipped by its region and it counts. That is the
right default and it is what an inline expander wants — options that push into the flow and are
scrolled down to.

#### How far out

Stated, not defaulted, because no one answer is right everywhere. A menu on a row of a table with
its own scrollbar, inside a settings panel with another, wants out of both. A tooltip on a file in a
list inside a sidebar wants out of the list and *not* out of the sidebar, or it paints across the
page beside it. Nothing distinguishes the two pictures to the engine.

| | Held by |
|---|---|
| `Escape::Region` | whatever holds the region it left |
| `Escape::Surface` | nothing; a clip is never wider than what is drawn on |
| `Escape::Within(leaf)` | the element named, which still clips it |

`Within` names an element rather than counting regions, because a count is a fact about the tree's
current shape and would quietly mean something else the moment a wrapper was added between the two.
Naming something that is not above it holds nothing, and falls back to `Escape::Region` — an element
escapes no further than it was told to, and a mistake is not permission to leave everything.

Escape depth is about **clipping alone**. Extent needs no such argument: a region contributes its own
box and never its content to whatever contains it, so once an overlay is out of the region it is
grown in there is no second region left to be excluded from.

### Explicit extent

```
.extent(area)
```

Overrides derivation entirely. This is for virtualised content, where realised children are a
window onto a much longer list and deriving extent from them is exactly wrong — the scrollbar would
describe the window rather than the list.

Deriving is the default because it is right whenever the children *are* the content. Declaring is
available because sometimes they are not.

## 6. What is gone

`ScrollAxes`, `OverscrollPropagation`, `DirectionalLock`, `holds_drag`, `ScrollRefused` as a
distinct channel, `#[require(View)]` on `Grid`, and fraction-only writes.

What remains: `.scrolls(Axis)`, per-axis `contain`, `.pinned()`, `.floats(Escape)`, `.extent(..)`,
and `ScrollTo` with its unit named.

`ClipToViewport` is gone with them, and `floats` is what replaced it. The old flag did three things
at once — escaped the clip cascade, reset the elevation prefix, and dropped the subtree from its
parent's extent. The third and the first are one statement about content and are kept together here;
the second is `lifecycle.md`'s, and is answered by growing an element that must clear a stack
somewhere else rather than by a flag.

## 7. Proof obligations

Headless:

- an element with a `Grid` and no scroll declaration does not scroll, by drag, wheel, or `scroll`
- a `Y` view ignores horizontal drag and computes no horizontal extent
- a `visible(false)` child contributes nothing to extent, and nor does its subtree
- an element positioned outside its region is cut off at the region's edge, and `floats` is what
  stops it — at each of the three depths, including the one neither of the other two reaches
- a floating child contributes nothing to extent, and still travels with the content
- a child scrolled fully out of view still contributes, and can be scrolled back to
- a child at a negative offset creates no backward range
- an empty view has zero range
- extent tracks a child that grows, including one that grew by wrapping (`placement.md`)
- `.extent(..)` overrides derivation and survives children changing
- a chaining region at its end hands a continuing drag outward mid-gesture
- a `contain` region at its end absorbs it, and nothing outside moves
- nested chaining regions hand off outward in order
- `ScrollTo::px` and `ScrollTo::fraction` land on the same place for equivalent inputs
- `ScrollTo::show` brings a descendant just into view and no further
- reading the offset back after writing it returns the same pixels
- a disabled region does not scroll and does not chain (`lifecycle.md`)

## 8. Momentum and coast

A drag released with speed keeps the region moving, decaying until it stops or reaches an end.

**Coast is a view behaviour; the gesture only supplies its initial velocity.** `interaction.md`
hands over the release velocity and is then done — everything after that is this document's:
the decay curve, the clamp against extent, and whether reaching an end while coasting chains
outward or absorbs, exactly as a drag would.

It is deliberately not an Aspen tween. There is no target and no duration — a coast is an
integration that runs until it settles, which is a different shape from an interpolation between
two known endpoints.

A coasting view is pending work under F9, so frames keep running until it settles and stop
afterwards.

## 9. Open

Nothing. The remaining decision that touches views is the placement grammar, which is
`placement.md`'s.
