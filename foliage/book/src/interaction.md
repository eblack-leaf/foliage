# Interaction and Focus

[Leaf](./leaf.md) requires `InteractionShape`/`InteractionPropagation`/`FocusBehavior`
on every entity unconditionally -- so the interaction system never needs to special-case
"this entity might not be able to receive input." Whether it actually *does* is a
separate question, answered by `InteractionListener`.

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

## `InteractionPropagation`: pass-through and grab

Every entity defaults to `grab: true` (per `Leaf`'s unconditional requirement) -- a plain
`Panel`/`Text`/`Icon` with no `InteractionListener` still competes to *own* a click even
though it does nothing with it, since hit-testing is a flat, elevation-ranked scan over
every `Leaf`, not ancestor-bubbling (see [Popover](./composites/popover.md)'s doc comment
on exactly this constraint). `InteractionPropagation::pass_through()` is how a rendering
child (`Button`'s own `Panel`/`Text`) opts back out, so the click reaches the composite
root's own listener instead of being swallowed by whichever visual child happened to be
on top. `disable_drag()` (used by `Carousel`'s viewport, `Slider`'s knob) is the
narrower version -- still grabs for click purposes, but excludes drag-panning
specifically, so a composite that drives its own drag logic doesn't fight the generic
view-panning behavior a `Grid`-bearing entity gets by default.

## `FocusBehavior`

`FocusBehavior::ignore()` (used the same places `pass_through()` is, on `Button`'s
`Panel`/`Text` children) keeps a purely-visual child from stealing keyboard focus away
from the composite root that should actually hold it -- the same "rendering children
shouldn't compete with their own composite root" pattern `InteractionPropagation`
follows, applied to focus instead of clicks.

## After the grab: walking up to find a `View` to pan

Hit-testing itself (deciding *which* entity wins the grab, above) is a flat scan with no
ancestor walking involved. But once something's grabbed and actually dragged or
scrolled, a second, separate question comes up: *which* entity's `View` (see
[Grid](./grid.md)) actually gets panned? The grabbed entity itself might have no `View`
of its own (a `Slider`'s knob, a `TextInput`'s cursor) -- so `interactive_elements`
walks up the grabbed entity's real `Stem` chain looking for the nearest ancestor that
does:

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
wheel-scroll channel: a drag not meant for the nearest view (a `Carousel` swiping its own
pages) must still be able to reach whatever scrollable ancestor further out *is* meant to
receive it (the page behind the `Carousel`, on a platform where dragging is the only
scroll input there is).

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
hard recency cutoff, not a continued exponential decay. Decaying by `coast`'s own
`decay.powf(elapsed_ms)` here was tried and rejected: it can't guarantee a settled stop
for *any* fixed real-world pause, since a fast enough original swipe always needs
proportionally longer to decay under the same fraction (`decay.powf` approaches zero but
never reaches it) -- a pause long enough to neutralize a typical flick still isn't enough
for an unusually fast one. Only past the cutoff is the pointer read as having genuinely
stopped, independent of how fast it was moving before that. Then the (possibly-zeroed)
velocity is checked against
[`ScrollMomentum::velocity_threshold`](./grid.md#scrollmomentum-coasting-after-a-dragtouch-release):
above it, the same entity that would've received the final `ViewAdjustment` gets a
`Coasting` component instead (the final 1:1 diff still applies first); below it, nothing
further happens. Wheel-scroll release never coasts -- it already has its own per-tick
`ScrollInertia` scaling, a different mechanism for a discrete-pulse input rather than
continuous tracking.
