# Foliage

`Foliage` is a cross-platform UI written in `Rust`. It can target `Linux | Windows | MacOS` natively,
`Web` via `WebAssembly` and `Android` (via `cargo-ndk`). Capable of running on `iOS` but not ported
as of writing. It leverages `wgpu.rs` and `winit` for native-rendering on (almost) every platform.

## Overview

`Foliage` is the main class to interact with the library.

```rust
let mut foliage = Foliage::new();
```

Then it can be configured to fit your needs.

```rust
foliage.set_window_title("name");
foliage.set_desktop_size((800, 600));
// https://origin/foliage ("" for hosting @ root)
foliage.set_base_url("foliage");
```
By default, there is minimal data to run the program in `Foliage`.
To add `data` or make changes, you can create `branch`es on the tree.
```rust
// run a set of changes
foliage.grow_branch(Home {});
```

Where `Home` has defined the `trait` `Branch`.
```rust
pub trait Branch where Self: Clone + Send + Sync + 'static,
{
    fn grow(self, tree: Tree);
}
```

The `Tree` is a handle to the entirety of the data. Single `Entities` can be added via
`Leaf`s.
```rust
tree.add_leaf(Leaf::new(|l| {
    l.give(...);
    l.give_filtered(Layout::LANDSCAPE_MOBILE, ...);
    l.stem_from("other");
}));
```
Each `Leaf` can hold up to one `Component` of each type to compose complex structures.
You can link `Leaf`s together with one `stem`ming from another to inherit `Opacity`, `Remove`, and `Visibility` changes.
`LeafHandle`s are used as names to identify an `Entity`. `Filtered` components can be added to be
conditionally active at certain `Layout`s.
```rust
LeafHandle::new("name")
```
`Leaf`s must have a name + `GridLocation` and `Elevation`.
```rust
Leaf::new(|l| {...})
    .named("name")
    .located(GridLocation::new())
    .elevation(10)
```
`Elevation` refers to the `z` offset from `0` - `100` with `0` being the `near`-plane.

`GridLocation`s specify how to arrange `Leaf`s on screen.

```rust
let location = GridLocation::new()
    .bottom(screen().x() - 16.px())
    .top("header".y() + 10.percent().of("header"))
    .width(50.percent().of(screen()))
    .left("button".right() + 10.px());
```
Each statement is attached to a designator for what part of the `Placement`
(a collection of `Position`, `Area`, and `RenderLayer`) is defined.
Statements can use other `Leaf`s to base their placement on, or use a
templated `Grid` attached to a `Leaf` to get `column`s and `row`s.

```rust
GridLocation::new()
    // ...
    .right_at(Layout::LANDSCAPE_MOBILE, screen().x() + "footer".width())
    .left_at(Layout::PORTRAIT_MOBILE, 3.column().of("header"))
    .top_at(Layout::PORTRAIT_MOBILE, 2.row().of("header"))
```
A `Layout` can be specified to use that statement when the screen is at that size.
A main `Grid` is associated with the screen which reconfigures at breakpoints.
```rust
Layout::SQUARE => Grid::template(4, 4) // 4 columns, 4 rows
Layout::PORTRAIT_MOBILE => Grid::template(4, 8) // 4 columns, 8 rows
Layout::LANDSCAPE_MOBILE => Grid::template(8, 4)
Layout::TALL_DESKTOP => Grid::template(8, 12)
...
```

A `Twig` can be added to create compound-elements.
```rust
Twig::new(Button::new(...), |l| { 
    l.give(...); // extend base button if needed
});
```

Where the `Button` defines

```rust
pub trait TwigDef
where
    Self: Sized + Send + Sync + 'static,
{
    fn grow(self, twig_ptr: &mut TwigPtr);
}
```

A `TwigPtr` is a handle to the `Tree` and the root-entity of the `Twig`.
Here you can `bind` `Leaf`s to the `Twig` and configure logic.

```rust
twig_ptr.config_grid(Grid::template(3, 2));
twig_ptr.bind(Leaf::new(|l| { ... }).elevation(-1));
// ...
```

An `Elevation` of `-1` is used to place the bound `Leaf` up one `RenderLayer` from the
`Twig` entity. Each bound `Leaf` is automatically made to `stem_from` the `Twig` entity.

Examples of `Leaf` components that are included by the library are

### Panel

A `Panel` is a `square`-`circle` that can round itself from `0` - `1` to achieve either
a `square` (`0`) or `circle` (`1`).

```rust
Leaf::new(|l| {
    l.give(Panel::new(Rounding::all(0.2), Color::WHITE));
})
```

### Icon

An `Icon` is a one-size `png` of `feathericons` at `24x24` pixels. It will scale
on devices with a `ScaleFactor` greater than `1.0` to appear `24x24` (cannot manually scale yet).

```rust
Leaf::new(|l| {
    l.give(Icon::new(IconId(0), Color::WHITE));
})
```

The `Icon` must be loaded into memory. Then it can be referenced many times using `IconId`.

```rust
foliage.load_icon(IconId(0), include_bytes!("path/to/assets/name.icon"));
```

### Image

### Text

### Shape

Once everything is to your liking, you can run the system with

```rust
// run main-loop
foliage.photosynthesize();
```