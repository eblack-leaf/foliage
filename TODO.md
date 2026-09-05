# TODO

What is left. Every slice of the plan is implemented; what follows is either owed inside a slice
that landed, waiting on a platform that has no build, or the distance between a working engine and a
crate someone else can use.

Gates today: `cargo test --workspace` — 493 headless tests, 27 doctests, 14 compile-fail doctests.
`cargo check -p application` and `cargo check -p foliage --target wasm32-unknown-unknown` both pass.

## Decide

- **Whether a path gets an element of its own.** A chain of `Line`s is correct opaque at every
  weight and angle with `Cap::Round`, and wrong at every vertex under partial alpha, where two
  overlapping strokes paint the ground twice. Dashes and `Motion::DrawProgress` have no expression
  in a chain either way. `POLYLINE.md` is the design and what it costs; the call is whether a path
  in foliage can ever be translucent or fade.

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

## Platform

Four arms where the seam is built and the target is not. Each has a settled surface; only the arm
changes when it lands.

- **A native http client.** `Origin::url` is nameable everywhere and fetched only on the web; off it
  a URL is accepted and answered as `missing`. It lands behind a feature that brings the client and
  the TLS stack with it — the `TODO` sits at that arm in `asset.rs`.
- **A paste from another program, on the web.** `navigator.clipboard.readText()` is permission-gated
  and refused outside a user gesture, and a frame is not one; refused, the engine's own mirror
  answers. The road out is a `paste` event on the hidden input the keyboard already owns.
- **A soft keyboard anywhere but the web.** Android is the other platform with one and there is no
  Android build to raise it from, so `Keypad` is carried, reported and ignored there.
- **A download off the web.** A browser is what turns a URL into a file; off it the verb is traced
  and does nothing, since a program that wants a file on disk has `std::fs`.

## Crate

None of this is engine work, and all of it is between here and a library.

- **A `README.md` and licence files.** `Cargo.toml` claims `MIT OR Apache-2.0` and points
  `repository`, `homepage` and `documentation` at pages that do not exist.
- **`#![deny(missing_docs)]`.** The surface is documented today and nothing keeps it that way.
- **Examples.** Small ones, each doing one thing: a placement, a gesture, a field, a series.
- **A way to bake an icon field.** A picture is decoded here now, but `icon` still takes a baked MSDF
  and nothing in the repo produces one. Port the baker as its own tool, or document the field format
  so another can. This and the Android modules are the only reasons to keep `../working-foliage`.
- **CI.** A build matrix — native targets and wasm — and the test suite. Rasterisation is checked by
  eye rather than by test, deliberately, so no golden images.
- **A web and Android build path.** The wasm target compiles and nothing packages it: no
  `Trunk.toml`, no Android crate, no `xtask`.
- **A book.** The slice plan named a chapter each; none exist.

## Not ported, and not owed

Recorded so it is not mistaken for an oversight: `Outline` on a panel, `Repeat` on an animation,
horizontal and vertical alignment inside a box, `AspectRatio`, a source rect on an image, a
keybinding table, and video and document embeds drawn in the DOM over the canvas. Each was dropped by
decision, and nothing in the engine replaces any of them.
