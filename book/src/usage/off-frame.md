# Work off the frame

Everything the engine does happens inside a frame, on the thread the frame runs on. Some work
belongs somewhere else: a search that takes a second, a picture decoded on a worker, a callback
from the host. A [`Sprig`](https://eblack-leaf.github.io/foliage/api/foliage/struct.Sprig.html) is
the engine reached from there. This chapter streams results into a list from a worker thread, lets
the worker hear a cancel button, and hands a picture over from off the frame.

## A handle that carries every write

```rust
# use foliage::Grove;
# fn f(grove: &mut Grove) {
let sprig = grove.sprig();
# let _ = sprig;
# }
```

A `Sprig` implements [`Grow`](https://eblack-leaf.github.io/foliage/api/foliage/trait.Grow.html)
exactly as the `Grove` does, so a change reads the same wherever it is written: `plant`, `branch`,
`at`, `animate`, `prune`, all of it. It is cheap to clone, and on every platform with threads it is
`Send`. Every clone, and every call to `grove.sprig()`, is the same handle.

What differs is only *when* an op lands. The frame and every sprig push onto one queue, and the
queue is drained once a frame. An op from a sprig lands in the drain of the frame that was running
when it arrived, if it arrived before that frame's drain, and in the next frame's otherwise. Nothing
about how it is applied depends on which side it came from. If the loop was asleep, pushing the op
wakes it.

Names work the same way. `sprig.branch(..)` hands back a `Leaf` at once, from the same allocator
the frame uses, so a worker can grow an element and branch another off it on the next line without
waiting for anything. A name taken on a worker takes its place in the order names were asked for,
which is what settles which of two level elements is in front, exactly as a name taken in `frame`
does.

A leaf a worker holds may wither before the worker writes to it. That is safe on the same terms it
is safe anywhere: an op naming a withered element is dropped.

## Streaming into a list

```rust,no_run
use std::thread;
use std::time::Duration;

use foliage::{
    Area, Axes, Boxed, Foliage, FontSize, Grove, Grow, Leaf, Location, Panel, Place, Pollen, Root,
    Source, Sprig, Stem, Text, anchor, content, left, top,
};

struct App;

impl Root for App {
    fn take_root(grove: &mut Grove) -> Self {
        let page = grove.plant(Panel::new());
        let column = grove.branch(page, Stem::new().scrolls(Axes::Vertical));
        let sprig = grove.sprig();
        thread::spawn(move || stream(sprig, column));
        App
    }

    fn frame(&mut self, _grove: &mut Grove, _pollen: Pollen) {}
}

/// On its own thread: a result every quarter of a second, each placed under the last.
fn stream(mut sprig: Sprig, column: Leaf) {
    let mut last: Option<Leaf> = None;
    for n in 1..=40 {
        thread::sleep(Duration::from_millis(250));
        let row = Text::new(format!("result {n}")).font_size(FontSize::new().xs(14));
        let row = match last {
            Some(last) => row.anchored(last).at(Location::new().xs(
                left(16.px()).width(content()),
                top(anchor().bottom() + 6.px()).height(content()),
            )),
            None => row.at(Location::new().xs(
                left(16.px()).width(content()),
                top(16.px()).height(content()),
            )),
        };
        last = Some(sprig.branch(column, row));
    }
}

fn main() {
    let mut foliage = Foliage::new();
    foliage.title("stream");
    foliage.desktop_size(Area::new(320.0, 400.0));
    foliage.root::<App>();
    foliage.photosynthesize();
}
```

The app's `frame` does nothing at all. The worker grows forty rows over ten seconds, each anchored
to the one before, and each lands in the frame after it was written. The engine sleeps between
them.

On the web there are no threads to spawn, and a `Sprig` is not `Send` there. The same handle is
used from a promise or a callback instead (anything `wasm_bindgen_futures::spawn_local` runs), and
the ops land the same way.

## Reading from off the frame

A worker cannot *sample* the tree: a read needs the world, and the world belongs to the frame. So
the reads a worker needs are pushed to it instead, three ways.

**Conditions** are what is true of the whole tree, published at the end of every frame:

```rust
# use foliage::{Layout, Sprig};
# fn f(sprig: &Sprig) {
if let Some(conditions) = sprig.conditions() {
    let narrow = conditions.layout <= Layout::Sm;
    let _ = (narrow, conditions.viewport, conditions.scheme, conditions.focused);
}
# }
```

They are `None` until the first frame has run, and the fields always come from one frame, so the
viewport and the breakpoint in hand agree with each other.

**Watches** are one property of one element, pushed whenever it changes:

```rust
# use foliage::{Leaf, Sap, Sprig, Vein};
# fn f(sprig: &mut Sprig, column: Leaf) {
sprig.watch(column, Vein::Extent);
// Later, from the same thread or any other holding the handle:
if let Some(Sap::Area(extent)) = sprig.tap(column, Vein::Extent) {
    let _ = extent.height;
}
// And when it is no longer wanted:
sprig.unwatch(column, Vein::Extent);
# }
```

A watch is an op like any other, so it is drained in order: watching an element in the same breath
as growing it works for the reason writing to it does. Its first reading is taken at the end of
the frame it is drained in, with the value as it already stands. `tap` answers `None` until then,
and once the watch has ended or the element has withered. Watching the same property twice is one
watch.

**Reports** are the same `Pollen` the app's `frame` is handed, one per frame:

```rust
# use foliage::{Leaf, Sprig};
# fn f(sprig: &Sprig, cancel: Leaf) -> bool {
// The first call arms delivery, and answers with nothing.
let _ = sprig.pollen();
// Every call after answers with every frame's report since the last one, oldest first.
let cancelled = sprig.pollen().iter().any(|pollen| pollen.clicked(cancel));
# cancelled
# }
```

Nothing is collected until the first call, so a handle that only ever writes never fills an inbox
nobody reads. After that, a worker hears about a press on an element it grew without the app having
to relay it. There is one inbox per handle, shared by every clone, so two workers holding clones
read a single stream between them.

A reading is never older than the last report, and may be one frame newer, which is the ordinary
condition of reading a running engine from beside it.

## Pictures and fonts from a worker

A worker can register assets as well as write. The names come from the same allocator, and the
bytes arrive as the op a file read would have pushed:

```rust
# use foliage::{Area, Boxed, Grove, Grow, Image, Leaf, Location, Source, left, top};
# fn render() -> Vec<u8> { vec![255; 64 * 64 * 4] }
# fn f(grove: &mut Grove, page: Leaf) {
// A name now, with nothing behind it yet.
let plate = grove.plate();
grove.branch(
    page,
    Image::new(plate).at(Location::new().xs(left(16.px()).width(64.px()), top(16.px()).height(64.px()))),
);
// The pixels, whenever they are ready. The image appears on that frame.
let mut sprig = grove.sprig();
std::thread::spawn(move || {
    let rgba = render();
    sprig.load(plate, rgba, Area::new(64.0, 64.0));
});
# }
```

`sprig.font`, `sprig.icon` and `sprig.image` are the grove's registrations for a thread. One
difference is deliberate: **none of the three panics.** A face that turns out to be proportional,
or a field smaller than it was said to be, is reported `missing`, because a worker holds bytes it
built or was sent rather than bytes the program stated, and a panic on a thread ends the thread
rather than naming a callsite.

`load` and `pixels` are different, because they are `Grow` verbs rather than registrations: they
queue the pixels as an op, and the drain applies it. Pixels shorter than the size they were given
with still panic, and the panic is in the drain, on the frame's thread. A worker that builds pixels
should get the size right, as the `render` above does by construction.

## What not to do from a worker

A sprig is a way to reach the tree, not a second frame. Two things follow.

- **Do not drive per-frame motion from a thread.** A thread's clock is not the frame's, and an op
  written every few milliseconds lands in whichever frame it lands in. Motion belongs to `animate`
  and `tween`, which run on the frame's one clock.
- **Do not poll the tree.** There is nothing to poll: a thread reads what was pushed to it, and a
  watch pushes only when the value moves.
