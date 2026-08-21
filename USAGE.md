# Foliage, for someone who has never seen it

Orientation, not reference. Every type and method here is documented at the item itself, often
with the reasoning — `cargo doc --open`, or just read the doc comments, they are good. What this
file carries is the part rustdoc structurally cannot: **where things live**, **which surface to
reach for**, and **the invariants that span modules** and so have no single item to hang off.

Nothing below lists signatures. Signatures rot; a map does not.


## The shape of an app

```rust
pub fn run(mut foliage: Foliage) {
    foliage.enable_tracing("myapp=info".parse().unwrap());
    foliage.desktop_size((1440, 900));
    foliage.tune(ClearColor(some_color));
    icons::register(&mut foliage);   // startup-only: takes &mut Foliage
    fonts::register(&mut foliage);   // ditto
    foliage.root::<App>();
    foliage.photosynthesize();
}
struct App { /*...*/ }
impl Root for App {
  fn take_root(canopy: &mut Canopy) -> Self { /* one time boot; good for init */ }
  fn frame(&mut self, canopy: &mut Canopy, blooms: Vec<Bloom>) {
    // see if a Bloom::Clicked(leaf) was emitted and matches my widget.leaf
    if self.widget.clicked(&blooms) { /* ... */ } // read from blooms as needed
  }
}
```

Registration (fonts, icons, assets, `tune`) happens **before** `photosynthesize` and only there —
it needs `&mut Foliage`, which stops existing once the loop takes it. Everything after is the
event loop: drain emissions, then do per-frame work.

## Three surfaces, and which one you want

| you want to… | surface | where |
|---|---|---|
| describe an element as you spawn it | `Author` (builder methods on every `*Sprout`) | `author.rs` |
| change an element that already exists | `Grows` (verbs) | `boundary/verbs.rs` |
| read what the tree resolved to | `Canopy` (inherent methods) | `boundary/canopy.rs` |
| answer something that happened | `Bloom` | `boundary/bloom.rs` |

`Grows` is sealed — implemented for `Canopy` (queues into the frame's buffer) and `Sprig` (queues
across threads, behind a lock). Both read identically at the call site. Read `verbs.rs` top to
bottom once; it is the complete list of things an app can ask the engine to do, and it is short.

Every verb is safe to queue against a `Leaf` that has withered by the time it applies — it is
silently dropped, never a panic. That is why teardown races are mostly not your problem, and why
nothing here asks you to check `Presence` before every call.

## Elements

`Bare` (layout/hit only, draws nothing), `Panel`, `Text`, `TextInput`, `Icon`, `Image`, `Polygon`,
`Polyline`. Each is `X::new()` returning a sprout you chain `Author` methods onto, then
`canopy.branch(parent, sprout)` → `Leaf`.

There is **no widget library**. Buttons, chips, switches, scrollbars are app assembly: a `Bare`
hit target with decorative children that `.pass_through()`. This is deliberate; see
`application/` in this workspace for the idioms worth copying.

## Layout

`grid/location.rs` is the file. A `Location` pins each axis with **two values**:

```rust
Location::new().xs(
    16.px().as_left().with(100.pct().as_right().adjust(-16)),  // horizontal
    50.pct().as_center_y().with(24.px().as_height()),          // vertical
)
```

- Breakpoints: `xs` (required, the fallback) then `sm`/`md`/`lg`/`xl`, plus `short` for
  vertically-cramped viewports.
- Units come from `GridExt`: `px`, `pct`, `col`, `row`, `letters`.
- Designators turn a value into a role: `as_left`, `as_right`, `as_top`, `as_bottom`, `as_width`,
  `as_height`, `as_center_x`, `as_center_y`, and `as_x`/`as_y` (which resolve to `Points` rather
  than a `Section`).
- `anchor()` reads geometry off *another* element; `text_content()` reads a text run's own
  measured size.

> **The pairing trap.** Two values fully pin an axis, so they must be *(edge, size)* or
> *(edge, edge)* — never an edge measured from one end plus a size measuring the same span.
> `.as_right().adjust(-W)` together with `.with(W.px().as_width())` counts `W` twice and the
> element lands a `W` short of where you meant. This is the single most common layout bug.

## Parenting, anchoring, elevation — three independent axes

Do not conflate them. Each decides exactly one thing:

- **Parenting** (`canopy.branch(parent, …)`) decides ownership, **clipping**, and opacity
  inheritance. A child is clipped to the intersection of every ancestor's box.
- **Anchoring** (`Author::anchored`, or `Grows::anchor` to repoint later) decides only *whose
  geometry `anchor()` values read*. The target can be anywhere in the tree. A `Location` carrying
  `anchor()` values and no anchor cannot resolve.
- **Elevation** (`Author::elevate`) decides draw order and who wins a hit test. Two siblings at
  equal elevation that share pixels have **no defined order** — always separate them.

A floating panel that must not be clipped by a scrolling region therefore hangs off the root
(parenting) while anchoring to something deep inside it (anchoring).

## Views and scrolling

Giving an element a `Grid` makes it a **view**. Its scrollable `extent` is grown each frame from
its **children's sections** (`grid/view.rs`, `extent_check`).

> A child positioned *outside* its parent's box therefore enlarges what the parent can scroll to,
> even if it is invisible. An off-screen "parked" element is a scrollbar you did not order.

Drive with `Grows::scroll` + `ScrollTo`; read with `Canopy::sample(view, Sap::ScrollProgress)` or
`Canopy::scroll_offset`. `Author::overscroll` controls whether unconsumed scroll passes outward;
`Author::holds_drag` stops a drag on a control from also panning the region under it.

## Animation

`Grows::animate(leaf, Motion::…, Timing)` — or `animate_during(.., sequence)` to join a
`sequence()`, which emits one `Bloom::SequenceFinished` when the whole group lands.

`Timing::over(ms).after(ms).eased(Ease::…)`. Easings include `Ease::Linear` and `Ease::EMPHASIS`.

`Motion` does **not** cover everything a verb can write. Notably `draw_progress` (polyline
draw-in) and `scroll` are plain writes with no tween — drive them by hand from `frame_time()`.
`Grows::tween` exists for exactly this: it runs the engine's easing over bare numbers and reports
each frame as `Bloom::Tween`, so a library can build its own animatable properties.

> **One writer per property.** A plain `canopy.location()` landing while a `Motion::Location`
> tween is in flight resolves against that tween's cached difference rather than against the box,
> and the element jumps. If a property is animated anywhere, animate it everywhere — and build
> each target as a whole fresh `Location`, never by adjusting the current one.

## Interaction

`Author::interactive` makes an element compete for gestures; the topmost interactive element under
the pointer wins. `pass_through` marks decoration that is notified but never wins — put it on every
child of a control, or the control stops working. `round_hit_area` hit-tests an inscribed circle.

Hit testing is **rectangles and inscribed circles only**. Anything else — a polyline, a diagonal —
is not directly targetable; lay invisible `Bare().interactive()` boxes over the parts you want hit.

`Bloom` covers `Clicked` / `Engaged` / `Dragged` / `DragStarted` / `Disengaged`, focus, keys,
`TextChanged` / `TextAction`, `Tween` / `TweenDone`, `TimerFinished`, `SequenceFinished`,
`AssetLoaded`, `Withered`, `Resized`. Read the enum; the per-variant docs say precisely when each
fires and how they order.

> **Opacity is not visibility.** A zero-opacity element still draws and still takes clicks. Fade a
> control in and it is live and invisible for the length of the fade. `disable()` it and re-enable
> from a `timer()` when the fade lands — `application/src/site/mod.rs::arm_at` is the pattern.
> `Grows::visible` is the real hide: skipped by drawing *and* hit-testing.

## Text and text input

`Text` measures in character cells, so **fonts must be monospaced**; `1.letters()` is a unit for
exactly this reason. `text_content()` gives a run its own measured size.

> **Measure containers, not `Text`.** A `Text`'s section is what it *became* — a string that
> wrapped reports the wrapped block's width. A budget computed from that is wrong in the direction
> that caused the wrap. Put the text in a declared box and measure the box.

`TextInput` (`text_input/`) is a composite: field, text, hint and caret are separate entities, so
a click arrives naming one of *those*, never the `Leaf` you were handed. Match by geometry
(`Canopy::section(..).contains(Canopy::pointer().current)`) rather than by leaf, or presses into
your own field read as presses on whatever is behind it.

Style is one unit — `TextInputStyle`, poked via `Grows::input_style` — carrying foreground, hint
and accent only. `TextInput` draws no backdrop of its own (a rounded one needs an inset to avoid
clipping a glyph flush against it, which fought every other layout invariant here) — wrap it in
your own `Panel` for background/rounding/outline, sized and inset however that panel's own corners
need. `LineConstraint::Single` makes Enter a submission (`Bloom::TextAction` with
`TextInputAction::Enter`); `Multiple` makes it a newline.

## Colour, rounding, opacity

- `color.rs` — the full Tailwind palette as constructors (`Color::stone(950)`, `Color::sky(400)`,
  …) across luminance steps 50–950. Hues: red amber orange yellow lime green emerald teal cyan sky
  blue indigo violet purple fuchsia pink rose slate gray zinc neutral stone.
- `rounding.rs` — `Xs`/`Sm`/`Md`/`Lg` are fixed logical-px radii (clamped so a small element can't
  be asked for more curve than it has room for): the same visible curve on a chip and a card.
  `Full` is the one bracket that's proportional instead — half the shorter side, always a true
  pill/circle, which is the only shape where that has to track the box rather than stay fixed.
- `opacity.rs` — write `Opacity`; the engine maintains `InheritedOpacity` (product of ancestors)
  and `BlendedOpacity` (what the renderer multiplies in). Writing a parent's opacity propagates,
  so fading a subtree is one call on its root.

## Assets and icons

`Grows::load_asset` starts a load and hands back a key immediately; bytes arrive as
`Bloom::AssetLoaded` and are read with `Canopy::asset`.

Icons are MTSDF fields baked ahead of time by the `foliage_icons` crate in this workspace:

```
cargo run -p foliage_icons --release -- \
  gen --svg <svg-dir> --out <gen-dir> --field-size 48 --px-range 3
```

It emits a `generated.rs` with an `#[icon_handle]` enum and a `register(&mut Foliage)`. Call
`register` before `photosynthesize`; nothing can draw an icon that was not registered there.

## Where things live

`foliage` is a thin re-export; `foliage_proper` is the engine. Every path below is where a `pub`
item you'd actually reach for is *defined* — not an invitation to read the whole file. Each one
also holds several times its own weight in `pub(crate)` machinery (a drawable's `pipeline.rs`
equivalent, a composite's internal entities, reactive systems) that exists to make the `pub`
surface work, not to be called. `cargo doc --open` already filters that out for you; a raw file
does not.

| | |
|---|---|
| `foliage.rs` | `Foliage` — construction, `tune`, `font`, `icon`, `photosynthesize` |
| `boundary/verbs.rs` | `Grows` — every verb an app can call |
| `boundary/canopy.rs` | `Canopy` — every read |
| `boundary/bloom.rs` | `Bloom` — every emission |
| `author.rs` | `Author` — the spawn-time builder methods |
| `grid/location.rs` | `Location`, units, designators, `anchor()`, `text_content()` |
| `grid/view.rs` | `View` — scroll offsets and extents |
| `color.rs` `rounding.rs` `opacity.rs` `visibility.rs` | the small vocabularies |
| `text/mod.rs` | `Text`, `TextSprout`, `GlyphColors`, `FontSize` |
| `text_input/mod.rs` | `TextInput`, `TextInputStyle`, `LineConstraint` |
| `panel/` `icon/` `line/` `polygon/` `image/` | one module per drawable kind — `X::new()` + `Author` |

Nothing in `boundary/op.rs`, `ash/`, or `interaction/mod.rs`'s hit-testing is `pub` at all — they
are how the above gets executed, not a further surface to reach for. If you find yourself reading
one of those three to answer an app question, stop: either the answer is a behavioral note that
belongs in this file (say so, and it'll get added), or the question is actually about modifying
the engine, which is a different task.

## Reference app

`application/` in this workspace is a real app built on all of the above and is the best source of
idioms — icon buttons, drawers, staggered entrances, arming faded-in controls. Ignore `android/`.
