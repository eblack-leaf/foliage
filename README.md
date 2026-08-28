# Foliage

`Foliage` is a cross-platform UI framework written in Rust. Every element on screen is one
entity, held by an ECS ([`bevy_ecs`](https://crates.io/crates/bevy_ecs)) that stays an
implementation detail -- nothing in the public API is a bevy type. Rendering is
[`wgpu`](https://wgpu.rs) and windowing/input is [`winit`](https://docs.rs/winit). It targets
Linux, Windows, and macOS natively, the Web via WebAssembly, and Android -- see
[Platform support](#platform-support) below.

## Getting started

```toml
# Cargo.toml
[dependencies]
foliage = { git = "https://github.com/eblack-leaf/foliage" }
```

```rust
// src/main.rs
use foliage::{
    Forest, Color, Elevation, Foliage, GridExt, Grows, Location, Panel, Rounding, Sprout,
};
struct App;
impl Root for App {
  fn take_root(forest: &mut Forest) -> Self {
    // ... oneshot to create app structure
  }
  fn frame(&mut self, forest: &mut Forest, mosses: Vec<Moss>) {
    // ... per frame | events from mosses
  }
}
fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((300, 200));
    foliage.root::<App>();
    foliage.photosynthesize();
}
```

Runnable examples live in [`foliage/examples`](foliage/examples) --
`cargo run --example basic_shapes -p foliage` is a good first one, and `interaction`,
`responsive_split`, `polygon_animation`, `scrolling`, `text_input`, `keyboard`, `polyline`,
`opacity_and_elevation` and `off_thread` cover the rest of the surface. All of them run with
no feature flags.

## Composing and reacting

Elements nest via `forest.branch(parent, ..)`. Everything foliage reports back -- clicks,
keys, finished animations, resizes -- arrives as a `Moss` from `forest.take()`:

```rust
use foliage::{
    Moss, Forest, Color, Elevation, Foliage, Grid, GridExt, Grows, Leaf, Location, Panel,
    Rounding, Sprout, Root
};
struct App { overlay: Leaf }
impl Root for App {
  fn take_root(forest: &mut Forest) -> Self {
    let base = forest.leaf(
      Panel::new()
              .color(Color::orange(700))
              .at(Location::new().xs(
                20.px().as_left().with(140.px().as_width()),
                20.px().as_top().with(140.px().as_height()),
              ))
              .elevate(Elevation::up(1))
              .grid(Grid::default()), // children resolve their Location against this
    );
    let overlay = forest.branch(
      base, // stemmed to `base` -- moves, clips and withers with it
      Panel::new()
              .color(Color::green(500))
              .rounding(Rounding::Sm)
              .at(Location::new().xs(
                50.px().as_left().with(60.px().as_width()),
                50.px().as_top().with(60.px().as_height()),
              ))
              .elevate(Elevation::up(1)) // one layer in front of its stem
              .interactive(), // what puts it in the hit test at all
    );
    App { overlay }
  }
  fn frame(&mut self, forest: &mut Forest, mosses: Vec<Moss>) {
    for b in mosses {
      if let Moss::Clicked(leaf) = b {
        if leaf == self.overlay {
          // ... process clicked
        }
      }
    }
  }
}
fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((200, 200));
    foliage.root::<App>();
    foliage.photosynthesize();
}

```

`Elevation::up(n)`/`abs(n)` set draw order relative to a stem or absolute. After the initial
growth there are only verbs, and they all take the name first: `color`, `opacity`, `location`,
`text`, `visible`, `enable`, `disable`, `animate`, `prune`. A write *is* the change -- there is
no render call to make, and the next frame is already different.

A `Leaf` is safe to hold onto. Prune the element and the name withers: later commands naming it
are dropped, samples read absent, and a name is never reused, so nothing panics and nothing is
silently addressed.

## Under the hood

None of the above touches rendering directly -- every element's changed state is tracked
by a `Differential` cache and only what actually changed is drained into the `ash`
backend each frame, which feeds `wgpu` through `ginkgo`.

The line that matters is which types have a renderer and which are logical on top of one. Six
have a renderer -- `Panel`, `Text`, `Icon`, `Image`, `Polygon`, and `LineQuad`, which `Line`
fronts -- and each owns a wgpu pipeline and an instance buffer fed by that cache. `Text` is one
of the six in the sense that it owns a pipeline, not in the sense that it is simple: a string is
however many glyph instances it takes to set it.

Everything else is assembled from those six and draws nothing itself. `Polyline` is `Line`
segments with `Polygon` joints closing the wedges at each bend. `TextInput` is a panel, text, and
a caret. The [site](https://eblack-leaf.github.io/foliage/) in [`application/`](application) is
the largest worked example -- its cards, buttons, rail and figures are all assembly.

The [book](https://eblack-leaf.github.io/foliage/book/) covers all of this in depth, building
each piece up from nothing in the same order.

## Platform support

| Platform | Status |
|---|---|
| Linux / Windows / macOS | Native, built and run in CI on all three |
| Web (WASM) | Live -- this is the crate's actual deployed target, not just a CI build check |
| Android | Wired up end-to-end -- `Foliage::android(app)`, a cdylib entry point crate (`application_android`), and a Gradle-project scaffolding CLI (`foliage_android`); not yet in CI |
| iOS | No toolchain currently available to compile or verify against; the shared source has no iOS-specific branch behind it yet, so this is unverified rather than unsupported |

## Known gaps

- **Text input selection**: Shift+Click doesn't yet extend an existing selection (every click
  restarts it), and drag-selecting doesn't auto-scroll when the pointer nears the box's edges.
  Both are scoped but not implemented.
- **Widgets live above the framework, not in it.** foliage stays unopinionated: it gives you the
  renderers, layout, motion and input, and a button or a card is assembly on top. `lichen` is
  where that opinionated layer is going; it is not ready yet.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Bundled fonts

foliage embeds **JetBrains Mono** as its default font, so a binary built with it redistributes
that font — see [LICENSE-JETBRAINS-MONO](LICENSE-JETBRAINS-MONO) (SIL OFL 1.1), which its terms
require to accompany the binary. Registering your own font with `Foliage::font` does not remove
it; the bundled face remains the default.

**DejaVu Sans** and **DejaVu Sans Mono** ([LICENSE-DEJAVU](LICENSE-DEJAVU)) are used only by the
repo's own examples and fixtures, and are never compiled into a dependent's binary.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion
in this work by you shall be dual licensed as above, without any additional terms or
conditions.
