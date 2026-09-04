# What is left

The working checklist for foliage 1.0. `spec/` states the design and `spec/README.md` records how
each slice arrived; neither is a list of what is still owed. This is that list.

## Where it stands

| Gate | State |
|---|---|
| `cargo test --workspace` | 417 headless tests, 22 doctests, 14 compile-fail doctests — passing |
| `cargo check -p application` | passing (one dead-code warning: `Instances::len`, `Instances::holding`) |
| `cargo check -p foliage --target wasm32-unknown-unknown` | passing |

Landed: A7, B1–B9 and C1, plus the long-press primitive and drag auto-scroll. Not started: B10
(platform edges) and B11 (`Sprig`).

The engine is complete as a *model*. Every law in `spec/frame.md` is implemented and proven, the
tree resolves, six renderers draw through one stack, and one composite is built out of the same
parts an app has. What is missing is not model — it is reach: the app cannot hear a keyboard, cannot
load anything from disk or network, and cannot be built for anything but the desktop.

## Against the previous engine

Every concept that carries the model is ported. What is left in `../working-foliage` is platform
integration, tooling, and four features that were deliberately dropped.

| Only in the previous engine | Disposition |
|---|---|
| `asset` — `AssetSource`, `AssetLoader`, `OnRetrieval` | B10 |
| `clipboard` | B10 |
| `virtual_keyboard` | B10 |
| `web_ext` — `HrefLink`, download, video and document embeds | B10 |
| `platform`, `foliage_android`, `application_android` | not ported; no target builds for Android |
| `boundary::sprig` — `Sprig`, `Conditions`, watched `Moss` | B11 |
| `polyline` — `DashPattern`, `PolylineDrawProgress` | B8 owed; needs a renderer of its own |
| `panel::Outline` | not ported, and nothing replaces it — a bordered box has no expression |
| `anim::Repeat` | not ported, and nothing replaces it — a looping motion has no expression |
| `alignment` — `HorizontalAlignment`, `VerticalAlignment` | not ported; a run is placed by its box and nothing centres text inside one |
| `grid::AspectRatio` | not ported; no equivalent in the placement grammar |
| `image::ImageView` — a source rect | not ported; `Fit` answers the two cases the page needed |
| `text_input::KeyBindings`, `LineConstraint` | dropped by decision (B9): no binding table, one line only |
| `foliage_macros` | not needed — no targeted events and no icon handles in this engine |
| `foliage_icons` — SVG to MSDF | not ported, and it is the only thing that produced an icon field |
| `xtask` — site, serve, book, api, changelog, web | not ported |
| `lichen` | already empty upstream; nothing to port |
| `book/`, `examples/` | not ported |

**The old engine is worth keeping only as reference for the four platform modules and the MSDF
baker.** Everything else in it is either ported or replaced by something this engine states
differently on purpose, so the new shape should be judged on its own consistency.

## 1. Keys are not reachable from an app

The largest hole in the public surface, and it is not on any slice's list.

`Key`, `Modifiers` and `Keystroke` are `pub(crate)`. Dispatch sends `Tab` and `Escape` to focus and
everything else to the element holding focus — where only `TextInput` consumes one. There is no
`Pollen` for a keystroke, so an app cannot answer `Enter` on a focused button, arrow keys on a list,
or any shortcut it defines. Every app that is not a form needs this.

What it needs decided: which of `Key` becomes public, what a key that reaches an element with no use
for it does, and whether a report is keyed on the focused `Leaf` or stands on its own the way
`tween` does.

## 2. B10 — the platform edges

| | Unblocks |
|---|---|
| Assets | a font, an icon field or a picture that is not `include_bytes!` — including the web, where the fetch is asynchronous and the element has to exist before its pixels do (`plate`/`load` already have the shape for this) |
| Clipboard | `Ctrl+C`/`Ctrl+V`, which B9 left owed on the clipboard rather than on the modifier |
| Virtual keyboard | `TextInput` on a phone — a field that cannot raise a keyboard is not usable there |
| Web ext | a link that navigates, and a download |

## 3. B11 — `Sprig`

Off-thread ops, proven indistinguishable from in-frame ops. The names are already allocated from an
atomic counter for exactly this, so what is left is the channel, the drain-side receipt, and the
watched reads the previous engine's `Conditions` covered.

## 4. Owed inside slices that landed

- **`Polyline`** (B8). A path as one element, one coverage evaluation, a dash pattern and
  `Motion::DrawProgress`. `application/` draws its series as a chain of square-capped `Line`s.
- **`LineQuad::new` snaps a stroke whose ends share a coordinate** (B8). A hazard rather than a
  defect today: in a chain, one axis-aligned segment would stop meeting its neighbours. Whatever
  owns a path answers it.
- **Sheet eviction** (B8). Deferred with a reason — marks are bounded and pictures have a texture
  each, so nothing fills the shared sheet. Shelves are the reclaim unit if it is ever needed, and it
  needs the character kept per glyph to re-cut what it orphans.
- **`.extent(..)`** (B7). Waits for a virtualised list, and for the verb that undoes it.
- **A blinking caret** (B9). A frame owed for as long as a field holds focus, weighed against F9's
  idling.
- **Composition** (B9). A key that produced text is taken as that text; an inline preedit needs a
  run drawn in a state it does not have.
- **A text area** (B9). More than one line puts the caret back into the wrap walk. A second element,
  when something wants it.
- **Word-wise motion** (B9). `Ctrl+Left`/`Right` and friends — a decision about where a word ends.
- **Recolouring a field after it is grown** (B9). `color` reaches none of a field's four fills, and
  which part it would mean is unanswered.

## 5. What the crate needs before anyone else can use it

None of this is engine work, and all of it is between here and "a library".

- **No `README.md` and no licence files** in `foliage/`. `Cargo.toml` claims `MIT OR Apache-2.0` and
  points `repository`, `homepage` and `documentation` at pages that do not exist yet.
- **No `#![deny(missing_docs)]`.** The surface is documented today; nothing keeps it that way.
- **No examples.** `application/` is an API gate, not a teaching artifact — it is one page that uses
  everything at once, which is the opposite of what a first read wants.
- **No way to make an icon field or image pixels.** `Foliage::icon` takes a baked MSDF and
  `Foliage::image` takes decoded RGBA; nothing in the repo produces either, and `application/`
  generates both procedurally. Port the baker as its own tool, or document the field format and name
  a decoder an app is expected to bring.
- **No CI.** No golden images (`spec/harness.md` names them as the second tier), no native matrix,
  no wasm build, no `.github` at all.
- **No web or Android build path.** No `Trunk.toml`, no Android crate, no `xtask`. The wasm target
  compiles; nothing packages it.
- **No book.** `spec/README.md` says each slice lands with a chapter; none exist.

## 6. Settled — not open questions

Listed so they are not reopened by a reader who did not see them decided: absolute elevation, a
hit test that walks the stack, `ClipToViewport` (replaced by `floats`/`Escape`), a keybinding table,
a second cache beyond text shaping, per-element tracing events, and a `Polyline` built as a single
stack entry rather than a single coverage evaluation.

## 7. For the optimisation pass

Candidates, not measured, and all deliberate as built:

- Rowan recomputes the whole tree every frame by design; the frame's cost is the tree's size rather
  than the change's.
- Extraction gathers runs into a scratch kept between frames — an unchanged frame allocates nothing
  there, but a changed run is rewritten entire.
- `Pollen` builds its sets each frame and hands out an `Arc`.
- The shared walk cuts spans on renderer, clip and binding, and runs only when the stack moved.
- The sheet never reclaims; a failed pack draws blank and traces once.

## Two finish lines

**Useful to someone else** — §1 keys, §2 assets and clipboard, and §5's packaging, examples and an
answer for icon fields. Virtual keyboard as well, if a phone is in scope.

**Feature-complete against the plan** — the above, plus B11, `Polyline`, the B9 field items, CI with
golden images, and the book.
