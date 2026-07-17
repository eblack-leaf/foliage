# The Primitives

Before authoring anything, it helps to know what's already in the box — the vocabulary
`Authoring a Widget` assumes you recognize when it composes `Panel`/`Icon`/`Text`/`Grid`/
`Location`/`Elevation` without re-explaining any of them.

## The visual primitives

Each of these is itself a `Sprout` — spawned the same way as anything you'd author yourself
(`tree.leaf(...)`/`tree.branch(parent, ...)`), just with an empty `build` (see
[Authoring a Widget](./authoring.md)): they have nothing to build underneath, they *are* the
leaf content.

- **`Panel`** — a rectangle: `Panel::new().color(Color::gray(800)).rounding(Rounding::Md)
  .outline(2)`. The generic surface almost everything else sits on top of.
- **`Text`** — `Text::new("label")`, with `.size(FontSize::new(16))`. Content updates through
  `TextValue`, the same value component `Button` forwards internally.
- **`Icon`** — `Icon::new(id)`, where `id` comes from an `#[icon_handle]`-generated enum (see
  the icon pipeline — `foliage_icons` rasterizes SVGs ahead of time into a fixed-size mip chain,
  not a runtime vector renderer). Content updates through `IconValue`.
- **`Image`** — `Image::new(key)` where `key: AssetKey` comes from `load_asset!`/`Keyring`.
  `.view(ImageView::Aspect | Stretch | Crop)` controls how the source bitmap fills its `Location`
  — `Aspect` preserves aspect ratio and centers, `Stretch` fills exactly, `Crop` fills and clips.
- **`Line`**/**`Shape`** — a line segment between two points (`Line::new(weight)`, positioned via
  `Location`'s point-pair form rather than a rect) — used for the connective/underline marks
  you'll see in composite widgets like `Slider`.

## The composites

Everything in `foliage_proper/src/composite/` is authored with the same `Sprout` trait as
[A Complete Example](./example.md) — no library-only magic, each one a second example you can
read. The contract per widget is uniform: value component(s) in via `write_to`, a
`#[targeted_event]` out via `subscribe`.

- **`Button`** — `ButtonStyle`/`TextValue`/`IconValue` in, `OnClick` out.
- **`TextInput`** — `TextValue`/`TextInputStyle`/`HintText` in, `TextChanged` out
  (`composite/text_input/` — the deep-end example: cursor, selection, keybindings).
- **`Slider`** — `Progress` (0..=1) in, `ProgressChanged` out. Drag, tap-to-seek, and
  programmatic writes share the one `Progress` door; `.interactive(false)` is the
  display-only progress bar.
- **`Toggle`** — `ToggleState` in, `Toggled` out.
- **`Pagination`** — `PageIndex`/`PageCount` in, `PageChanged` out. `Dots` or `Numbered`
  (with `…` collapse); prev/next step buttons exist only when `.step_icons(..)` supplies
  icons — the library ships none.
- **`List`** — `ListItems`/`ListLayout` in, rows out as YOUR content (see the slot
  convention below). Scrolling and clipping come free from `View`.
- **`Dropdown`** — `DropdownOptions`/`Selected` in, `SelectionChanged` out. Options are
  deliberately plain strings; rich rows = compose your own trigger + `List`.
- **`Carousel`** — `CarouselPages`/`PageIndex` in, `PageChanged` out (the same event
  Pagination defines); swipe + optional embedded dot strip.
- **`Modal`** — full-screen overlay hosting your content, growing from an optional anchor
  entity; `Closed` out, `CloseModal` in for programmatic closes.

**The slot convention** (`SlotFn`/`IndexedSlotFn`, `composite/mod.rs`): content-hosting
composites (List rows, Carousel pages, Modal body) never define what your content *is* —
you hand them a closure, they hand it a positioned **slot** entity, and you `tree.branch`
whatever you want under it with full `Location`/`Grid`/`Anchor` freedom. The closure lives
in the widget's config component, so rewriting that config via `write_to` re-runs it —
config writes ARE the re-render API, the same `react` door as every value.

## Positioning

Every spawnable carries these through `LeafSprout` (`.at()`/`.elevate()`/`.with()` — see
[Authoring a Widget](./authoring.md) for where these live in the `Sprout` trait):

- **`Location`** — where an entity sits, described per breakpoint (`.xs(...)`, `.sm(...)`, ...
  falling back to the next-smallest defined one). A `Location` expression is a pair of
  horizontal/vertical descriptors built from anchors like `0.pct().as_left()`,
  `1.col().as_left()`, `16.px().as_top()`, each `.with(...)` a matching far edge
  (`.as_right()`/`.as_bottom()`) or a size (`.as_width()`/`.as_height()`).
- **`Grid`** — `Grid::new(columns, rows)` on a parent, e.g. `12.col().gap(8)` /
  `40.px().gap(8)`, is what makes a child's `1.col()`/`(i+1).row()` in the child's own
  `Location` resolve to real pixels — a child's `Location` resolves against its `Stem`'s
  (parent's) `Grid`, or the viewport if it has no parent (see `tree.leaf` vs `tree.branch`,
  [The Contract](./contract.md)).
- **`Anchor`** — positions an entity relative to *another specific entity* (not its parent's
  grid) — how a modal's backdrop tracks whatever it opened from, or a line tracks the button
  it's annotating.
- **`Elevation`** — z-ordering/layering, not physical size. `.elevate(Elevation::abs(n))` sets
  an absolute layer; `.elevate(Elevation::up(n))` sets it relative to the parent's layer — the
  mandatory argument every `Sprout` requires before spawning (skipping it is a compile-reachable
  panic on purpose, see `tree.rs`'s `Sow::grow`).

## Styling

`Color::gray(l)` / `Color::red(l)` / etc. take a luminance value, not raw RGB — the palette is
closed and thematic rather than arbitrary hex. `Opacity`, `Rounding`, `Outline` are ordinary
components any visual primitive accepts via `.with(...)` or a dedicated builder method
(`Panel::new().rounding(Rounding::Full).outline(2)`), and (per `Outline`/`Opacity`'s
`Attachment::attach`) are themselves animatable — see below.

## Animation

- **`Animation<C>`** — `Animation::new(end_value).targeting(entity).start(ms).finish(ms)
  .eased(Ease::EMPHASIS)` — interpolates a single component on one entity over a time range.
  Any `Animate`-implementing component can be animated this way (`Location`, `Opacity`,
  `Outline`, ... — the ones already wired via `enable_animation::<C>()` in their own
  `Attachment::attach`).
- **`Sequence`** — `Sequence::new(tree).animate(...).animate(...).end(|trigger: Trigger<OnEnd>,
  tree: Tree| { .. })` — batches animations that share a starting reference point (each
  animation's own `start`/`finish` are offsets *within* the sequence, not wall-clock times),
  with `.end(...)` firing once every animation in the sequence has finished — the usual place to
  do cleanup (`tree.remove(...)`) after a close/exit animation completes.

With this vocabulary in hand, [Authoring a Widget](./authoring.md) is just: which of these
you spawn as children, and how `react` wires their values to your own widget's config.
