# Interaction

Hit-testing, gestures, and focus.

The previous system carried five flags — `interactive`, `pass_through`, `holds_drag`, `overscroll`,
`directional_lock` — of which three tried to predict, at spawn, what a gesture would turn out to
mean. Those three go. `interactive` and `pass_through` state what cannot be derived and stay.

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

**Membership is universal.** Nothing opts in, and `pass_through` does not opt out — it changes what
stops a gesture, never what is in the stack. This is geometry, and it is what makes two other
things work:

- **Occlusion is per-point and automatic.** There is no such thing as an obscured *element*, only
  an element that is below another *at this point*. A panel covering half a button is above it for
  those pixels and absent for the rest, with no special handling.
- **A drag knows where it started**, in tree terms, whoever is or is not a target there.

## 3. Stopping and receiving

Two declarations. Neither is derived from the other, and neither is derived from geometry.

> **A gesture goes to the top of the box stack** — the one element nearest the viewer at that
> point, whatever it draws and whether or not it receives. `pass_through` takes an element out of
> the stack for this purpose, so the element beneath it is the top.
>
> **Only an element that called `interactive()` receives.**

An element that is at the top without receiving **eats** the gesture. A backdrop, a sheet backing
and a menu's padding are all this, and none of them declares anything to be it.

### The hit test does not search

Reading the top of the stack is the whole of it. The engine never continues downward looking for an
element willing to take the gesture, and it never passes over one because it is undeclared or draws
nothing. What is in the stack is the author's statement; which member of it wins is geometry.

A search would have to judge, at each element it passed, whether that element is *part of* what it
covers or a *layer over* it. At a point the two are one picture — a target with an undeclared drawn
element above it — and ancestry, paint, elevation and targethood each answer one of them correctly
and the other wrong. A searching hit test therefore attributes presses by inference, which is wrong
silently and at a distance from anything the author wrote.

It also decides more than a press. Where a gesture lands is where a drag that follows it looks for
its scrolling region (§4), so a press attributed to the wrong element scrolls the wrong region. An
inferred target is not one mistake but two.

So the distinction is stated where it is known. A composite marks its decoration `pass_through`,
and nothing else about targeting is declared at all.

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

**A claim is settled once.** The axis a gesture resolved on, and who took it, hold for the life of
that gesture — a drag that turns is still the drag it was claimed as. Re-deciding per move would
hand the gesture back and forth through the middle of a diagonal, which is worse than either answer,
and it would do it at the moment a hand is least steady. The one thing that moves afterwards is the
claim travelling *outward* when its holder can no longer consume (§7), which is one direction and
never back.

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
.drags(Axes::Horizontal)   // or Vertical, or Both
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

### Owed: a press that was held

The lifecycle above has one threshold, and it is a distance. There is no time in it, so a gesture
that sits still is indistinguishable from one that has not moved yet — `resolving` covers both.

That is a gap rather than a simplification, and touch is where it shows. Dragging to scroll and
dragging to select are the same motion, and nothing in a gesture separates them; both platforms
answer it the same way, with a plain drag scrolling and selection beginning from a press that was
**held**. Without the notion, an element that wants both has to take one and give up the other —
which is what a `TextInput` does today, and why a drag across one can never scroll it.

So `resolving` gains a second way out: held past a duration, reported as its own gesture fact.

```
opened ──▶ resolving ──▶ claimed ──▶ ended
               └──────▶ held ──▶ claimed ──▶ ended
```

One duration, tuned once beside `Claim` (§9), because it is input feel and input feel that varies
element to element is what makes an app feel unpredictable. It is general — context menus, reorder
handles and press-and-hold affordances all want it — so it does not arrive as anything one seed
asked for, and nothing about it mentions text.

Double-tap belongs to the same missing dimension and is not part of this: what it is *for* on touch
is selecting a word, and that waits on word boundaries rather than on gesture timing.

## 7. Overscroll is a handoff

`overscroll(bool)` predicted at spawn whether unconsumed scroll should escape. Whether a region
*can* consume is a runtime fact — it depends on where it currently sits in its own extent.

> A claimant that can no longer consume **yields**, and the claim passes to the next scrollable
> ancestor.

A region scrolled to its end hands the gesture outward mid-drag, which is what the flag was
approximating. One rule, nothing to configure, nothing to set wrong.

A region is never handed a gesture on an axis it does not scroll: the claim passes over it to the
next one out, and if nothing in the chain scrolls that axis the gesture is held by nothing and moves
nothing for the rest of its life. So `ScrollRefused` has no special channel to be — refusal is the
absence of a claimant, which is the same rule read from the other side.

## 8. Focus

Focus is a first-class surface, not a byproduct of clicking.

**`focus(leaf)` is an ordinary verb.** Today the only way to move focus is `click_on`, because
`TextInput`'s caret responds to the click path rather than to focus. That is an implementation
limitation dictating the public API — the inversion this rewrite exists to undo. The caret responds
to focus; focus becomes a verb.

**A gesture moves focus nowhere on its own.** The element a person pressed and the element they want
to type into are different questions, and only the app knows when they coincide. Nothing is inferred
from a gesture here for the same reason nothing is inferred in §3. It also deletes the judgement
about which press counts as pressing *away* from a field: an app that wants a popover to close when
something else is pressed asks about the element it put behind its content, which `pollen.md`
already requires of anything asking "what was pressed".

**Focus goes to what was tapped, and no declaration decides it.** `interactive()` is already the
statement that an element takes input, and focus already rests only on what said that — so the
target of a tap is by definition somewhere focus can be. A second flag would have been the same
question asked twice, with the two free to disagree.

A tap that reached nothing which receives takes focus away. That is not a rule about dismissing: it
is this rule, reading that what was tapped cannot hold focus, so nothing does.

The verb keeps the last word. A tap settles focus at dispatch, which is the frame *before* an app is
handed the `clicked` it produced — so an app writing `focus` elsewhere is simply the later write.

**A gesture that became a drag was never a statement about what it began on.** It earns no tap, and
so it moves no caret and takes no focus: the same rule, applied to all three. That is why a caret
lands on the tap rather than on the press — a drag out of a field to scroll the page behind it
leaves the field exactly as it found it.

**What can hold focus is what declared `interactive()`.** The set that asked to receive input is the
set a keyboard should be able to reach, so there is no second declaration to keep in step with the
first, and no way for the two to disagree.

**Focus order is reading order** — top to bottom, left to right — within the current scope, derived
rather than declared, and overridable where a layout's meaning differs from its geometry.

**Focus scopes trap.** A drawer or dialog declares itself a scope and focus cycles within it until
it closes. Without this, keyboard navigation inside an overlay walks off into the page behind it,
which is why most apps are mouse-only in practice.

**Visible focus is the app's job.** foliage reports focus through `Pollen` and draws nothing. A
focused element may be a `Stem` with no visible part at all, so there is no mark the engine could
draw that would be right.

## What is gone

`holds_drag`, `directional_lock`, `overscroll`, `InteractionPropagation`, `ScrollRefused` as a
distinct channel, and the drag-cancels-click retraction.

What remains: `interactive()`, `pass_through()`, `drags(..)`, `round_hit_area()`, `disable()`, and
focus's own two, `focus_scope()` and `focus_order(..)`.

## Proof obligations

Headless, with synthetic pointer input:

- a decorative child marked `pass_through` never wins a tap, and wins it when unmarked
- an undeclared element at the top of the stack eats the gesture, and the target beneath it
  receives nothing — the hit test does not continue past it
- a target partially covered by another element is pressable on its uncovered pixels and blocked on
  the covered ones
- a target covered by a `pass_through` element is pressable through it
- a full-screen target blocks everything beneath it
- a fully transparent element is not in the stack
- a drag starting on plain decoration inside a region scrolls that region
- a drag starting on a *button* inside a region scrolls the region, and the button gets no tap
- a press and release on that button with no movement gets a tap
- a press held past the duration is reported as held, and a drag out of it is claimed by whoever took the hold
- a press held past the duration and released without moving is not also a tap
- a tap focuses what it landed on, and reports it in the frame the tap was in
- a tap that reached nothing which receives takes focus away
- a drag moves focus nowhere, and leaves the element it began on untouched
- an app writing `focus` from `clicked` beats the tap
- `focus(leaf)` reaches an element shown in the same frame it was written
- a drag along a `drags(Horizontal)` slider moves the slider, not the column
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

A gesture that has travelled far enough on both is a drag **down**: down is the gesture a hand makes
without meaning anything by it, and across is the one it means.

It is a **global tuning value, not a per-element flag** — this is input feel, and input feel that
varies element to element is what makes an app feel unpredictable. It joins the existing startup
tunables (`Foliage::tune`) alongside scroll momentum and key bindings, and callers wanting the two
axes the same simply set them the same.

## 10. Keys

A keystroke is input, and it arrives where a press arrives: at intake, in order, and is dispatched
against what the last frame settled. It is the one thing in the engine whose **order is part of what
it means** — `frame.md` says so of emissions, and this is what that clause was reserving.

**A key goes to whatever it is about, and nothing searches for that.** `Tab` is focus's own and is
answered wherever focus is, including nowhere, so a keyboard reaches a page that has never been
pressed. Everything else goes to the element holding focus, and to nothing at all if that element
has no use for it. There is no bubbling and no chain: the same law §3 states about a point, stated
about a key.

**What a key produced is the platform's answer, not the engine's.** A layout, a dead key and a
composed sequence are all resolved before a keystroke reaches intake, so a key that produced text is
taken as the text it produced. A key that produced none is one of the few that mean something
instead. Nothing here maps a scancode, and nothing here holds a binding table.

**What is held arrives as its own event, in the same stream.** A modifier is only ever a statement
about the keys pressed after it, so what a key was held with is read from the order the two arrived
in rather than from a flag kept beside the queue. It is engine state, which is what lets the
headless suite hold a modifier by writing the event a window writes — the alternative is a second
path only the platform can reach, and a whole modifier the suite cannot test.

`shift` extends a selection where a bare arrow moves a caret, and steps `Tab` backwards. `control`
makes a key a command rather than a character: `Ctrl+A` selects a field's whole value, and a chord
nothing answers is nothing rather than a character inserted. Word-wise motion and the clipboard are
the rest of that story and are not here — the first is a decision about word boundaries and the
second needs a platform clipboard, and neither is a modifier problem.

**A keystroke is queued, not applied.** Dispatch decides which element a key is about; the drain
decides what it does. That keeps one queue and one drain (F1), and it is what puts what a key
changed on F7's ordinary footing rather than on a second, faster path.

### Proof obligations

- several keystrokes arriving in one frame are applied in the order they arrived
- `Tab` steps focus with no field focused, and with one
- `Tab` is never typed into the field holding focus
- `Escape` takes focus away, and is never typed into the field it was pressed in
- a keystroke with nothing focused reaches nothing, and one with a non-field focused reaches nothing
- a modifier held across a keystroke reaches it, and is released with it
- `Ctrl+A` selects a field's whole value and inserts nothing

## What handing off looks like

When a gesture ends with speed, interaction supplies the **release velocity** — the mean over a
trailing window of the last hundred milliseconds, because one frame is too noisy a sample to measure
a fling from — and stops there. Everything after — decay, clamping, whether hitting an end chains or
absorbs — is `views.md`'s.

A press that lands on a region still coasting **stops it and is spent on stopping it**. The gesture
stays open and keeps its chain, so a drag out of the catch scrolls as any other does, but the
element under the hand is told nothing: it does not engage, and the release is not a tap. Reaching
for a list that is still going is not a choice about whatever was under the hand when it halted.

`disable()` stays, cascades to the subtree, and **swallows** rather than passing input through —
see `lifecycle.md`, which owns the three ways an element can be off and how they differ.
