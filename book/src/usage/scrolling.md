# Scrolling

An element scrolls because it said so, and for no other reason. Having more children than fit, or
being divided by a grid, says nothing about scrolling. This chapter builds a log: a column that
scrolls under a header that stays put, lines that are added at the bottom and scrolled to in the
same frame, a button that glides back to the top, and a thumb that shows where the column is.

## Declaring a region

```rust
# use foliage::{Axes, Place, Scroll, Stem};
Stem::new().scrolls(Axes::Vertical);
Stem::new().scrolls(Scroll::new(Axes::Both).contain(Axes::Vertical));
```

`scrolls` names the axes an element moves on. An axis that is not named does not scroll and has no
extent. It is not a scrolling axis with a range of zero; there is simply nothing to move, which is
what stops a column that scrolls down from growing a sideways scroll nobody asked for.

A drag anywhere inside a region scrolls it, whether or not what the drag landed on receives
anything. On touch that is the only way to scroll at all, and it has to work over plain text and
decoration. So scrolling is structural: the region the press landed in is found by walking up the
tree from wherever the press landed.

When a region reaches the end of an axis, the drag **chains** outward to the next region
containing it that scrolls that axis, which is what lets a drag inside a list keep moving the page
once the list is done. `Scroll::contain` names axes that **absorb** instead: the region keeps the
gesture at its end and nothing outside moves. A map, an editor, or a pane inside a fixed shell
wants that; reaching the bottom and having the whole page lurch is a bug in all three.

A wheel scrolls the innermost region under the pointer that can still move, and chains outward the
same way.

## A log

```rust,no_run
use foliage::{
    Area, Axes, Boxed, Ease, Elevation, Foliage, FontSize, Grove, Grow, Leaf, Location, Motion,
    Palette, Panel, Place, Pollen, Root, Rounding, Sap, Scroll, ScrollTo, Source, Stem, Text,
    Timing, Vein, anchor, bottom, center_x, center_y, content, left, right, top,
};

const HEADER: f32 = 48.0;
const BAR: f32 = 64.0;
const THUMB: f32 = 40.0;

struct Log {
    column: Leaf,
    last: Option<Leaf>,
    thumb: Leaf,
    add: Leaf,
    back: Leaf,
    lines: usize,
    /// How far through its range the column was when the thumb was last placed.
    read: f32,
}

fn button(grove: &mut Grove, under: Leaf, x: f32, says: &str) -> Leaf {
    let button = grove.branch(
        under,
        Panel::new()
            .color(Palette::Accent)
            .rounding(Rounding::Sm)
            .interactive()
            .at(Location::new().xs(left(x.px()).width(120.px()), top(12.px()).height(40.px()))),
    );
    grove.branch(
        button,
        Text::new(says)
            .color(Palette::Accent.on())
            .font_size(FontSize::new().xs(14))
            .intangible()
            .at(Location::new().xs(
                center_x(50.pct()).width(content()),
                center_y(50.pct()).height(content()),
            )),
    );
    button
}

/// The thumb, `progress` of the way down its rail: its top at the rail's top at `0.0`, and its
/// bottom at the rail's bottom at `1.0`.
fn thumb_at(progress: f32) -> Location {
    Location::new().xs(
        left(2.px()).width(4.px()),
        top((progress * 100.0).pct() - (progress * THUMB).px()).height(THUMB.px()),
    )
}

impl Root for Log {
    fn take_root(grove: &mut Grove) -> Self {
        let page = grove.plant(Panel::new());
        let column = grove.branch(
            page,
            Stem::new()
                .scrolls(Scroll::new(Axes::Vertical).contain(Axes::Vertical))
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(0.px()).bottom(100.pct() - BAR.px()),
                )),
        );
        // Inside the column, and not moving with it.
        let header = grove.branch(
            column,
            Panel::new()
                .color(Palette::Raised)
                .pinned()
                .elevate(Elevation::up(1))
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(0.px()).height(HEADER.px()),
                )),
        );
        grove.branch(
            header,
            Text::new("log").at(Location::new().xs(
                left(16.px()).width(content()),
                center_y(50.pct()).height(content()),
            )),
        );
        // The thumb's rail: beside the column, from under the header to above the bar.
        let rail = grove.branch(
            page,
            Stem::new()
                .elevate(Elevation::up(2))
                .intangible()
                .at(Location::new().xs(
                    right(100.pct()).width(8.px()),
                    top(HEADER.px()).bottom(100.pct() - BAR.px()),
                )),
        );
        let thumb = grove.branch(
            rail,
            Panel::new()
                .color(Palette::Muted)
                .rounding(Rounding::Full)
                .intangible()
                .at(thumb_at(0.0)),
        );
        let bar = grove.branch(
            page,
            Stem::new().at(Location::new().xs(
                left(0.px()).right(100.pct()),
                bottom(100.pct()).height(BAR.px()),
            )),
        );
        let add = button(grove, bar, 16.0, "add a line");
        let back = button(grove, bar, 152.0, "back to top");
        Log {
            column,
            last: None,
            thumb,
            add,
            back,
            lines: 0,
            read: 0.0,
        }
    }

    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        if pollen.clicked(self.add) {
            self.lines += 1;
            let line = Text::new(format!("line {}", self.lines));
            let line = match self.last {
                Some(last) => line.anchored(last).at(Location::new().xs(
                    left(16.px()).width(content()),
                    top(anchor().bottom() + 8.px()).height(content()),
                )),
                None => line.at(Location::new().xs(
                    left(16.px()).width(content()),
                    top((HEADER + 8.0).px()).height(content()),
                )),
            };
            self.last = Some(grove.branch(self.column, line));
            // The column grows in this frame, and `end` is answered against this frame's extent.
            grove.scroll(self.column, ScrollTo::end());
            // It lands after this code has run, so one more frame is needed to read where.
            grove.again();
        }
        if pollen.clicked(self.back) {
            grove.animate(
                self.column,
                Motion::Scroll(ScrollTo::start()),
                Timing::ms(400).ease(Ease::Decelerate),
            );
        }
        // The thumb stands as far down its rail as the column is through its range.
        if let Some(Sap::Progress(progress)) = grove.tap(self.column, Vein::Progress)
            && progress.y != self.read
        {
            self.read = progress.y;
            grove.at(self.thumb, thumb_at(progress.y));
            // Still moving, so keep reading until it stops.
            grove.again();
        }
    }
}

fn main() {
    let mut foliage = Foliage::new();
    foliage.title("log");
    foliage.desktop_size(Area::new(360.0, 480.0));
    foliage.root::<Log>();
    foliage.photosynthesize();
}
```

Add lines until they pass the bottom of the window. Each new one is scrolled to on the frame it
appears, drag or wheel and the column moves under the header, and "back to top" glides there.

## Extent

A region's **extent** is how far its content reaches, and its range is the extent less its own box.
It is measured from where its children *landed*, never from what is currently drawn: content
scrolled out of sight is exactly what an extent describes.

Four rules keep it honest:

- **Only along a declared axis.** A child hanging off the side of a column that scrolls down is
  simply out of frame.
- **Outward from the content's origin.** A child placed above or left of the region's near edges
  creates no room to scroll *back* into.
- **Never smaller than the region's own box**, so an empty region has a range of zero.
- **A visible child far away is content**, and is counted. What takes a child out of the extent is
  the app saying so: `visible(false)` takes it and everything under it out, and `pinned` and
  `floats` take out something that is inside the region without being part of its content.

A child that scrolls in its own right contributes its box to its parent's extent and not its
content; what overflows inside it is its own to reach.

Unlike `height(content())`, an extent has no rule about which children count by *how* they were
placed. The log's lines are each anchored to the one before, which would leave them out of a
content-sized box's measure, and they are all counted here. A stack of things of unknown height
belongs in a region for exactly that reason.

## Pinned and floating

Two declarations let an element sit inside a region without being part of its content:

- **`pinned()`** does not travel with the content, and contributes nothing to the extent. The log's
  header is pinned: the lines slide under it. It keeps its place in the tree, and with it the
  clipping, the opacity and the disabling it would have lost by being grown outside the region.
- **`floats(Escape)`** is not clipped by the region and contributes nothing to its extent, but
  *does* travel with the content. A menu that opens past the edge of the list its row is in is
  this, so it follows the row as the list scrolls.

The two are opposites: a pinned element keeps the clip and escapes the movement, and a floating
one keeps the movement and escapes the clip. How far a float escapes is stated rather than
assumed, because both answers are right somewhere:

```rust
# use foliage::{Escape, Leaf, Panel, Place};
# fn f(sheet: Leaf) {
Panel::new().floats(Escape::Region);        // out of the region it is in, and no further
Panel::new().floats(Escape::Surface);       // out of every region above it
Panel::new().floats(Escape::Within(sheet)); // out of every region up to the sheet, which still holds it
# }
```

## Moving a region

```rust
# use foliage::{Axes, Grove, Grow, Leaf, ScrollTo};
# fn f(grove: &mut Grove, column: Leaf, section: Leaf, canvas: Leaf) {
grove.scroll(column, ScrollTo::px(240.0));     // 240 pixels from the content's origin
grove.scroll(column, ScrollTo::fraction(0.5)); // half way through its range
grove.scroll(column, ScrollTo::start());
grove.scroll(column, ScrollTo::end());
grove.scroll(column, ScrollTo::show(section)); // the least movement that brings it into view
grove.scroll(canvas, ScrollTo::px(120.0).on(Axes::Horizontal));
# }
```

Every form lands as a number of pixels from the content's origin, clamped to what the region can
reach, and reading the offset back afterwards gives those pixels. `show` moves nothing if the
element is already in view, and is dropped if the element is not grown under the region.

Every form but one is stated relative to the region's range, so it means the same thing on either
axis. `px` is the exception: two hundred pixels down and two hundred across are unrelated distances
that happen to share a number, so on a region that scrolls both ways a `px` has to name its axis
with `on`, and one that does not is dropped rather than moving both.

**A destination is answered against the frame it lands in.** Scrolling to the end of a list that
grew in the same frame lands at the end of the list as it now is, not where it used to stop; that
is why the log can add a line and scroll to it in one frame. The same holds for
`Motion::Scroll`, which re-answers its destination every frame, so a glide toward the end of a list
that keeps growing still lands on the end.

A direct `scroll` cancels a scroll motion still running, and ends a coast. A drag on the region
cancels both too. The reader wins.

## Momentum

A drag released with speed keeps the region moving. The speed is the mean over the last 100
milliseconds of the gesture rather than the last frame, because a hand slows as it lifts and a
frame on its own reads a flick as a stop. The coast decays continuously with a half-life (350ms by
default), so a fling travels the same distance at 30 frames a second as at 120, and it stops when
it falls below a minimum speed (40 pixels a second). Both are `Momentum`, tuned once for the app
with `Foliage::tune`.

A coast that reaches an end hands itself outward exactly as a drag does, or stops there if the
axis contains.

Pressing a coasting region stops it where the hand met it, and the press is spent on the catch:
whatever was under the finger hears nothing, because stopping a moving list is not also a press on
whatever it happened to stop over.

## Reading a region

| Vein | Answers |
|---|---|
| `Vein::Offset` | `Sap::Position`: how far the region has moved, in pixels |
| `Vein::Extent` | `Sap::Area`: how far its content reaches from its near edges |
| `Vein::Progress` | `Sap::Progress`: how far through its range it is, per axis, `0.0..=1.0` |

Each answers `None` on something that does not scroll. A region with nowhere to go reads zero
progress on that axis.

Inside a scrolled region, an element has two boxes. `Vein::Placed` is where the layout put it;
`Vein::Drawn` is that less every scrolling ancestor's offset, which is what was drawn and what a
hit test runs against. The two differ only inside a region that has moved.

## Following a region from the app

The log's thumb is the app following the column by reading it. Only the progress is read. How far
the thumb can travel is stated in the grammar, as a percentage of its rail less its own height, so
a resize that makes the column taller moves the thumb without the app hearing about it: whatever
can be stated in the grammar is kept right by the layout, and only what cannot is the app's to
follow.

A read taken in `frame` is what the last frame settled, which is what is on screen. A drag moves
the offset before `frame` runs, so the thumb is exactly in step while a finger is on the column.
But a coast, a scroll motion and a `scroll` write are all answered *later* in the frame, once the
frame's extent has been measured. So while one of those is moving the column, the thumb placed
from a read trails the content by a frame.

That is harmless while the column is moving, because every frame of the motion reads the frame
before it. It matters at the end: the frame in which a coast settles, or a `scroll` lands, may be
the last frame the engine runs, since nothing is owed after it. The log therefore asks for one more
frame with [`again`](https://eblack-leaf.github.io/foliage/api/foliage/struct.Grove.html#method.again)
whenever what it read has changed, and once after writing a `scroll`. A read that has stopped
changing stops asking, and the engine goes back to sleep.
