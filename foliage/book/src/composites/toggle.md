# Toggle

Same persistent-state shape as [Checkbox](./checkbox.md) -- one `ToggleState(bool)`,
flipped by a click or a programmatic write, redrawn by one reaction regardless of which
-- but with an animated knob instead of an instant redraw, and Material 3's specific
track/knob sizing behavior:

```rust
// foliage_proper/src/composite/toggle.rs
fn knob_location(on: bool) -> Location {
    if on {
        // half-track-width pill, inset 4px from the track's own edges
        Location::new().xs(50.pct().as_left().adjust(2).with(100.pct().as_right().adjust(-4)), ...)
    } else {
        // roughly half that diameter, centered in the track's left portion
        Location::new().xs(12.5.pct().as_left().with(37.5.pct().as_right()), ...)
    }
}
```

The doc comment on this function explains a real, tested design choice: the knob doesn't
just slide left/right at a fixed size -- it visibly *shrinks* while off, the same size
drop Material's own switch uses to keep the empty state reading as genuinely smaller,
not just "the same knob, slid over." Both states use pure percentage anchors (no fixed
pixel insets) specifically so the shrink scales with the track's own height instead of
being crushed at whatever small track size a real app actually uses.

## Sliding, not snapping

```rust
// foliage_proper/src/composite/toggle.rs (abridged)
if place_directly {
    tree.write_to(knob, knob_location(on)); // first fire: nothing to slide from yet
    place_directly = false;
} else {
    Sequence::new(&mut tree).animate(
        Animation::new(knob_location(on)).targeting(knob).start(0).finish(150).eased(Ease::EMPHASIS),
    );
}
```

A captured `place_directly` flag (the same "first fire is special" pattern the crate
calls out elsewhere as the ProjectCard-image pattern) distinguishes initial placement
from every later toggle: the very first fire has no prior knob position to slide from,
so it writes the target `Location` directly; every subsequent state flip animates to it
instead of snapping.

## Track exclusivity mirrors `Checkbox` exactly

```rust
// foliage_proper/src/composite/toggle.rs
if on {
    tree.write_to(track, (style.on_fill, Outline::default()));
    tree.write_to(knob, style.knob_on);
} else {
    tree.write_to(track, (style.off_outline, Outline::new(2)));
    tree.write_to(knob, style.knob_off);
}
```

The same [`Panel`](../panel.md) fill-vs-outline switch [Checkbox](./checkbox.md) uses for
its box carries `Toggle`'s track state: solid fill while on, stroke-only (whatever's
behind the track shows through) while off. The optional check-icon glyph on the knob
follows the same "structural config, spawned lazily, registered before the style
reaction" pattern `Checkbox` uses too -- parented to the *knob* itself here, specifically
so the icon's position rides along with the knob's own slide for free rather than being
repositioned by a second, separate reaction.
