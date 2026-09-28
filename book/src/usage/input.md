# Input and focus

This chapter builds a slider that can be dragged, tapped and driven from the keyboard, then a
dialog that holds focus while it is open. Between them they cover how a gesture finds an element,
what it can turn into, how focus moves, and how keys arrive.

## Two questions

A press raises two questions, and foliage answers them separately:

1. **What is at this point?** Every element that is shown and not clipped away is in the *box
   stack* at the points it covers, whatever it draws and whatever it declared. This is geometry,
   and nothing opts in.
2. **Who receives it?** Only an element that declared
   [`interactive`](https://eblack-leaf.github.io/foliage/api/foliage/trait.Place.html#method.interactive).

A gesture goes to the top of the stack at the point it landed, the element nearest the viewer
there, and what that element declared decides what happens:

| The top element declared | Effect |
|---|---|
| `interactive()` | It receives the gesture, and focus can rest on it |
| `intangible()` | The gesture passes through to whatever is beneath it |
| neither | It **eats** the gesture: nothing receives it |

The engine never searches down the stack for something willing to take a press. A search would
have to decide, at each element it passed, whether that element is part of what it covers or a
layer over it, and at a single point those are the same picture. So the rule is the top, and a
composite marks its own decoration `intangible`. A backdrop behind a dialog, the padding of a
menu, the ground of a sheet: those are the elements that should eat a press, and they do so by
declaring nothing.

Two more declarations shape how an element meets a hand:

- `drags(Axes)` says which drags the element takes. Undeclared, it takes none.
- `round_hit_area()` tests hits against the ellipse inscribed in the box, so a round control does
  not take presses in the corners it does not draw.

Hit-testing runs against **what was drawn in the last frame**. A pointer event was made by a
person looking at the screen, and the screen is the last frame's render; testing against geometry
that has moved since would resolve the gesture against something nobody saw.

## What a gesture becomes

```text
engaged ──▶ resolving ──▶ clicked            (released without becoming anything else)
                 ├──────▶ held ──▶ drag     (still for Hold::after, then moved)
                 └──────▶ drag              (moved past Claim, on an axis it takes)
disengaged: however it ended
```

[`Pollen`](https://eblack-leaf.github.io/foliage/api/foliage/struct.Pollen.html) reports each
stage to the element holding the gesture:

- `engaged(leaf)`: a gesture went down on it and is being held. The hook for a pressed visual.
- `clicked(leaf)`: it ended without ever becoming a drag or a hold. `clicked_at(leaf)` is where it
  *began*, which is the point that was hit-tested and therefore the one certainly on the element.
- `held(leaf)`: it stayed still for `Hold::after` (500ms by default). A hold is never also a
  click. `held_at(leaf)` is where the press landed.
- `drag_started(leaf)`, then `dragged(leaf)` each frame it moves, carrying a
  [`Drag`](https://eblack-leaf.github.io/foliage/api/foliage/struct.Drag.html): where it began,
  where it is now, and how far it moved this frame.
- `disengaged(leaf)`: however it ended. Always somewhere to put a pressed visual back.
- `activated(leaf)`: clicked, *or* sent `Enter` or the space bar while it held focus. The one
  reading of a press a keyboard can make as well as a pointer.

Apart from `engaged`, which says only that something went down, nothing is reported while a
gesture is still deciding what it is, so nothing is ever taken back. A tap is not a click that was
issued early and then retracted when the pointer moved; it is what a gesture that never became
anything else turns out to have been.

### Drags are claimed

A gesture moves past a threshold on one axis (by default 8 pixels down, or 16 across; a tie goes
down, because down is what a hand does without meaning anything by it) and becomes a drag on that
axis. If the element holding it declared `drags` on that axis, it takes the drag. If not, it
**yields**: it hears `disengaged`, and the drag passes to the nearest scrolling region containing
it.

That is what makes a button in a scrolling list behave on touch. Press it and it holds; drag and
the list scrolls; release without moving and it gets a tap. Nothing about the button had to know it
was in a list.

The drag out of a **hold** is the exception: it belongs to whoever took the hold, whichever way it
goes and whatever that element declared. That is how an element scrolls like anything else and
still takes a drag when it wants one, as a text field does to select: it declares no drags, and a
press held still before moving is a selection.

## A slider

```rust,no_run
use foliage::{
    Area, Axes, Boxed, Foliage, Grove, Grow, Key, Leaf, Location, Palette, Panel, Place, Pollen,
    Root, Rounding, Sap, Source, Stem, Vein, center_x, center_y, left, top,
};

const KNOB: f32 = 20.0;

struct Slider {
    /// What takes the gesture and holds focus. Everything drawn on it is decoration.
    hit: Leaf,
    ring: Leaf,
    fill: Leaf,
    knob: Leaf,
    value: f32,
}

impl Slider {
    fn grow(grove: &mut Grove, under: Leaf) -> Self {
        let hit = grove.branch(
            under,
            Stem::new()
                .interactive()
                .drags(Axes::Horizontal)
                .at(Location::new().xs(
                    left(40.px()).right(100.pct() - 40.px()),
                    top(48.px()).height(32.px()),
                )),
        );
        let ring = grove.branch(
            hit,
            Panel::new()
                .color(Palette::Accent.mark())
                .rounding(Rounding::Sm)
                .visible(false)
                .intangible()
                .at(Location::new().xs(
                    left((-8).px()).right(100.pct() + 8.px()),
                    top(0.px()).bottom(100.pct()),
                )),
        );
        let track = grove.branch(
            hit,
            Panel::new()
                .color(Palette::Muted)
                .rounding(Rounding::Full)
                .intangible()
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    center_y(50.pct()).height(6.px()),
                )),
        );
        let fill = grove.branch(
            track,
            Panel::new()
                .color(Palette::Accent)
                .rounding(Rounding::Full)
                .intangible(),
        );
        let knob = grove.branch(
            hit,
            Panel::new()
                .color(Palette::Accent)
                .rounding(Rounding::Full)
                .intangible(),
        );
        let mut slider = Slider {
            hit,
            ring,
            fill,
            knob,
            value: 0.0,
        };
        slider.set(grove, 0.5);
        slider
    }

    fn set(&mut self, grove: &mut Grove, value: f32) {
        self.value = value.clamp(0.0, 1.0);
        let at = self.value * 100.0;
        grove.at(
            self.fill,
            Location::new().xs(left(0.px()).width(at.pct()), top(0.px()).bottom(100.pct())),
        );
        grove.at(
            self.knob,
            Location::new().xs(
                center_x(at.pct()).width(KNOB.px()),
                center_y(50.pct()).height(KNOB.px()),
            ),
        );
    }

    /// The value a point on the surface stands for, against where the slider was drawn.
    fn value_at(&self, grove: &Grove, x: f32) -> Option<f32> {
        let Some(Sap::Section(drawn)) = grove.tap(self.hit, Vein::Drawn) else {
            return None;
        };
        Some((x - drawn.left()) / drawn.width())
    }

    fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) {
        let pointed = pollen
            .dragged(self.hit)
            .map(|drag| drag.current)
            .or(pollen.clicked_at(self.hit));
        if let Some(value) = pointed.and_then(|at| self.value_at(grove, at.x)) {
            self.set(grove, value);
        }
        for stroke in pollen.keys(self.hit) {
            let step = if stroke.modifiers.shift { 0.2 } else { 0.05 };
            match stroke.key {
                Key::Left => self.set(grove, self.value - step),
                Key::Right => self.set(grove, self.value + step),
                Key::Home => self.set(grove, 0.0),
                Key::End => self.set(grove, 1.0),
                _ => {}
            }
        }
        if pollen.focused(self.hit) {
            grove.visible(self.ring, true);
        }
        if pollen.unfocused(self.hit) {
            grove.visible(self.ring, false);
        }
    }
}

struct App {
    slider: Slider,
}

impl Root for App {
    fn take_root(grove: &mut Grove) -> Self {
        let page = grove.plant(Panel::new());
        App {
            slider: Slider::grow(grove, page),
        }
    }

    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        self.slider.frame(grove, &pollen);
    }
}

fn main() {
    let mut foliage = Foliage::new();
    foliage.title("slider");
    foliage.desktop_size(Area::new(420.0, 160.0));
    foliage.root::<App>();
    foliage.photosynthesize();
}
```

Tap anywhere along it and the knob jumps there. Drag and it follows. Drag *down* instead and it
does nothing: the slider takes drags across only, so a vertical drag yields to whatever scrolls
around it, which here is nothing. Press `Tab` and the ring appears; the arrow keys move it, with
shift for bigger steps.

What to notice:

- **One element receives, and everything on it is `intangible`.** The ring, the track, the fill
  and the knob are all on top of the hit area at the pixels they cover. Without `intangible` each
  would eat any press that landed on it, and the slider would only work in the gaps.
- **The hit area is larger than the track.** A six-pixel line is hard to hit with a finger. The
  `Stem` is 32 pixels tall and draws nothing, so the target is as large as the hand needs while
  the track is as thin as the design wants.
- **A point is read against `Vein::Drawn`**, the box the slider was drawn at. Pointer positions
  are in the same logical pixels, on the same surface, so the arithmetic is a subtraction.
- **The slider is a struct with its own `frame`.** The app holds it and forwards the pollen.
  [Composing parts](components.md) builds on that shape.

## Focus

Focus is where keys go, and it rests only on something that declared `interactive`: the set that
asked to receive input is the set a keyboard should be able to reach, so there is no second
declaration to keep in step.

It moves in these ways and no others:

- **A tap** on something interactive moves focus to it. A tap that lands on nothing that receives
  takes focus away.
- **`Tab` and `Shift+Tab`** step to the next and previous element in focus order, and **`Escape`**
  takes focus away. The engine answers these three itself, wherever focus is (including nowhere),
  and they are not reported as keys.
- **The verbs**: `grove.focus(leaf)`, `grove.unfocus()`, `grove.focus_next()` and
  `grove.focus_previous()`.

A drag and a hold move focus nowhere, because neither was a tap. And when whatever holds focus is
hidden, disabled or pruned, focus lets go of it at the end of that frame's drain.

`Pollen::focused` and `Pollen::unfocused` report focus arriving and leaving, and `Grove::focused`
says what holds it now. foliage **draws nothing** for focus. A focused element may be a `Stem`
with nothing visible at all, so there is no mark the engine could draw that would be right; the
slider's ring is the app's.

### Order and scope

Focus order is **reading order**: top to bottom, then left to right, by where elements were drawn.
`focus_order(n)` pulls one element earlier (negative) or later (positive) where a layout's meaning
differs from its geometry, and everything sharing a value keeps reading order among itself.

`focus_scope()` makes focus cycle inside an element while it is in there. It is what a dialog or a
drawer declares, so that stepping through one does not walk off into the page behind it:

```rust
# use foliage::{Boxed, Elevation, Grove, Grow, Leaf, Location, Palette, Panel, Place, Rounding, Source, center_x, center_y, left, top};
# fn open(grove: &mut Grove, page: Leaf) -> (Leaf, Leaf, Leaf) {
// The page goes inert: it still draws, and swallows every press, without a scrim to arrange.
grove.disable(page);
let dialog = grove.plant(
    Panel::new()
        .color(Palette::Raised)
        .rounding(Rounding::Md)
        .elevate(Elevation::up(10))
        .focus_scope()
        .at(Location::new().xs(
            center_x(50.pct()).width(280.px()),
            center_y(50.pct()).height(160.px()),
        )),
);
let keep = grove.branch(
    dialog,
    Panel::new().color(Palette::Muted).interactive().at(Location::new().xs(
        left(16.px()).width(116.px()),
        top(100.pct() - 56.px()).height(40.px()),
    )),
);
let discard = grove.branch(
    dialog,
    Panel::new().color(Palette::Danger).interactive().at(Location::new().xs(
        left(148.px()).width(116.px()),
        top(100.pct() - 56.px()).height(40.px()),
    )),
);
// Focus can go into what was grown in the same frame: the drain grows it first.
grove.focus(keep);
# (dialog, keep, discard)
# }
```

Tab now cycles between the two buttons and never leaves the dialog. Closing it is the reverse:
`grove.prune(dialog)` and `grove.enable(page)`.

`Escape` takes focus away rather than being reported, so a dialog that should close on `Escape`
watches for focus leaving it: `pollen.unfocused(keep)` in a frame where `grove.focused()` is
`None`.

## Keys

A key goes to whatever holds focus:

```rust
# use foliage::{Grove, Grow, Key, Leaf, Pollen};
# fn f(grove: &mut Grove, pollen: &Pollen, list: Leaf) {
for stroke in pollen.keys(list) {
    match stroke.key {
        Key::Down => grove.focus_next(),
        Key::Up => grove.focus_previous(),
        Key::Typed('n') if stroke.modifiers.control => { /* a chord */ }
        _ => {}
    }
}
# }
```

`keys(leaf)` is what that element was sent, **in the order it arrived**. It is the one ordered
thing in `Pollen`, because two keys in a frame mean different things in each order. Everything
else there is a set.

**A key is reported a frame after a tap would be.** A tap is decided at dispatch: it is reported
in the frame that took it, and the focus it moves has moved before `frame` runs. A key is decided
by whatever holds focus at that moment too, but what it *does* is an op: it is queued at dispatch,
ahead of anything the app writes that frame, and applied in the drain, which is where a field
types it. So a key's effect is on screen at the end of the frame that took it, and `keys` and
`edited` report it in the next. That keeps every change on one road, in arrival order. What it
asks of an app is only this: a click and the typing just before it can reach the app in different
frames, so a field that stands for one thing should not be repointed at another on a click. The
[notes walkthrough](notes.md#one-area-per-note) shows the shape that makes this impossible to get
wrong.

A [`Keystroke`](https://eblack-leaf.github.io/foliage/api/foliage/struct.Keystroke.html) is a
[`Key`](https://eblack-leaf.github.io/foliage/api/foliage/enum.Key.html) and the
[`Modifiers`](https://eblack-leaf.github.io/foliage/api/foliage/struct.Modifiers.html) held with
it. `Key::Typed(char)` is a character the platform's layout produced, taken as itself; dead keys,
compose sequences and layouts are all answered before it arrives. The other keys are the few that
mean something rather than say something: the arrows, `Home`, `End`, `Backspace`, `Delete` and
`Enter`. `Key` is non-exhaustive, so a match needs a fallback arm.

`Modifiers` carries two flags, `shift` and `control`, and `control` is the Control key. The one
exception is a browser while a field holds focus: keys are read there through a hidden input (see
[Platforms](platforms.md#web)), which counts the Command key as `control` too, so that a field's
`Cmd+C` copies on a Mac.

A key that arrives while focus rests nowhere is not dropped. It is reported to the app itself, as
`pollen.root_keys()`, which is where a shortcut that is about the whole app belongs. A key is
reported either to an element or to the root, never both.

## Off, three ways

An element can be off in three ways, and they are different on purpose:

| | Draws | In the box stack | Receives |
|---|---|---|---|
| `visible(false)` | no | no | no |
| `disable` | yes | **yes** | no: swallows presses, passes scrolling on |
| `opacity(0.0)` | nothing to see | no | no |

A disabled element still draws and still blocks, which is what makes it different from
decoration, and what makes disabling a page enough on its own when a dialog opens over it. A drag
over a disabled control still scrolls the list it is in, but a disabled region does not scroll.

A fully transparent element is not there at all. That closes the case of an element faded out that
went on taking presses. Anything above zero opacity is there and takes presses normally.

All three are inherited as a product over the whole ancestry, recomputed every frame. An element
grown under a disabled trunk is disabled on its first frame, and enabling the trunk leaves anything
disabled in its own right disabled. `Vein::Visible`, `Vein::Disabled` and `Vein::Opacity` read
what the element itself declared, not the product.

## Feel

How a gesture feels is three numbers, set once for the whole app with `Foliage::tune`:

```rust
# use core::time::Duration;
# use foliage::{Claim, Foliage, Hold, Momentum};
# let mut foliage = Foliage::new();
foliage.tune(Claim { horizontal: 16.0, vertical: 8.0 });
foliage.tune(Hold { after: Duration::from_millis(500) });
foliage.tune(Momentum { half_life: Duration::from_millis(350), minimum: 40.0 });
```

Those are the defaults. `Claim` is how far a gesture travels before it is a drag, per axis;
`Hold` is how long a still press takes to become a hold; `Momentum` is how a released drag coasts
([Scrolling](scrolling.md)). They are global rather than per element, because input feel that
varies from element to element is what makes an app feel unpredictable.

There is one pointer. A second finger is not a second gesture. And a wheel is not a gesture at all:
a notch moves the scrolling region under the pointer and is over.
