# What is left

The working checklist for foliage 1.0, and the one document meant to outlive the planning ones.

## Where it stands

| Gate | State |
|---|---|
| `cargo test --workspace` | 435 headless tests, 24 doctests, 14 compile-fail doctests — passing |
| `cargo check -p application` | passing (one dead-code warning: `Instances::len`, `Instances::holding`) |
| `cargo check -p foliage --target wasm32-unknown-unknown` | passing |

Landed: A7, B1–B9 and C1, plus the long-press primitive, drag auto-scroll, the public key surface,
and the asset road. Not started: B11 (`Sprig`); B10 has its assets half and not its platform half.

The engine is complete as a *model*. The frame law is implemented and proven end to end, the tree
resolves, six renderers draw through one stack, one composite is built out of the same parts an app
has, and a font, a mark or a picture can be read from a path or a URL. What is missing is not model
— it is reach: no clipboard, no keyboard on a phone, and no build for anything but the desktop.

## Against the previous engine

Every concept that carries the model is ported. What is left in `../working-foliage` is platform
integration, tooling, and four features that were deliberately dropped.

| Only in the previous engine | Disposition |
|---|---|
| `asset` — `AssetSource`, `AssetLoader` | ported as `Origin` and an arrival op. `OnRetrieval` (a callback) and `bundled_asset!` (a macro over a `cfg`) were rejected |
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

## 1. B10 — the platform edges

Assets landed; three edges are left.

| | Unblocks |
|---|---|
| Clipboard | `Ctrl+C`/`Ctrl+V`, which B9 left owed on the clipboard rather than on the modifier |
| Virtual keyboard | `TextInput` on a phone — a field that cannot raise a keyboard is not usable there |
| Web ext | a link that navigates, and a download |

Beside them, one gap the asset road left on purpose: **`Origin::url` is nameable everywhere and
fetched only on the web.** Off it a URL is accepted and answered as `missing`, because an http client
and a TLS stack are a large addition nothing has asked for yet. The surface is settled and the
arrival is the only thing that changes when it lands, behind a feature that brings the client with
it — the `TODO` sits at that arm in `asset.rs`.

## 2. B11 — `Sprig`

Off-thread ops, proven indistinguishable from in-frame ops. The names are already allocated from an
atomic counter for exactly this, so what is left is the channel, the drain-side receipt, and the
watched reads the previous engine's `Conditions` covered.

## 3. Owed inside slices that landed

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

## 4. What the crate needs before anyone else can use it

None of this is engine work, and all of it is between here and "a library".

- **No `README.md` and no licence files** in `foliage/`. `Cargo.toml` claims `MIT OR Apache-2.0` and
  points `repository`, `homepage` and `documentation` at pages that do not exist yet.
- **No `#![deny(missing_docs)]`.** The surface is documented today; nothing keeps it that way.
- **No examples.** `application/` is an API gate, not a teaching artifact — it is one page that uses
  everything at once, which is the opposite of what a first read wants.
- **No way to bake an icon field.** A picture is answered now — PNG or JPEG in, decoded here — but
  `icon` still takes a baked MSDF, and nothing in the repo produces one. Port the baker as its own
  tool, or document the field format so another can.
- **No CI.** No golden images — the second tier the headless suite explicitly cannot reach — no
  native matrix, no wasm build, no `.github` at all.
- **No web or Android build path.** No `Trunk.toml`, no Android crate, no `xtask`. The wasm target
  compiles; nothing packages it.
- **No book.** The slice plan named a chapter per slice; none exist.

## 5. Settled — not open questions

Listed so they are not reopened by a reader who did not see them decided: absolute elevation, a
hit test that walks the stack, `ClipToViewport` (replaced by `floats`/`Escape`), a keybinding table,
a second cache beyond text shaping, per-element tracing events, and a `Polyline` built as a single
stack entry rather than a single coverage evaluation.

**B8's "pixels and decoding are the app's" is reversed, deliberately.** foliage decodes PNG and JPEG
now, and that is what makes one road serve three destinations: `font`, `icon` and `image` each take
bytes an app holds or an [`Origin`] to read them at, and each hands back its ordinary handle at once.
Without the decoder a retrieval could only yield bytes, which meant a second handle for "bytes I am
waiting for" and a decode at every callsite. A picture states no size coming that way, because the
decode answers it and two answers can disagree. Pixels an app makes itself are `pixels`, which states
one because nothing can be read from them.

What follows from a name being valid before its bytes: a mark or a picture that has not arrived
occupies its box and draws nothing, and is absent from the batch rather than held as blank — so the
frame it lands is the frame it appears, with nothing to undo. A **font** that has not arrived is
measured in the bundled face instead, because a run has a cell either way and measuring zero would
collapse every column address on the page and spring it back.

Keys settled the same way, and the shape is worth stating once: a key goes to whatever holds focus,
focus rests only on what declared `interactive()`, so **`interactive()` is the whole declaration** —
there is no second flag saying an element listens. `Pollen::keys(leaf)` is ordered, which nothing
else in `Pollen` is, because two keys in a frame mean different things in each order. `Tab` and
`Escape` are focus's own and never reach an element. A key that arrived with focus nowhere is the
app's, through `Pollen::root_keys()`. Nothing consumes a key on another's behalf: a field is sent
what it edited with and the app is told the same keys.

## 6. For the optimisation pass

Candidates, not measured, and all deliberate as built:

- Rowan recomputes the whole tree every frame by design; the frame's cost is the tree's size rather
  than the change's.
- Extraction gathers runs into a scratch kept between frames — an unchanged frame allocates nothing
  there, but a changed run is rewritten entire.
- `Pollen` builds its sets each frame and hands out an `Arc`.
- The shared walk cuts spans on renderer, clip and binding, and runs only when the stack moved.
- The sheet never reclaims; a failed pack draws blank and traces once.

## Two finish lines

**Useful to someone else** — §1's assets and clipboard, and §4's packaging, examples and an answer
for icon fields. Virtual keyboard as well, if a phone is in scope.

**Feature-complete against the plan** — the above, plus B11, `Polyline`, the B9 field items, CI with
golden images, and the book.
