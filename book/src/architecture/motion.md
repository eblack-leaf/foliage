# Motion

`Aspen` is the engine's motion: every running motion, channel and timer, advanced once a frame on the
frame's one clock. It lives in
[`aspen/mod.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/aspen/mod.rs) and
[`aspen/ease.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/aspen/ease.rs), and
it is step 5 of the frame, between the drain that starts motions and the resolution that applies
most of them.

## The hard case

Animating a color is easy: two colors and a number between them. Animating a *placement* is the case
the engine was shaped around, because a `Location` is not a value. It is a function from a context
(the breakpoint, the trunk's box, the anchor's box, a measured size) to a box, and the context can
change while the motion runs. So "animate to this placement" cannot be answered once at the start:

```text
box(t) = blend( resolve(from, context now), resolve(to, context now), ease(t) )
```

Both ends are resolved again every frame, in that frame's context. Nothing about the motion is
cached, so nothing about it can go stale: a resize, a crossed breakpoint or a moved anchor reaches
both ends at once, and the motion lands exactly on its target rather than near it. This is only
possible because [the resolver](resolver.md) is pure, and can be asked twice about one element on one
axis in one frame.

## The target is written at once

Starting a motion (the drain applying `Op::Animate`) writes the **target** to the element
immediately, and the motion carries what the element **left**. So from the moment it is told to
move, an element declares where it is going, and three things follow with no code of their own:

- **Arriving changes nothing.** The blend at the end is exactly the plain reading of the declaration,
  so there is no settling step that could land a pixel off.
- **Cancelling is a removal.** A direct write replaces the declaration and drops the motion; the
  element is at what was written, and there is nothing left to reconcile.
- **Stopping early lands on the target.** `stop` and `finish` give up the duration, not the
  destination, so an interrupted motion leaves the element where running it out would have.

What a motion departed from is one of two things:

```rust,ignore
// aspen/mod.rs
pub(crate) enum Departed<D, S> {
    Declared(D),  // a value the app wrote: re-resolved every frame
    Snapshot(S),  // where the element was when a second motion replaced the first
}
```

A **declared** departure re-resolves every frame like the target does. A **snapshot** is what a
retarget takes: when a second `animate` replaces a running one, the element is somewhere between two
placements, which is a box and not a placement, so the new motion departs from that box. That is
right rather than a compromise. A mid-motion blend never corresponded to any declared state, so there
is nothing for it to be stale relative to; it is a starting pixel, and the target is still fully live.
Starting the second motion from the first one's origin instead would make the element jump back.

## Progress, and who applies it

Step 5 advances every motion against the clock and computes its eased progress once. It does *not*
apply every motion itself. Which phase applies a motion follows from one rule:

> A blend of the same type as the declaration is written back over it here. A blend of a different
> type is left to the phase that reads the declaration.

| `Motion` | Blends to | Applied in |
|---|---|---|
| `Opacity` | a number, like its declaration | step 5, written back through the tree |
| `Polygon` | a shape, like its declaration | step 5, written back |
| `Location` | a box, where the declaration is a placement | R2, when the element is resolved |
| `Trace` | two ends, where the declaration is a trace | R2 |
| `Scroll` | an offset, where the target is a `ScrollTo` | R4 |
| `Color`, `Palette` | a color, where the declaration is a fill | extraction |

The fill case is the one that needs saying. A fill is a `Palette` role or a literal color, and a blend
of two fills is always a color, so not even two literals could be written back as a fill. The blend
happens where a fill becomes a color, which is extraction, and extraction is also where the
element's opacity is folded in. Nothing then holds a second color that a repaint would have to find:
a role at either end of a motion follows a repaint mid-motion, because both ends are read against
the scheme every frame.

A motion held beside the tree is invisible to the tree's write records, so step 5 records one for
it every frame it runs: a placement motion marks its element as written (so R2 resolves it again),
and a fill motion marks it restyled (so extraction draws it again). An opacity or a shape is written
through the tree's own setters, which record themselves.

```rust,ignore
// aspen/mod.rs
fn stirred(tree: &mut Tree, leaf: Leaf, moving: &Moving) {
    match moving {
        Moving::Location(_) | Moving::Trace(_) => tree.declared(leaf),
        Moving::Fill(_) => tree.restyled(leaf),
        Moving::Opacity { .. } | Moving::Shape { .. } | Moving::Scroll { .. } => {}
    }
}
```

## Where the motions are held

```rust,ignore
// aspen/mod.rs
pub(crate) struct Aspen {
    motions: Named<Moves, Motioning>,       // keyed by (element, property)
    channels: HashMap<Tween, Channel>,      // tweens and timers
    sequences: HashMap<Sequence, usize>,    // how many are still running in each group
    finished: Vec<Sequence>,                // groups that emptied, to report
}
```

Beside the tree rather than on the elements, because what is moving is a small set and almost never
the tree. Step 5 walks what is running and nothing else, and the loop can ask whether anything is
running (one of the clauses of [whether a frame is owed](loop.md#whether-a-frame-is-owed)) without a
pass over anything.

A motion is keyed by its element and its **property**: location (which a box and a trace share),
opacity, fill (which `Color` and `Palette` share), scroll and shape. That one key is what makes the
rules cheap. A second motion on the same property replaces the first by being inserted under the same
key. A direct write cancels a motion by removing that key. An element that withers has each of its
five keys removed.

## The first frame is not charged

A motion started in the drain begins at this frame's instant. The clock's delta is how long the
interval *ending* at that instant took, which is time that elapsed before the motion existed. So a
motion's first advance charges nothing, and the element does not move on the frame it was told to
start from where it currently is:

```rust,ignore
// aspen/mod.rs
fn advance(&mut self, delta: Duration) -> f32 {
    match self.charged {
        true => self.elapsed += delta,
        false => self.charged = true,
    }
    self.at = self.timing.at(self.elapsed);
    self.at
}
```

The eased progress is computed once, here, and held: R2 reads it once per axis, and a shaped ease is
a curve solved for rather than a multiply.

## Landing, and what is reported

When a motion's time is up, it is removed, and anything not already on the element is written out: an
opacity's or a shape's end value, or a scroll's destination (queued for R4 as though it had been
written directly). A placement or a fill needs nothing written, since the element already declares
it, but it is recorded, because the element was last resolved or drawn at the blend and the frame the
motion goes has to resolve or draw it at its declaration.

Three reports can come of it, each answering a different question, all delivered in the next frame's
`Pollen`:

- **`landed(leaf)`**: the element settled. One report for the element, however many of its properties
  arrived together.
- **`finished(tween)`**: this motion is the one that ended, on its target. Also reported by `finish`,
  and never by a motion that was stopped, cancelled, replaced or withered, which is what makes
  stopping a motion the way to break a chain waiting on it.
- **`sequence_finished(sequence)`**: the last member of a group ended, however it ended.

## Sequences

A sequence is a name and a count, nothing more. Anything that runs on the clock joins one through its
`Timing` (`Timing::ms(200).within(intro)`), from any callsite, at any frame. The count goes up when a
member starts and down however it ends (landing, stopped, cancelled, replaced, or taken down with its
element), and the group is reported once, the frame it reaches zero. A group owns nothing that joined
it: stopping one member is nothing to the others, because there is no list of them for it to be
anything to.

## Channels and timers

`tween(from, to, timing)` runs a number on the engine's clock and easing and writes it nowhere; each
frame's value is reported as `pollen.tween(name)`, and the frame it ends reports its end value and
`finished` together, so there is never an end value to infer from an absence. `timer(timing)` is a
tween from 0 to 1 whose value nobody reads. This is the answer for everything `Motion` deliberately
leaves out: a font size, a count, a value foliage has never heard of. `Motion` is closed because the
engine's obligations should be, not because an app's are, and what is on the list is there because it
*cannot* be animated from outside: nothing an app holds can resolve a placement, a role against the
scheme in force, or a scroll destination against an extent the frame has not measured yet.

## Easing

Every shape is a cubic bezier through (0, 0) and (1, 1): `Linear`, `Decelerate`, `Accelerate`,
`Emphasis`, or a `Curve` with two stated control points. A bezier is parametric, so the curve
parameter for a given fraction of the duration has to be recovered before the progress can be read:
Newton's method for up to eight steps, then bisection for the shapes where the slope is too flat for
it. Both are bounded, so it cannot fail to return and cannot be slow.

Both ends are exact rather than close. `Ease::at` returns exactly 0 and 1 at the ends of the
duration whatever the shape, and `blend` returns exactly `from` at 0 and exactly `to` at 1 rather
than computing `from + (to - from) * 1.0`, which can land a rounding error short. A motion lands
*on* its target, so no landing ever needs a correction after it. A curve whose control points
overshoot is left to overshoot in the middle.

Timing is milliseconds throughout. `after(n)` holds the motion where it was for `n` milliseconds
rather than queuing it: the motion exists from the frame it was asked for, and a direct write cancels
it whether or not it has begun to move.

## One clock, capped

Every motion reads the clock sampled at intake, so two motions started together stay together. The
loop caps how far the clock may move in one frame at 100 milliseconds
([The loop](loop.md#time)), so a stalled machine defers motion rather than skipping it: a motion
resumes where it was and takes longer in wall time, and never reaches its end without having been
drawn on the way.

A test advances the clock by hand, exactly, and uncapped, so the suite's motion tests are arithmetic:
advance 120 milliseconds into a 240-millisecond linear motion and the element is exactly half way.
