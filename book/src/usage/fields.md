# Text fields

[`TextInput`](https://eblack-leaf.github.io/foliage/api/foliage/struct.TextInput.html) is an
editable line, and [`TextArea`](https://eblack-leaf.github.io/foliage/api/foliage/struct.TextArea.html)
is the same with as many lines as its value wraps to. Each is several elements under the hood (a
run of text, a placeholder, a caret, a selection), and each is **one `Leaf`** to the app: every
verb and every read is addressed to the field, and what it is made of is never something the app
has to keep in step. This chapter builds a list that filters as a name is typed.

## A filter

```rust,no_run
use foliage::{
    Area, Boxed, Foliage, FontSize, Grove, Grow, Leaf, Location, Palette, Panel, Place, Pollen,
    Root, Rounding, Sap, Source, Text, TextInput, Vein, content, left, right, top,
};

const NAMES: [&str; 8] = [
    "ash", "aspen", "elm", "fern", "ginkgo", "lichen", "rowan", "willow",
];

struct Filter {
    field: Leaf,
    rows: Vec<(Leaf, &'static str)>,
    count: Leaf,
}

/// Where the `n`th row that is showing stands.
fn row_at(n: usize) -> Location {
    Location::new().xs(
        left(28.px()).width(content()),
        top((80.0 + n as f32 * 28.0).px()).height(content()),
    )
}

impl Root for Filter {
    fn take_root(grove: &mut Grove) -> Self {
        let page = grove.plant(Panel::new());
        // A field draws no ground or border of its own; it sits in whatever box it is given.
        let well = grove.branch(
            page,
            Panel::new()
                .color(Palette::Raised)
                .rounding(Rounding::Sm)
                .at(Location::new().xs(
                    left(16.px()).right(100.pct() - 16.px()),
                    top(16.px()).height(40.px()),
                )),
        );
        let field = grove.branch(
            well,
            TextInput::new()
                .placeholder("filter")
                .font_size(FontSize::new().xs(15))
                .at(Location::new().xs(
                    left(12.px()).right(100.pct() - 12.px()),
                    top(0.px()).bottom(100.pct()),
                )),
        );
        let rows = NAMES
            .iter()
            .enumerate()
            .map(|(n, name)| {
                let row = grove.branch(page, Text::new(*name).at(row_at(n)));
                (row, *name)
            })
            .collect();
        let count = grove.branch(
            page,
            Text::new(format!("{} of {}", NAMES.len(), NAMES.len()))
                .color(Palette::Ink.recede())
                .font_size(FontSize::new().xs(12))
                .at(Location::new().xs(
                    right(100.pct() - 16.px()).width(content()),
                    top(62.px()).height(content()),
                )),
        );
        grove.focus(field);
        Filter { field, rows, count }
    }

    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        if pollen.edited(self.field) {
            let Some(Sap::Text(query)) = grove.tap(self.field, Vein::Text) else {
                return;
            };
            let mut showing = 0;
            for (row, name) in &self.rows {
                let matched = name.contains(query.trim());
                grove.visible(*row, matched);
                if matched {
                    grove.at(*row, row_at(showing));
                    showing += 1;
                }
            }
            grove.text(self.count, format!("{showing} of {}", self.rows.len()));
        }
        if pollen.submitted(self.field) {
            // Enter clears it. An app's own write is not reported back as `edited`.
            grove.text(self.field, "");
            for (n, (row, _)) in self.rows.iter().enumerate() {
                grove.visible(*row, true);
                grove.at(*row, row_at(n));
            }
            grove.text(self.count, format!("{} of {}", self.rows.len(), self.rows.len()));
        }
    }
}

fn main() {
    let mut foliage = Foliage::new();
    foliage.title("filter");
    foliage.desktop_size(Area::new(320.0, 360.0));
    foliage.root::<Filter>();
    foliage.photosynthesize();
}
```

The field has focus from the first frame, so typing goes straight into it. Typing is reported as
`edited` (once per frame, however many keys it took), the app reads the value back, hides the rows
that do not match and moves the rest up. Enter submits, and the app clears it.

## What a field does by itself

A field answers the keyboard and the pointer without any app code:

| Input | Does |
|---|---|
| typing | inserts at the caret, replacing any selection |
| `Backspace`, `Delete` | removes the selection, or the character before or after the caret |
| `Left`, `Right`, `Home`, `End` | moves the caret; with `Shift`, extends the selection |
| `Ctrl+A` | selects everything |
| `Ctrl+C`, `Ctrl+X` | copies or cuts the selection to the system clipboard |
| `Ctrl+V` | asks the clipboard, and inserts what it holds when it answers |
| `Enter` | reports `submitted` |
| a tap | puts the caret where it landed and takes focus |
| a drag | scrolls the value inside the field |
| a hold, then a drag | selects from where the hold landed |

An unshifted arrow against a selection collapses it to the edge it points at rather than stepping
from the caret, which is what every editor does.

Dragging to scroll and dragging to select are the same motion, and the only thing that separates
them is time: a field takes no drags, so a plain drag is the field's scrolling, and a press held
still for [`Hold::after`](input.md#feel) first is the start of a selection. That is the same rule
on a phone and a desktop.

The app still hears everything: a field's own keys are reported in `pollen.keys(field)` as well,
alongside the `edited` they produced.

## Reading and writing a field

- `pollen.edited(field)`: the person at the keyboard changed the value (typed, deleted, cut or
  pasted into it). Keys are applied in the drain, so the change is on screen at the end of the
  frame that took the keys and reported in the frame after; the value read then is the one that
  was drawn.
- `pollen.submitted(field)`: `Enter` was pressed in it. What that means is the app's; a field holds
  no opinion about whether anything is to be submitted.
- `grove.tap(field, Vein::Text)`: the value, as `Sap::Text`.
- `grove.tap(field, Vein::Selection)`: the selection, as `Sap::Selection(range)`, in characters of
  the value. An empty range is a caret with nothing selected, standing at the range's position, so
  the caret and the selection are one read rather than two that could disagree.

And the writes:

- `grove.text(field, value)` rewrites the value and leaves the caret at its end. It is **not**
  reported as `edited`: an app that wrote a value already knows what it wrote.
- `grove.select(field, from..to)` selects a span. The selection is anchored at `from` and the caret
  goes to `to`, so a range whose end comes before its start is a selection reaching backwards, and
  an empty range just places the caret. A range past the end is held to the end.
- `grove.color(field, fill)` refills the value's text. The placeholder, the caret and the selection
  keep the fills they were grown with.

A useful combination: selecting the whole value whenever the field takes focus, so that typing
replaces it.

```rust
# use foliage::{Grove, Grow, Leaf, Pollen};
# fn f(grove: &mut Grove, pollen: &Pollen, field: Leaf) {
if pollen.focused(field) {
    grove.select(field, 0..usize::MAX);
}
# }
```

It works when focus arrives by a tap too, even though the tap places a caret: the tap's own
`select` is queued before the app's frame runs, and the app's is queued after it, so the app's
lands last.

## Building one

```rust
# use foliage::{FontSize, Keypad, Palette, Place, TextInput};
TextInput::new()
    .value("")                        // what it starts out saying
    .placeholder("quantity")          // read in the field's place while it is empty
    .color(Palette::Ink)              // the value
    .hint(Palette::Muted)             // the placeholder
    .caret(Palette::Accent)           // the caret
    .selection(Palette::Muted)        // what is drawn behind a selected span
    .keypad(Keypad::Number)           // which soft keyboard to raise
    .read_only(false)
    .font_size(FontSize::new().xs(15));
```

Those are the defaults, apart from the placeholder, the keypad and the size.

A field is a scrolling region along the axis its value grows on, which is what clips a value longer
than its box and what keeps the caret in view: every edit asks the field to show the caret, and
the frame that typed past the edge is the frame that scrolls.

**It draws no border and no ground.** A field is a run and a caret inside whatever box an app puts
it in, and a chrome the engine drew would be one more thing to talk an app out of. It draws no
focus ring either: the caret and the selection are shown only while it holds focus, and that is
the only change focus makes to it. An app that wants a ring watches `focused` and `unfocused`, as
the slider in the last chapter does.

## Many lines

A `TextArea` wraps at its box and scrolls down rather than across. Everything above applies,
and four keys change meaning because a line means something:

| Key | On a `TextArea` |
|---|---|
| `Enter` | inserts a newline |
| `Ctrl+Enter` | reports `submitted` |
| `Up`, `Down` | move the caret a line, to the same column or the end of a shorter line |
| `Home`, `End` | go to the ends of the line the caret is on, as it wrapped |

A newline that arrives on the clipboard is kept, where a one-line field drops it.

```rust
# use foliage::{Boxed, FontSize, Grove, Grow, Leaf, Location, Palette, Place, Pollen, Sap, Source, Text, TextArea, Vein, content, left, top};
# fn f(grove: &mut Grove, page: Leaf) -> (Leaf, Leaf) {
let notes = grove.branch(
    page,
    TextArea::new()
        .placeholder("notes")
        .font_size(FontSize::new().xs(14))
        .at(Location::new().xs(
            left(16.px()).right(100.pct() - 16.px()),
            top(16.px()).height(200.px()),
        )),
);
let counter = grove.branch(
    page,
    Text::new("0 characters")
        .color(Palette::Ink.recede())
        .font_size(FontSize::new().xs(12))
        .at(Location::new().xs(left(16.px()).width(content()), top(224.px()).height(content()))),
);
# (notes, counter)
# }
# fn frame(grove: &mut Grove, pollen: &Pollen, notes: Leaf, counter: Leaf) {
if pollen.edited(notes) {
    if let Some(Sap::Text(value)) = grove.tap(notes, Vein::Text) {
        grove.text(counter, format!("{} characters", value.chars().count()));
    }
}
# }
```

## Read-only

`TextInput::read_only(true)`, or `grove.read_only(field, true)` later, keeps the value and nothing
else from the person at the keyboard. The field still receives: a drag scrolls it, a hold selects
in it, and with focus the arrows walk it and `Ctrl+C` copies out of it. Typing, deleting and
pasting do nothing, a cut is only a copy, no caret is drawn, and no soft keyboard is raised.

That is the answer for a value that must be readable but not editable. Disabling the field would
refuse far more: a disabled region takes no gesture, so a long value could not even be scrolled
to be read.

## Soft keyboards

On a platform with a keyboard of its own to raise, a field raises one while it holds focus and
lowers it when it lets go. Nothing is declared for this beyond being a field. `keypad` says which
keys to offer (`Keypad::Text`, `Number` or `Telephone`); it is a hint about what is easy to type
and never a rule about what the field accepts, so a number field can still be pasted a word into
and what a value is allowed to be is the app's to check.

On the web, the browser's keyboard is raised through a hidden input, which hands every key back to
the engine. On Android the activity raises it directly. See [Platforms](platforms.md) for what is
and is not wired up on each.
