# Placing elements

Every element placed by a box states its box as a
[`Location`](https://eblack-leaf.github.io/foliage/api/foliage/struct.Location.html). This chapter
builds a card whose text can grow, with a button that stays under it and a ground that stays
around all of it, and uses it to cover the placement grammar.

## A card that follows its text

```rust,no_run
use foliage::{
    Area, Boxed, Foliage, FontSize, Grove, Grow, Leaf, Location, Palette, Panel, Place, Pollen,
    Root, Rounding, Source, Text, anchor, center_x, center_y, content, left, top,
};

const SHORT: &str = "A value is a role and a source.";
const LONG: &str = "A value is a role and a source. The role says what a number describes for \
                    this element, and the source says where the number comes from. The two are \
                    independent, which is what lets any source fill any coordinate.";

struct Card {
    body: Leaf,
    more: Leaf,
    long: bool,
}

impl Root for Card {
    fn take_root(grove: &mut Grove) -> Self {
        let page = grove.plant(Panel::new());
        // Grown first, so it sits behind everything grown after it.
        let ground = grove.branch(
            page,
            Panel::new()
                .color(Palette::Raised)
                .rounding(Rounding::Md)
                .at(Location::new().xs(
                    left(16.px()).right(100.pct() - 16.px()),
                    top(16.px()).bottom(anchor().bottom() + 16.px()),
                )),
        );
        let title = grove.branch(
            page,
            Text::new("Placement")
                .font_size(FontSize::new().xs(18))
                .at(Location::new().xs(
                    left(32.px()).right(100.pct() - 32.px()),
                    top(32.px()).height(content()),
                )),
        );
        let body = grove.branch(
            page,
            Text::new(SHORT)
                .color(Palette::Ink.recede())
                .font_size(FontSize::new().xs(14))
                .anchored(title)
                .at(Location::new().xs(
                    left(32.px()).right(100.pct() - 32.px()),
                    top(anchor().bottom() + 8.px()).height(content()),
                )),
        );
        let more = grove.branch(
            page,
            Panel::new()
                .color(Palette::Accent)
                .rounding(Rounding::Sm)
                .interactive()
                .anchored(body)
                .at(Location::new().xs(
                    left(32.px()).width(96.px()),
                    top(anchor().bottom() + 12.px()).height(32.px()),
                )),
        );
        grove.branch(
            more,
            Text::new("more")
                .color(Palette::Accent.on())
                .font_size(FontSize::new().xs(14))
                .intangible()
                .at(Location::new().xs(
                    center_x(50.pct()).width(content()),
                    center_y(50.pct()).height(content()),
                )),
        );
        // The ground is placed by what it holds: it ends 16px below the button.
        grove.anchor(ground, more);
        Card {
            body,
            more,
            long: false,
        }
    }

    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        if pollen.clicked(self.more) {
            self.long = !self.long;
            grove.text(self.body, if self.long { LONG } else { SHORT });
        }
    }
}

fn main() {
    let mut foliage = Foliage::new();
    foliage.title("card");
    foliage.desktop_size(Area::new(360.0, 400.0));
    foliage.root::<Card>();
    foliage.photosynthesize();
}
```

Press the button and the body grows to three times its length. It wraps at the card's width, grows
taller, pushes the button down, and the ground stretches to hold both. All of that happens in the
frame the text changed, and none of it is written by the app: the app rewrote one string.

Narrow the window and the same thing happens the other way. Every line of text wraps at its new
width, every height follows, and every anchored element moves.

## A value is a role and a source

```rust,ignore
left(32.px()).right(100.pct() - 32.px())
```

Each axis is written as two values. A **role** (`left`, `right`, `width`) says what the value
describes for this element. A **source** (`32.px()`, `100.pct()`) says where the number comes
from. The two are independent: any source can fill any role, and arithmetic joins sources into
one value.

The role is written first because it is read first. "My left is 32 pixels in; my right is 32
pixels short of my trunk's right edge." Each opener returns a type that carries only the
completions legal with it, so an axis has exactly four forms:

```text
left(..).width(..)      left(..).right(..)      right(..).width(..)      center_x(..).width(..)
top(..).height(..)      top(..).bottom(..)      bottom(..).height(..)    center_y(..).height(..)
```

`left(..).center_x(..)` or a second `width` is not rejected at run time. The method is not there.

**Every edge role is a coordinate in the parent's space**, `right` and `bottom` included. Sixteen
in from the right is `right(100.pct() - 16.px())`, not `right(16.px())`, which would put the
element's right edge sixteen pixels from its trunk's *left*.

The horizontal half always comes first in `xs(..)`, and the two halves are different types. The
wrong order does not compile:

```rust,compile_fail
use foliage::{Location, Source, left, top};
Location::new().xs(top(0.px()).height(10.px()), left(0.px()).width(10.px()));
```

## Sources

| Source | Reads |
|---|---|
| `20.px()` | Logical pixels. The only source that reads no geometry |
| `50.pct()` | A fraction of the trunk's extent, on the axis being resolved |
| `2.col()`, `2.row()` | A one-based track of the trunk's grid ([Responsive layout](responsive.md)) |
| `8.letters()` | A count of the element's own character cells |
| `content()` | The element's own measured extent |
| `aspect(ratio)` | A height in proportion to the element's own width |
| `anchor()`, `trunk()` | Another element's edges, extents, tracks, cells and measure |

`px`, `pct`, `col`, `row` and `letters` are methods on plain numbers, from the
[`Source`](https://eblack-leaf.github.io/foliage/api/foliage/trait.Source.html) trait, and read
the trunk. Expressions are sums of scaled terms: `+`, `-`, and `*` by a plain number.

```rust,ignore
left(20.px()).right(100.pct() - 16.px())          // inset 20 and 16
center_x(50.pct()).width(50.pct() + 40.px())      // centred, half again as wide plus a bit
top(0.px()).height(aspect(16.0 / 9.0))             // a 16:9 box, whatever its width came to
```

An element that states no location fills its trunk. An element with no trunk (one that was
planted rather than branched) fills the window, so `pct` at the top level is a fraction of the
viewport.

## Lengths and coordinates

The sources come in four types, and the split is what makes a whole class of mistakes impossible
to write:

- a **length** (`Length`) is an extent with no position: `px`, `pct`, `letters`, `content()`,
  `anchor().width()`. Legal in any role.
- a **vertical length** (`VerticalLength`) is a length only the vertical pass can answer: `row`,
  `aspect`, `anchor().height()`. Legal in vertical roles only.
- a **coordinate** (`HorizontalCoordinate`, `VerticalCoordinate`) is a position: an edge of
  another element, or a track of another element's grid. Legal where a position is wanted, and
  never as a size.

A length used as a coordinate is measured from the trunk's near edge, which is why `left(20.px())`
means twenty pixels in. An edge is already a position on the surface and is measured from
nothing, which is why `top(anchor().bottom() + 8.px())` means eight pixels below the anchor
wherever the anchor is.

Two coordinates subtracted give the length between them, which is how two edges become a size:

```rust,ignore
left(anchor().left()).width(anchor().right() - anchor().left())   // exactly as wide as the anchor
```

Adding two coordinates is not an operation, a coordinate on one axis has no reading on the other,
and a height cannot be a width:

```rust,compile_fail
use foliage::{Source, left};
// A row is a vertical length: the horizontal pass cannot know it yet.
left(0.px()).width(2.row());
```

The last rule is the interesting one, and it comes from how resolution is ordered.

## Width flows down, height flows up

Text wrapping makes a height depend on a width, and a width comes from layout. A box sized to its
contents is therefore a cycle in general. foliage breaks it at two passes and never iterates:

1. The **horizontal** axis resolves for the whole tree. A run of monospaced text knows the widest
   it could want to be before any layout (its longest line's character count times the cell
   width), so nothing on this pass has to be measured.
2. Every run **wraps** at the width it was just given, which settles how tall it is.
3. The **vertical** axis resolves for the whole tree, reading those heights.

The types are that order stated in advance. A horizontal role cannot read a height because, when
it is resolved, no height exists. The reverse is fine and useful: `height(2.col())` is a
two-column span used as a height, and `aspect` is a height read off the element's own width.

`content()` is one word with a different question per axis. In a width role it is the widest the
element wants to be (its text, unwrapped). In a height role it is how tall it turned out at the
width it was given. So `left(..).right(..)` with `height(content())` is a paragraph: as wide as its
parent allows, as tall as it wraps to.

## Clamps

Either axis can be held between two extents:

```rust,ignore
left(0.px()).width(content()).at_most(240.px())    // fit-content: as wide as its text, up to 240
left(0.px()).right(100.pct()).at_least(160.px())   // stretch, but never narrower than 160
top(0.px()).height(content()).at_most(aspect(1.0)) // as tall as it wraps to, no taller than wide
```

The ceiling is applied first and the floor second, so a floor always wins, and no extent is ever
negative. Over `content()`, `at_most` is fit-content: whichever of the two is smaller.

## Anchors

```rust,ignore
Text::new(SHORT)
    .anchored(title)
    .at(Location::new().xs(
        left(32.px()).right(100.pct() - 32.px()),
        top(anchor().bottom() + 8.px()).height(content()),
    ))
```

Bare sources read the trunk. [`anchor()`](https://eblack-leaf.github.io/foliage/api/foliage/fn.anchor.html)
reads the one other element an element is anchored to, with the same vocabulary a trunk offers:
its edges, its width and height, its grid's tracks, its character cells, and its measured content.
An element has **at most one** anchor, given with `anchored` on the seed or `anchor` on the grove.
A placement reading an anchor it has not been given reads a zero box.

Anchoring is the answer to "sit below that, wherever it ended up". It holds up under wrapping and
resizing where a fixed offset does not, and it costs one read of a box that has already been
resolved: the engine orders resolution so that an element always comes after whatever it anchors
to.

An anchor can point anywhere in the tree: a sibling, a later sibling, a cousin, an element in
another subtree entirely. The card's ground anchors to the button, which was grown after it.
What an anchor cannot do is close a loop. An element anchored to something that is anchored
(however indirectly) back to it is refused with a panic that names both elements, where each was
planted, and the call that made the loop:

```text
anchor cycle: leaf 4294967290 cannot anchor to leaf 4294967292, which already reaches back to it
  leaf 4294967290 was planted at src/main.rs:40:21
  leaf 4294967292 was planted at src/main.rs:31:20
  the anchor was written at src/main.rs:58:15
```

A cycle between two anchors is a contradiction rather than a scheduling problem, so there is no
state to fall back to, and the mistake is stopped where it can still be fixed.

[`trunk()`](https://eblack-leaf.github.io/foliage/api/foliage/fn.trunk.html) carries the same
vocabulary for the trunk, for the two readings the bare sources cannot spell:
`trunk().content()` (how large the trunk's contents measured, as against its box) and
`trunk().letters(n)` (cells in the trunk's font rather than the element's own).

## Sizing a box to what it holds

The card's ground is not the parent of the text it frames. It is a sibling, grown first so that
it sits behind, and anchored to the last thing it frames. That is the idiom, and the reason for it
is worth knowing.

An element sized `height(content())` takes the greater of two things: how tall its own run of
text wrapped to, and how far down its children reach. **Only children that describe their own
extent are counted**: a child placed in pixels, in letters, by its own content, or by any
horizontal reading. A child whose vertical placement reads *another* vertical box (a percentage of
its trunk, a row of its trunk's grid, an anchor's edge) is asking how tall something else is, so
it cannot be what decides how tall this is. It is left out of the measure and given its real
position afterwards.

So a box *can* be sized to children at known offsets. A badge whose label sits 6 pixels in:

```rust
# use foliage::{Boxed, FontSize, Grove, Grow, Leaf, Location, Palette, Panel, Place, Rounding, Source, Text, content, left, top};
# fn f(grove: &mut Grove, under: Leaf) {
let badge = grove.branch(
    under,
    Panel::new()
        .color(Palette::Signal)
        .rounding(Rounding::Full)
        .font_size(FontSize::new().xs(12))
        // Two letters of room either side of a five-letter label, in the label's own cells.
        .at(Location::new().xs(
            left(0.px()).width(9.letters()),
            top(0.px()).height(content() + 6.px()),
        )),
);
grove.branch(
    badge,
    Text::new("draft")
        .color(Palette::Signal.on())
        .font_size(FontSize::new().xs(12))
        .at(Location::new().xs(
            left(2.letters()).width(content()),
            top(6.px()).height(content()),
        )),
);
# }
```

The label reaches 6 pixels plus one line down, and the badge adds 6 more for the padding below.
Two things in it are worth copying:

- **Padding below is added, not measured.** The measure is how far the children reach, so a
  bottom margin is `content() + 6.px()`.
- **A width in letters needs a font size.** As a *width*, `content()` is only the element's own
  text, so a panel does not take its width from a label on it. It can state the same width in
  characters instead, and `letters` counts cells of the element's own font and size, which is why
  the badge declares the label's `font_size`. An element that names no font and no size has no
  cell and reads zero for every letter.

A stack of things of unknown height, each placed under the last with an anchor, is the case that
cannot be measured this way. Frame it the way the card does, with a sibling anchored to the last
element, or put it in a scrolling region, whose extent is measured from wherever its children
landed ([Scrolling](scrolling.md)).

## Elevation

Where an element sits in depth is relative to its trunk:

```rust,ignore
Panel::new().elevate(Elevation::up(2))   // two steps in front of its trunk
Panel::new().elevate(Elevation::down(1)) // one step behind it
```

Elevation accumulates down the tree, so raising a card raises everything on it, and nothing
inside it is rewritten. An element that says nothing sits at its trunk's own elevation, which puts
it just in front of its trunk. Two elements that come to the same elevation are ordered by when
their names were handed out: **the one asked for later is in front**. That is why the card's
ground, grown first, is behind the title, the body and the button without a word about it, and why
a label branched off a button is on top of the button.

There is deliberately no way to state an absolute layer. An element that has to clear the stack it
was grown in (a dropdown over a list, a tooltip over a page) is grown somewhere else, and anchored
back to what it belongs to:

```rust
# use foliage::{Boxed, Elevation, Grove, Grow, Leaf, Location, Palette, Panel, Place, Rounding, Source, anchor, left, top};
# fn f(grove: &mut Grove, button: Leaf) {
let menu = grove.plant(
    Panel::new()
        .color(Palette::Raised)
        .rounding(Rounding::Sm)
        .elevate(Elevation::up(10))
        .anchored(button)
        .at(Location::new().xs(
            left(anchor().left()).width(180.px()),
            top(anchor().bottom() + 4.px()).height(120.px()),
        )),
);
# }
```

The trunk decides what takes an element down, what clips it, and what it stacks among. The anchor
decides where it sits. Keeping the two apart is what lets a menu live at the top of the tree and
still follow the button that opened it.

## Lines

A [`Line`](https://eblack-leaf.github.io/foliage/api/foliage/struct.Line.html) has no box to
state. It has two ends, and each end is a
[`Point`](https://eblack-leaf.github.io/foliage/api/foliage/struct.Point.html): one coordinate
per axis, in the same grammar.

```rust
# use foliage::{Grove, Grow, Leaf, Line, Palette, Place, Point, Source, anchor};
# fn f(grove: &mut Grove, card: Leaf, title: Leaf) {
// A rule across the card, 8px under its title.
grove.branch(
    card,
    Line::new()
        .weight(1.0)
        .color(Palette::Muted)
        .anchored(title)
        .between(
            Point::new(16.px(), anchor().bottom() + 8.px()),
            Point::new(100.pct() - 16.px(), anchor().bottom() + 8.px()),
        ),
);
# }
```

A line's box is the rectangle around its two ends, grown by half its weight on every side, so a
horizontal rule still has a box to be clipped and hit-tested by. Its ends are moved with
`Grow::between`, and read back with `Vein::Ends`. A line is not `Boxed`, so there is no `at` to
hand it.

A path is a chain of lines meeting end to end, and `Cap::Round` on each is what closes the joins:
each end is a half-disc, and the disc one stroke puts on a shared end covers the wedge two strokes
leave open. The [`polyline`](https://github.com/eblack-leaf/foliage/blob/main/foliage/examples/polyline.rs)
example draws a series this way.
