# Layout: Grid, Location, Anchor

Every example so far has quoted calls like `8.px().as_left().with(160.px().as_width())`
without explaining them. This chapter is that explanation.

## The grammar: `LocationValue` → `ValueDescriptor` → `ConfigurationDescriptor`

`GridExt` is implemented for the plain number types (`i32`, `f32`, `u32`, ...), which is
what makes `8.px()` legal at all:

```rust
// foliage_proper/src/grid/location.rs
pub trait GridExt {
    fn pct(self) -> LocationValue;
    fn px(self) -> LocationValue;
    fn col(self) -> LocationValue;
    fn row(self) -> LocationValue;
    fn letters(self) -> LocationValue;
}
pub enum LocationValue {
    Percent(f32), Px(CoordinateUnit), Column(i32), Row(i32), Anchor(Designator),
    TextContent, Letters(i32),
}
```

`.as_left()`/`.as_right()`/`.as_top()`/`.as_bottom()`/`.as_width()`/`.as_height()` wrap a
`LocationValue` into a `ValueDescriptor` (the value plus which edge it describes);
`.with(..)` pairs two `ValueDescriptor`s (one axis-start edge and its counterpart) into a
`ConfigurationDescriptor`. So `8.px().as_left().with(160.px().as_width())` reads exactly
as written: left edge is 8px in, width is 160px. `Location::new().xs(horizontal,
vertical)` takes one `ConfigurationDescriptor` per axis; `.sm()`/`.md()`/`.lg()`/`.xl()`
override it at larger breakpoints, falling back to the next-smaller configured one if a
given breakpoint has none of its own (`Grid::config`'s `at_least_sm`/`at_least_md`/
`at_least_lg` chain).

`col()`/`row()` resolve against the parent's `Grid` (see below) instead of raw pixels --
`1.col().as_left().with(1.col().as_right())` means "the first column, full width," the
pattern every composite's own internal skeleton uses (see [Button](./composite-button.md)'s
panel, sized to `1.col()`/`1.row()` -- one full cell). `letters()` sizes against the
current font's monospaced character width, used for text-content-driven width like
`TextInput`'s field.

## `Grid`: the coordinate system a `Location` resolves against

```rust
// foliage_proper/src/grid/mod.rs
#[derive(Component, Copy, Clone)]
#[require(View)]
pub struct Grid {
    pub xs: GridConfiguration,
    pub sm: Option<GridConfiguration>,
    // md, lg, xl -- same breakpoint-override shape as Location
}
```

Any entity that's going to size children with `.col()`/`.row()`/`.pct()` needs `Grid` on
itself -- resolving a child's `Location` always reads its parent's `Grid` unconditionally,
regardless of which units the child actually uses (`examples/opacity_and_elevation.rs`'s
own comment on this: a grid-less parent panics resolving *any* child, even a plain-`px`
one). `Grid::default()` is `Grid::new(1.col(), 1.row())` -- one column, one row, the
common case for "this entity is just a positioned box," not an actual multi-cell layout.

## `Anchor`: relative-to-another-entity, not relative-to-parent

`anchor().left().as_left()` and friends (used throughout `Button`'s icon positioning,
`Dropdown`'s option surface) resolve against a *named sibling or ancestor* entity's own
resolved `Section`, via `Anchor::new(target)`, instead of the parent's `Grid`. This is
what lets a Popover's content surface size itself relative to its trigger, or an icon
size itself relative to a text label it sits beside, without either one needing to know
the other's absolute position.

`View` (required by `Grid`) is the scroll/pan state a `Location` resolves *through* --
covered where it matters most, in [TextInput](./composites/text-input.md)'s
scroll-into-view logic and `List`/`Carousel`'s viewport.
