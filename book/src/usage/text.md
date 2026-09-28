# Text

A [`Text`](https://eblack-leaf.github.io/foliage/api/foliage/struct.Text.html) is a run of
glyphs. It is the one element whose box can be *measured* rather than declared, and the thing
everything else measured in characters is measured against. This chapter covers fonts, sizing,
wrapping, per-character color, and placing things against a particular character of a run.

## Every font is monospaced

foliage lays text out on a grid of identical cells. A run's width is its character count times the
cell width, a line is a whole number of cells, and `letters` and letter-pitched grid tracks are
counts of cells. A proportional face does not degrade under that model; it silently puts every
column address somewhere it does not belong. So a proportional font is **refused**:

- bytes handed to `Foliage::font` or `Grove::font` outright are checked on the spot, and a
  proportional face panics with a message naming two characters whose advances differ;
- bytes read from a path or a URL are refused without panicking and reported as `missing`, because
  what a file turned out to hold is not something the program stated.

The bundled face is JetBrains Mono NL Medium. It is what an element that names no font composes
in, and it is always there, so there is always a cell to measure.

## Registering a font

```rust
# use foliage::{Font, Foliage, Grove};
# fn boot(foliage: &mut Foliage, bytes: &'static [u8]) {
// At boot, before anything runs:
let italic: Font = foliage.font(bytes);
# }
# fn later(grove: &mut Grove, bytes: &'static [u8]) {
// Or at any frame, which is where an app that takes root usually does it:
let italic: Font = grove.font(bytes);
# }
```

A [`Font`](https://eblack-leaf.github.io/foliage/api/foliage/struct.Font.html) is a name and
nothing else. An element chooses one with `font(..)`, and a size with `font_size(..)`:

```rust
# use foliage::{Font, FontSize, Place, Text};
# fn f(italic: Font) {
Text::new("an aside")
    .font(italic)
    .font_size(FontSize::new().xs(13).lg(15));
# }
```

A size is in logical pixels, per breakpoint, and defaults to `FontSize::DEFAULT` (16). The cell
is the face's advance and line height at that size, each rounded up to a whole logical pixel, so
that the hundredth column is exactly a hundred cells in.

Both `font` and `font_size` are on every element, not only on text. A font and a size are what
give an element a character cell, and `letters` counts the element's *own* cells, so a panel meant
to be eight characters wide states a size to be eight characters of.

## Sizing text

In a width role, `content()` is the widest the run would like to be: its longest line, unwrapped.
In a height role it is how tall the run came out at the width it was given. Those two combine into
the three shapes text is placed in:

```rust
# use foliage::{Boxed, Location, Source, Text, center_x, content, left, right, top};
// A label: exactly as large as its text.
Text::new("ok").at(Location::new().xs(
    left(16.px()).width(content()),
    top(16.px()).height(content()),
));

// A paragraph: as wide as it is allowed, as tall as it wraps to.
Text::new("a paragraph").at(Location::new().xs(
    left(16.px()).right(100.pct() - 16.px()),
    top(16.px()).height(content()),
));

// A line centred or right-aligned: size the box to the text, and place the box.
Text::new("centred").at(Location::new().xs(
    center_x(50.pct()).width(content()),
    top(16.px()).height(content()),
));
Text::new("right").at(Location::new().xs(
    right(100.pct() - 16.px()).width(content()),
    top(16.px()).height(content()),
));
```

A run has no alignment of its own: its lines start at the left of its box. Centring and right
alignment are placements of a box sized to the text, which is exact for a line and not available
for a paragraph.

Where the count is known ahead of time, `letters` says it directly and costs nothing to resolve:
`width(12.letters())` is twelve cells of the element's own font, whatever it says.

The measure follows the value in the same frame. `grove.text(run, ..)` is drained, shaped,
wrapped and placed before anything is drawn, so a box sized to its text is the right size on the
frame the text changed rather than the frame after, and whatever is anchored below it moves with
it on that frame too.

## Wrapping

A run wraps at the width of its box, on word boundaries, greedily. There are three rules and no
fourth:

- a newline ends a line;
- a word that does not fit in what is left of a line starts the next one, and the spaces before it
  go with the break rather than trailing off the end;
- a word longer than a whole line fills what is left and breaks inside itself, because there is
  no width at which it would fit.

A run with nothing in it takes no lines at all, so an empty element measures to zero rather than
to one line of nothing.

## Color, per character

A run is filled like anything else: `Text::new(..)` defaults to `Palette::Ink`, and `color` takes
a tone or a literal. Part of a run can be filled differently with a **tint**, over a range of the
run's own character indices:

```rust
# use foliage::{Color, Grove, Grow, Leaf, Palette, Text};
# fn f(grove: &mut Grove, line: Leaf) {
Text::new("ok  warn  error")
    .color(Palette::Ink)
    .tint(4..8, Palette::Caution.mark())
    .tint(10..15, Palette::Danger.mark());

// Rewritten later, all at once:
grove.tint(line, [(0..2, Palette::Positive.mark())]);
// And taken off:
grove.untint(line);
# }
```

The indices are **characters**, not bytes, spaces and newlines included: the same space a caret
and a selection are addressed in. Where two tints overlap, the later one wins. `grove.tint`
replaces every tint on the run rather than adding one, for the reason `at` replaces a whole
placement.

A tint is a `Fill`, so a tone follows a repaint and a literal does not. Use a hue's `mark()` form
for a word set apart on a neutral ground; that is what the form is for.

## Placing things against a character

A run knows where each of its characters landed after wrapping, and that is readable from the
grammar. `anchor().character(n)` is where character `n` of the anchor's run stands: its column
across and its line down, each measured in the anchor's cells from the run's top-left. It is exact
on the frame the run wraps differently, because it is read off the wrap the layout just made.

This page highlights a word, and the highlight follows it when the text reflows:

```rust,no_run
use foliage::{
    Area, Boxed, Elevation, Foliage, FontSize, Grove, Grow, Location, Palette, Panel, Place,
    Pollen, Root, Rounding, Source, Step, Text, anchor, content, left, top,
};

const SAYS: &str = "Every measurement foliage makes is a count of character cells, which is \
                    why every font it draws is monospaced.";

struct Highlight;

impl Root for Highlight {
    fn take_root(grove: &mut Grove) -> Self {
        let page = grove.plant(Panel::new());
        let run = grove.branch(
            page,
            Text::new(SAYS)
                .font_size(FontSize::new().xs(16))
                // In front of the highlight, which is grown after it.
                .elevate(Elevation::up(1))
                .at(Location::new().xs(
                    left(24.px()).right(100.pct() - 24.px()),
                    top(24.px()).height(content()),
                )),
        );
        let word = "character";
        let from = SAYS[..SAYS.find(word).unwrap()].chars().count();
        let length = word.chars().count() as f32;
        grove.branch(
            page,
            Panel::new()
                .color(Palette::Caution.at(Step::Farthest))
                .rounding(Rounding::Xs)
                .anchored(run)
                .at(Location::new().xs(
                    left(anchor().left() + anchor().character(from)).width(anchor().letters(length)),
                    top(anchor().top() + anchor().character(from)).height(anchor().letters(1.0)),
                )),
        );
        Highlight
    }

    fn frame(&mut self, _grove: &mut Grove, _pollen: Pollen) {}
}

fn main() {
    let mut foliage = Foliage::new();
    foliage.title("highlight");
    foliage.desktop_size(Area::new(480.0, 240.0));
    foliage.root::<Highlight>();
    foliage.photosynthesize();
}
```

Resize the window until "character" is pushed to the next line: the highlight goes with it on the
same frame.

Three things make it work:

- `anchor().character(from)` is a **length**, measured from the run's own corner, which is why it
  is added to `anchor().left()` and `anchor().top()` rather than used alone. On the horizontal axis
  it is the column times the cell width; on the vertical axis, the line times the cell height.
- `anchor().letters(n)` counts cells of the *anchor's* font, which is what a highlight on its
  text has to be measured in. A bare `letters` would count the panel's own cells, and the panel
  has no font.
- The run is raised one step, so the highlight (grown after it, and so otherwise in front of it)
  sits behind its glyphs.

A highlight across a line break needs one box per line. The text fields do exactly that for their
selection, with three boxes whose sizes come out as zero when a span does not need them; [the
Fronds chapter](../architecture/fronds.md) shows how.

## Reading text back

`grove.tap(run, Vein::Text)` answers what a run says, as `Sap::Text(String)`. It is what was
written, not how it wrapped; where the lines fell is a function of the box, and the box is
`Vein::Drawn`. For a field, the same vein answers the field's value.
