# Interaction and Focus

[`Node`](./tree.md) requires `InteractionShape`/`InteractionPropagation`/`FocusBehavior`
on every on-screen entity unconditionally -- so the interaction system never needs to
special-case "this entity might not be able to receive input." Whether it actually
*does* is a separate question, answered by `InteractionListener`. From an app, this
whole system is what turns into [`Sprout::interactive`/`Sprout::pass_through`/etc.](./spawning.md)
on the way in and [`Bloom::Clicked`/`Engaged`/`Dragged`/etc.](./canopy.md) on the way
out.

## Hit-testing: shape + clip, not bounding-box-only

```rust
// foliage_proper/src/interaction/listener.rs
pub(crate) fn is_contained(shape: InteractionShape, section: Section<Logical>, clip: ResolvedClip, event: Position<Logical>) -> bool {
    let section_contained = match shape {
        InteractionShape::Rectangle => section.contains(event),
        InteractionShape::Circle => section.center().distance(event) <= section.width() / 2f32,
    };
    section_contained && clip.0.contains(event)
}
```

A click has to land inside both the entity's own shape *and* whatever ancestor clip
region applies -- a `Panel` with `Rounding::Full` switches to `InteractionShape::Circle`
automatically (see [Panel](./panel.md)'s `Rounding::on_insert`), so a circular button
doesn't grab clicks that land in its bounding box's square corners.

## Three independent gates, not one flag

```rust
// foliage_proper/src/interaction/listener.rs
bitflags! {
    impl InteractionState: u8 {
        const ENABLED = 1 << 0;
        const AUTO_ENABLED = 1 << 1;
        const INHERIT_ENABLED = 1 << 2;
    }
}
pub fn disabled(&self) -> bool {
    !(self.state.contains(ENABLED) && self.state.contains(AUTO_ENABLED) && self.state.contains(INHERIT_ENABLED))
}
```

All three must be set for a listener to actually respond. `ENABLED` is the author's own
explicit `Disable`/`Enable`; `AUTO_ENABLED` is the library's own internal opt-out (a
widget disabling one of its own children's interactivity as part of its own logic, not
an author choice); `INHERIT_ENABLED` is what a parent's `Disable` cascades down as (see
[Lifecycle](./lifecycle.md)). Three independent bits mean disabling a parent doesn't
require touching every descendant's own `ENABLED` state directly -- it only has to flip
`INHERIT_ENABLED` down the tree, leaving the author's and the library's own separate
opt-outs untouched and independently restorable.

## The two drag emissions, and the threshold between them

A gesture reports its motion twice over, and the pair is not redundant. `Dragged` is the
stream: one per frame that carries a pointer move, from the first pixel, which is what a
knob or a slider following the pointer consumes. `DragStarted` is the threshold: sent
once, on the move where the gesture's travel first exceeds
`InteractionListener::DRAG_THRESHOLD` on either axis, ahead of that same move's
`Dragged`.

The threshold is the moment the gesture stops being a pending click -- a release after it
produces `Disengaged` alone, where a release before it produces `Clicked` as well. That
makes `DragStarted` the hook for anything that has to commit at exactly that point, and
the reason it exists at all: measured travel is a per-axis comparison against a constant
the engine owns, so an app inferring the same moment from the `Dragged` stream would be
keeping a second copy of both, and the obvious straight-line reading of the distance
disagrees with the engine diagonally.

## `InteractionPropagation`: pass-through and grab

Every entity defaults to `grab: true` (per `Node`'s unconditional requirement) -- a plain
`Panel`/`Text`/`Icon` with no `InteractionListener` still competes to *own* a click even
though it does nothing with it, since hit-testing is a flat, elevation-ranked scan over
every on-screen entity, not ancestor-bubbling. `InteractionPropagation::pass_through()`
is how a purely-visual child (a widget's own inner `Panel`/`Text`) opts back out, so the
click reaches the widget root's own listener instead of being swallowed by whichever
visual child happened to be on top. `disable_drag()` is the narrower version -- still
grabs for click purposes, but excludes drag-panning specifically, so a widget that drives
its own drag logic (a knob, a swipeable page) doesn't fight the generic view-panning
behavior a `Grid`-bearing entity gets by default.

## `FocusBehavior`

`FocusBehavior::ignore()` (used the same places `pass_through()` is, on a widget's own
purely-visual children) keeps a rendering child from stealing keyboard focus away from
the widget root that should actually hold it -- the same "rendering children shouldn't
compete with their own root" pattern `InteractionPropagation` follows, applied to focus
instead of clicks.

## After the grab: walking up to find a `View` to pan

Hit-testing itself (deciding *which* entity wins the grab, above) is a flat scan with no
ancestor walking involved. But once something's grabbed and actually dragged or
scrolled, a second, separate question comes up: *which* entity's `View` (see
[Grid](./grid.md)) actually gets panned? The grabbed entity itself might have no `View`
of its own (a widget's own draggable knob, a `TextInput`'s cursor) -- so
`interactive_elements` walks up the grabbed entity's real `Parent` chain looking for the
nearest ancestor that does:

```rust
// foliage_proper/src/interaction/mod.rs (abridged, the drag-move case)
let mut context = *contexts.get(p).unwrap();
while let Some(id) = context.id {
    if let Ok(_) = views.get(id) {
        if !all.get(id).unwrap().4.disable_drag {
            tree.entity(id).insert(ViewAdjustment(diff));
            break;
        }
    }
    // a disabled view doesn't stop the search -- keep walking up for the next one.
    if let Ok(up) = contexts.get(id) { context = *up; } else { break; }
}
```

The crate's own comments call this the interaction system's "LCA walk" (see
`ash/clip.rs`'s doc comment on `ClipContext`) -- informally: it's a single-entity ancestor
walk from the grabbed entity outward, not a two-entity lowest-common-ancestor
computation in the strict graph-theory sense. A disabled view along the way doesn't stop
the search either -- it keeps walking further out, since touch has no separate
wheel-scroll channel: a drag not meant for the nearest view (a widget swiping its own
pages) must still be able to reach whatever scrollable ancestor further out *is* meant to
receive it (the page behind it, on a platform where dragging is the only scroll input
there is).

## After the release: velocity, and handing off to a coast

Every drag-move above stays exactly 1:1 -- but `interactive_elements` also tracks a
smoothed (EMA, not the single latest sample) px/ms velocity alongside that raw `diff`,
so release can tell a flick from a drag that was already settling:

```rust
// foliage_proper/src/interaction/mod.rs (abridged)
let now = Moment::now();
if let Some(last_time) = current.last_drag_time {
    let elapsed_ms = now.duration_since(last_time).as_secs_f32() * 1000.0;
    let instant = diff / elapsed_ms;
    current.velocity = current.velocity * (1.0 - SMOOTHING) + instant * SMOOTHING;
}
current.last_drag_time = Some(now);
```

`last_drag_time` is seeded the moment `past_drag` first flips true (crossing
`DRAG_THRESHOLD`), not just at the initial `Start` -- a fling that only sends a couple of
move samples before release (a real fast flick easily can) still needs a prior timestamp
to diff its first real move against, or it could never compute a nonzero velocity at all.

On release, `current.velocity` is first zeroed outright if more than
`ScrollMomentum::stillness_cutoff_ms` has passed since the last actual move sample -- a
hard recency cutoff, not a continued exponential decay. Decay cannot do this job: it
guarantees a settled stop for no fixed real-world pause, since a faster original swipe
always needs proportionally longer to fall under the same threshold (`decay.powf`
approaches zero without reaching it), so a pause long enough to neutralize a typical
flick still leaves an unusually fast one coasting. Past the cutoff the pointer is read as
having stopped, independent of how fast it was moving before that. Then the (possibly-zeroed)
velocity is checked against
[`ScrollMomentum::velocity_threshold`](./grid.md#scrollmomentum-coasting-after-a-dragtouch-release):
above it, the same entity that would've received the final `ViewAdjustment` gets a
`Coasting` component instead (the final 1:1 diff still applies first); below it, nothing
further happens. Wheel-scroll release never coasts -- it already has its own per-tick
`ScrollInertia` scaling, a different mechanism for a discrete-pulse input rather than
continuous tracking.

That target is the first `View` walking up from the grab, which is routinely a card
carrying an internal layout rather than the list the content visibly scrolls -- every
`Grid`-bearing entity has a `View`, for reasons that have nothing to do with scrolling
(see [Grid](./grid.md)). The motion reaches the real scroller from there through
`OverscrollPropagation`, exactly as the live drag's own pan did, and keeping the coast on
the pan's own target is what makes it continue precisely the motion the drag was
producing. It also means two cards in one list can each be a coast target, which is why
stopping a coast can't be phrased in terms of the tree -- see
[`ScrollMomentum`](./grid.md#scrollmomentum-coasting-after-a-dragtouch-release).
