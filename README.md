# Foliage

`Foliage` is a cross-platform UI framework written in Rust. Every widget is one ECS entity
(via [`bevy_ecs`](https://crates.io/crates/bevy_ecs)); rendering is [`wgpu`](https://wgpu.rs)
and windowing/input is [`winit`](https://docs.rs/winit). It targets Linux, Windows, and macOS
natively, the Web via WebAssembly, and Android (in progress) -- see [Platform support](#platform-support)
below.

## Getting started

```toml
# Cargo.toml
[dependencies]
foliage = { git = "https://github.com/eblack-leaf/foliage" }
```

```rust
// src/main.rs
use foliage::{Button, Color, EcsExtension, Elevation, Foliage, Location, Rounding, Sprout};

fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((300, 200));
    foliage.world.leaf(
        Button::new()
            .text("Button")
            .rounding(Rounding::Sm)
            .colors(Color::gray(900), Color::green(500))
            .at(Location::new().xs(
                8.px().as_left().with(160.px().as_width()),
                8.px().as_top().with(52.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );
    foliage.photosynthesize(); // hands off to the window/event loop and runs
}
```

`foliage.world.leaf(..)` spawns a widget at the top level (no parent). `Button::new()`
returns a `ButtonSprout` -- a config builder, not yet an entity -- and `.at(..)`/
`.elevate(..)`/`.text(..)`/`.colors(..)` all just set fields on it. It only becomes a
real, spawned entity once handed to `leaf`/`branch`. More runnable examples (composing
several widgets, click handling, animation, text input) live in
[`foliage/examples`](foliage/examples) -- `cargo run --example controls -p foliage` is a
good first one to try.

## Composing and reacting

Widgets nest via `tree.branch(parent, ..)`, and clicks are handled with `on_click`:

```rust
use foliage::{
    Color, EcsExtension, Elevation, Foliage, Grid, GridExt, Location, OnClick, Opacity, Panel,
    Sprout, Tree, Trigger,
};

let mut foliage = Foliage::new();
let base = foliage.world.leaf(
    Panel::new()
        .color(Color::orange(700))
        .at(Location::new().xs(20.px().as_left().with(140.px().as_width()), 20.px().as_top().with(140.px().as_height())))
        .elevate(Elevation::abs(0))
        .with((Opacity::new(1.0), Grid::default())), // children resolve their Location against this
);
let overlay = foliage.world.branch(
    base, // parented to `base` -- moves and is clipped with it
    Panel::new()
        .color(Color::green(500))
        .at(Location::new().xs(50.px().as_left().with(60.px().as_width()), 50.px().as_top().with(60.px().as_height())))
        .elevate(Elevation::up(1)) // one layer in front of its parent
        .with(Opacity::new(0.6)),
);
foliage.on_click(overlay, |_: Trigger<OnClick>, mut tree: Tree| {
    tree.write_to(overlay, Color::gray(200)); // any later write re-triggers whatever reacts to Color
});
```

`Elevation::up(n)`/`abs(n)` set draw order relative to a parent or absolute; `.with(..)`
folds extra components (here, `Opacity`) into the same one-shot spawn. Writing to an
entity's components later (`tree.write_to`) is how you update anything after the fact --
composites like `Button` restyle themselves by reacting to exactly that kind of write.

## Under the hood

None of the above touches rendering directly -- every widget's changed state is tracked
by a `Differential` cache and only what actually changed is drained into the `ash`
backend each frame, which feeds `wgpu` through `ginkgo`. Rendering primitives (`Panel`,
`Text`, `Icon`, `Image`, `Polygon`, `Line`) are built on exactly that machinery, and
composites like `Button` are combinations of several primitives under one root entity.

The [book](https://eblack-leaf.github.io/foliage/book/) covers all of this in depth,
building each piece up from nothing in the same order, and ends by building `Button`
itself from scratch using only what came before it.

## Platform support

| Platform | Status |
|---|---|
| Linux / Windows / macOS | Native, CI-tested (`cargo test --workspace` on all three) |
| Web (WASM) | Live -- this is the crate's actual deployed target, not just a CI build check |
| Android | Code paths exist (`cfg(target_os = "android")`, winit's android-game-activity feature) but no app project is wired up end-to-end yet; not yet in CI |
| iOS | No toolchain currently available to compile or verify against; the shared source has no iOS-specific branch behind it yet, so this is unverified rather than unsupported |

## Known gaps

- **Text input selection**: Shift+Click doesn't yet extend an existing selection (every click
  restarts it), and drag-selecting doesn't auto-scroll when the pointer nears the box's edges.
  Both are scoped but not implemented.
- **Router has no URL/browser-history integration, on purpose.** This was designed through and
  deliberately rejected, not left undone: a deep link would let a route become a visitor's
  *first* scene rather than one reached through the app's own authored navigation, which can
  silently break anything that assumed it was entered in-session. Router only supports
  in-session, app-authored navigation.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion
in this work by you shall be dual licensed as above, without any additional terms or
conditions.
