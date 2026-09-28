# Interaction

Step 2 of the frame turns what the platform said (a press here, a move there, a key) into what it
meant: a tap on this element, a drag of that one, a scroll of the list behind them. It lives in
[`interaction/`](https://github.com/eblack-leaf/foliage/tree/main/foliage/src/interaction): the box
stack in `stack.rs`, gestures in `mod.rs`, focus in `focus.rs`.

## Two questions, not one

1. **What is at this point?** Geometry, answered for every element, always.
2. **Who receives this gesture?** Intent, answered only by elements that asked, with one bit:
   `interactive`.

Engines that answer both with one mechanism get a familiar bug: a label drawn over a button steals
the button's taps, and the only way to stop it is to take the label out of the geometry, which is the
same geometry a drag needs to find the list it should scroll. Keeping the questions apart is what
lets a label be `intangible` (not something a press lands on) while still being inside the list a
drag scrolls.

## The box stack

R8 builds the box stack at the end of every frame that changed it: every element that is present
(visible, and not fully transparent) and not clipped away entirely, front-most first.

```rust,ignore
// interaction/stack.rs
pub(crate) struct Region {
    pub(crate) leaf: Leaf,
    pub(crate) section: Section,   // where it was drawn
    pub(crate) clip: Section,      // what a scrolling ancestor left of it
    pub(crate) shape: Shape,       // its box, or the ellipse inside it
    pub(crate) tangible: bool,     // false for `intangible`
    pub(crate) receives: bool,     // `interactive`
    pub(crate) disabled: bool,     // in its own right or by an ancestor
}
```

**Membership is universal.** Nothing opts in: an element is in the stack because it is there. That
makes occlusion per point and automatic. There is no such thing as an obscured element, only an
element that is below another *at this point*, so a panel covering half a button is above it for
those pixels and absent for the rest, with no special handling.

The whole hit test is one read:

```rust,ignore
// interaction/stack.rs
pub(crate) fn top(&self, point: Position) -> Option<Region> {
    self.regions
        .iter()
        .find(|region| region.tangible && region.holds(point))
        .copied()
}
```

A gesture goes to the **top** of the stack: the front-most tangible element at the point, whatever it
draws and whether or not it receives. `intangible` is what a gesture passes through, so the element
beneath becomes the top. An element at the top that does not receive **eats** the gesture, which is
what a backdrop, a sheet's backing and a menu's padding are, without any of them declaring anything.

The read never searches downward for an element willing to take the gesture. A search would have to
judge, at each element it passed, whether that element is *part of* what it covers or a *layer over*
it, and at a single point those look identical, so it would be answering by inference. Inference is
wrong silently, and here it would be wrong twice, because where a press lands is also where a drag
that follows looks for its scrolling region.

The stack is read at dispatch, which is step 2 of the *next* frame, before the drain. So a press is
tested against the picture the person was looking at when they pressed ([F5](frame.md#f5-hit-testing-runs-against-what-was-drawn)),
even if this frame's ops are about to move everything.

## A gesture's life

Nothing at spawn time knows what a gesture will turn out to be. A press on a slider inside a
scrolling column belongs to the slider if it moves along the slider and to the column if it moves
down. So a gesture opens **unclaimed**, and is claimed when it becomes known what it is:

```text
opened ──▶ resolving ──▶ claimed ──▶ ended
               └──────▶ held ──▶ claimed ──▶ ended
```

**The press** reads the stack once. If the top receives, it becomes the gesture's **target**, and is
reported `engaged` (the hook for a pressed look). If the top is disabled, the press is swallowed.
Either way the gesture records its **chain**: every scrolling ancestor of where it landed, innermost
first, leaving out any that is disabled. The chain is fixed at the press, because where a gesture
landed is where it looks for a region, whatever it passes over afterwards.

If any region in the chain is still coasting from an earlier fling, the press **catches** it: the
coast stops where the hand met it, and the press is spent on the catch, so stopping a moving list is
not also a tap on whatever happened to be under the finger.

**Moves** accumulate travel from the press point. While the gesture is resolving, nothing is
reported, and when the travel crosses a threshold the gesture is claimed:

```rust,ignore
// interaction/mod.rs
fn claimed(&self, travel: Position) -> Option<Axis> {
    let (across, down) = (travel.x.abs(), travel.y.abs());
    if down >= self.vertical && down >= across {
        return Some(Axis::Vertical);
    }
    if across >= self.horizontal && across > down {
        return Some(Axis::Horizontal);
    }
    None
}
```

The two axes have different thresholds (16 logical pixels across, 8 down, by default, tunable once
for the whole app with `Foliage::tune`), because they compete at different scales. On touch,
scrolling down is the dominant gesture and wants an eager claim; a claim across contends with it and
wants a larger one, or every attempt to scroll steals into a carousel. A tie goes down, because down
is the gesture a hand makes without meaning anything by it.

**The claim** is settled once. If the target declared `drags` on that axis, it takes the drag and is
told `drag_started`, and every move after is reported as `dragged`. Otherwise the target **yields**:
it is told `disengaged` and hears nothing more, and the drag passes to the innermost region in the
chain that scrolls on that axis. That is what makes a button inside a scrolling list behave: press it
and it holds, drag and it lets go, so the list scrolls and the button gets no tap.

Movement made while the gesture was still resolving is not paid out afterwards. The move that
crosses the threshold is applied, and the ones before it are not, so nothing that claims a drag
starts with a jump of everything it took to decide.

**A region holding a drag** consumes as much of each move as it can, and at its end hands the claim
*outward* to the next region in the chain that scrolls on the axis, unless it declared `contain` on
that axis, which is where the walk stops. The claim never travels back inward, or a drag would hand
itself between regions every time it reversed. Moving a region is a write to its offset, made right
here at dispatch, and like any write it cancels a scroll motion on that region and ends any coast.

**The release** is the last move, then the end. A gesture that ended while still resolving is a
**tap**: the target is told `clicked`, at the point the press landed, and focus moves to it. A tap
that landed on nothing that receives takes focus away. Nothing was reported early, so nothing is
retracted: the threshold is not a rule about taking a click back, it is the point at which the
kind of gesture becomes known.

If a region was holding the drag, the release hands it a velocity, and the region may coast. The
velocity is the mean over the last 100 milliseconds of the gesture, not over its last frame: a hand
slows as it lifts, a mouse reports nothing in the frame the button came up, and a flick can put all
of its movement into the frame before. Measured over one frame, all three read as a hand that
stopped. A hand that genuinely rested before lifting still reads as stopped, because its resting
frames are in the window. [Scrolling](views.md) takes it from there.

## A press that was held

A distance cannot tell a gesture sitting still from one that has not moved yet, so resolving has a
second way out: a press held for `Hold::after` (500 milliseconds by default) without becoming a drag
is **held**. It is reported once, to the target, as `held`, at the point it landed.

A hold is the one transition nothing arrives to cause, so it is checked at the top of every dispatch
against the frame's clock, and the loop owes frames for as long as a still press could still become
one. After a hold, a release is not a tap, and the first movement is a drag that belongs to the
target whatever its axis and however far it went: the hold already settled who is holding the
gesture. That is how a text field scrolls under a plain drag but selects from a press that was held
first, with one rule on touch and on a desktop.

## The wheel

A wheel notch is not a gesture. It has no lifecycle, no claim and no target: it reads the stack at
the pointer, takes the dominant axis of its delta, and moves the innermost region under the pointer
that scrolls on that axis, handing outward at an end exactly as a drag would.

## Keys

`Tab`, `Shift+Tab` and `Escape` belong to focus, and dispatch answers them wherever focus is,
including nowhere: each becomes a `Focus` op. Every other key becomes a `Keyed` op carrying whatever
held focus at that moment, or nothing, and the drain applies it: it is reported to its target (or to
the app, as a root key) and then handed to the element, which is how a field types. Because the key's
effect is an op, it lands behind anything queued before it and is reported in the next frame, a frame
later than a tap. [The frame](frame.md#f7-the-collection-window-is-since-your-last-frame) tabulates
when each report arrives.

Modifiers arrive as their own input, in the same stream as the keys, and dispatch keeps what is held
as engine state. So what a key was pressed with is decided by the order the two arrived in, and the
test suite reaches it by sending the same events a window does.

## Focus

Focus rests only on an element that declared `interactive`: the set that asked to receive input is
the set a keyboard can reach, with no second declaration to keep in step. It moves in four ways (a
tap, `Tab` and `Shift+Tab`, `Escape`, and the verbs) and every one of them goes through one function,
`focus::moved`.

Whether an element can take focus is answered by walking its ancestry asking whether anything is
hidden or disabled. The walk reads **declarations**, not R7's product from last frame, so it is
current as of the moment in the drain it is asked. That is what lets an app show a drawer and focus
into it in the same frame, and it means focus is final before resolution runs: a field's caret, which
shows only while it holds focus, is an ordinary `visible` write resolved like anything else.

A focus op that names something that cannot take focus is dropped, and focus stays where it was
rather than moving somewhere the app did not name. At the end of the drain, `focus::sweep` lets go of
whatever holds focus if it has since been hidden, disabled or pruned.

**Order** is reading order: every focusable element, sorted by `focus_order` first, then by the top
and then the left of where it was drawn, then by allocation order, so the sort is total and never
arbitrary. **Scopes** trap: stepping is confined to the innermost `focus_scope` ancestor of whatever
holds focus, so `Tab` inside a dialog cycles inside the dialog. A scope is read from where focus *is*,
so a dialog that declares itself a scope while focus is elsewhere traps nothing until focus enters it.

The engine draws nothing for focus. A focused element may be a `Stem` with nothing visible at all,
so there is no mark the engine could draw that would be right.

## What the fields make of it

A text field is several elements under one name, and what a tap or a drag means to it (a caret
placed, a selection extended) is its own business rather than dispatch's. Dispatch reports taps,
holds and drags like any other; straight after dispatch, before the app's `frame`, each field reads
what was reported about it and queues what that means. [Fronds](fronds.md) covers it.
