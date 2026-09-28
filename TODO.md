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
