# Testing an app

An app's [`Root`] does not know whether a window is around it. The `Greenhouse` grows the same
`Root` with no window: input is written in, time is moved by hand, frames run when the test asks
for one, and a frame can be drawn to a picture and read back. It is in foliage itself, so an app's
own tests need nothing else.

```rust
use foliage::{
    Area, Boxed, Greenhouse, Grove, Grow, Leaf, Location, Palette, Panel, Place, Pollen, Position,
    Root, Source, left, top,
};

struct Counter {
    button: Leaf,
    count: u32,
}

impl Root for Counter {
    fn take_root(grove: &mut Grove) -> Self {
        let button = grove.plant(
            Panel::new()
                .color(Palette::Accent)
                .interactive()
                .at(Location::new().xs(left(20.px()).width(80.px()), top(20.px()).height(40.px()))),
        );
        Counter { button, count: 0 }
    }

    fn frame(&mut self, _grove: &mut Grove, pollen: Pollen) {
        if pollen.clicked(self.button) {
            self.count += 1;
        }
    }
}

#[test]
fn a_click_counts() {
    let mut house = Greenhouse::<Counter>::new(Area::new(200.0, 100.0));
    house.frame(); // take_root runs here, as it does in a window's first frame
    house.tap(Position::new(60.0, 40.0));
    house.settle(); // frames until nothing more is owed
    assert_eq!(house.root().count, 1);
}
```

## Nothing happens between two lines

Every input method queues what a window would have queued, and returns. Nothing is taken in
until a frame runs, so a test reads the way the loop runs:

| Call | What it is |
|---|---|
| `frame()` | one frame, with the app's turn in it. Hands back the `Pollen` the app was given |
| `advance(ms)` | time passes, to be taken up by the next frame. Exactly: five seconds is five seconds, with none of the cap a window puts on a long gap |
| `step(ms)` | `advance`, then `frame` |
| `settle()` | frames 16ms apart until nothing more is owed: every motion landed, every report delivered. Frames asked for with `again_in` are not waited on |
| `press`, `release`, `tap`, `point`, `cancel`, `leave` | the pointer. `point` with nothing held is where the cursor rests, which is what is hovered |
| `wheel(at, delta)` | a scroll, as the distance a drag would cover |
| `key`, `chord(modifiers, key)`, `type_text` | the keyboard |
| `resize(area)` | the surface, at the next frame |

A report lands a frame after what caused it, as it does in a window: a tap is taken in by one
frame and handed to the app in the next. `settle` covers that; with single frames, run one more.

## Reading what happened

- `root()` / `root_mut()`: the app itself, to read or change between frames.
- `grove()`: what the last frame settled. `tap(leaf, Vein::Drawn)` for where an element ended
  up, `focused()`, `hovered()`.
- `grove_mut()`: verbs, written between frames, landing in the next frame's drain.
- `pollen()`: what the last frame handed the app.

## A picture of a frame

`render()` draws what the last frame settled and hands back a `Picture`: RGBA, at one device
pixel per logical pixel, so a point in the picture is the point the tree was laid out in.

```rust,ignore
let Some(picture) = house.render() else {
    return; // no GPU and no software rasteriser: nothing to draw with
};
let at = picture.at(Position::new(60.0, 40.0)).unwrap();
assert!(at.red > 0.5);
std::fs::write("counter.png", picture.png()).unwrap();
```

It uses the same renderers and the same pass a window uses. The difference is the target: a
texture that is read back instead of presented. So the picture is what the app shows. The first
call acquires a device, and every frame after that is handed to it as it runs. `None` means the
machine has no adapter at all, so check for it rather than failing.
