# Slider

Drag, tap-to-seek, and a programmatic write all funnel through one door -- the shared
[`Progress`](../composites-overview.md) value channel -- so nothing downstream can tell
which one moved it, and doesn't need to:

```rust
// foliage_proper/src/composite/slider.rs
pub struct Slider {}
pub struct SliderStyle { pub track: Color, pub fill: Color, pub weight: i32, pub knob_size: i32 }
pub struct SliderBehavior { pub interactive: bool } // separate from style on purpose
```

`SliderBehavior` is deliberately its own component, not a `SliderStyle` field -- the doc
comment explains why: flipping `.interactive(false)` at runtime (a media player locking
its timeline while buffering, say) shouldn't require re-stating every color just to
change one behavior flag.

## Drag and tap-to-seek compute the same percentage

```rust
// foliage_proper/src/composite/slider.rs (drag handler, abridged)
let bounds = sections.get(this).unwrap();
let pct = ((interaction.click().current.left() - bounds.left()) / bounds.width()).clamp(0.0, 1.0);
tree.write_to(this, Progress(pct));
```

Both the drag observer (subscribed on the knob) and the tap-to-seek `on_click` (on the
root, anywhere along the track) reduce to this same cursor-position-to-percentage
computation, each writing `Progress` -- neither one touches geometry directly; that's
the render reaction's job entirely.

## The knob-inset correction

```rust
// foliage_proper/src/composite/slider.rs (abridged)
let inset = styles.get(e).unwrap().knob_size as f32 / 2.0;
let correction = (inset * (1.0 - 2.0 * value)) as i32;
```

The knob is anchored to the *fill* line's own endpoint with no inset of its own, so
without this correction it would clip past the track's edges at the extremes (half the
knob rendering outside the parent rect at `value == 0.0` or `1.0`). Rather than mapping
`value` onto a narrower `(inset, track_width - inset)` range, the doc comment explains
the crate takes the closed-form equivalent: a percentage position plus a pixel correction
that shrinks linearly from `+half-knob` at zero to `-half-knob` at one, vanishing exactly
at the midpoint -- because percentage and pixel units can't otherwise be combined into
one `Location` value directly.

## Disabling cascades to the knob for free

```rust
if behaviors.get(e).unwrap().interactive {
    tree.enable(e);
    tree.write_to(knob, Visibility::new(true));
} else {
    tree.disable(e);
    tree.write_to(knob, Visibility::new(false));
}
```

`tree.disable(e)` disables at the *root*; the [`INHERIT_ENABLED` cascade](../interaction.md)
reaches the knob's own listener too, so re-enabling later restores both without the
slider needing to track and re-enable each interactive child individually. A
non-interactive slider swallows no clicks -- everything underneath it is genuinely
disabled, not just visually dimmed.
