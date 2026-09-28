# TODO

## Word boundaries

`Ctrl+Left`/`Right` (and `Ctrl+Backspace`/`Delete` if wanted) and double-tap to select.

- Rule: a run of letters and digits, a run of punctuation and a run of whitespace are each one
  unit; a word move skips whitespace and then one unit. Hand-rolled on `char` classes, no new
  dependency.
- Keys: `applied` in `foliage/src/text_input.rs` is pure (value, `Editing`, `Keystroke`, wrap) and
  already branches on `stroke.modifiers.control` first — that branch currently holds the clipboard
  chords and select-all. Word motion goes there, as another `Toward` in `moved`.
- Double-tap: nothing in `interaction/` counts taps today. Taps reach a field through
  `grove.drift` in `Field::gestured` (`text_input.rs`, the `Fronds` impl). A tap count needs a time
  and distance window in `interaction/`, which is gesture work, separate from the word rule.
- Tests: the pure half is `edit(...)` in `foliage/src/tests/text_area.rs` / `text_input.rs`.

## Remembered column

`Up`/`Down` through a short line should come back to the column the caret set out from.

- Add a goal to `Editing` (`text_input.rs`, beside `caret` and `anchor`). An x in cells or logical
  pixels, not a character index. Cells are uniform, so a column count works too.
- `moved` handles `Toward::Up`/`Down` with `wrap.cell_of` and `wrap.index_at`. Set the goal on the
  first vertical move, keep it across vertical moves, and clear it on every other key and on a press
  (`gestured`).
- The existing test `a_column_past_a_shorter_line_lands_at_its_end` asserts today's behaviour and
  will need to change.

## IME commit (Android typing)

Android's soft keyboard types through the IME, which winit reports as `WindowEvent::Ime`. Nothing
handles it, so fields take no typing on Android.

- Translation lives in `foliage/src/photosynthesize.rs`, in the `window_event` match (next to
  `KeyboardInput`). Map `Ime::Commit(text)` to one `Input::Keyed(Key::Typed(c))` per character,
  the same way the web's hidden input feeds `Incoming` (`keyboard.rs`, `captured`).
- Call `window.set_ime_allowed(true/false)` when a field gains or loses focus. The natural place is
  `Keyboard::raise` (`keyboard.rs`), which `settled` already calls with the focused field's keypad.
- Ignore `Ime::Preedit` for now (inline preedit is not planned). Check whether desktop platforms
  also start sending Commit instead of `KeyboardInput` text once IME is allowed, so text doesn't
  arrive twice.
- The `keyboard.rs` module docs ("what is typed arrives as ordinary key events") are wrong; fix them.

## `content()` width from children

`content()` differs by axis. As a height, it is the greater of the element's wrapped run and the
furthest any child reaches down (R2m: `wrap` → `reach` in `foliage/src/rowan.rs`). As a width, it
is the run's max-content only (R1: `measure` in `rowan.rs` writes `Area::new(width, 0.0)`), so a
container sized `width(content())` comes out 0 whatever it holds. That rules out a button or chip
that hugs a separate label child.

- Mirror R2m horizontally: a bottom-up measure before R2a, where each element's intrinsic width is
  the max of its run and the furthest right any child reaches.
- Same exclusion rule as vertical: a child whose horizontal placement reads the trunk's width
  (`pct`, `col`, trunk or anchor edges) is left out of the measure, so nothing is circular. The
  vertical version of that rule is `measurable` / `known` in `placement/role.rs`, and a horizontal
  twin goes beside it.
- Dirtying: `wrap` marks `measuring` up the trunk chain and sets `dirty` when a measure moves. The
  horizontal pass needs the same thing.
- Update the `content()` docs in `placement/source.rs`, which currently describe width as the
  element's own max-content only.

## Outline on a panel

A border drawn around a panel: text field borders, cards, focus rings, outlined buttons. Stacking
a panel inside a larger one fakes it only when the inside is filled; a border around a transparent
inside cannot be done today.

- Shader: `foliage/src/ash/panel.wgsl` already computes `d = sd_rounded_box(...)` per fragment.
  An outline of width `w` is coverage of `d` inside the ring `-w..0`: `sd_coverage(d)` minus
  `sd_coverage(d + w)`, or `abs(d + w/2) - w/2` as the ring's distance. Fill and outline are
  composited in the same fragment, so there is no second draw.
- Data path: `PanelPigment` (`elm.rs`: `fill`, `rounding`) → `PanelInstance::new` (`panel.rs`:
  `section`, `color`, `radii`) → the vertex layout (`@location(1..4)`). Add an outline width and an
  outline colour at each step, with new vertex attributes after `depth`.
- Draw it inside the box, as CSS `box-sizing: border-box` does. That is the natural answer with an
  SDF, and it keeps the outline out of layout entirely.
- Decide: whether the outline colour is a `Fill` that animates with `Motion::Color` the way a
  panel's fill does (see how `color`/`Op::Recolor` reaches `PanelPigment.fill`), or fixed at
  planting for now.
- Builder on `Panel` (`foliage/src/panel.rs`), e.g. `.outline(width, fill)`. An op to change it
  after planting only if focus rings need it — a focus ring is exactly that case.

## Glyph atlas

`foliage/src/ash/atlas.rs`: a fixed 2048² sheet keyed by `Cut { font, size, density, character }`.
It packs shelves and never frees anything. When full, `pack` reports once and new characters stop
drawing for good.

- On a scale-factor change, clear it. `reconfigure` in `photosynthesize.rs` already detects the
  change and calls `grove.elm.recut()`, which re-extracts every instance but does not touch the
  atlas. Reset the `Atlas` (held by the text renderer, `ash/text.rs`) there, so glyphs are cut
  again at the new density.
- When full, rebuild from only the cuts this frame draws, and report only if those alone don't fit.
  The rebuild has to happen before this frame's instances are written with their uvs.
- The icon sheet (`ash/sheet.rs`) needs neither rule: its marks are a fixed set named at boot.
