# Composing parts

foliage has no widgets. It has six things that draw, a layout, motion and input, and everything
an app calls a button, a card or a switch is assembled from those. This chapter is about making
that assembly reusable: the shape a part takes, the handful of rules that keep parts composable,
and a settings page built from a switch written once.

## The shape of a part

A part is an ordinary Rust value. It grows its elements under a trunk it is handed, keeps the names
it will need, and is carried through each frame by whatever owns it:

```rust,ignore
pub struct Switch { /* the names it writes to */ }

impl Switch {
    /// Grows it under `under`, standing where `at` says.
    pub fn grow(grove: &mut Grove, under: Leaf, at: Location, name: &str) -> Self;
    /// Carries it for a frame, and says what happened.
    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) -> bool;
    /// Verbs the owner can use.
    pub fn set(&mut self, grove: &mut Grove, on: bool);
}
```

Nothing about this is a framework. There is no trait to implement and nothing to register. The
engine has no idea parts exist: it sees elements and ops, as it always does. That is also why a
part costs nothing it does not use.

## A switch

```rust,no_run
use foliage::{
    Area, Boxed, Ease, Foliage, FontSize, Grove, Grow, Leaf, Length, Location, Motion, Palette,
    Panel, Place, Pollen, Root, Rounding, Source, Stem, Text, Timing, center_y, content, left, top,
};

/// A track with a knob on it, and a name beside. The one shape a two-way choice is offered in.
pub struct Switch {
    /// What takes the press and holds focus. Everything drawn on it is decoration.
    hit: Leaf,
    track: Leaf,
    knob: Leaf,
    on: bool,
}

const SIZE: u32 = 14;

impl Switch {
    pub fn grow(grove: &mut Grove, under: Leaf, at: Location, name: &str) -> Self {
        let hit = grove.branch(
            under,
            Stem::new()
                .interactive()
                .font_size(FontSize::new().xs(SIZE))
                .at(at),
        );
        let track = grove.branch(
            hit,
            Panel::new()
                .color(Palette::Muted)
                .rounding(Rounding::Full)
                .intangible()
                .at(Location::new().xs(
                    left(0.px()).width(36.px()),
                    center_y(50.pct()).height(20.px()),
                )),
        );
        let knob = grove.branch(
            track,
            Panel::new()
                .color(Palette::Ink)
                .rounding(Rounding::Full)
                .intangible()
                .at(knob_at(false)),
        );
        grove.branch(
            hit,
            Text::new(name)
                .font_size(FontSize::new().xs(SIZE))
                .intangible()
                .at(Location::new().xs(
                    left(48.px()).width(content()),
                    center_y(50.pct()).height(content()),
                )),
        );
        Switch {
            hit,
            track,
            knob,
            on: false,
        }
    }

    /// How wide a switch named `name` is: the track, the room beside it, and the name. In the
    /// switch's own cells, so it is for the switch's own placement.
    pub fn width(name: &str) -> Length {
        48.px() + (name.chars().count() as f32).letters()
    }

    pub fn on(&self) -> bool {
        self.on
    }

    /// Carries the switch for a frame. Whether it flipped.
    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) -> bool {
        if !pollen.activated(self.hit) {
            return false;
        }
        self.set(grove, !self.on);
        true
    }

    /// Puts it on or off: the knob slides there, and the track lights, or not.
    pub fn set(&mut self, grove: &mut Grove, on: bool) {
        self.on = on;
        let timing = Timing::ms(160).ease(Ease::Emphasis);
        let track = if on { Palette::Accent } else { Palette::Muted };
        grove.animate(self.knob, Motion::Location(knob_at(on)), timing);
        grove.animate(self.track, Motion::Palette(track), timing);
    }

    /// Puts it in reach, or out of it. Out of reach it still stands and still says where it is.
    pub fn reach(&self, grove: &mut Grove, within: bool) {
        match within {
            true => grove.enable(self.hit),
            false => grove.disable(self.hit),
        }
    }
}

fn knob_at(on: bool) -> Location {
    let x = if on { 18.0 } else { 2.0 };
    Location::new().xs(left(x.px()).width(16.px()), center_y(50.pct()).height(16.px()))
}

struct Settings {
    sync: Switch,
    wifi_only: Switch,
}

impl Root for Settings {
    fn take_root(grove: &mut Grove) -> Self {
        let page = grove.plant(Panel::new());
        let row = |n: f32, name: &str| {
            Location::new().xs(
                left(16.px()).width(Switch::width(name)),
                top((16.0 + n * 44.0).px()).height(32.px()),
            )
        };
        let sync = Switch::grow(grove, page, row(0.0, "sync"), "sync");
        let wifi_only = Switch::grow(grove, page, row(1.0, "only on wi-fi"), "only on wi-fi");
        // Only means anything while syncing.
        wifi_only.reach(grove, false);
        Settings { sync, wifi_only }
    }

    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        if self.sync.frame(grove, &pollen) {
            self.wifi_only.reach(grove, self.sync.on());
        }
        self.wifi_only.frame(grove, &pollen);
    }
}

fn main() {
    let mut foliage = Foliage::new();
    foliage.title("settings");
    foliage.desktop_size(Area::new(320.0, 160.0));
    foliage.root::<Settings>();
    foliage.photosynthesize();
}
```

The page is two lines of app code per switch, and the switch is written once. Everything below is
what made that possible.

## Rules that keep parts composable

**One element receives; the rest is decoration.** The switch's `hit` is the only element that
declared `interactive`, and everything drawn on it declared `intangible`. So a press anywhere on the
switch is a press on `hit`, focus rests on one element, and the visual is free to change without
the input changing with it.

**The caller says where it stands.** `grow` takes a `Location` rather than choosing one. Where
things go on a page is the page's business, and a part that placed itself could only ever be used
in the one layout it assumed. What a part *can* say is how large it wants to be, as the switch's
`width` does. That width is stated in `letters` of the switch's own font size, which is why `hit`
declares a font size though it draws no text: `letters` counts the cells of the element it is
written on, and an element with no size has no cells.

**Hold only the names you will use.** The switch keeps three names and throws the label's away.
Names are cheap, but a part that keeps every name it grew is a part whose shape leaks.

**Report what happened; do not hand out the pollen's meaning.** `frame` returns whether the switch
flipped, and the owner never asks the pollen about `hit`. That keeps what counts as a flip (a tap,
or `Enter` or space with focus, which is what `activated` means) inside the part.

**Dress in tones, not colors.** The track is `Muted` or `Accent`, and a state is a step, so a
repaint re-dresses every switch on the page without any of them being written. A part that took
literal colors would have to be told about every theme change.

**Make verbs safe to repeat.** An owner that says the same thing every frame should cost nothing
when it is already true. A write that restates what an element already holds is cheap in the
engine (nothing is placed again, and nothing is sent to the GPU), but a verb that *animates* does
not know whether it is already at its target, and starts a new motion every time. So a part worth
driving from a loop checks first. The chips in [`lichen`](#lichen) remember the state they were
last armed in, and write nothing when asked to stand where they already stand.

**Use the engine's off-states.** `reach` disables the switch rather than hiding it or inventing a
flag. A disabled element still draws, still blocks presses, and still lets a drag over it scroll
the page, and anything under it is disabled with it.

## Helpers generic over the seed

A helper that places *any* element takes a seed type bounded by the traits it needs:

```rust
# use foliage::{Boxed, Grove, Grow, Leaf, Location, Place, Seed, Source, anchor, content, left, top};
/// Grows `seed` under `under`, 12px below `last` if there is one, as wide as its trunk allows and
/// as tall as its content.
fn below<S: Seed + Place + Boxed>(grove: &mut Grove, under: Leaf, last: Option<Leaf>, seed: S) -> Leaf {
    let across = left(16.px()).right(100.pct() - 16.px());
    let placed = match last {
        Some(last) => seed
            .anchored(last)
            .at(Location::new().xs(across, top(anchor().bottom() + 12.px()).height(content()))),
        None => seed.at(Location::new().xs(across, top(16.px()).height(content()))),
    };
    grove.branch(under, placed)
}
```

`Seed` is what `branch` takes, `Place` gives `anchored`, and `Boxed` gives `at`; `Line` is the one
seed that is not `Boxed`, and a helper like this refuses it at compile time. The site's `Stack`
(in [`application/src/parts.rs`](https://github.com/eblack-leaf/foliage/blob/main/application/src/parts.rs))
is this helper with a memory of what it placed last.

## Helpers generic over where the writes go

Every write is a method on the [`Grow`](https://eblack-leaf.github.io/foliage/api/foliage/trait.Grow.html)
trait, and both the `Grove` and a [`Sprig`](off-frame.md) implement it. A helper written against
`Grow` works on either side of the frame:

```rust
# use foliage::{Boxed, FontSize, Grow, Leaf, Location, Place, Source, Text, content, left, top};
fn note<G: Grow>(sink: &mut G, under: Leaf, says: &str, y: f32) -> Leaf {
    sink.branch(
        under,
        Text::new(says)
            .font_size(FontSize::new().xs(13))
            .intangible()
            .at(Location::new().xs(left(16.px()).width(content()), top(y.px()).height(content()))),
    )
}
```

The same function grows a note from `take_root` and from a worker thread, and the result is
identical, because an op is applied the same way whichever side issued it.

## Growing on demand

A part that is not showing yet does not have to exist yet. The site's showcase grows each of its
five themes the first time it is chosen, keeps it, and after that only toggles which one is
`visible`:

```rust
# use foliage::{Boxed, Grove, Grow, Leaf, Location, Place, Stem};
# struct Theme;
# impl Theme { fn grow(_grove: &mut Grove, _page: Leaf) -> Self { Theme } }
struct Tabs {
    details: Leaf,
    pages: Vec<Option<(Leaf, Theme)>>,
    showing: usize,
}

impl Tabs {
    fn show(&mut self, grove: &mut Grove, n: usize) {
        if let Some((page, _)) = &self.pages[self.showing] {
            grove.visible(*page, false);
        }
        match &self.pages[n] {
            Some((page, _)) => grove.visible(*page, true),
            None => {
                let page = grove.branch(self.details, Stem::new().at(Location::new()));
                self.pages[n] = Some((page, Theme::grow(grove, page)));
            }
        }
        self.showing = n;
    }
}
```

Hidden rather than faded or pruned. A hidden subtree keeps its state and its names, draws nothing,
is out of the box stack, and is not content: a scrolling column holding the tabs scrolls only as
far as the tab that is showing reaches. Pruning would lose the state, and fading to zero opacity
would leave it counted in the extent.

## lichen

[`lichen`](https://github.com/eblack-leaf/foliage/tree/main/lichen) is a set of parts on foliage
written this way, and it is also an argument about what a part layer should decide. foliage decides
nothing about how a thing looks. lichen decides a great deal (timings, proportions, how a state is
dressed), so that an app built from it need not, and so that two apps built from it look like
relatives. What an app still chooses is its colors (through the scheme), its density, and where
everything stands.

```toml
lichen = { git = "https://github.com/eblack-leaf/foliage" }
```

```rust
# use foliage::{Elevation, Field, Grove, Leaf, Location, Pollen};
# use lichen::{Chip, Press};
# fn f(grove: &mut Grove, pollen: &Pollen, under: Leaf, at: Location, check: Field) {
let mut save = Chip::grow(grove, under, at, Elevation::up(1), check, "save");
// The state a press stands in; what that looks like is lichen's to decide.
save.arm(grove, Press::Armed);
if save.pressed(pollen) {
    save.arm(grove, Press::Chosen);
}
# }
```

Its chips, switches, fields, badges and pings follow every rule above, and its source is the best
reference for writing more.
