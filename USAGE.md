# Usage

The concepts foliage is built out of, and the vocabulary each is written in. The
[API reference](https://eblack-leaf.github.io/foliage/api/foliage/) states every type and every
method; this states how they fit together and which one answers a given question.

The code is the source of truth. Every section below names the module the concept lives in, and
those modules carry the reasoning that is not repeated here.

## The shape of a program

An app is a value implementing `Root`. The engine is a `Foliage`, built at boot, told what to
grow, and then run.

```rust
use foliage::{Area, Foliage, Grove, Pollen, Root};

struct App { /* whatever the app keeps between frames */ }

impl Root for App {
    fn take_root(grove: &mut Grove) -> Self { /* grow the tree, return the app */ }
    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) { /* what happened, and what to do */ }
}

fn main() {
    let mut foliage = Foliage::new();
    foliage.title("app");
    foliage.app_id("app");
    foliage.desktop_size(Area::new(390.0, 844.0));
    foliage.root::<App>();
    foliage.photosynthesize();
}
```

`take_root` runs once, inside the first frame, with a `Grove` no different from any other frame's.
`frame` runs once per frame thereafter.

Nothing the app holds is handed to the engine, and the engine has no way to reach it. The value
stays on the app's side and touches the tree only through the `Grove` it is lent for the length of
a frame. No engine type is handed out and nothing an app holds borrows from the world.

Three things cross the seam, and all of them are plain data:

| Direction | What | Where |
|---|---|---|
| App to engine | ops, issued through `Grow` | `src/verbs.rs` |
| Engine to app | one `Pollen` per frame | `src/pollen.rs` |
| App to engine, read-only | a `Vein` tapped at the app's own callsite | `src/vein.rs` |

*Modules: `src/foliage.rs`, `src/root.rs`, `src/grove.rs`.*

## Names

A `Leaf` names one element. It is allocated the moment it is asked for and is usable immediately —
as a trunk, as the target of a write — even though the element itself comes into existence when the
frame's ops are drained.

A `Leaf` naming something pruned is inert rather than dangerous: every op targeting it is dropped
and every tap of it reads absent. **A name is never reused**, so a stale one cannot come to address
whatever grew after it. `Grove::presence` answers what a name currently means: `Planted`, `Live`
or `Withered`, in that order and terminal at the end.

*Module: `src/leaf.rs`.*

## Growing

A `Seed` describes an element before it exists. `Grow::plant` grows one at the top level,
`Grow::branch` grows one under another, and both hand back the `Leaf`. `Grow::prune` takes an
element and everything beneath it down; each is reported once as `Pollen::withered`.

The set of seeds is closed:

| Seed | What it is | Draws |
|---|---|---|
| `Stem` | Structure: a box that holds children and takes hits | nothing |
| `Panel` | A filled rectangle with rounded corners | yes |
| `Text` | A run of monospaced glyphs | yes |
| `Icon` | A vector mark, from a registered distance field | yes |
| `Image` | A registered picture, fitted into its box | yes |
| `Polygon` | A regular polygon: sides, corner rounding, rotation | yes |
| `Line` | A stroke between two points | yes |
| `TextInput` | An editable run, with a caret and a selection | assembled |

The six that draw own a render pipeline and an instance buffer. Everything else is assembly on top
of them: a `TextInput` is a panel, a run of glyphs and a caret, and draws nothing of its own.

foliage provides no widgets. A button or a card is assembled from the seeds above.

*Modules: `src/seed.rs`, and one per element.*

## Declaring, and writing

A seed **declares** what an element is, through `Place` (every seed) and `Boxed` (every seed
placed by a box). After it exists, the same properties are **written** through `Grow`. Both are
sealed traits: they can be called, never implemented.

| Declared on the seed | Written afterwards |
|---|---|
| `Boxed::at` | `Grow::at` |
| `Line::between` | `Grow::between` |
| `Place::grid` | `Grow::grid` |
| `Place::anchored` | `Grow::anchor` |
| `Place::elevate` | `Grow::elevate` |
| `color` on the seed | `Grow::color` |
| `rounding` on the seed | `Grow::round` |
| `Place::visible` | `Grow::visible` |
| `Place::opacity` | `Grow::opacity` |

A placement is one value rather than a set of edges, so there is no half-written state between two
writes and no question of which edge a later write meant. The same holds of a set of tints and of a
polygon's shape.

An op naming something it does not apply to is **dropped**, not an error: a `text` write to a panel,
a `round` on a polygon, a `scroll` on something that does not scroll. Every drop is traced.

*Modules: `src/place.rs`, `src/verbs.rs`, `src/op.rs`.*

## Placement

A `Location` states one `Horizontal` and one `Vertical` per breakpoint. The two axes are
distinct types and are given in that order, so they cannot be written the wrong way round.

```rust
Location::new()
    .xs(
        left(20.px()).right(100.pct() - 16.px()),
        top(anchor().bottom() + 8.px()).height(content()),
    )
    .md(left(2.col()).right(3.col()), top(0.px()).height(120.px()))
```

### A value is a role and a source

A **role** says what a number describes for this element; a **source** says where the number comes
from. The two are independent, which is what lets any source fill any coordinate. The role is
written first.

Each opener returns a type carrying only the completions legal with it, so an axis has exactly four
forms and the illegal ones cannot be written at all:

```
left(..).width(..)      left(..).right(..)      right(..).width(..)      center_x(..).width(..)
top(..).height(..)      top(..).bottom(..)      bottom(..).height(..)    center_y(..).height(..)
```

Every edge role is a coordinate in the parent's space, `right` and `bottom` included, so sixteen in
from the right is `right(100.pct() - 16.px())`. Expressions are sums of scaled terms: addition,
subtraction, and scaling by a plain number.

`Horizontal::at_least` / `Horizontal::at_most` clamp the extent, and the vertical axis has the
same pair. Over `content()`, `at_most` is fit-content.

### Sources

| Source | Reads |
|---|---|
| `Source::px` | Logical pixels. The only source that reads no geometry |
| `Source::pct` | A fraction of the trunk's extent on the resolving axis |
| `Source::col` / `Source::row` | A one-based track of the trunk's grid |
| `Source::letters` | A count of the element's own character cells |
| `content()` | The element's own intrinsic extent |
| `anchor()` / `trunk()` | Another element's edges, extents, tracks, cells and measure |

### Whose geometry

An element resolves against three things: itself, the trunk it was grown under, and the one element
it anchors to. Bare sources read the trunk. `trunk()` says so outright and adds the two readings
the bare sources cannot spell — its `content` and its `letters`. `anchor()` carries the same
vocabulary, which is what lets an element grown somewhere else go on addressing the element it
describes.

An element has **at most one** anchor, set with `Place::anchored` or `Grow::anchor`. An anchor
that would close a cycle is refused with a panic naming both elements and the write that did it.

### Breakpoints

`Location`, `Grid` and `FontSize` are all written in the same chain: `xs`, `sm`, `md`, `lg`,
`xl`, and `short`. A breakpoint with nothing of its own takes the nearest smaller one that has, so
only `xs` is ever required, and a chain that runs out falls back to the whole of the parent's box.

| Breakpoint | Viewport width |
|---|---|
| `Xs` | below 420 |
| `Sm` | 420 and above |
| `Md` | 600 and above |
| `Lg` | 840 and above |
| `Xl` | 1200 and above |

`Short` is orthogonal to width: the viewport becomes `Short::Yes` below 400 logical pixels tall
and returns to `No` at 440, and a configuration keyed to it wins over the width-derived one. Nothing
without such a configuration is affected.

### Grids

A `Grid` is how a parent's box is divided for the elements grown under it — what a child's
`Source::col` and `Source::row` address. Every element has one; undeclared it is a single column
and a single row. Tracks are stated as a count (`Divide::columns`, `Divide::rows`), a pixel
width (`Columns::px`), or a count of character cells (`Columns::letters`) in the font of the
element the grid is on. `Columns::gap` is the space between adjacent tracks, so an axis of `n`
tracks has `n - 1` gaps and no outer margin.

A grid decides layout and nothing else. Whether an element scrolls is a separate declaration.

### Width flows down, height flows up

Text wrapping makes height depend on width, and width comes from layout. The cycle is bounded at two
passes with no iteration: the horizontal axis resolves for the whole tree, each run wraps at its
now-known width, and the vertical axis resolves and reads the heights that came out.

The order is visible in the types. A `VerticalLength` — `Source::row`, `anchor().height()` — is
refused by a horizontal role. The reverse is fine and useful: `height(2.col())` is a two-column span
used as a height.

One consequence is worth stating outright. An element sized with `height(content())` takes the
furthest its children reach, **counting only the children that describe their own extent**. A child
whose vertical placement reads a vertical box — a percentage of its trunk, a row of its trunk's
grid, an anchor's edge — is asking how tall something else is, so it cannot be what decides how tall
this is; it is left out of the measure and given its real height afterwards. Pixels, letters, the
child's own content, and any horizontal reading all count.

### Elevation

`Elevation::up` and `Elevation::down` are relative to the trunk, and accumulate down the tree, so
raising a card carries its whole subtree and nothing inside it is rewritten. There is no form that
states a layer outright. Two elements that accumulate to the same elevation are separated by
allocation order, which is monotonic and never reused.

An element that has to clear the stack it was grown in is grown somewhere else and anchored back.

*Modules: `src/placement/`, `src/elevation.rs`, `src/rowan.rs` (the resolution passes).*

## Color

An element declares a **tone**, and the `Scheme` decides what it resolves to. A tone is a
`Palette` role and a `Step` on that role's ramp.

| Role | What it is for |
|---|---|
| `Surface` | The ordinary fill, and what an element that says nothing takes |
| `Raised` | A surface in front of another: a card against the page |
| `Muted` | A quieter fill, for a division or a rule |
| `Accent` | The emphatic color, and what the app's own content is marked with |
| `Signal` | The second hue: what reports the system's own state rather than the app's content |
| `Ink` | What is read against a surface |
| `Contrast` | What is read against a hue — `Accent` or `Signal` |

Each role names its own base step, so `Palette::Accent` is a tone in its own right. `Palette::at`
takes a role to a named `Step` — `Farthest`, `Far`, `Base`, `Near`, `Nearest` — and
`Palette::recede` / `Palette::advance` move one step at a time. A step is named for where it
stands relative to the ground rather than for which way it moves in lightness, so a state written
once is correct against a dark scheme and a light one.

States are steps. A hover, a press, something disabled: each is a step on the role's own ramp rather
than a color picked beside it, so it survives a repaint without being restated.

A `Fill` is either a tone or a `Color` stated outright. A literal is an element saying it is not
part of the scheme: `Grow::repaint` moves the first and not the second.

A `Scheme` is stated in seven colors, one per role, and derives the other four steps of each ramp in
OKLab — so a theme is seven decisions rather than thirty-five. `Scheme::new` is a dark reading,
`Scheme::light` a light one, and `Scheme::set` replaces a role's seed (re-deriving its ramp) or
one step outright. `Grow::repaint` is the one write that names no element.

*Modules: `src/palette.rs`, `src/color.rs`.*

## Text

Every font is monospaced, and every measurement foliage makes is a count of character cells. A
proportional face is refused.

`Text` is a run of glyphs. It is the one element whose box can be measured rather than declared:
in a width role `content()` is the widest the run wants to be, and in a height role it is how tall
it turned out at the width the layout gave it. `Source::letters` is the other half, for where the
count is known ahead of time.

`FontSize` is stated per breakpoint. `Place::font` and `Place::font_size` are on **every**
element, not only on ones that draw glyphs: a font and a size are what give a character cell its
size, and a cell is what `letters` and a letter-pitched track are measured in. An element that names
neither reads zero for both.

`Text::tint` and `Grow::tint` fill part of a run differently from the rest, over the run's own
character index space — the same space a caret and a selection are addressed in. A tint is a `Fill`
like any other, so a role follows a repaint and a literal does not. `Grow::untint` takes them off.

`TextInput` is an editable run: one `Leaf` to hold, `Grow::select` to move its caret,
`Pollen::edited` and `Pollen::submitted` to hear what was typed. It answers `Ctrl+C`, `Ctrl+X`
and `Ctrl+V` itself and raises the `Keypad` it named where there is one to raise. Its value is
read with `Vein::Text` and its selection with `Vein::Selection`.

*Modules: `src/text/`, `src/text_input.rs`.*

## Input

### Who receives a gesture

A gesture goes to the top of the box stack whatever is there. What the element at the top does with
it is its own declaration:

| Declaration | Effect |
|---|---|
| `Place::interactive` | Receives gestures, and is reachable by focus |
| `Place::intangible` | The gesture goes to whatever is beneath it. What a composite marks its own decoration with |
| neither | **Eats** the gesture. What a backdrop, a sheet backing and a menu's padding are |
| `Place::drags` | Which axes this element takes drags on |
| `Place::round_hit_area` | Hits are tested against the inscribed ellipse rather than the box |

`intangible` does not take an element out of the box stack, only out of what may be the top of it:
the drag that follows a press still finds the region containing it, so scrolling works over
decoration exactly as over anything else.

### What a gesture becomes

`Pollen::engaged` says a gesture went down and is being held. It says nothing about what the
gesture will turn out to be. From there it becomes one of:

- `Pollen::clicked` — it ended without ever becoming a drag. `Pollen::clicked_at` is where it
  began, which is the point it was hit-tested against.
- `Pollen::held` — it stayed still past `Hold`'s duration. A hold is never also a click.
- `Pollen::drag_started`, then `Pollen::dragged` carrying a `Drag` — reported to an element
  that declared `drags` on the axis the gesture went, or that took the hold this drag came out of.

`Pollen::disengaged` is reported however it ended, including a drag this element does not take
being passed to a region containing it. It is always somewhere to put a pressed visual back.

An element that takes no drags holds a gesture only until that gesture becomes a drag, and then
yields — which is what makes a button inside a scrolling list behave on touch.

Nothing is emitted while a gesture is resolving, so there is nothing to retract when it turns out to
be a drag.

### Focus and keys

Focus rests only on something that declared `interactive`. It is not a byproduct of pressing
anything: a press moves focus nowhere, and an app that wants a field focused when tapped writes that
from `clicked`.

`Grow::focus`, `Grow::unfocus`, `Grow::focus_next` and `Grow::focus_previous` move it;
`Pollen::focused` and `Pollen::unfocused` report it; `Grove::focused` answers what holds it.
Focus order is reading order, adjusted by `Place::focus_order`. `Place::focus_scope` makes focus
cycle inside an element while it is in there.

`Pollen::keys` is what an element was sent, and `Pollen::root_keys` is what arrived with focus
nowhere. Both are ordered, which nothing else in `Pollen` is, because two keys in a frame mean
different things in each order. A `Keystroke` carries a `Key` and its `Modifiers`.

### Feel

Three global tuning values, set once at boot with `Foliage::tune`. They are global because input
feel that varies from element to element is what makes an app feel unpredictable.

| Value | What it sets | Default |
|---|---|---|
| `Claim` | How far a gesture travels before it is a drag, per axis | 16 across, 8 down |
| `Hold` | How long a press is down before it is a hold | 500ms |
| `Momentum` | How a released drag coasts: half-life and minimum speed | 350ms, 40px/s |

*Modules: `src/interaction/`, `src/keyboard.rs`.*

## Views

An element scrolls because it said so, and for no other reason. Dividing a box with a grid says
nothing about scrolling.

`Place::scrolls` takes an `Axes` or a `Scroll`. An axis not named does not scroll and has no
extent. A drag anywhere inside a region scrolls it, whether or not what the drag landed on is a
target, and an axis that reaches its end hands the drag outward to the next region containing it —
unless `Scroll::contain` names it, in which case the region absorbs it.

Two ways to be inside a region without being part of its content, each one declaration:

- `Place::pinned` — does not travel with the content, and contributes nothing to the extent. A
  header that stays put. It keeps its place in the tree, and with it the clipping, the opacity
  product and the disable cascade.
- `Place::floats` — sits over the region rather than in it: not clipped by it, and contributing
  nothing to its extent, while still travelling with the content. `Escape` says how far the clip
  escape reaches — `Region`, `Surface`, or `Within(leaf)`.

`Grow::scroll` moves a region by name, with `ScrollTo::px`, `ScrollTo::fraction`,
`ScrollTo::start`, `ScrollTo::end` or `ScrollTo::show`. Every form lands as a number of pixels
from the content's origin, clamped to what the region can reach. The destination is answered against
the extent of the frame it lands in, so scrolling to the end of a list that grew in the same frame
lands at the end of the list. `px` is the one absolute distance and names its axis with
`ScrollTo::on` where the region scrolls both.

Extent is measured from where children landed, never from what is currently drawn: content scrolled
out of sight is exactly what an extent describes. A visible child that is simply far away is
content, and is counted; what removes a child is `visible(false)` or `pinned`.

`Vein::Offset`, `Vein::Extent` and `Vein::Progress` read a region back. Each answers `None` on
something that does not scroll.

*Module: `src/view.rs`.*

## Motion

`Grow::animate` moves a property over time and hands back a `Tween` naming that motion.

The target is written to the element at once and the tween carries what it left, so the element
declares where it is going from the moment it is told to go there. Nothing has to be undone on
arrival.

| `Motion` | Moves |
|---|---|
| `Opacity(f32)` | How opaque the element is |
| `Color(Color)` | The fill, stated outright |
| `Palette(Palette)` | The fill, as a role — so a repaint mid-motion moves the motion |
| `Location(Location)` | Where the element sits; both ends re-resolve every frame |
| `Scroll(ScrollTo)` | Where a region is moved to; the destination re-resolves every frame |
| `Polygon(Shape)` | Sides, corner rounding and rotation together |

The list is closed. Everything else — a font size, a value foliage has no concept of — is
`Grow::tween`, which runs a number from one end to the other, reports it each frame, and writes
nowhere. `Grow::timer` is a tween whose value is not read.

`Timing::ms` is the duration; `Timing::after` delays the start; `Timing::ease` shapes it with
an `Ease` — `Linear`, `Decelerate`, `Accelerate`, `Emphasis`, or a `Curve` through two control
points. `Timing::within` counts the motion into a `Sequence`, from any callsite at any frame,
whose last ending is reported as `Pollen::sequence_finished`.

A second `animate` on a property already moving replaces it, starting from where the element
currently is. A **direct write** to that property cancels it, and the element is at what was written.

Two ways to end one early:

- `Grow::stop` — nothing is reported, so a chain waiting on it never runs. The motion is left on
  its target: giving up the duration leaves the element where it was told to end.
- `Grow::finish` — the same, reported as an arrival, so a chain waiting on it runs this frame.

`Pollen::tween` is how far a tween has come — a motion's eased progress, a channel's value.
`Pollen::finished` is the frame it ended on the end it was going to. `Pollen::landed` is the
element settling, and says nothing about which of its properties did.

*Module: `src/aspen/`.*

## Reading

### What the frame reports

`Pollen` is a set to interrogate, not a list to walk: ask it about the elements you own. It is
handed to `frame` once per frame.

| Question | Answers |
|---|---|
| `withered` | An element was taken down |
| `resized` | The surface's new size |
| `engaged`, `disengaged`, `clicked`, `clicked_at`, `held`, `held_at` | The gesture lifecycle |
| `drag_started`, `dragged` | A drag this element took |
| `focused`, `unfocused` | Focus arriving and leaving |
| `keys`, `root_keys` | What was typed, in order |
| `edited`, `submitted` | What a field was told |
| `pasted` | What the clipboard held |
| `loaded`, `missing` | A font, mark or picture arriving, or failing to |
| `landed`, `tween`, `finished`, `sequence_finished` | Motion |

An app's own writes are never reported back to it.

### What an element holds

`Grove::tap` reads one property, named by a `Vein`, and answers a `Sap`. It is exhaustive by
construction: if a `Vein` is not there, an app cannot see it — and everything an app can declare is
there, because a value you can set and cannot read back is a value you have to keep a copy of.

Some read what was **declared** (`Color`, `Elevation`, `Visible`, `Opacity`, `Disabled`) and some
what was **resolved** (`Placed`, `Drawn`, `Ends`, `Offset`, `Extent`, `Progress`). `Placed` is where
the layout put an element; `Drawn` is that less every scrolling ancestor's offset, which is what was
drawn and what a hit test runs against. The two differ only inside a region that has been scrolled.

`tap` answers `None` where the element has withered, has not been grown yet, or does not carry that
property.

### What is true of the frame

`Grove::viewport`, `Grove::layout`, `Grove::short`, `Grove::scheme`, `Grove::focused`,
`Grove::frame_time`, `Grove::elapsed`, `Grove::presence`.

*Modules: `src/pollen.rs`, `src/vein.rs`, `src/grove.rs`.*

## Assets

A font, an icon's field and a picture are all bytes. Each is registered by handing over the bytes or
an `Origin` to read them from, and the name comes back at once either way — so an app that fetches
writes the same line as one that bundles.

| Registers | At boot | At any frame |
|---|---|---|
| A font, giving a `Font` | `Foliage::font` | `Grove::font` |
| A mark, giving a `Field` | `Foliage::icon` | `Grove::icon` |
| A set of marks | `Foliage::marks` | `Grove::marks` |
| An encoded picture, giving a `Plate` | `Foliage::image` | `Grove::image` |
| Pixels the app made | `Foliage::pixels` | `Grow::pixels` |

`Grow::plate` names a picture whose pixels have not arrived, and `Grow::load` fills it. A name is
valid the moment it is handed out: an element drawing a plate or a mark with nothing behind it
occupies its box, draws nothing, and appears on the frame the pixels do. That is what keeps "is it
loaded yet" out of an app's state.

`Pollen::loaded` and `Pollen::missing` ask about a `Font`, a `Field` and a `Plate` alike, through
`Arrival`. A **font** is the one worth waiting for: a run composed in one that has yet to land is
measured in the bundled face and reflows when the real one arrives.

`Marks` is the set of marks an app draws, registered at once and named by the app — the same
inversion `Root` is. `foliage-icons` bakes a mark from an SVG and generates the module that
implements it.

*Modules: `src/asset.rs`, `src/icon.rs`, `src/image.rs`, `src/text/font.rs`.*

## Off the frame

`Grove::sprig` hands out a `Sprig`: the engine reached from a thread, a promise or a callback. It
is cloneable and, where the platform has threads, `Send`. It carries `Grow` entire, so **a change
reads identically wherever it is issued**.

An op from a `Sprig` lands in the drain of the frame that was running when it arrived, or the next
one. That is the whole of the difference. Nothing about how an op is applied depends on which side
it came from, and a `Leaf` that withered before the write reaches it drops the write as ever.

Reading is pushed rather than sampled, because a thread has no world to read:

- `Sprig::conditions` — what is true of the tree as a whole, as of the last frame.
- `Sprig::watch` / `Sprig::unwatch` — ask to be told one property of one element, and told again
  whenever it changes. `Sprig::tap` reads what the watch last saw.
- `Sprig::pollen` — every frame's report since the last call. Nothing is collected until the first
  call, which is what arms delivery.

Every clone is the same handle: one queue, one stream of reports, one set of watches.

*Module: `src/sprig.rs`.*

## The host

Four things an app can ask for that the engine cannot do itself:

| Verb | What it does |
|---|---|
| `Grow::copy` | Puts text on the system clipboard |
| `Grow::paste` | Asks what it holds, reported as `Pollen::pasted`, never in the frame that asked |
| `Grow::navigate` | Goes to a URL. **On the web this replaces the page**; off it the desktop is asked to open it |
| `Grow::download` | Asks the host to save what is at a URL. The web's only; elsewhere it is traced |

A `TextInput` answers `Ctrl+C`, `Ctrl+X` and `Ctrl+V` for itself and does not go through these.

*Modules: `src/clipboard.rs`, `src/link.rs`.*

## Frames

There is no render call to make. Ops issued from `frame` are drained in the order written as soon as
it returns, so nothing an app queues can land while it is still running, and the next frame is
already different.

The engine idles when nothing is owed: a frame is owed while ops are queued, while a tween is
running, while a region is coasting, and while a report is waiting to be delivered. An app driving
its own motion from `Grove::frame_time` has nothing the engine can detect and calls
`Grove::again` for as long as it is doing something.

Only the state that actually changed is drained into the backend each frame.

*Modules: `src/photosynthesize.rs`, `src/rowan.rs`, `src/ash/`, `src/ginkgo/`.*

## Platforms

| Platform | Notes |
|---|---|
| Linux, Windows, macOS | `Foliage::new()` |
| Web (`wasm32-unknown-unknown`) | `Foliage::new()`. The surface is taken from the page |
| Android | `Foliage::android(activity)`, from `android_main`. `foliage-android` scaffolds and drives the Gradle project |
| iOS | The shared source carries iOS arms; unverified |

Only construction differs. Everything after it is the same statements.

One feature, off by default: **`origin-url`** fetches an `Origin::url` off the web, and is the only
thing in the crate needing an http client and a TLS stack. On the web a URL is fetched by the
browser and the feature selects nothing; off it, without the feature, a URL is accepted and reported
`Pollen::missing`.

## Where to look

| Concept | Module |
|---|---|
| Boot, the platform loop | `src/foliage.rs`, `src/photosynthesize.rs`, `src/willow.rs` |
| The app's seam | `src/root.rs`, `src/grove.rs`, `src/verbs.rs`, `src/pollen.rs`, `src/vein.rs` |
| Ops and the queue | `src/op.rs`, `src/queue.rs` |
| The tree | `src/tree.rs`, `src/leaf.rs`, `src/lifecycle.rs` |
| Declaring an element | `src/place.rs`, `src/seed.rs` |
| Placement grammar | `src/placement/` |
| Layout resolution | `src/rowan.rs` |
| Breakpoints | `src/layout.rs` |
| Color | `src/palette.rs`, `src/color.rs` |
| Elements | `src/panel.rs`, `src/text/`, `src/icon.rs`, `src/image.rs`, `src/polygon.rs`, `src/line.rs`, `src/stem.rs`, `src/text_input.rs` |
| Input, focus | `src/interaction/`, `src/keyboard.rs` |
| Scrolling | `src/view.rs` |
| Motion | `src/aspen/` |
| Assets | `src/asset.rs` |
| Off-frame access | `src/sprig.rs` |
| Extraction and the backend | `src/elm.rs`, `src/ash/`, `src/ginkgo/` |

Runnable examples, each doing one thing, are in [`foliage/examples`](foliage/examples):

```sh
cargo run -p foliage --example palette
cargo run -p foliage --example animate
cargo run -p foliage --example polyline
```

