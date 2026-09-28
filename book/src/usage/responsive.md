# Responsive layout

A phone and a desk want different layouts from the same app. foliage answers that in the
placement itself: a `Location` says where an element sits *at each breakpoint*, and a `Grid` says
how a box is divided at each one. This chapter builds a page whose navigation is a bar along the
bottom on a phone and a rail down the left on a desk.

## Breakpoints

The viewport's width puts it at one of five breakpoints:

| Breakpoint | Viewport width |
|---|---|
| `Xs` | below 420 |
| `Sm` | 420 and above |
| `Md` | 600 and above |
| `Lg` | 840 and above |
| `Xl` | 1200 and above |

The bounds are constants on [`Layout`](https://eblack-leaf.github.io/foliage/api/foliage/enum.Layout.html)
(`Layout::SM`, `Layout::MD` and so on) and `grove.layout()` says which one is in force.

Four things are written per breakpoint, all in the same chain: `Location`, `Grid`, `FontSize`, and
a line's `Trace`. Each has `xs`, `sm`, `md`, `lg` and `xl`, and a breakpoint with nothing of its
own takes the nearest smaller one that has. So only `xs` is ever needed, and an element that only
differs above some width states that width and nothing else:

```rust,ignore
Location::new()
    .xs(left(16.px()).right(100.pct() - 16.px()), top(16.px()).height(content()))
    .lg(left(4.col()).right(12.col()), top(32.px()).height(content()))
// Sm and Md fall back to xs; Xl falls back to lg.
```

A chain that states nothing at all falls back to the whole of the parent's box.

### Short

Width alone reads a landscape phone wrong: it is `Md`-wide, with a fraction of the height a
portrait one has. So the viewport has a second, independent reading,
[`Short`](https://eblack-leaf.github.io/foliage/api/foliage/enum.Short.html): it becomes
`Short::Yes` below 400 logical pixels tall, and goes back to `Short::No` at 440.

The gap between the two is deliberate. On mobile web the address bar hides and shows as a page
scrolls, and a single threshold would flip on every scroll and re-lay the whole page. Leaning
toward `Yes` is the safer mistake: a layout slightly more compact than it needed to be, rather than
content running off the bottom.

Every chain also takes `short(..)`. A configuration keyed to `short` wins outright while the
viewport is cramped, whatever its width, and nothing without one is affected.

## Grids

Every element divides its box for the elements grown under it. Undeclared, it is one column and
one row. A [`Grid`](https://eblack-leaf.github.io/foliage/api/foliage/struct.Grid.html) states the
division, per breakpoint:

```rust
# use foliage::{Divide, Grid};
Grid::new()
    .xs(4.columns().gap(12.0), 1.rows())
    .md(12.columns().gap(16.0), 1.rows());
```

`4.columns()` (from the [`Divide`](https://eblack-leaf.github.io/foliage/api/foliage/trait.Divide.html)
trait) is four equal tracks, with the gaps taken out first. A gap is between tracks only, so `n`
tracks have `n - 1` gaps and no margin at either end. Two other pitches size the track rather than
count them:

- `Columns::px(80.0)` is tracks of 80 pixels, as many as fit.
- `Columns::letters(12.0)` is tracks of twelve character cells, in the font of the element the grid
  is *on*, not of the children addressing it. A table whose columns line up with its text is a grid
  like this.

A child addresses the grid with `col` and `row`, which are one-based. The role decides which part
of the track is meant:

| Written | Means |
|---|---|
| `left(2.col())` | the left edge of column 2 |
| `right(3.col())` | the right edge of column 3 |
| `center_x(2.col())` | the middle of column 2 |
| `width(2.col())` | the width of two columns and the gap between them |
| `left(1.col()).right(1.col())` | exactly column 1 |

A grid decides how children are laid out and nothing else. In particular it says nothing about
scrolling: a column with more rows than fit is simply taller than its parent, and an element
scrolls only because it says so ([Scrolling](scrolling.md)).

## A page with navigation

```rust,no_run
use foliage::{
    Area, Boxed, Divide, Foliage, FontSize, Grid, Grove, Grow, Line, Location, Palette, Panel,
    Place, Point, Pollen, Root, Source, Text, Trace, anchor, bottom, content, left, top,
};

const ABOUT: &str = "The navigation is a bar along the bottom on a phone and a rail down the left \
                     on a desk. Nothing in the app decides which: every element says where it \
                     sits at each breakpoint, and the viewport picks.";

struct Page;

impl Root for Page {
    fn take_root(grove: &mut Grove) -> Self {
        let page = grove.plant(
            Panel::new().grid(
                Grid::new()
                    .xs(4.columns().gap(12.0), 1.rows())
                    .md(12.columns().gap(16.0), 1.rows()),
            ),
        );
        // A bar along the bottom, then a rail three columns wide.
        let nav = grove.branch(
            page,
            Panel::new().color(Palette::Raised).at(
                Location::new()
                    .xs(
                        left(0.px()).right(100.pct()),
                        bottom(100.pct()).height(56.px()),
                    )
                    .md(left(0.px()).width(3.col()), top(0.px()).bottom(100.pct())),
            ),
        );
        // A rule along the navigation's inner edge: its top as a bar, its right as a rail.
        grove.branch(
            page,
            Line::new().color(Palette::Muted).anchored(nav).trace(
                Trace::new()
                    .xs(
                        Point::new(anchor().left(), anchor().top()),
                        Point::new(anchor().right(), anchor().top()),
                    )
                    .md(
                        Point::new(anchor().right(), anchor().top()),
                        Point::new(anchor().right(), anchor().bottom()),
                    ),
            ),
        );
        // The content, full width on a phone and to the right of the rail on a desk.
        grove.branch(
            page,
            Text::new(ABOUT)
                .font_size(FontSize::new().xs(14).lg(16).short(12))
                .at(
                    Location::new()
                        .xs(
                            left(16.px()).right(100.pct() - 16.px()),
                            top(16.px()).height(content()),
                        )
                        .md(left(4.col()).right(12.col()), top(24.px()).height(content())),
                ),
        );
        Page
    }

    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        if let Some(size) = pollen.resized() {
            println!("{} wide, at {:?}", size.width, grove.layout());
        }
    }
}

fn main() {
    let mut foliage = Foliage::new();
    foliage.title("responsive");
    foliage.desktop_size(Area::new(390.0, 700.0));
    foliage.root::<Page>();
    foliage.photosynthesize();
}
```

Open it narrow and drag the window wider. At 600 pixels the grid changes from four columns to
twelve, the bar becomes a rail, the rule turns from horizontal to vertical, and the text moves to
the right of the rail and re-wraps at its new width. Drag the window short and the text drops a
size.

Three details in it:

- **The rule is placed against the navigation**, not against the grid. A point reads an anchor
  exactly as a box does, so at each breakpoint the rule names the edge of the navigation it runs
  along, and it stays on that edge however the navigation was placed. Stating it in columns
  instead would work, but a column in a *point* is always the track's near edge (a point is a
  position, and a position is what a near role reads), while the rail's right edge is a far edge.
  The two differ by a gap.
- **`bottom(100.pct())` is the trunk's bottom edge.** An edge role is a coordinate, so the bar's
  bottom sits on the page's bottom and its height grows upward from there.
- **Content right of the rail starts at `4.col()`**: the near edge of column 4, one gap past the
  rail's far edge at `3.col()`. The gutter between them is the grid's own gap.

## Font sizes

`FontSize` is the same chain:

```rust
# use foliage::FontSize;
FontSize::new().xs(14).lg(18).short(12);
```

A size is stated on *any* element, not only one that draws text, because a font and a size are
what give an element a character cell, and a cell is what `letters` and a letter-pitched grid are
measured in. An element that states neither has no cell at all.

## Reading the breakpoint

Most of the time an app never asks: every element says what it does at each breakpoint and the
viewport decides. Where an app does need to know (to grow a different set of parts on a phone, or
to choose between two behaviours) it reads the frame:

```rust
# use foliage::{Grove, Layout, Pollen, Short};
# fn f(grove: &Grove, pollen: &Pollen) {
match grove.layout() {
    Layout::Xs | Layout::Sm => { /* narrow */ }
    _ => { /* wide */ }
}
if grove.short() == Short::Yes { /* cramped */ }
if let Some(size) = pollen.resized() {
    // The surface's new size, reported once, in the frame it changed.
    let _ = (size.width, size.height, grove.viewport());
}
# }
```

`pollen.resized()` is reported on the frame the new size took effect, including the first frame,
which reports the size the platform actually gave the window.

## What a resize costs

The viewport, the breakpoint and the short reading are read by every placement in the tree, so a
change to any of them is a write to every element: the whole tree is resolved again. Ordinary
frames resolve only what was written to and what depends on it, so a resize is the most expensive
thing that routinely happens. Dragging a window's edge on a desktop does it every frame of the
drag. [What a frame costs](../architecture/performance.md) has the numbers.
