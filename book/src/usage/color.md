# Color

An element does not usually name a color. It names a **tone**, which is what the color is *for*,
and a [`Scheme`](https://eblack-leaf.github.io/foliage/api/foliage/struct.Scheme.html) decides
what every tone resolves to. Changing the scheme changes every element carrying an affected tone,
with one op. This chapter builds a page with a light/dark toggle and a pressed state, and covers
tones, schemes and the difference between the two kinds of fill.

## Fills

Everything that can be filled takes a [`Fill`](https://eblack-leaf.github.io/foliage/api/foliage/enum.Fill.html),
which is one of two things:

- a **tone**, a [`Palette`](https://eblack-leaf.github.io/foliage/api/foliage/struct.Palette.html)
  value such as `Palette::Accent`, resolved against whichever scheme is in force when it is drawn;
- a **literal**, a [`Color`](https://eblack-leaf.github.io/foliage/api/foliage/struct.Color.html)
  stated outright, which no scheme moves.

Both convert into a `Fill`, so every `color(..)` takes either:

```rust
# use foliage::{Color, Palette, Panel};
Panel::new().color(Palette::Raised);             // part of the scheme
Panel::new().color(Color::rgb(0.9, 0.35, 0.42)); // not
```

A literal is an element saying it is not part of the scheme, and it is meant to be visible as one.
A picture of a swatch, a chart series coloured by data, a brand mark: those are literals. Anything
that is surface, text, emphasis or state is a tone.

## Tones

A tone is a **role**, a **form**, and a **step**.

### Roles

Four roles are neutral:

| Role | What it is for |
|---|---|
| `Surface` | The ordinary fill, and what an element that says nothing takes. Also what the window is cleared to |
| `Raised` | A surface in front of another: a card against the page |
| `Muted` | A quieter fill, for a division or a rule |
| `Ink` | What is read against a surface |

The rest are hues:

| Role | What it is for |
|---|---|
| `Accent` | The emphatic hue, and what the app's own content is marked with |
| `Signal` | The second hue: what reports the system's own state rather than the app's content |
| `Danger` | What cannot be taken back, and what went wrong |
| `Caution` | What wants care before it is done |
| `Positive` | What went right, or was chosen |
| `Palette::hue(0..4)` | Numbered slots, for an app to name for itself |

A slot is named once and used by name:

```rust
# use foliage::Palette;
const MOSS: Palette = Palette::hue(0);
```

A slot no scheme has seeded answers as `Accent` does, so a forgotten one is plainly the accent
rather than black.

### Forms

One color cannot do the three jobs a hue is given, so each hue is held in three forms:

| Form | Written | What it is |
|---|---|---|
| ground | `Palette::Accent` | what is filled with the hue |
| mark | `Palette::Accent.mark()` | the hue carried on a neutral ground: a word set apart, a rule, an icon |
| on | `Palette::Accent.on()` | what is read on the ground form |

The neutrals answer the forms too. `Surface.on()` and `Surface.mark()` are `Ink`, and `Ink.on()`
is `Surface`, so `on()` names the partner of anything. `Palette::Contrast` is `Accent.on()`, kept
by an older name.

### Steps

Every form of every role is a ramp of five steps:
[`Step`](https://eblack-leaf.github.io/foliage/api/foliage/enum.Step.html)`::Farthest`, `Far`,
`Base`, `Near` and `Nearest`. A role names its own base step, so `Palette::Accent` is a tone in its
own right, and `at`, `recede` and `advance` move along the ramp:

```rust
# use foliage::{Palette, Step};
Palette::Accent.at(Step::Near);
Palette::Accent.advance(); // one step nearer: Base to Near
Palette::Ink.recede();     // one step farther: Base to Far
```

A step is named for where it stands relative to the ground rather than for which way it moves in
lightness. Against a dark ground, nearer is lighter; against a light one, nearer is darker. So a
state written once as `advance()` is correct in both readings, and switching a scheme from dark to
light needs no element to be written.

**States are steps.** Pressed, held, disabled, placeholding: each is a step on the tone's own
ramp rather than a color picked beside it, which is what lets it survive a repaint without being
restated. There is no hover, because a pointer that is not pressed reports nothing to the engine.

## A page with a theme toggle

```rust,no_run
use foliage::{
    Area, Boxed, Color, Foliage, FontSize, Grove, Grow, Leaf, Location, Palette, Panel, Place,
    Pollen, Polygon, Root, Rounding, Scheme, Source, Text, center_x, center_y, content, left, top,
};

struct Themes {
    toggle: Leaf,
    label: Leaf,
    dark: bool,
}

impl Root for Themes {
    fn take_root(grove: &mut Grove) -> Self {
        let page = grove.plant(Panel::new());
        let card = grove.branch(
            page,
            Panel::new()
                .color(Palette::Raised)
                .rounding(Rounding::Md)
                .at(Location::new().xs(
                    left(16.px()).right(100.pct() - 16.px()),
                    top(16.px()).height(160.px()),
                )),
        );
        grove.branch(
            card,
            Text::new("ink on a raised ground, and a word in the accent")
                .font_size(FontSize::new().xs(14))
                .tint(42..48, Palette::Accent.mark())
                .at(Location::new().xs(
                    left(16.px()).right(100.pct() - 16.px()),
                    top(16.px()).height(content()),
                )),
        );
        // A literal. It is the same color in every scheme.
        grove.branch(
            card,
            Polygon::circle()
                .color(Color::rgb(0.9, 0.35, 0.42))
                .at(Location::new().xs(
                    left(16.px()).width(24.px()),
                    top(100.pct() - 40.px()).height(24.px()),
                )),
        );
        let toggle = grove.branch(
            page,
            Panel::new()
                .color(Palette::Accent)
                .rounding(Rounding::Sm)
                .interactive()
                .at(Location::new().xs(
                    left(16.px()).width(120.px()),
                    top(192.px()).height(40.px()),
                )),
        );
        let label = grove.branch(
            toggle,
            Text::new("light")
                .color(Palette::Accent.on())
                .font_size(FontSize::new().xs(14))
                .intangible()
                .at(Location::new().xs(
                    center_x(50.pct()).width(content()),
                    center_y(50.pct()).height(content()),
                )),
        );
        Themes {
            toggle,
            label,
            dark: true,
        }
    }

    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        // A pressed state is a step, so it is right in either scheme.
        if pollen.engaged(self.toggle) {
            grove.color(self.toggle, Palette::Accent.advance());
        }
        if pollen.disengaged(self.toggle) {
            grove.color(self.toggle, Palette::Accent);
        }
        if pollen.clicked(self.toggle) {
            self.dark = !self.dark;
            let (scheme, next) = match self.dark {
                true => (Scheme::new(), "light"),
                false => (Scheme::light(), "dark"),
            };
            // One op, naming no element.
            grove.repaint(scheme);
            grove.text(self.label, next);
        }
    }
}

fn main() {
    let mut foliage = Foliage::new();
    foliage.title("themes");
    foliage.desktop_size(Area::new(360.0, 260.0));
    foliage.root::<Themes>();
    foliage.photosynthesize();
}
```

Press the toggle. The window's ground, the card, the text, the accent word and the button all
move to the light reading together. The red circle does not, because it was stated outright.

`engaged` and `disengaged` bracket every gesture on the button, so the pressed step goes on when a
finger lands and comes off however the gesture ended: released, dragged away, or cancelled. A
press and release inside one frame report all three of `engaged`, `clicked` and `disengaged`, and
the writes land in the order they were made, so the button ends the frame at its base step.

## Schemes

A scheme holds a ramp for every form of every role. It is stated in very few colors:

- `Scheme::new()` is a dark reading with a green accent, and `Scheme::light()` is a light one.
- `set(tone, color)` at a tone's base step **seeds** its ramp: the color is written at `Base`
  exactly, and the other four steps are derived from it in OKLab, holding its hue and chroma and
  moving only its lightness. A step that would leave sRGB has its chroma backed off until it fits.
- Seeding a hue's **ground** also derives its mark (the hue at text lightness) and what is on it
  (near-black or near-white, whichever the ground is not, keeping a tint of the hue), unless either
  was seeded outright.
- `set` at any other step replaces that one step and nothing else, which is the way out when a
  derived step is not the one wanted.

So a theme is a handful of decisions:

```rust
# use foliage::{Color, Palette, Scheme};
const MOSS: Palette = Palette::hue(0);

let scheme = Scheme::new()
    .set(Palette::Accent, Color::rgb(0.93, 0.52, 0.17))
    // The derived mark is fine for most hues; this one wants to be warmer.
    .set(Palette::Accent.mark(), Color::rgb(0.98, 0.66, 0.36))
    .set(Palette::Positive, Color::rgb(0.14, 0.36, 0.17))
    .set(MOSS, Color::rgb(0.45, 0.55, 0.30));
```

It takes effect when it is written with `grove.repaint(scheme)`, usually first thing in
`take_root`. `repaint` is the one write that names no element, and like every write it lands at
the drain: `grove.scheme()` answers what the frame was drawn in, not what a repaint queued this
frame will make it.

## Grounds and what is read on them

A ramp reaches two notches either side of its seed. The distance from a fill to something legible
on that fill is several times that, so **every seed is either a ground or a mark, and no step moves
one into the other**. That is arithmetic rather than convention, and it is why the tones come in
pairs:

| Ground | Read against it |
|---|---|
| `Surface`, `Raised`, `Muted` | `Ink`, or any hue's `mark()` |
| any hue | that hue's `on()` |

Nothing enforces this. A tone is an index into a table, and text lettered in `Muted` on a `Muted`
panel draws exactly as asked, which is to say invisibly. The failure the pairs prevent is silent,
because an illegible tone still renders.

## Colors an app computes

Some colors are not one tone: a gradient across a row of elements, a colour interpolated from a
value. A scheme holds up to four numbered **spectra** for these, so that every color an app draws
from is stated in one place:

```rust
# use foliage::{Color, Scheme};
let scheme = Scheme::new().spectrum(
    0,
    &[
        Color::rgb(0.97, 0.76, 0.30),
        Color::rgb(0.82, 0.28, 0.13),
        Color::rgb(0.58, 0.12, 0.11),
    ],
);
let stops = scheme.stops(0);
// Half way along the first two stops, through OKLab rather than channel by channel.
let middle = stops[0].toward(stops[1], 0.5);
# let _ = middle;
```

Nothing in foliage reads a spectrum; they are for the app. A spectrum no one stated reads as the
accent's ramp, farthest to nearest.

Colors computed this way are literals, so a repaint does not move them. `pollen.repainted()` says
the scheme was replaced, which is the frame to compute them again from `grove.scheme()`:

```rust
# use foliage::{Grove, Grow, Leaf, Pollen};
# fn f(grove: &mut Grove, pollen: &Pollen, cells: &[Leaf]) {
if pollen.repainted() {
    let stops = grove.scheme().stops(0);
    let last = (cells.len().max(2) - 1) as f32;
    for (n, cell) in cells.iter().enumerate() {
        grove.color(*cell, stops[0].toward(stops[stops.len() - 1], n as f32 / last));
    }
}
# }
```

`Color::toward` and `Color::lightened` both work in OKLab, so equal fractions read as equal
distances and the middle of two saturated colors is not a grey between them.

## Animating a fill

Anything that can be filled can be animated to a fill, either kind:

```rust
# use foliage::{Color, Grove, Grow, Leaf, Motion, Palette, Timing};
# fn f(grove: &mut Grove, card: Leaf) {
grove.animate(card, Motion::Palette(Palette::Raised.advance()), Timing::ms(180));
grove.animate(card, Motion::Color(Color::rgb(0.2, 0.2, 0.25)), Timing::ms(180));
# }
```

A motion toward a tone resolves the tone every frame, so a repaint in the middle of the motion
moves the motion too. [Motion](motion.md) covers the rest.
