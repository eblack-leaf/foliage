# Growing the tree

Everything on screen is an element, and every element sits somewhere in one tree. This chapter
builds a list that rows can be added to and removed from, and uses it to cover how elements are
grown, named, written to, read, and taken down.

## Seeds

A seed describes an element before it exists. The set is closed:

| Seed | What it is | Draws |
|---|---|---|
| `Stem` | Structure: a box that holds children and takes hits | nothing |
| `Panel` | A filled rectangle with rounded corners | yes |
| `Text` | A run of monospaced glyphs | yes |
| `Icon` | A vector mark, from a registered distance field | yes |
| `Image` | A registered picture, fitted into its box | yes |
| `Polygon` | A regular polygon: sides, corner rounding, rotation | yes |
| `Line` | A stroke between two points | yes |
| `TextInput` | An editable run of one line, with a caret and a selection | assembled |
| `TextArea` | The same, with as many lines as its value wraps to | assembled |

The six that draw each own a render pipeline. The two fields are assembled from those six. A
`Stem` draws nothing at all, and is what to reach for whenever structure is wanted and pixels are
not: a column that holds rows, a region that scrolls, a box that other things are placed inside.

There are no buttons, cards or menus in the list. foliage gives the renderers, the layout, the
motion and the input. A button is a panel that receives presses with a label on it, and it is the
app's to assemble. [Composing parts](components.md) shows how to make that assembly reusable.

## A list with rows

```rust,no_run
use foliage::{
    Area, Boxed, Foliage, FontSize, Grove, Grow, Leaf, Location, Palette, Panel, Place, Pollen,
    Root, Rounding, Source, Text, center_x, center_y, content, left, right, top,
};

const ROW: f32 = 44.0;

struct Row {
    ground: Leaf,
    remove: Leaf,
}

struct List {
    page: Leaf,
    add: Leaf,
    rows: Vec<Row>,
    grown: u32,
}

/// Where row `n` stands: below the button, one row's height apart.
fn row_at(n: usize) -> Location {
    Location::new().xs(
        left(16.px()).right(100.pct() - 16.px()),
        top((72.0 + n as f32 * ROW).px()).height((ROW - 4.0).px()),
    )
}

impl List {
    fn grow_row(&mut self, grove: &mut Grove) {
        self.grown += 1;
        let ground = grove.branch(
            self.page,
            Panel::new()
                .color(Palette::Raised)
                .rounding(Rounding::Sm)
                .at(row_at(self.rows.len())),
        );
        grove.branch(
            ground,
            Text::new(format!("row {}", self.grown))
                .font_size(FontSize::new().xs(14))
                .at(Location::new().xs(
                    left(12.px()).width(content()),
                    center_y(50.pct()).height(content()),
                )),
        );
        let remove = grove.branch(
            ground,
            Panel::new()
                .color(Palette::Danger)
                .rounding(Rounding::Full)
                .interactive()
                .at(Location::new().xs(
                    right(100.pct() - 8.px()).width(24.px()),
                    center_y(50.pct()).height(24.px()),
                )),
        );
        self.rows.push(Row { ground, remove });
    }
}

impl Root for List {
    fn take_root(grove: &mut Grove) -> Self {
        let page = grove.plant(Panel::new());
        let add = grove.branch(
            page,
            Panel::new()
                .color(Palette::Accent)
                .rounding(Rounding::Sm)
                .interactive()
                .at(Location::new().xs(
                    left(16.px()).width(120.px()),
                    top(16.px()).height(40.px()),
                )),
        );
        grove.branch(
            add,
            Text::new("add a row")
                .color(Palette::Accent.on())
                .font_size(FontSize::new().xs(14))
                .intangible()
                .at(Location::new().xs(
                    center_x(50.pct()).width(content()),
                    center_y(50.pct()).height(content()),
                )),
        );
        List {
            page,
            add,
            rows: Vec::new(),
            grown: 0,
        }
    }

    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        if pollen.clicked(self.add) {
            self.grow_row(grove);
        }
        if let Some(n) = self.rows.iter().position(|row| pollen.clicked(row.remove)) {
            let row = self.rows.remove(n);
            grove.prune(row.ground);
            // Everything below it moves up a place.
            for (at, row) in self.rows.iter().enumerate().skip(n) {
                grove.at(row.ground, row_at(at));
            }
        }
    }
}

fn main() {
    let mut foliage = Foliage::new();
    foliage.title("rows");
    foliage.desktop_size(Area::new(360.0, 480.0));
    foliage.root::<List>();
    foliage.photosynthesize();
}
```

Each row is three elements: a raised ground, a label on it, and a round red button on its right
edge. The app keeps two of the three names, because those are the two it will ever write to or ask
about. The label's name is thrown away, and that is fine: it goes when its ground goes.

## Names

`branch` hands back a [`Leaf`](https://eblack-leaf.github.io/foliage/api/foliage/struct.Leaf.html)
at once. The element it names does not exist yet: the op that grows it is queued, and the queue is
drained when `frame` returns. The name is usable anyway, as a trunk and as the target of any write.
`grow_row` branches the label and the button off `ground` a line after asking for it, and the three
are grown together in the order they were asked for.

A name moves through three states, in order, and the last is final:

```rust
# use foliage::{Grove, Leaf, Presence};
# fn f(grove: &Grove, leaf: Leaf) {
match grove.presence(leaf) {
    // Named, and the op that grows it has not been drained yet.
    Presence::Planted => {}
    // In the tree.
    Presence::Live => {}
    // Pruned, or taken down with an ancestor. Growing again means a new name.
    Presence::Withered => {}
}
# }
```

**A name is never reused.** Once an element has withered, its name refers to nothing for the rest
of the run, so a stale name held in some corner of an app cannot come to mean whatever grew after
it. Writing to a withered name does nothing and reading one answers `None`. Nothing panics.

`Planted` is also what a name reads as when the op that would have grown it was dropped, because
its trunk had withered by the time the drain reached it.

## Pruning

`prune` takes an element down along with everything grown beneath it. Removing a row prunes its
ground, and the label and the button go too.

Every element that went is reported once, in the next frame's `Pollen`:

```rust
# use foliage::{Grove, Leaf, Pollen};
# struct App { remove: Leaf }
# impl App {
# fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
if pollen.withered(self.remove) {
    // The button is gone, whichever ancestor took it down.
}
# }
# }
```

In the list above nothing needs this, because the app pruned the row itself and already knows. It
matters where something else decides: a part that takes itself down when an animation finishes, or
a worker thread pruning what it grew.

## Writing

Removing a row leaves a gap, so every row below it is moved up with `at`:

```rust,ignore
for (at, row) in self.rows.iter().enumerate().skip(n) {
    grove.at(row.ground, row_at(at));
}
```

`at` replaces an element's whole placement. A placement is one value rather than a set of edges,
so there is no half-written state between two writes and no question of which edge a later write
meant.

Most things a seed declares can be written again afterwards, by a verb on
[`Grow`](https://eblack-leaf.github.io/foliage/api/foliage/trait.Grow.html):

| Declared on the seed | Written afterwards |
|---|---|
| `at` | `Grow::at` |
| `Line::between`, `Line::trace` | `Grow::between`, `Grow::trace` |
| `grid` | `Grow::grid` |
| `anchored` | `Grow::anchor` |
| `elevate` | `Grow::elevate` |
| `color` | `Grow::color` |
| `rounding` on a panel or an image | `Grow::round` |
| a polygon's `sides`, `rounding`, `rotation` | `Grow::reshape` |
| `Text::new(value)`, a field's `value` | `Grow::text` |
| `Text::tint` | `Grow::tint`, `Grow::untint` |
| `Icon::new(field)` | `Grow::mark` |
| `Image::new(plate)`, `Image::fit` | `Grow::depict`, `Grow::fit` |
| a field's `read_only` | `Grow::read_only` |
| a field's `masked` | `Grow::masked` |
| `visible` | `Grow::visible` |
| `opacity` | `Grow::opacity` |
| (nothing) | `Grow::disable`, `Grow::enable` |

The rest is fixed when the element is grown: its font and font size, whether it is `interactive`
or `intangible`, which axes it `drags` on or `scrolls` on, whether it is `pinned` or `floats`, its
focus order, and a line's weight and cap. What an element *is* never changes either. A panel does
not become a run of text; that is a different element.

**An op that names something it does not apply to is dropped.** `text` written to a panel, `round`
written to a polygon, `scroll` written to something that does not scroll: none of them is an error
and none of them panics. The element is left as it was, and the drop is written to the trace with
the verb and the reason. [Debugging](debugging.md) shows how to see it.

## A frame reads one tree

Every write is queued, and the queue is drained after `frame` returns. So nothing an app writes
can change what it reads while it is still running:

```rust
# use foliage::{Grove, Grow, Leaf, Location, Sap, Source, Vein, left, top};
# fn f(grove: &mut Grove, row: Leaf) {
let before = grove.tap(row, Vein::Drawn);
grove.at(
    row,
    Location::new().xs(left(0.px()).width(10.px()), top(0.px()).height(10.px())),
);
let after = grove.tap(row, Vein::Drawn);
// The move has not happened yet. It lands at the drain, and the next frame reads it.
assert_eq!(before, after);
# }
```

That is a guarantee rather than an accident: the state read at the top of `frame` is the state
read at the bottom, so a read never has to be understood by where in the function it was made. The
same goes for a prune. An element pruned in this frame is still there for the rest of it.

It also means an element planted this frame cannot be read this frame. Its name is `Planted`, and
a tap of it answers `None` until the next one.

## Reading

`tap` reads one property of one element, named by a
[`Vein`](https://eblack-leaf.github.io/foliage/api/foliage/enum.Vein.html), and hands back a
[`Sap`](https://eblack-leaf.github.io/foliage/api/foliage/enum.Sap.html) holding a copy:

```rust
# use foliage::{Grove, Leaf, Sap, Vein};
# fn f(grove: &Grove, row: Leaf, label: Leaf) {
if let Some(Sap::Section(drawn)) = grove.tap(row, Vein::Drawn) {
    // Where the row is on screen, in logical pixels.
    let _ = (drawn.left(), drawn.top(), drawn.width(), drawn.height());
}
if let Some(Sap::Text(says)) = grove.tap(label, Vein::Text) {
    let _ = says;
}
if let Some(Sap::Leaves(under)) = grove.tap(row, Vein::Branches) {
    // Everything grown directly under the row, in the order it was grown.
    let _ = under.len();
}
# }
```

Everything an app can declare can be read back, because a value that can be set and not read is
one an app has to keep a copy of. Some veins read what was *declared* (`Color`, `Visible`,
`Opacity`, `Elevation`); others read what the engine *resolved* (`Placed`, `Drawn`, `Ends`,
`Offset`, `Extent`). `tap` answers `None` where the element has withered, has not been grown yet,
or does not carry that property. Asking a panel for `Vein::Text` is `None`, not a panic.

A read is a copy. It borrows nothing and holds nothing; the engine keeps its own state and the app
keeps the answer for as long as it wants it.

## What an app holds

Taken together, this is the shape of every foliage app:

- The app's own value keeps **names** (and whatever else it likes). Names are small copyable
  values, and holding one keeps nothing alive.
- **Writes** are verbs on a `Grow`, queued and drained after the app's code returns.
- **Reports** arrive once a frame, as `Pollen`.
- **Reads** are copies, taken with `tap` whenever the app needs one.

No reference into the engine is ever handed out. That is what lets an app store its names in any
structure it likes, pass them between parts of itself, and hand them to another thread, without
anything to borrow or lock.
