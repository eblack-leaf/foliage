# Responsive Segment

A `ResponsiveSegment` is used for placing elements on the `MacroGrid`.
The `MacroGrid` is made up of `column`|`row` counts to divide the screen into 
subsections for deterministic placement across varying screen sizes.

```rust
MacroGrid::new(8, 6)
```

The `Gap` size starts at `8.0` and will scale to be a maximum of `0.15`% of the
`column_width`|`row_height`.

The `Gap` can be configured with

```rust
MacroGrid::new(4, 8).assign_gap(GapDescriptor::Vertical, 4.0)
```

would assign a vertical gap of `4.0` as there are more `rows` than `columns`.

Then elements are placed from `begin`-`end` ranges on the grid.

```rust
1.near().to(4.far())
```

This describes starting at the `first` column on the near side,
then spanning to the  `fourth` column on the far side.

If that does not suit your needs when scaling screen size,
add a `.minimum(...)` | `.maximum(...)`

```rust
1.near().to(4.far())..minimum(45.0)
```
```rust
1.near().to(4.far()).minimum(45.0).maximum(150.0)
```
which will constrain the dimension to that value(s).

You can add fixed offsets to the base locations with `.offset(...)`.

```rust
1.near().offset(10.0).to(4.far())
```

You can also just `fix` the size for exact element sizing

```rust
1.near().to(4.far()).fixed(44.0)
```

which will put both a `min`|`max` of `44.0`.

Relative and absolute placements can be accomplished with 

```rust
1.near().to(0.3.relative())

45.absolute().to(4.far())
```

where `#.relative()` will use the corresponding dimension of the `Viewport` to determine location,
and `#.absolute()` will be exactly there

An entire `ResponsiveSegment` is made up of a `horizontal`|`vertical` range of these specifiers,
with a few extra tweaks.

```rust
ResponsiveSegment::base(
    Segment::new(
        4.near().to(5.far()),
        4.near().to(4.far()).minimum(30.0).maximum(40.0),
    ))
    .justify(Justify::Top)
    .without_portrait_mobile()
```

You can `Justify`, elements to achieve cleaner looks

```rust
.justify(Justify::Center) or .justify(Justify::TopLeft)
```

You can `negate` certain [`Layout`](layout.md)s that are not aesthetically pleasing
with an element at certain screen sizes.

```rust
.without_landscape_tablet()
```

would optionally disable the element at that configuration

You can redefine the elements location at certain `Layout`s as well

```rust
ResponsiveSegment::base(...).exception([Layout::LANDSCAPE_MOBILE], 4.near().to(5.far()))
```

You can also define an `AspectRatio` for the dimensions to adhere to

```rust
ResponsiveSegment::base(...).aspect_ratio(16 / 9)
```