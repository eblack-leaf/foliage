# Interaction

Hit-testing, gestures, and focus.

The previous system was, in its author's words, overdone and underwhelming — five flags
(`interactive`, `pass_through`, `holds_drag`, `overscroll`, `directional_lock`) that between them
tried to predict, at spawn, what a gesture would turn out to mean. Most of this document is
deletion, but not as much as a first pass suggested: one of those flags was carrying real
information and survives in a smaller form.

## 1. Two questions, not one

Everything that went wrong here comes from conflating them:

1. **What is at this point?** — geometry. Answered for every element, always.
2. **Who receives this gesture?** — intent. Answered only for elements that asked.

The previous engine answered both with one mechanism, so making decoration stop stealing taps also
meant taking it out of the geometry — which would break scrolling. Separating them lets each have
the right default.

## 2. The box stack

At a pointer position, the **box stack** is every element whose resolved box contains that point
and which is not clipped away, hidden, or fully transparent — ordered top-first by resolved
elevation.

**Membership is universal.** Nothing opts in. This is geometry, and it is what makes two other
things work:

- **Occlusion is per-point and automatic.** There is no such thing as an obscured *element*, only
  an element that is below another *at this point*. A panel covering half a button is above it for
  those pixels and absent for the rest, with no special handling.
- **A drag knows where it started**, in tree terms, whoever is or is not a target there.

## 3. Targets

> **Only an element that called `interactive()` receives gestures.**

Targeting walks the box stack from the top and stops at the first target. Non-targets are passed
over: they are present for geometry, not for receiving.

**A target blocks targets beneath it.** That is how a backdrop works — a full-screen panel that
should stop presses reaching the page is a target, so it blocks, and it also receives the press,
which is usually what a dismiss-on-outside-click wanted anyway. Decoration is not a target, so a
badge drawn over a button's corner does not break the button.

This is exactly what `pass_through` was arranging by hand, so it becomes the default and the flag
goes. The site marks 20+ elements `pass_through` against 14 `interactive()` calls — more code
spent saying "not me" than "me" — and its own comment records the discovery:

> *the card itself is the target — its contents draw above it, so without this each one wins the
> hit test on the pixels it covers and only the bare backing is clickable*

## 4. Scroll ownership is structural

A drag anywhere inside a scrolling region must scroll it. On touch it is the only way to scroll at
all, and it must work on plain decoration, which asked for nothing.

This never needed decoration to be a target. Given the point, the box stack names what is there;
the drag then walks **up the tree** to the nearest scrollable ancestor, and that region owns it.
Targethood is not consulted.

So scrolling works everywhere by construction, and it is not the reason anything opts in.

## 5. Gestures are claimed, not declared

A drag beginning on a slider inside a scrolling column belongs to the slider if it moves along the
slider and to the column if it moves down. Nothing at spawn time knows which it will be.

> A gesture opens **unclaimed**. The target it landed on holds it while it resolves. If it resolves
> to a drag the target does not take, the target yields and it passes to the nearest scrollable
> ancestor.

**Contention runs up the tree, never sideways.** A target contends only with its own ancestors —
the slider with the column that contains it. Two unrelated elements never contend, because
targeting already picked one of them. Claiming does not widen who is eligible; it decides, among an
element and the things containing it, which one the gesture meant.

This is also what makes a button inside a scrolling list behave on touch: press it and the button
holds the gesture; drag and the button yields, so the list scrolls; release without moving and the
button gets a tap.

### What a target has to declare

The engine can derive most of this, but not which drags an element wants. A slider taking
along-axis drags is information only the app has.

```
.takes_drag(Axis::Horizontal)   // or Vertical, or Both
```

Default: **takes no drags.** So a target holds a gesture only until it becomes a drag, then yields
— which is the mobile-correct default, because it means drags fall through to scrolling unless
something deliberately wants them.

This one declaration replaces `holds_drag`, `directional_lock`, and the drag half of `scroll_axes`.
It is not cruft: it is the minimum the engine cannot work out for itself.

## 6. A tap is an outcome, not a cancelled click

The drag threshold felt bolted on because it was: a click was issued and then retracted once the
pointer had moved far enough.

One gesture, one lifecycle:

```
opened ──▶ resolving ──▶ claimed ──▶ ended
```

A **tap** is what a gesture that ended without ever resolving to a drag emits. Nothing is
cancelled, because nothing was issued early. The threshold is not a retraction rule; it is the
point at which the kind becomes known.

## 7. Overscroll is a handoff

`overscroll(bool)` predicted at spawn whether unconsumed scroll should escape. Whether a region
*can* consume is a runtime fact — it depends on where it currently sits in its own extent.

> A claimant that can no longer consume **yields**, and the claim passes to the next scrollable
> ancestor.

A region scrolled to its end hands the gesture outward mid-drag, which is what the flag was
approximating. One rule, nothing to configure, nothing to set wrong.

A region that holds a gesture on an axis it does not scroll reports it rather than moving —
replacing `ScrollRefused` as a special channel with a consequence of the same rule.

## 8. Focus

Focus is a first-class surface, not a byproduct of clicking.

**`focus(leaf)` is an ordinary verb.** Today the only way to move focus is `click_on`, because
`TextInput`'s caret responds to the click path rather than to focus. That is an implementation
limitation dictating the public API — the inversion this rewrite exists to undo. The caret responds
to focus; focus becomes a verb.

**Focus order is reading order** — top to bottom, left to right — within the current scope, derived
rather than declared, and overridable where a layout's meaning differs from its geometry.

**Focus scopes trap.** A drawer or dialog declares itself a scope and focus cycles within it until
it closes. Without this, keyboard navigation inside an overlay walks off into the page behind it,
which is why most apps are mouse-only in practice.

**Visible focus is the app's job.** foliage reports focus through `Pollen` and draws nothing. A
focused element may be a `Stem` with no visible part at all, so there is no mark the engine could
draw that would be right.

## What is gone

`pass_through`, `holds_drag`, `directional_lock`, `overscroll`, `InteractionPropagation`,
`ScrollRefused` as a distinct channel, and the drag-cancels-click retraction.

What remains: `interactive()`, `takes_drag(..)`, `round_hit_area()`, `disable()`.

## Proof obligations

Headless, with synthetic pointer input:

- a decorative child of an interactive parent never wins a tap, with nothing marked
- a target partially covered by another target is pressable on its uncovered pixels and blocked on
  the covered ones
- a target partially covered by *decoration* is pressable everywhere
- a full-screen target blocks everything beneath it
- a fully transparent element is not in the stack
- a drag starting on plain decoration inside a region scrolls that region
- a drag starting on a *button* inside a region scrolls the region, and the button gets no tap
- a press and release on that button with no movement gets a tap
- a drag along a `takes_drag(Horizontal)` slider moves the slider, not the column
- a drag down the same slider moves the column, not the slider
- a region at its extent hands a continuing drag outward mid-gesture
- nested regions hand off outward in order
- `focus(leaf)` on a text input places a usable caret with no click involved
- tab order follows reading order and honours an override
- focus inside a scope cycles within it and does not escape

## 9. The claim threshold

**Per axis, and tunable.**

The two axes compete at different scales. On touch, vertical scrolling is the dominant gesture and
wants an eager claim; a horizontal claim contends with it and wants a larger threshold, or every
attempt to scroll steals into a carousel. One number is too eager for one and too reluctant for the
other.

It is a **global tuning value, not a per-element flag** — this is input feel, and input feel that
varies element to element is what makes an app feel unpredictable. It joins the existing startup
tunables (`Foliage::tune`) alongside scroll momentum and key bindings, and callers wanting the two
axes the same simply set them the same.

## What handing off looks like

When a gesture ends with speed, interaction supplies the **release velocity** and stops there.
Everything after — decay, clamping, whether hitting an end chains or absorbs — is `views.md`'s.

`disable()` stays, cascades to the subtree, and **swallows** rather than passing input through —
see `lifecycle.md`, which owns the three ways an element can be off and how they differ.
