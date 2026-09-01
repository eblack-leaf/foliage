# Aspen — animation

Timing, easing, and how an animated value is applied. Written early, ahead of its slice, because
one requirement it makes of `Rowan` shapes that system's core.

## The problem it has to solve

Animating a `Location` is the hard case, and the previous engine never solved it properly.

A `Location` is not a value. It is a *function* from context to a box — and the context is
breakpoint, parent geometry, anchor targets, and measured text. All four can change while a tween
is running:

- the window resizes, or crosses a breakpoint
- the element it anchors to moves
- the text it sizes itself from is rewritten
- its parent's grid resolves differently

So "animate to this `Location`" cannot be answered once at tween start. Whatever the target
resolves to at that instant may not be where the target *is* by the time the tween lands.

### Why the old approach could not work

`impl Animate for Location` interpolated one channel from `1.0` to `0.0` into an
`animation_percent` field **on the `Location` itself**, blended against a cached `Resolution`
component. The animation's state lived inside the animated value.

That is an artifact of the era when every value had to interpolate itself, because app code held
components directly. It produced the documented footgun: a plain write mid-tween replaced the
`Location` but left the percent and the cached resolution behind, so the element resolved against a
stale difference and jumped.

## The model

> **Resolution is a pure function. A tween resolves both of its endpoints every frame, in the
> current context, and interpolates between the results.**

```
section(t) = lerp( resolve(from, ctx_now), resolve(to, ctx_now), ease(t) )
```

Both endpoints re-resolve each frame. Nothing is cached, so nothing can go stale.

| Mid-tween event | What happens |
|---|---|
| Window resizes | Both endpoints re-resolve at the new size; motion stays correct; lands exactly on target |
| Breakpoint crosses | Both endpoints re-resolve at the new breakpoint |
| Anchor target moves | `to` re-resolves against its new geometry; the element follows |
| Text is rewritten | A `text_content()` endpoint re-measures |
| Nothing changes | Identical to caching, at the cost of arithmetic over a handful of values |

The cost is resolving twice instead of once, for animating elements only. Resolution is arithmetic
over a few numbers; this is not a real cost.

## What it requires of Rowan

**Resolution must be a pure function of `(Location, context)`, callable more than once per entity
per frame** — not a stateful pass that mutates its way to an answer.

This is the constraint to hold on to when `rowan.md` is written. The previous design could not
resolve an entity twice because resolution *was* the mutation; this one must separate deciding from
writing.

Dependency order is unchanged: an entity resolves after whatever it anchors to. Both of its
endpoints are resolved at its own position in that order, against the same already-settled
ancestors, so the two answers are consistent with each other.

An element anchored to an animating element sees that element's **interpolated** box, which is
correct — it should follow the motion, not the destination.

## Where the target lives

Starting a `Location` tween writes the target to the element **immediately**, and the tween carries
the *old* `Location` as `from`:

```
animate(leaf, Motion::Location(to), timing)
  → drain:  from = element's current Location
            element's Location = to          ← the declared value is already the target
            start tween { from, timing }
  → resolve: lerp(resolve(from), resolve(Location), eased(t))
  → t = 1:   tween ends, resolution is just the Location. No discontinuity, nothing to clean up.
```

Two properties fall out for free:

- **The tween ending changes nothing.** The blend at `t = 1` already equals the plain resolution, so
  there is no settle-to-final step that could land a pixel off.
- **Cancellation is trivial.** A direct write (F8) replaces the `Location` and drops the tween; the
  element resolves at the written target. No stale state exists to reconcile.

### Retargeting mid-tween

A second `animate` on a property already tweening replaces the tween. Its `from` is a **snapshot of
the current blended box**, not the old `from` — otherwise the element jumps back to where the old
motion started.

The snapshot does not re-resolve, and that is correct rather than a compromise. A mid-tween blend
is a synthesized intermediate that never corresponded to any declared layout, so there is nothing
for it to be stale *relative to* — it is a starting pixel, and a starting pixel is all it needs to
be. The `to` side stays fully responsive, so the landing is exact regardless of what changes on the
way.

## Progress and application are separate

Aspen computes **progress**; it does not apply values.

Once per frame, at the `animate` phase, every live tween advances against the one clock (F6) and
computes its eased `t`. Applying that `t` belongs to whichever phase owns the property:

| Property | Applied in | Why |
|---|---|---|
| Location | `resolve` | Needs the resolution context, which does not exist earlier |
| Color, opacity, outline, polygon shape | `animate` | Plain values, no context needed |
| Scalar channels (`tween`) | `animate` | Reported outward as `Pollen`; foliage writes nothing |

This split is why F8 holds structurally. The drain runs before any of it and cancels tweens on
written properties, so no property ever reaches its applying phase with both a pending write and a
running tween.

## Non-geometric animation

Everything that is a plain value — color, opacity, outline, polygon shape — interpolates channel-wise
and needs none of the above. It is listed here only to make clear that the machinery above exists
for one reason, and does not tax the simple cases.

## Proof obligations

Headless, deterministic clock:

- resize mid-tween → lands exactly on target
- breakpoint crossed mid-tween → lands exactly on target
- anchor target moved mid-tween → follows, lands correct
- `text_content()` endpoint rewritten mid-tween → re-measures, lands correct
- retarget mid-tween → no jump at the retarget frame
- direct write mid-tween → tween drops, element at written value, no jump
- element anchored to an animating element → tracks the interpolated box each frame
- tween end → the frame at `t = 1` and the frame after are pixel-identical

## What `Motion` covers

A property belongs in `Motion` if **animating it is a normal thing to want**, or if it **cannot be
animated from outside** because it needs context only the engine has.

```
Motion::Opacity(f32)
Motion::Color(Color)
Motion::Location(Location)     ← the second kind: nothing outside can resolve it
Motion::Polygon(Polygon)       ← sides, corner rounding, rotation: all numbers
Motion::Outline(Outline)
Motion::DrawProgress(f32)      ← polyline draw-in
Motion::Scroll(ScrollTo)       ← smooth scroll-to
```

`DrawProgress` and `Scroll` close gaps the previous engine documented and apologised for. Drawing a
polyline in *is* what `draw_progress` exists for — animating it was never optional — and a nav that
scrolls to a section is what the reference site itself had to hand-roll.

`Scroll` interacts with input exactly as it should: a drag is a write, so under F8 the reader
taking hold of a scrolling region cancels an animation moving it. The person wins.

### Excluded, and why

| Property | Reason |
|---|---|
| `Elevation` | Invisible — decides draw order and hit priority, never appearance. Its correctness is what stops geometry tearing at odd z levels. Ranking stays a pass over stable integers, which is the whole reason it can be trusted |
| `FontSize` | Responsive per breakpoint like `Location`, so it would need `Location`'s whole double-resolve machinery, for a far rarer want |
| `Points`, `GlyphColors` | Lists whose interpolation is only defined when the counts match. Not "expensive" — *undefined*, which is a harder line and the right one |
| `Rounding`, `Icon`, `Visible`, `Text` | Discrete. There is nothing between two of them |
| `ImageView` | Well-defined, and a candidate later. Not a normal enough want for 1.0 |

### Everything else is `tween`

`tween` takes start/end pairs and reports each frame's values as `Pollen`, writing nothing itself —
the engine's clock and easing made available to values it has no concept of. A single `0.0..1.0`
channel is plain eased progress, if that is all you want.

This is the answer for every excluded property above, and for anything an app invents. The list is
closed because the engine's obligations should be, not because an app's are.
