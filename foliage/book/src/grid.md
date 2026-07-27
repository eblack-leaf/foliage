# Layout: Grid, Location, Anchor

Every example so far has quoted calls like `8.px().as_left().with(160.px().as_width())`
without explaining them. This chapter is that explanation.

## The grammar: `LocationValue` → `ValueDescriptor` → `ConfigurationDescriptor`

`GridExt` is implemented for the plain number types (`i32`, `f32`, `u32`, ...), which is
what makes `8.px()` legal at all:

```rust
// foliage_proper/src/grid/location.rs
pub trait GridExt {
    fn pct(self) -> LocationValue;
    fn px(self) -> LocationValue;
    fn col(self) -> LocationValue;
    fn row(self) -> LocationValue;
    fn letters(self) -> LocationValue;
}
pub enum LocationValue {
    Percent(f32), Px(CoordinateUnit), Column(i32), Row(i32), Anchor(Designator, f32),
    TextContent, Letters(i32),
}
```

`.as_left()`/`.as_right()`/`.as_top()`/`.as_bottom()`/`.as_width()`/`.as_height()` wrap a
`LocationValue` into a `ValueDescriptor` (the value plus which edge it describes);
`.with(..)` pairs two `ValueDescriptor`s (one axis-start edge and its counterpart) into a
`ConfigurationDescriptor`. So `8.px().as_left().with(160.px().as_width())` reads exactly
as written: left edge is 8px in, width is 160px. `Location::new().xs(horizontal,
vertical)` takes one `ConfigurationDescriptor` per axis; `.sm()`/`.md()`/`.lg()`/`.xl()`
override it at larger breakpoints, falling back to the next-smaller configured one if a
given breakpoint has none of its own (`Grid::config`'s `at_least_sm`/`at_least_md`/
`at_least_lg` chain).

`col()`/`row()` resolve against the parent's `Grid` (see below) instead of raw pixels --
`1.col().as_left().with(1.col().as_right())` means "the first column, full width," the
pattern every composite's own internal skeleton uses (see [Button](./composite-button.md)'s
panel, sized to `1.col()`/`1.row()` -- one full cell).

`letters()` has two distinct, separately-resolved paths, both against the *current
font's* real monospaced advance width (never a guessed constant):

- As a `Location` value directly, on the entity's own `FontSize`: `5.letters().as_width()`
  sizes a box to exactly 5 character-widths.
- As a `Grid` column/row pitch, on a *parent*: `Grid::new(1.letters(), 1.letters())` makes
  one column/row worth exactly one letter, so a child's `1.col()`/`5.col()` resolves
  through the *parent's* own `FontSize` (`stem_letters`) rather than the child's own --
  `TextInput`'s field/cursor grid is built entirely on this: the cursor's column is a real
  `.col()` address into a letter-pitched grid, not a hand-computed pixel offset.

## `Grid`: the coordinate system a `Location` resolves against

```rust
// foliage_proper/src/grid/mod.rs
#[derive(Component, Copy, Clone)]
#[require(View)]
pub struct Grid {
    pub xs: GridConfiguration,
    pub sm: Option<GridConfiguration>,
    // md, lg, xl -- same breakpoint-override shape as Location
}
```

Any entity that's going to size children with `.col()`/`.row()`/`.pct()` needs `Grid` on
itself -- resolving a child's `Location` always reads its parent's `Grid` unconditionally,
regardless of which units the child actually uses (`examples/opacity_and_elevation.rs`'s
own comment on this: a grid-less parent panics resolving *any* child, even a plain-`px`
one). `Grid::default()` is `Grid::new(1.col(), 1.row())` -- one column, one row, the
common case for "this entity is just a positioned box," not an actual multi-cell layout.

## `AspectRatio`: constraining a resolved box to a ratio

```rust
// foliage_proper/src/grid/aspect_ratio.rs
#[derive(Component, Copy, Clone)]
pub struct AspectRatio {
    pub xs: Option<f32>,
    // sm, md, lg, xl -- same breakpoint-override shape as Location/Grid
}
```

Not part of `Location` resolution itself -- a separate, later pass a *renderer* applies
against an already-resolved `Section` (`ImageView::on_insert` is the one built-in
consumer: see [Image](./image.md), which computes this straight from the decoded image's
own real pixel dimensions). `.constrain(section, layout)` shrinks the box down onto the
ratio (never grows it); `.fit(section, layout)` grows it back up to the ratio instead
(cropping the excess is the caller's own problem either way -- this only computes the
box). Both recenter the result within the original `section` on whichever axis actually
changed, rather than pinning it to the original top-left corner -- pinning to the corner
would silently shift anything else anchored to the shape's own center or far edge (a
sibling positioned via `Anchor::new(this)`) away from where it actually expects that
center to be, purely as a side effect of the ratio constraint having nothing to do with
where the shape's own center should end up.

## `Anchor`: relative-to-another-entity, not relative-to-parent

`anchor().left().as_left()` and friends (used throughout `Button`'s icon positioning,
`Dropdown`'s option surface) resolve against a *named sibling or ancestor* entity's own
resolved `Section`, via `Anchor::new(target)`, instead of the parent's `Grid`. This is
what lets a Popover's content surface size itself relative to its trigger, or an icon
size itself relative to a text label it sits beside, without either one needing to know
the other's absolute position.

An anchor value can also scale: `anchor().width() * 0.5` sizes a dependent at half its
target's *live* resolved width, `anchor().center_x() * 1.0` (the identity, what every
plain `anchor()...` call carries implicitly) left unscaled.

```rust
// foliage_proper/src/grid/location.rs
impl Mul<f32> for LocationValue {
    type Output = LocationValue;
    fn mul(self, rhs: f32) -> LocationValue { .. }
}
```

This can't just multiply a stored number the way `Percent`/`Px` do (`Percent(50.0) *
2.0`) -- an anchor's real value isn't known until it's actually looked up against the
live target at resolve time, so `Mul` instead scales a *factor* carried alongside the
`Designator`, applied once that lookup happens. `navigator.rs`'s shadow offset is the
worked example: `(anchor().width() * scale).as_width().with(anchor().center_x().as_right())`
mixes a scaled value (`width`) with an unscaled one (`center_x`) in the same box pair --
legal, and exactly what a shadow that's a smaller copy pinned to a reference point on its
target needs, but easy to get backwards (only some anchors mid-formula being scaled is
non-obvious at a glance) -- `Column`/`Row`/`Letters`/`TextContent` have no numeric value
scaling could mean anything for, and hit a loud `debug_assert!` instead of silently
no-op'ing if multiplied.

## `View`: the scroll/pan state a `Location` resolves *through*

```rust
// foliage_proper/src/grid/view.rs
#[derive(Component, Copy, Clone, Debug)]
#[require(ViewAdjustment, OverscrollPropagation, ScrollInertia, ScrollProgress)]
pub struct View {
    offset: Position<Logical>, // pub(crate) -- only `extent_check`'s own clamp writes it
    extent: Section<Logical>,  // read externally via `.offset()`/`.extent()`
}
```

`View` (required by `Grid`, so every `Grid`-carrying entity has one whether or not it
ever actually scrolls) is what `resolution.section.position -= view.offset` in
`grid/location.rs` subtracts from *every* resolved child underneath it -- this is why an
entity meant to stay visually fixed (chrome, a scrollbar) can't be spawned as a structural
child of something that scrolls: it would get panned right along with the content.
`.offset()` is the current pan in px; `.extent()` is the union of the entity's own
`Section` with every `Stem`-descendant's live (offset-adjusted) bounds -- effectively "how
big the scrollable content actually is," recomputed by `extent_check`
(`DiffMarkers::Prepare`) whenever a descendant's `Section`/`Stem` changes. Both fields are
`pub(crate)` (write-access stays inside the crate, in `extent_check`'s own clamp) with
public getters -- from outside the crate this is read-only raw state, not a door to park
the view somewhere a real drag/wheel gesture could never reach.

Two more pieces round out the scroll story, added alongside a hand-built scrollbar in the
`application` demo that needed a way to both *read* and *set* scroll position without
reaching into `offset`/`extent` raw pixel math itself:

```rust
// foliage_proper/src/grid/view.rs
pub struct ScrollTo { pub x: Option<f32>, pub y: Option<f32> } // a request: write-only, by design
pub struct ScrollProgress { x: f32, y: f32 } // a readout: `.x()`/`.y()` getters, no public constructor
```

- **`ScrollTo`** is an author-facing *request*, as a 0..1 fraction of the entity's current
  scrollable range per axis (`ScrollTo::y(0.5)`, `ScrollTo::xy(x, y)`) -- not a raw pixel
  offset, since that would go stale the instant `extent`/`Section` next shift, and could
  park the view somewhere a real drag/wheel gesture would never have been allowed to
  reach. `extent_check` resolves it the same way it resolves a drag (recomputes the real
  pixel delta from the live `View`/`Section`, clamped identically), then removes it --
  one-shot, consumed the same pass it lands in. Either axis left `None` leaves that axis's
  current scroll position untouched.
- **`ScrollProgress`** is the read-only counterpart: the same 0..1 fraction, kept live by
  `extent_check` on every entity with a `View`. This is a real `.insert()`, not a `Query`
  mutation, specifically so `tree.react::<ScrollProgress, _>(entity, ..)` fires on every
  scroll change (drag, wheel, or a `ScrollTo` write) and not just the first.

This split -- author states intent via `ScrollTo`, a resolved value comes back via
`ScrollProgress` -- mirrors `Visibility`/`ResolvedVisibility` (see
[Lifecycle](./lifecycle.md)): several inputs can influence the real state, but only one
resolved value is ever the ground truth, and nothing outside the resolver mutates it
directly.

## `ScrollMomentum`: coasting after a drag/touch release

A raw pointer-drag stays exactly 1:1 while it's actually moving (see
[Interaction](./interaction.md)'s own walk-up code) -- but on release, a fast enough
flick should keep the view coasting on its own instead of stopping dead the instant a
finger lifts, the way every native touch-scroll implementation behaves ("momentum
scrolling" is the term most of them use for exactly this).

```rust
// foliage_proper/src/grid/view.rs
#[derive(Resource, Copy, Clone, Debug)]
pub struct ScrollMomentum {
    pub velocity_threshold: f32,   // px/ms below which a release just stops, no coast
    pub decay: f32,                // fraction of velocity retained per elapsed ms while coasting
    pub stop_epsilon: f32,         // px/ms below which a coast is finished
    pub stillness_cutoff_ms: f32,  // ms since the last real move past which velocity is zeroed outright
}
```

`ScrollMomentum` is a real `Resource`, inserted with a default at startup -- insert your
own before `photosynthesize()` to retune the feel for a given app (a dense list and a
wide gallery don't want the same threshold/decay). `Coasting` (`pub(crate)`, not
author-facing) is the actual per-view runtime state -- a release velocity plus a last-tick
timestamp, present only while a coast is in flight. A small `coast` system ticks every
`Coasting` view each frame, writing `ViewAdjustment` from `velocity * elapsed_ms` and
decaying `velocity` by `decay.powf(elapsed_ms)` (exponential, so it's frame-rate
independent) until it drops under `stop_epsilon`, at which point `Coasting` removes
itself.

`coast` also reads `CurrentInteraction` directly and removes a view's own `Coasting`
outright, before any of the decay logic above runs, if the pointer is down right now
(`current.pressed`) *and* that view is either the currently grabbed `current.primary`
itself, or is reached by walking *up from* `current.primary` through its own `Stem`
chain -- the same walk (starting at the grabbed entity, going toward the root)
`interactive_elements` uses to find a view to pan (see
[Interaction](./interaction.md)). The direction matters: grabbing something *inside* a
coasting view (a row in a list, a nested scrollable region within it) walks up through
that content and reaches the view, stopping it -- correct, you're interacting with its
own content. Grabbing the coasting view's own *parent* directly does not stop it: walking
up from that parent goes further toward the root, away from the view (which is *below*
it, not above), so an unrelated container being touched never reaches down into a
completely different view's own coast. `interactive_elements` separately clears
`Coasting` off the same chain right on `Start` too, but that's a `Tree`-queued command
with no guaranteed same-frame ordering against `coast`'s own run within the same
schedule; `coast`'s own resource read is what actually guarantees a fresh drag's real
`ViewAdjustment` never gets overwritten by a stale coasting one on the same view.

`current.pressed` matters here specifically because `current.primary` isn't cleared on
release -- it's deliberately left set between gestures (only the *next* `Start` clears
it), so a released entity's own click/`Disengaged` can still be judged against where it
was grabbed. Checking `primary`'s mere presence would read as "still grabbed" for the
exact entity that was *just* released, on the very tick its own `Coasting` was created --
`pressed` is the field that actually answers "is the pointer down right now."

This is distinct from `ScrollInertia` (the `#[require(..)]` above) -- that one scales
each *new* wheel tick while ticks keep arriving close together; `ScrollMomentum` instead
governs what happens once a drag/touch release has no more input left to scale at all.

Covered where scrolling matters most beyond this: [TextInput](./composites/text-input.md)'s
scroll-into-view logic and `List`/`Carousel`'s viewport.
