# TODO

What is left, which is all engine: work owed inside a slice that landed, rather than anything
between the engine and a crate someone else can use.

Gates today: `cargo test --workspace` — 496 headless tests, 34 doctests, 12 compile-fail doctests.
`cargo check -p application`, `cargo check -p foliage --features origin-url` and `cargo check -p
foliage --target wasm32-unknown-unknown` all pass.

## Engine

- **A blinking caret.** The caret is solid while focused. A blink is a frame owed for as long as a
  field holds focus, which is what F9's idling is weighed against.
- **Composition.** A key that produced text is taken as the text it produced, which covers a dead key
  and a committed sequence. An inline preedit needs a run drawn in a state it does not have.
- **A text area.** A field is one line. More than one puts the caret back into the wrap walk, which
  answers a cell per character rather than a column for one — a second element, when something wants
  it.
- **Word boundaries.** Word-wise motion (`Ctrl+Left`/`Right` and friends) and double-tap to select
  wait on the same decision: where a word ends. Neither is about gesture timing or which modifier was
  held.
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
