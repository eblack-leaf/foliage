# Motion

`grove.animate(leaf, motion, timing)` moves one property of an element over time, and hands back
a [`Tween`](https://eblack-leaf.github.io/foliage/api/foliage/struct.Tween.html) naming that
motion. This chapter builds a toast that slides up, waits, fades and takes itself down, then
covers the other things the clock is used for: staggered entrances, values the engine has no
concept of, and motion an app drives by hand.

## What can move

[`Motion`](https://eblack-leaf.github.io/foliage/api/foliage/enum.Motion.html) is a closed set:

| `Motion` | Moves |
|---|---|
| `Opacity(f32)` | how opaque the element is |
| `Color(Color)` | the fill, stated outright |
| `Palette(Palette)` | the fill, as a tone, so a repaint in the middle moves the motion |
| `Location(Location)` | where the element sits; both ends re-resolve every frame |
| `Trace(Trace)` | where a line's two ends are |
| `Scroll(ScrollTo)` | where a region is moved to; the destination re-resolves every frame |
| `Polygon(Shape)` | a polygon's sides, rounding and rotation together |

Anything that can be filled converts into a motion toward that fill, so
`Motion::from(Palette::Accent)` and `Palette::Accent.into()` are both `Motion::Palette`.

The list is closed because it is the list of the engine's obligations. Most of it is there because
it cannot be animated from outside: nothing an app holds can resolve a placement against a
breakpoint and an anchor, a tone against the scheme in force, or a destination against an extent
the frame has not measured yet. Everything else (a font size, a count, a value foliage has never
heard of) is a [tween](#values-the-engine-does-not-know).

## The target is written at once

Starting a motion writes its target to the element immediately, and the motion carries what the
element left behind. So an element declares where it is *going* from the moment it is told to go
there, and two things follow without any code:

- **Arriving changes nothing.** The blend at the end is exactly the plain reading of the
  declaration, so there is no settling step that could land a pixel off.
- **Anything that moves the target moves the motion.** A `Location` motion resolves both of its
  ends every frame, in the frame's own context. A resize, a crossed breakpoint or a moved anchor in
  the middle of a motion reaches both ends at once, and it still lands exactly on target.

## A toast

```rust,no_run
use foliage::{
    Area, Boxed, Ease, Foliage, FontSize, Grove, Grow, Leaf, Location, Motion, Palette, Panel,
    Place, Pollen, Root, Rounding, Source, Text, Timing, Tween, center_x, center_y, content, left,
    top,
};

/// Where the toast sits: just below the window, or resting above its bottom edge.
fn toast_at(shown: bool) -> Location {
    let y = match shown {
        true => 100.pct() - 72.px(),
        false => 100.pct() + 8.px(),
    };
    Location::new().xs(left(16.px()).right(100.pct() - 16.px()), top(y).height(56.px()))
}

/// What the toast is doing, and the name of the tween that says when it is done.
enum Stage {
    Arriving(Tween),
    Resting(Tween),
    Leaving(Tween),
}

struct Toast {
    ground: Leaf,
    stage: Stage,
}

struct App {
    page: Leaf,
    button: Leaf,
    toast: Option<Toast>,
}

impl App {
    fn show(&mut self, grove: &mut Grove) {
        let ground = grove.branch(
            self.page,
            Panel::new()
                .color(Palette::Raised)
                .rounding(Rounding::Md)
                .interactive()
                .at(toast_at(false)),
        );
        grove.branch(
            ground,
            Text::new("saved")
                .font_size(FontSize::new().xs(14))
                .intangible()
                .at(Location::new().xs(
                    left(16.px()).width(content()),
                    center_y(50.pct()).height(content()),
                )),
        );
        // Grown below the window and sent up in the same frame.
        let arriving = grove.animate(
            ground,
            Motion::Location(toast_at(true)),
            Timing::ms(260).ease(Ease::Decelerate),
        );
        self.toast = Some(Toast {
            ground,
            stage: Stage::Arriving(arriving),
        });
    }
}

impl Root for App {
    fn take_root(grove: &mut Grove) -> Self {
        let page = grove.plant(Panel::new());
        let button = grove.branch(
            page,
            Panel::new()
                .color(Palette::Accent)
                .rounding(Rounding::Sm)
                .interactive()
                .at(Location::new().xs(left(16.px()).width(120.px()), top(16.px()).height(40.px()))),
        );
        grove.branch(
            button,
            Text::new("save")
                .color(Palette::Accent.on())
                .font_size(FontSize::new().xs(14))
                .intangible()
                .at(Location::new().xs(
                    center_x(50.pct()).width(content()),
                    center_y(50.pct()).height(content()),
                )),
        );
        App {
            page,
            button,
            toast: None,
        }
    }

    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        if pollen.clicked(self.button) && self.toast.is_none() {
            self.show(grove);
        }
        let Some(toast) = &mut self.toast else {
            return;
        };
        // A tap on the toast sends it away now, whatever it was doing.
        let dismissed = pollen.clicked(toast.ground);
        let next = match toast.stage {
            // Still sliding in: the fade starts and the slide carries on, because the two move
            // different properties.
            Stage::Arriving(_) if dismissed => Some(leave(grove, toast.ground)),
            Stage::Resting(timer) if dismissed => {
                grove.stop(timer);
                Some(leave(grove, toast.ground))
            }
            Stage::Arriving(arriving) if pollen.finished(arriving) => {
                Some(Stage::Resting(grove.timer(Timing::ms(2000))))
            }
            Stage::Resting(timer) if pollen.finished(timer) => Some(leave(grove, toast.ground)),
            Stage::Leaving(leaving) if pollen.finished(leaving) => {
                grove.prune(toast.ground);
                self.toast = None;
                return;
            }
            _ => None,
        };
        if let Some(next) = next {
            toast.stage = next;
        }
    }
}

fn leave(grove: &mut Grove, ground: Leaf) -> Stage {
    Stage::Leaving(grove.animate(
        ground,
        Motion::Opacity(0.0),
        Timing::ms(300).ease(Ease::Accelerate),
    ))
}

fn main() {
    let mut foliage = Foliage::new();
    foliage.title("toast");
    foliage.desktop_size(Area::new(360.0, 320.0));
    foliage.root::<App>();
    foliage.photosynthesize();
}
```

Press "save". The toast is grown below the window and slides up; two seconds later it fades out
and is pruned. Tap it while it is up and it leaves at once.

Every step of the chain hangs off the name the step before handed back, and `pollen.finished(name)`
is the frame that step ended on. The app never counts frames or reads a clock.

## Ending a motion

A motion ends in one of these ways:

- **It runs out.** The element reports `pollen.landed(leaf)`, and the motion reports
  `pollen.finished(tween)`. `landed` says the element settled and not which of its properties did;
  `finished` says which motion it was.
- **Another `animate` on the same property replaces it**, starting from wherever the element
  currently is rather than where the first motion began, so nothing jumps back.
- **A direct write to the property cancels it**, and the element is at what was written. `at`
  cancels a `Location` motion (and a `Trace` one, since both are where the element is), `color`
  cancels a fill motion, `opacity` an opacity motion, `reshape` a shape motion, and `scroll` a
  scroll motion. A write to one property leaves a motion on another alone.
- **`grove.stop(tween)`** ends it early and reports nothing, so a chain waiting on it never runs.
- **`grove.finish(tween)`** ends it early and reports it as an arrival, so a chain waiting on it
  runs this frame.
- **Its element withers.**

Only running out and `finish` report `finished`. Stopping, replacing, cancelling and withering are
all ways the name stops meaning anything, and a chain hanging off it quietly never runs, which is
what makes `stop` the way to break a chain.

**Stopping gives up the duration, not the destination.** A motion stopped half way leaves its
element where the motion was going, because that is what the element was told to be when the
motion started; the duration was only how long it was to take. The engine never writes back a
value from part way along, since a blend part way along is something the app never declared. An
element that is to stop somewhere else stops there by being written there.

Because a direct write cancels, a property that is animated in one place does not have to be
animated everywhere. The toast example's dismissal could have written `grove.opacity(ground, 0.0)`
instead of animating, and the arrival motion would simply have been cancelled.

## Timing

```rust
# use foliage::{Ease, Timing};
Timing::ms(240)                        // 240 milliseconds, starting now, unshaped
    .after(60)                         // hold where it is for 60ms first
    .ease(Ease::Decelerate);           // the shape it moves in
```

The duration and the shape are independent: one says when a motion is over, and the other says
where it is on the way. [`Ease`](https://eblack-leaf.github.io/foliage/api/foliage/enum.Ease.html)
has three named shapes for the three things a surface does, and a curve for anything else:

| `Ease` | For |
|---|---|
| `Linear` | the default: constant rate |
| `Decelerate` | something arriving: quick to start, gentle to settle |
| `Accelerate` | something leaving: gentle to start, quick to go |
| `Emphasis` | a change in place: eased at both ends, quick in the middle |
| `Curve { x1, y1, x2, y2 }` | a cubic bezier through two control points |

Every shape starts exactly where the motion started and ends exactly on its target. A `Curve` may
overshoot in between. `Ease::at(fraction)` is public, for anything built on top of a motion that
needs to say where it would be part way through.

A delay is part of the motion, not a queue in front of it: the motion is running from the frame it
was asked for and holds where it was for the delay, so a write that cancels it cancels it whether
or not it has started to move.

## Several at once

A [`Sequence`](https://eblack-leaf.github.io/foliage/api/foliage/struct.Sequence.html) names a
group of tweens so that the *last* of them ending is reported too:

```rust
# use foliage::{Ease, Grove, Grow, Leaf, Motion, Pollen, Timing};
# fn f(grove: &mut Grove, pollen: &Pollen, rows: &[Leaf]) {
// Each row was grown with `.opacity(0.0)`.
let intro = grove.sequence();
for (n, row) in rows.iter().enumerate() {
    grove.animate(
        *row,
        Motion::Opacity(1.0),
        Timing::ms(220).after(n as u64 * 40).ease(Ease::Decelerate).within(intro),
    );
}
// In a later frame:
if pollen.sequence_finished(intro) {
    // every row is in
}
# }
```

A sequence is a name and a count, not a container. Anything on the clock joins one with
`Timing::within`, from any callsite at any frame, which is the point: it times together things
that have no reason to be written together. It is over when nothing is running under it any more,
however each member ended, and it has no reach over its members: stopping one is nothing to the
others. The offsets stay on each tween's `after`, so a delay is stated in one place.

## Values the engine does not know

`grove.tween(from, to, timing)` runs a number from one end to the other and writes it nowhere.
`pollen.tween(name)` reports where it is each frame:

```rust
# use foliage::{Ease, Grove, Grow, Leaf, Pollen, Timing, Tween};
# fn start(grove: &mut Grove) -> Tween {
let counting = grove.tween(0.0, 1280.0, Timing::ms(900).ease(Ease::Decelerate));
# counting
# }
# fn frame(grove: &mut Grove, pollen: &Pollen, counting: Tween, label: Leaf) {
if let Some(value) = pollen.tween(counting) {
    grove.text(label, format!("{}", value.round()));
}
# }
```

The frame a tween ends reports its end value and `finished` together, so the last value never has
to be inferred from its absence. For a motion, `pollen.tween(name)` is the motion's *eased*
progress from `0.0` to `1.0`: how far along it looks, which differs from how much of the duration
has passed under every ease but `Linear`.

`grove.timer(timing)` is a tween whose value nobody reads: what is wanted is `finished`. A timer of
zero fires no earlier than the next frame. It is queued when the app writes it, applied at the
drain, advanced once, and reported to the app at the start of the next frame, which is honest
rather than special-cased.

## Motion an app drives itself

Some motion has no target: a spinner turning for as long as something loads, a value following a
spring. That is written from the clock by hand:

```rust
# use foliage::{Grove, Grow, Leaf, Shape};
# struct Spinner { leaf: Leaf, angle: f32, loading: bool }
# impl Spinner {
fn frame(&mut self, grove: &mut Grove) {
    if !self.loading {
        return;
    }
    self.angle += grove.frame_time().as_secs_f32() * 3.0;
    grove.reshape(
        self.leaf,
        Shape {
            sides: 3.0,
            rounding: 0.3,
            rotation: self.angle,
        },
    );
    // Nothing the engine can see is running, so it has to be asked for the next frame.
    grove.again();
}
# }
```

The engine idles when nothing is owed, and a value an app is moving itself is invisible to it.
`grove.again()` asks for one more frame, and calling it every frame for as long as the motion runs
keeps the loop going for exactly that long.

`grove.frame_time()` is how long the last frame took, on the one clock every tween reads. When the
machine stalls (it slept, a tab was in the background, a window was dragged) a frame is never told
more than 100 milliseconds passed, so everything on the clock is deferred by the gap rather than
jumping through it. A motion resumes where it was and takes longer in wall time, which is a
better failure than arriving at its end without having been drawn.
