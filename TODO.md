# TODO

What is left, which is all engine: work owed inside a slice that landed, rather than anything
between the engine and a crate someone else can use.

Gates today: `cargo test --workspace` — 543 headless tests, 34 doctests, 15 compile-fail doctests.
`cargo check -p application`, `cargo check -p foliage --features origin-url` and `cargo check -p
foliage --target wasm32-unknown-unknown` all pass.

## Engine

- **A blinking caret.** The caret is solid while focused. A blink is a frame owed for as long as a
  field holds focus, which is what F9's idling is weighed against.
- **Composition.** A key that produced text is taken as the text it produced, which covers a dead key
  and a committed sequence. An inline preedit needs a run drawn in a state it does not have.
- **Word boundaries.** Word-wise motion (`Ctrl+Left`/`Right` and friends) and double-tap to select
  wait on the same decision: where a word ends. Neither is about gesture timing or which modifier was
  held.
- **A remembered column.** `Up` and `Down` through a short line land at its end and come back from
  it, where an editor remembers the column the caret set out from. A goal column is one more field
  on `Editing` and a rule for which keys clear it.
- **A selection re-split on the frame it rewraps.** Its geometry is the layout's, but how many
  boxes it takes is decided at the drain against the width the run was last drawn at, so a resize
  under a selection that crosses lines is right on the frame after.
- **Recolouring a field after it is grown.** Its four fills are stated when it is planted and `color`
  reaches none of them, because which part of a field a fill means is unanswered. What an app usually
  wants — an error state, a focus mark — is the ground it put the field in, which is its own panel
  and already writable.
- **Sheet eviction.** Nothing fills the shared sheet today: marks are a bounded set packed once and
  pictures have a texture each. Shelves are the reclaim unit if it is ever needed, and it needs the
  character kept per glyph to re-cut what it orphans.

## Not ported, and not owed

Recorded so it is not mistaken for an oversight: `Outline` on a panel, `Repeat` on an animation,
horizontal and vertical alignment inside a box, `AspectRatio`, a source rect on an image, a
keybinding table, and video and document embeds drawn in the DOM over the canvas. Each was dropped by
decision, and nothing in the engine replaces any of them.
