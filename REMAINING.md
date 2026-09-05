# What is left

The working checklist for foliage 1.0, and the one document meant to outlive the planning ones.

## Where it stands

| Gate | State |
|---|---|
| `cargo test --workspace` | 450 headless tests, 24 doctests, 14 compile-fail doctests — passing |
| `cargo check -p application` | passing (one dead-code warning: `Instances::len`, `Instances::holding`) |
| `cargo check -p foliage --target wasm32-unknown-unknown` | passing |

Landed: A7, B1–B10 and C1, plus the long-press primitive, drag auto-scroll, and the public key
surface. Not started: B11 (`Sprig`).

The engine is complete as a *model*, and it now reaches the host. The frame law is implemented and
proven end to end, the tree resolves, six renderers draw through one stack, one composite is built
out of the same parts an app has, a font, a mark or a picture can be read from a path or a URL, and
the clipboard, a soft keyboard and a URL handed over are all reached through the same queue every
other change goes through. What is missing is not model and no longer reach — it is **packaging**:
no build for anything but the desktop, no CI, and nothing that turns artwork into an icon field.

## Against the previous engine

Every concept that carries the model is ported, and so is every platform edge that had a target to
reach. What is left in `../working-foliage` is the Android integration, the tooling, and the
features that were deliberately dropped.

| Only in the previous engine | Disposition |
|---|---|
| `asset` — `AssetSource`, `AssetLoader` | ported as `Origin` and an arrival op. `OnRetrieval` (a callback) and `bundled_asset!` (a macro over a `cfg`) were rejected |
| `clipboard` | ported as `copy`/`paste` and `Pollen::pasted`. The read is an arrival op rather than a return value, so the web's promise and a desktop's round trip answer alike |
| `virtual_keyboard` | ported as `Keypad`, raised by focus alone. `VirtualKeyboardAdapter::open`/`close` were rejected: a keyboard an app raises by hand is one it can leave up over nothing |
| `web_ext` — `HrefLink`, download | ported as `navigate` and `download` |
| `web_ext` — video and document embeds | not ported. A DOM overlay above the canvas is the browser's own player in front of the engine rather than an edge of it, and nothing has asked for one |
| `platform`, `foliage_android`, `application_android` | not ported; no target builds for Android, which is why a keypad is raised on the web and nowhere else |
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

**The old engine is worth keeping only as reference for the Android modules and the MSDF baker.**
Everything else in it is either ported or replaced by something this engine states differently on
purpose, so the new shape should be judged on its own consistency.

## 1. B11 — `Sprig`

Off-thread ops, proven indistinguishable from in-frame ops. The names are already allocated from an
atomic counter for exactly this, so what is left is the channel, the drain-side receipt, and the
watched reads the previous engine's `Conditions` covered.

## 2. Owed inside slices that landed

- **A native http client** (B10). `Origin::url` is nameable everywhere and fetched only on the web;
  off it a URL is accepted and answered as `missing`, because a client and a TLS stack are a large
  addition nothing has asked for yet. The surface is settled and the arrival is the only thing that
  changes when it lands, behind a feature that brings the client with it — the `TODO` sits at that
  arm in `asset.rs`.
- **A paste from another program, on the web** (B10). `navigator.clipboard.readText()` is
  permission-gated and refused outside a user gesture, and a frame is not one. Refused, the engine's
  own mirror answers — so a copy inside the app round-trips and one from outside it may not. The
  road out is a `paste` event on the hidden input the keyboard already owns, which is the one place
  a browser hands over what was pasted without asking.
- **A soft keyboard anywhere but the web** (B10). Android is the other platform that has one and
  there is no Android build to raise it from, so `Keypad` is carried, reported and ignored there.
  What is missing is the target, not the seam.
- **A download off the web** (B10). A browser is what turns a URL into a file in someone's
  downloads; off it the verb is traced and does nothing, because a program that wants a file on disk
  already has `std::fs` to write it with.
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

## 3. What the crate needs before anyone else can use it

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

## 4. Settled — not open questions

Listed so they are not reopened by a reader who did not see them decided: absolute elevation, a
hit test that walks the stack, `ClipToViewport` (replaced by `floats`/`Escape`), a keybinding table,
a second cache beyond text shaping, per-element tracing events, a `Polyline` built as a single
stack entry rather than a single coverage evaluation, a verb that opens or closes the soft keyboard,
a clipboard read that returns what it read, and a media overlay drawn in the DOM over the canvas.

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

**The host is reached through the queue, and only through it.** A `copy`, a `navigate` and a
`download` are ops, drained in arrival order with everything else; a `paste` is an op whose *answer*
is another op, pushed from wherever the read finished and drained in the frame it landed in. That is
the asset road, and taking it twice is what makes a paste mean the same thing on a promise and on a
round trip instead of landing in this frame on one target and the next on the other. There is no
call on the engine that reaches the platform and returns.

**Every platform edge is opened by `photosynthesize` and shut under the headless suite** — the seam
`Wake` already sat on. The clipboard then answers from the engine's own mirror, the keyboard records
what it was asked to raise and raises nothing, and a URL goes nowhere: which is what proves the rules
on this side of the seam and keeps a test off the clipboard and the browser of whoever runs it.

**A soft keyboard is raised by focus and by nothing else.** A field is the only thing that is typed
into and focus already rests only on what receives, so there is no verb for opening one and none for
closing it — the same reasoning that made `interactive()` the whole declaration for keys. `Keypad`
is the one thing a field says about it, and it is a hint about which keys are easy rather than a
rule about what the value may be.

## 5. For the optimisation pass

Candidates, not measured, and all deliberate as built:

- Rowan recomputes the whole tree every frame by design; the frame's cost is the tree's size rather
  than the change's.
- Extraction gathers runs into a scratch kept between frames — an unchanged frame allocates nothing
  there, but a changed run is rewritten entire.
- `Pollen` builds its sets each frame and hands out an `Arc`.
- The shared walk cuts spans on renderer, clip and binding, and runs only when the stack moved.
- The sheet never reclaims; a failed pack draws blank and traces once.

## Two finish lines

**Useful to someone else** — §3 entire: a `README`, licence files, examples, packaging, and an
answer for icon fields. Nothing in §1 or §2 stands between the engine and someone else using it.

**Feature-complete against the plan** — the above, plus B11, `Polyline`, the B9 field items, the B10
items in §2, CI with golden images, and the book.
