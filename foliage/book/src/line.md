# Line

Every other primitive is positioned with a `Location` box (`.at(Location::new().xs(..))`).
`Line` is the exception -- it's defined by two points, not a box:

```rust
// foliage_proper/src/line/mod.rs
#[derive(Component, Copy, Clone)]
#[require(LineQuad)]
pub struct Line {
    pub weight: i32,
}
```

`Points<Logical>` (two `Position<Logical>`s, see [Coordinates](./coordinate.md)) is what
an author actually writes; `Line`/`weight` just says how thick to draw the segment
between them. `LineQuad` is the derived, GPU-facing representation neither the author
nor `Line` itself computes directly:

```rust
// foliage_proper/src/line/mod.rs
pub(crate) fn distill_descriptor(
    mut lines: Query<(&Points<Logical>, &Line, &mut LineQuad), Or<(Changed<Line>, Changed<Points<Logical>>)>>,
    scale_factor: Res<ScaleFactor>,
) {
    // turns two points + a weight into four corner positions (a thin quad,
    // perpendicular-offset from the line's own slope by half the weight)
}
```

Given two points and a weight, `distill_descriptor` computes the perpendicular normal to
the line's slope, offsets each endpoint by half the weight along that normal, and
produces the four corners of a thin quad -- converted to physical pixels via
[`ScaleFactor`](./ginkgo.md) at the same point every other logical-to-physical crossing
happens. This only re-runs when `Line` or `Points<Logical>` actually changes
(`Or<(Changed<Line>, Changed<Points<Logical>>)>`), not every frame.

## Registration

```rust
// foliage_proper/src/line/mod.rs
impl Attachment for Line {
    fn attach(foliage: &mut Foliage) {
        foliage.diff.add_systems(Line::distill_descriptor.in_set(DiffMarkers::Finalize));
        foliage.remove_queue::<LineQuad>();
        foliage.differential::<LineQuad, LineQuad>();
        // BlendedOpacity, ResolvedElevation, ClipContext, Color
    }
}
```

Notice the differentials are registered against `LineQuad`, not `Line` -- the GPU only
ever needs the *derived* quad geometry, never the raw two-point/weight description, so
that's what's tracked and queued for the renderer, exactly the same "derive once,
diff the derived form" shape [Text](./text.md)'s `ResolvedGlyphs` follows relative to
the author's raw `value` string.
