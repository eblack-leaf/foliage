# foliage

A cross-platform UI library for Rust. One element on screen is one entity, held by an ECS
([`bevy_ecs`](https://crates.io/crates/bevy_ecs)) that stays an implementation detail — no engine
type is ever handed out, and nothing an app holds borrows from the world. Rendering is
[`wgpu`](https://wgpu.rs) and windowing and input are [`winit`](https://docs.rs/winit).

Everything crossing the seam between an app and the engine is plain data: ops in, emissions out,
and read-only taps taken at the app's own callsite. An app declares what an element *is* — where
it sits, what fills it, whether it takes a gesture — and then only writes verbs against the name
it got back. There is no render call to make; the next frame is already different.

## Getting started

```toml
# Cargo.toml
[dependencies]
foliage = { git = "https://github.com/eblack-leaf/foliage" }
```

```rust
// src/main.rs
use foliage::{
    Area, Boxed, Foliage, Grove, Grow, Leaf, Location, Palette, Panel, Place, Pollen, Root,
    Rounding, Source, left, top,
};

/// What the app keeps between frames. Nothing here is handed to the engine.
struct Counter {
    button: Leaf,
    pressed: bool,
}

impl Root for Counter {
    /// Runs once, inside the first frame. A `Leaf` it hands back is usable immediately.
    fn take_root(grove: &mut Grove) -> Self {
        let page = grove.plant(Panel::new().color(Palette::Surface));
        let button = grove.branch(
            page,
            Panel::new()
                .color(Palette::Accent)
                .rounding(Rounding::Md)
                // What puts it in the hit test at all.
                .interactive()
                .at(Location::new().xs(
                    left(40.px()).width(120.px()),
                    top(40.px()).height(48.px()),
                )),
        );
        Counter {
            button,
            pressed: false,
        }
    }

    /// Runs once per frame: what happened, and what to do about it.
    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        if pollen.clicked(self.button) {
            self.pressed = !self.pressed;
            let fill = match self.pressed {
                true => Palette::Accent.recede(),
                false => Palette::Accent,
            };
            grove.color(self.button, fill);
        }
    }
}

fn main() {
    let mut foliage = Foliage::new();
    foliage.title("counter");
    foliage.app_id("counter");
    foliage.desktop_size(Area::new(320.0, 240.0));
    foliage.root::<Counter>();
    foliage.photosynthesize();
}
```

The crate root of the [API reference](https://eblack-leaf.github.io/foliage/api/foliage/) is the
map of the whole surface: what every type is for, and which verb writes it.

## Examples

Runnable, each doing one thing, in [`foliage/examples`](foliage/examples). All of them run with
no feature flags:

```sh
cargo run -p foliage --example palette
cargo run -p foliage --example animate
cargo run -p foliage --example polyline
```

## Icons

A mark has no size of its own — the same artwork is a 16px affordance and a 96px empty state — so
an icon is not a bitmap but a distance field, reconstructed sharp at whatever box a layout hands
it. [`foliage-icons`](foliage-icons) bakes one from an SVG:

```sh
cargo install --git https://github.com/eblack-leaf/foliage foliage-icons
foliage-icons bake --svg assets/svg --out src/icons --marks Icons
```

That writes one `.icon` per source file and a module beside them that registers the set. Both are
committed. The module is generated, so nothing in it is written by hand:

```rust
pub struct Icons {
    /// The `arrow-up` mark.
    pub arrow_up: Field,
    /// The `check` mark.
    pub check: Field,
}

impl Marks for Icons {
    fn register(grove: &mut Grove) -> Self { /* ... */ }
}
```

An app registers the set by naming it, the same way it names its [`Root`], and then reaches a mark
by the name it was given:

```rust
let icons = grove.marks::<Icons>();
grove.branch(bar, Icon::new(icons.check).color(Palette::Accent).at(/* ... */));
```

Adding or removing a mark and regenerating moves no callsite, because nothing addresses a mark by
its position. `foliage-icons preview` renders a baked field to a PNG, sampling exactly as the
shader does, to judge one without running an application.

## What draws, and what is assembled

Six elements own a render pipeline and an instance buffer: `Panel`, `Text`, `Icon`, `Image`,
`Polygon` and `Line`. Everything else is assembly on top of those — `TextInput` is a panel, a run
of glyphs and a caret — and draws nothing of its own. Only the state that actually changed is
drained into the backend each frame.

foliage stays unopinionated about widgets. It gives the renderers, the layout, the motion and the
input; a button or a card is yours to assemble.

## Platform support

| Platform | Status |
|---|---|
| Linux, Windows, macOS | Supported. Built and tested on all three in CI. |
| Web (WebAssembly) | Supported. Built for `wasm32-unknown-unknown` in CI. |
| Android | Planned. `foliage-android` is the entry point and the Gradle scaffolding that goes with it; the engine's own Android arms — surface acquisition, device limits — are written. |
| iOS | Untested. The shared source carries iOS arms where a platform decision is forced, and there is no toolchain here to verify against, so it is unverified rather than unsupported. |

## Repo layout

| Path | What it is |
|---|---|
| [`foliage/`](foliage) | The library. |
| [`foliage-icons/`](foliage-icons) | The icon baker: SVG in, the field a mark is drawn from out. |
| [`application/`](application) | A page written against nothing but foliage's public surface, and what `cargo xtask site` builds. `cargo check -p application` is a gate on the API rather than on this crate: an API that cannot build a page is an incomplete API. |
| [`book/`](book) | The book. |
| [`xtask/`](xtask) | Repo tasks. Not part of the library and never published. |

## Repo tasks

`cargo xtask <command>`, via the alias in `.cargo/config.toml`:

| Command | What it does | Needs |
|---|---|---|
| `site` | Builds the site into `docs/`. | [`trunk`](https://trunkrs.dev) |
| `serve` | Serves the site locally with auto-reload. | `trunk` |
| `book` | Builds the book into `docs/book`. | [`mdbook`](https://crates.io/crates/mdbook) |
| `api` | Builds the API reference for `foliage` and `foliage-icons` into `docs/api`. | — |
| `docs` | The book and the API reference. | `mdbook` |
| `web` | Everything: the site, then the book, then the API reference. | `trunk`, `mdbook` |

`docs/` is what GitHub Pages serves, so its contents are committed. `site` clears it — every build
emits differently-named bundles, which would otherwise pile up forever — so `web` is what rebuilds
all three in the order that survives it.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
this work by you shall be dual licensed as above, without any additional terms or conditions.

### Bundled font

foliage embeds **JetBrains Mono** as its default font, so a binary built with it redistributes
that font. Its licence — SIL OFL 1.1, which requires that it accompany the binary — ships beside
the face at
[`foliage/src/text/LICENSE-JETBRAINS-MONO`](foliage/src/text/LICENSE-JETBRAINS-MONO). Registering
your own font with `Foliage::font` does not remove it; the bundled face remains the default.
