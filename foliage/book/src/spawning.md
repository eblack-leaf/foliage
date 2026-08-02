# Specs and Sprout: What You Grow

`Panel::new()` doesn't create anything -- it returns a `PanelSprout`, a plain config
struct you build up with a chained call, then hand to [`Grows::leaf`/`Grows::branch`](./canopy.md):

```rust
// foliage/examples/interaction.rs
canopy.leaf(
    Panel::new()
        .color(Color::orange(600))
        .rounding(Rounding::Sm)
        .at(Location::new().xs(
            20.px().as_left().with(120.px().as_width()),
            70.px().as_top().with(120.px().as_height()),
        ))
        .elevate(Elevation::up(1))
        .grid(Grid::default())
        .interactive(),
)
```

`.color(..)`/`.rounding(..)` are `PanelSprout`'s own vocabulary -- every primitive has
its own. `.at(..)`/`.elevate(..)`/`.grid(..)`/`.interactive(..)` are common to all of
them, chained from one trait every builder shares.

## `Sprout`: the placement vocabulary every builder shares

```rust
// foliage_proper/src/author.rs
pub trait Sprout: Author {
    fn at(mut self, location: Location) -> Self;
    fn elevate(mut self, e: Elevation) -> Self;
    fn grid(mut self, grid: Grid) -> Self;
    fn anchored(mut self, leaf: Leaf) -> Self;
    fn opacity(mut self, value: f32) -> Self;
    fn align(mut self, h: HorizontalAlignment, v: VerticalAlignment) -> Self;
    fn interactive(mut self) -> Self;
    fn pass_through(mut self) -> Self;
    fn holds_drag(mut self) -> Self;
    fn round_hit_area(mut self) -> Self;
    fn overscroll(mut self, passes: bool) -> Self;
    fn clip_to_viewport(mut self) -> Self;
    fn aspect(mut self, ratio: AspectRatio) -> Self;
    fn sized_by_content(mut self, width: bool, height: bool) -> Self;
    fn font(mut self, font: FontId) -> Self;
}
```

`elevate` is the one with no default -- not an oversight, a deliberate hard requirement.
Any default would have to pick something (front, back, "one above whatever the parent is
this moment"), and whichever it picked would be silently wrong for any element that meant
something else. Leaving `Elevation` unset is what turns a forgotten `.elevate(..)` into a
loud failure at grow time instead of a UI that's silently in the wrong stacking order.

`grid` is required on anything that will have children -- a child's `Location` resolves
against its parent's grid, and growing a child under a parent with none is a failure
rather than a silent misplacement.

`Sprout`'s supertrait, `Author`, is `pub(crate)` -- you can call every method above on any
builder foliage hands you, but you cannot implement `Sprout` for a type of your own. The
set of things that can be grown is closed to what the engine itself provides; see
[Inside the Engine](./tree.md) for what `Author` actually does and why.

## `Spec`: the closed vocabulary of what can be grown

```rust
// foliage_proper/src/boundary/op.rs
pub enum Spec {
    Bare(LeafSprout),
    Panel(PanelSprout),
    Text(TextSprout),
    Icon(IconSprout),
    Image(ImageSprout),
    Line(LineSprout),
    Polygon(PolygonSprout),
    Polyline(PolylineSprout),
    TextInput(TextInputSprout),
}
```

`Grows::leaf`/`Grows::branch` take `impl Into<Spec>`, and every builder type converts
into it, so a call site writes `canopy.branch(under, Panel::new()...)` rather than
`canopy.branch(under, Spec::Panel(Panel::new()...))` -- the enum is the queued form, not
something you name yourself. `Bare` is the one variant with no primitive of its own: a
container for children, or a bare interaction hit area, built with
[`Bare::new()`](./panel.md) and taking the same `Sprout` chain as anything else.

This is the whole vocabulary foliage knows how to grow. There is no way to hand
`Canopy`/`Sprig` a type they don't already know about -- extending the set of things that
can be grown, or composing these primitives into a reusable widget, is engine-internal
work, covered in [Inside the Engine](./tree.md).

## `Motion`/`Timing`: animating a grown element

Changing a property once is `Grows::color`/`Grows::location`/etc. Tweening one is
[`Grows::animate`](./canopy.md), which takes a `Motion` (the closed set of things foliage
knows how to interpolate on an element) and a `Timing`:

```rust
// foliage_proper/src/boundary/op.rs
pub enum Motion {
    Opacity(f32),
    Color(Color),
    Elevation(Elevation),
    Location(Location),
    Polygon(Polygon),
    Outline(Outline),
}
pub struct Timing {
    pub start: u64,
    pub finish: u64,
    pub ease: Ease,
    pub repeat: Repeat,
    pub backtrack: bool,
}
```

`Timing::over(finish)` is the common case -- a single linear pass from now to `finish`
milliseconds -- with `.after(..)`/`.eased(..)`/`.repeat(..)`/`.backtrack()` layered on.
`Grows::animate_during` joins the tween to a sequence opened with
[`Grows::sequence`](./canopy.md), so its completion counts toward that sequence's
[`Bloom::SequenceFinished`](./canopy.md) -- the hook for chaining one stage of motion onto
the next. For values foliage has no concept of, [`Grows::tween`](./canopy.md) runs the
same easing and timing over plain numbers you supply, reporting each frame's values as
[`Bloom::Tween`](./canopy.md) instead of writing them anywhere.
