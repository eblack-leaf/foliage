# A first app

This chapter builds a counter: a button, and a label on it that says how many times it has been
pressed. It is small, but every part of the way a foliage app is put together is in it.

## Setting up

foliage is a library crate. Add it to a binary:

```toml
# Cargo.toml
[package]
name = "counter"
edition = "2024"

[dependencies]
foliage = { git = "https://github.com/eblack-leaf/foliage" }
```

It builds on a current stable toolchain. On Linux, winit and wgpu link against the windowing stack
at compile time, so the development headers have to be present. On Debian or Ubuntu that is:

```sh
sudo apt-get install libx11-dev libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev \
    libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

macOS and Windows need nothing beyond Rust itself.

## The whole program

```rust,no_run
use foliage::{
    Area, Boxed, Foliage, FontSize, Grove, Grow, Leaf, Location, Palette, Panel, Place, Pollen,
    Root, Rounding, Source, Text, center_x, center_y, content, left, top,
};

/// What the app keeps between frames. Nothing here is handed to the engine.
struct Counter {
    button: Leaf,
    label: Leaf,
    count: u32,
}

impl Root for Counter {
    /// Runs once, inside the first frame.
    fn take_root(grove: &mut Grove) -> Self {
        let page = grove.plant(Panel::new().color(Palette::Surface));
        let button = grove.branch(
            page,
            Panel::new()
                .color(Palette::Accent)
                .rounding(Rounding::Md)
                .interactive()
                .at(Location::new().xs(
                    left(40.px()).width(160.px()),
                    top(40.px()).height(48.px()),
                )),
        );
        let label = grove.branch(
            button,
            Text::new("count 0")
                .color(Palette::Accent.on())
                .font_size(FontSize::new().xs(14))
                .intangible()
                .at(Location::new().xs(
                    center_x(50.pct()).width(content()),
                    center_y(50.pct()).height(content()),
                )),
        );
        Counter {
            button,
            label,
            count: 0,
        }
    }

    /// Runs once per frame: what happened, and what to do about it.
    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        if pollen.clicked(self.button) {
            self.count += 1;
            grove.text(self.label, format!("count {}", self.count));
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

Run it with `cargo run`. A window opens with an accent-coloured button, and each press counts
up. The rest of this chapter reads it from the bottom.

## `main`: the engine

`Foliage` is the engine. `Foliage::new()` builds it without touching a window or a device; every
platform resource is acquired later, when the platform hands one over. That is also why the same
type can be built inside a test.

Before it runs it is told three things about the window and one about the app:

- `title` is the prose a window is labelled with.
- `app_id` is how the desktop recognises the application, as against how it labels the window. A
  shell matches it against the entry that launched the program, and without it some desktops draw a
  second, generic icon beside the one that was clicked. Conventionally it is the binary's name.
- `desktop_size` is the size the window opens at, in logical pixels. The web and Android take
  their size from the surface they are given and ignore it.
- `root::<Counter>()` names the app. The engine does not construct it yet; it calls
  `Counter::take_root` inside the first frame.

`photosynthesize()` runs the platform's event loop and does not return. It consumes the engine,
because from here on the loop belongs to the platform and everything that happens is a frame.

## `Root`: the app

An app is a value implementing [`Root`](https://eblack-leaf.github.io/foliage/api/foliage/trait.Root.html).
It has two methods:

- `take_root` runs once, inside the first frame. It grows the starting tree and returns the app's
  own value.
- `frame` runs once per frame after that, and on the first frame too, straight after `take_root`.

Whatever the app keeps (here, two names and a count) is its own. The engine has no way to reach
it. The value is lent a [`Grove`](https://eblack-leaf.github.io/foliage/api/foliage/struct.Grove.html)
for the length of each call, and the grove is the whole of what an app can touch.

## `plant` and `branch`: growing elements

```rust,ignore
let page = grove.plant(Panel::new().color(Palette::Surface));
let button = grove.branch(page, Panel::new() /* ... */);
```

A `Panel::new()` is a *seed*: an element described before it exists. `plant` grows one at the top
level and `branch` grows one under another element, and both hand back a
[`Leaf`](https://eblack-leaf.github.io/foliage/api/foliage/struct.Leaf.html): the name of the new
element.

The name is usable the moment it is handed back, which is why `button` can be branched off `page`
on the very next line. The element itself does not exist yet. `plant` and `branch` queue an op,
and the queue is drained once the frame's app code has returned. [The next
chapter](tree.md) says more about names and what they are good for.

The page is a panel with no placement, so it fills its parent. It has no parent, so it fills the
window. It is filled with `Palette::Surface`, the ordinary ground.

## `Location`: where it sits

```rust,ignore
.at(Location::new().xs(
    left(40.px()).width(160.px()),
    top(40.px()).height(48.px()),
))
```

A location states one horizontal and one vertical placement. Each is a *role* (`left`, `width`,
`top`, `height`) filled with a *source* (`40.px()`). The horizontal half comes first and the
vertical half second, and the two are different types, so they cannot be swapped.

`xs` is the smallest breakpoint. A location can say something different at wider breakpoints, but
`xs` alone is enough: every wider one falls back to it. [Placing elements](placement.md) covers the
grammar properly.

The label is placed against its trunk, the button:

```rust,ignore
.at(Location::new().xs(
    center_x(50.pct()).width(content()),
    center_y(50.pct()).height(content()),
))
```

`50.pct()` is half of the button's width, measured from the button's left edge, so the label is
centred on it. `content()` is the label's own measured size: as wide as its text, and as tall as
the text turned out.

## `interactive` and `intangible`: who takes the press

A press goes to whatever is on top at that point. The label is on top of the button (it was grown
under it, so it sits in front of it), which means that without anything said, a press on the
label's pixels would land on the label.

Two declarations settle it:

- `interactive()` on the button says it **receives** gestures. It is what `clicked` can report,
  and what focus can rest on.
- `intangible()` on the label says a gesture over it **passes through** to whatever is beneath.

An element on top that says neither **eats** the press: nothing receives it, and nothing behind it
does either. That is the right answer for a backdrop or a sheet, and the wrong one for a label
drawn over a button. Try deleting `.intangible()` and pressing the middle of the button: only the
rim still counts.

## `frame`: answering

```rust,ignore
fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
    if pollen.clicked(self.button) {
        self.count += 1;
        grove.text(self.label, format!("count {}", self.count));
    }
}
```

[`Pollen`](https://eblack-leaf.github.io/foliage/api/foliage/struct.Pollen.html) is what the frame
reported. It is a set to ask questions of, not a list to walk: `clicked(leaf)` answers whether that
element was tapped, meaning a gesture began on it and ended without turning into a drag.

`grove.text(label, …)` rewrites what the label says. Like `plant`, it is queued and drained after
`frame` returns, so reading the label back inside the same call still gives the old value. The
label's width is `content()`, so it re-measures itself to the new string in the same frame and
stays centred on the button.

## Imports

The imports are part of the lesson:

| Import | Is | Carries |
|---|---|---|
| `Grow` | a trait | every write: `plant`, `branch`, `text`, `color` and the rest |
| `Place` | a trait | the declarations every seed takes: `interactive`, `intangible`, `font_size` and the rest |
| `Boxed` | a trait | `at`, on every seed placed by a box |
| `Source` | a trait | `px`, `pct`, `col`, `row`, `letters` on plain numbers |

All four are sealed: they can be called and never implemented. A missing one shows up as "no
method named `plant`", or `px`, or `at`, and the fix is always the import.

## What happens when the button is pressed

The whole path, in the order the engine runs it:

1. The platform reports the press and the release. They are queued as input.
2. The next frame's **dispatch** hit-tests the press against what was drawn last frame and finds
   the button. When the release arrives before the gesture has become a drag or a hold, it records
   a tap.
3. `frame` is called with a `Pollen` in which `clicked(button)` is true. It queues a `text` write.
4. The **drain** applies the write.
5. **Resolution** measures the new string and re-centres the label.
6. **Extraction** notices that the label's glyphs changed and sends only those to the GPU.
7. The frame is drawn.

All of that is one frame. There is no render call to make, and nothing to invalidate: an app
states what is true and the next frame is already different. [The frame](../architecture/frame.md)
walks the same path from the inside.

## The engine idles

Nothing happens between presses. The engine runs a frame only when one is owed: input arrived, an
op is waiting, an animation is running, a report has not been delivered yet. Otherwise the loop
sleeps. An app that drives something itself from the clock, where the engine has nothing to
notice, asks for the next frame with
[`grove.again()`](https://eblack-leaf.github.io/foliage/api/foliage/struct.Grove.html#method.again)
for as long as it is doing it.
