# foliage spec

The normative documents for foliage 1.0. Written before the implementation, and the authority when
they and anything else disagree — including any plan file, prior notes, or the previous engine's
own documentation.

`../../working-foliage` is a **reference to consult, never a source to copy from.** Its docs in
particular have been wrong three times about behaviour its code contradicts. Read the code, not the
prose.

## Documents

| | |
|---|---|
| `lexicon.md` | The vocabulary. Read first — every other document is written in it |
| `frame.md` | The frame law. F1–F9: one queue, one drain, one clock, what runs when |
| `lifecycle.md` | `Planted`/`Live`/`Withered`, and the three separate ways to be off |
| `pollen.md` | Emissions as a set you interrogate, not a list you walk |
| `rowan.md` | Resolution. Recompute totally, diff at the edge. The pure resolver |
| `placement.md` | Width down, height up; the `Location` grammar |
| `views.md` | Scrolling, extent, pinning, momentum |
| `interaction.md` | Hit-testing, gesture claiming, focus |
| `aspen.md` | Animation. What `Motion` covers and why |
| `harness.md` | How foliage is proven: the frame minus the rasteriser |
| `tracing.md` | What foliage reports about itself. The counterpart to silent op drops |

## The decisions these turn on

- **bevy_ecs stays; the internals do not.** The boundary means the ECS layer is free to be whatever
  the engine needs, and it is not a port of the previous one.
- **Rowan recomputes; Elm decides what changed.** No dirty tracking, no invalidation, no reactive
  observers for derived state. The one measured exception is text shaping.
- **The resolver is a pure function** of `(Location, Context)`, callable twice per entity per frame.
  This is what makes animated and content-sized layout work at all.
- **One op queue, one drain.** `Sprig` and in-frame ops are indistinguishable.
- **Nothing may depend on emission order** except keystrokes, which are genuinely ordered.
- **Static errors are compile errors.** Elevation, grids, and illegal axis pairings are types, not
  runtime panics. Proven with compile-fail doctests pinned to an error *code*, never a snapshot of
  the compiler's wording.
- **Everything is proven headlessly** against the same `Fern` the event loop runs, reached the way
  the loop reaches it rather than through public API.
- **A trace of one frame reads as the frame law.** Instrumentation is a convention every slice
  follows, not something added later once the engine is confusing.

## Slice order

Each slice lands complete: spec section, implementation, headless tests discharging every proof
obligation that section states, a section of `application/` that exercises it, and a book chapter.

| | Slice | Proves |
|---|---|---|
| A7 | Skeleton + the headless suite | Built first — it is what every slice below is checked with |
| B1 | `Leaf`, `Grove`, `Grow`, ops, `Pollen` | F1–F3: names, single-drain FIFO, withering, collection |
| B2 | Coordinates, `Grid`, placement | The layout model and grammar |
| B3 | `Panel`, `Elm`, R6 `rank` | Extraction: change → instance, and that an unchanged frame costs nothing |
| C1 | `Willow`, `Ginkgo`, `Ash`, `photosynthesize` | First pixels; change → GPU end to end. F9's loop |
| B4 | Interaction: stack, claiming, focus | Targeting, handoff, focus order and trapping |
| B5 | `Aspen`: tweens, sequences, timers | Timing, easing, one writer per property |
| B6 | `Text` and fonts | The character cell, wrapping, `content()` |
| B7 | Views and scrolling | Extent, pinning, chain vs contain |
| B8 | `Icon`, `Image`, `Polygon`, `Line`, `Polyline` | The remaining renderers |
| B9 | `TextInput` | The one composite |
| B10 | Assets, clipboard, web ext, virtual keyboard | Platform edges |
| B11 | `Sprig` | That off-thread ops are genuinely identical to in-frame ops |

`C1` is lettered apart because its evidence is a different kind. Every `B` slice is discharged by the
headless suite; `C1` is the platform layer that suite explicitly cannot reach (`harness.md`), and is
answered for by the site running, by the native matrix, and later by golden images. It sits directly
after `B3` because an extraction nothing consumes is a shape nothing has checked, and because eight
slices of renderers with nothing ever on screen is how a wrong instance layout survives to `B8`.

Crate layout: one `foliage` crate — `foliage_proper` and its facade collapse — plus
`foliage_macros`, `foliage_icons`, `application`, `xtask`, and `lichen` later.

## Verification

- `cargo test --workspace` — the headless suite. A real gate: no slice merges without tests for the
  obligations its spec section states
- compile-fail doctests for every illegal axis pairing (`placement.md`), pinned by error code
- `cargo check -p application` — the site builds against the public API only. If it cannot be
  built, the API is incomplete, and that *is* the test
- `cargo check -p foliage --target wasm32-unknown-unknown`
- Golden images for the renderers, once B8 lands — wgpu to a texture, no window, runs in CI
- Native matrix (linux/macos/windows)

## Status

Phase A is complete. All eleven documents are written and carry no open items.

A7 has landed. One `foliage` crate on `bevy_ecs` and `tracing`: `Fern` runs the frame, `Grove` is
the surface it runs against, and `Foliage` holds both at boot. `Leaf`, `Presence`, `Seed`, `Stem`,
`Grow`, the one queue and its drain, `Pollen`, `Vein`/`Sap`, and `Root`. Twenty-two headless tests.

The engine has no entry point until `photosynthesize` lands: nothing outside the suite calls
`Fern`, and F9 has no loop to govern.

B2 has landed. Logical-pixel coordinates; `Layout` and `Short`; the placement grammar as types —
role-first openers, the coordinate/length split, and the axis asymmetry that width-down/height-up
implies; `Grid` with its own `columns()`/`rows()` vocabulary; the pure per-axis resolver; `Rowan`'s
R2a/R2b in dependency order, with anchor cycles refused at the op. Eighty headless tests and nine
compile-fail doctests.

Still owed by B2: `content()` resolves against an intrinsic that is always empty, and `letters()`
against a character cell that is always zero, because both come from a font. `Tree::cell` is the one
seam B6 fills; the resolver arithmetic behind them is proven now.

B1 has landed. Every verb on `Grow` is proven against F1–F3: a write lands in the frame that planted
its leaf, two writes to one property resolve in arrival order, ops of different kinds are not
reordered against each other, and neither a move nor a teardown is visible inside the frame that
made it. A dropped op is proven for `at`, `grid` and `anchor` against both reasons a name is not
live — it withered, or its own grow was dropped — and the drain is proven total across one. Ninety
-three headless tests.

`plant` has no dropped case to prove: it allocates its own name, and a name is never reused, so
there is no occupied name for one to land on.

`application/` is a workspace member, and `cargo check -p application` is a gate. It plants a page
against the public surface alone: nested grids, a rail that turns from a strip into a column at
`md`, a marker that reads whichever entry it was last anchored to, a reading column under `at_most`,
and a notice pruned and then let go of when `Pollen` reports it. `content()` and `letters()` are
absent from it deliberately, because both resolve to zero until B6.

Writing it settled the authoring surface for breakpoints. `Location` and `Grid` are chains, one link
per breakpoint, each link stating both axes:

```rust
Location::new()
    .xs(left(1.col()).right(4.col()), top(0.px()).height(56.px()))
    .md(left(1.col()).right(3.col()), top(0.px()).bottom(100.pct()))
```

Both axes together, because what an element looks like at a given width is one thought and belongs
in one place. A chain per axis would scatter it across two links per breakpoint, which is the
model Tailwind uses and is the thing to avoid. `xs` is a link like any other rather than a required
constructor argument, so a chain that is never written falls back to the whole of the parent's box
and an element that only differs above a certain width states that width and nothing else.

The shape matters because a two-argument call at the head of a chain formats badly — it either runs
long or bursts, and the override then hangs off its closing paren. A trivial head makes every link
uniform, which is what keeps a responsive placement readable inline instead of forcing it into a
temporary.

B3 has landed. `Panel`, the palette, corner rounding, `Elm` and R6 `rank`. A hundred and thirty-two
headless tests and nine compile-fail doctests.

What an element draws is `Chlorophyll`, and it is the *only* thing that says so: extraction routes
on it and never on which components an element happens to carry, because a set of components that
looks like a panel is not a panel — and will not be, once `Icon` also has a fill, a rounding and a
rank. The renderer's declared state is `PanelPigment` beside it, grown at the same single site, so
`color` and `round` write the state and nothing ever writes the decision.

The holding `Elm` compares against is updated in place. Rebuilding it would allocate a map per
renderer per frame, on every frame including the empty ones, which would contradict the one claim
the phase makes.

A fill is a **role**, not a color: `Panel::new().color(Palette::Accent)`. What a role resolves to is
the palette's answer and changes in one place; `Color` stays the form a renderer reads and the form
a palette is stated in. The ramp behind the roles — tints, and a scheme to read them against — is
still owed, and until it lands each role resolves to one fixed value. No callsite names one, so the
ramp replaces those values and nothing else.

Elevation is relative and only relative, and the absolute form was cut for a reason stronger than
the one that first cut it: see `lifecycle.md`. Its tie-break is allocation order, taken where the
`Leaf` is handed out rather than where the drain grows it, from an atomic counter — `Sprig` will
allocate off-thread and F1 says those names are ordered by nothing but when they arrived.

B2 amended. An anchor is a **basis**, not a set of edges: `Context` holds one `Basis` per element it
can read, and every term that reads geometry names whose it reads. `anchor().col(2)`,
`anchor().content()` and `trunk().letters(8)` are all sayable, `Origin` and the separate parent-cell
field are gone, and `parent` becomes `trunk` throughout — the lexicon's word, and the one already in
`Vein::Trunk`.

The amendment is load-bearing rather than a convenience. Relative-only elevation says that an
element which must clear its stack is grown somewhere else and anchored back; if the anchor grammar
were the smaller one, that move would silently cost `col`, `row`, `pct` and `letters` against the
element it still cares about. Escaping has to be free, or it is not an escape.

`application/` plants the page in panels now: a segmented rail whose ends round outward with `Side`,
a marker elevated over it, an entry recoloured as the tour moves, and a badge grown beside the rail
rather than inside it — which draws over the rail's elevation while still addressing the rail's grid
through its anchor.

Next: C1 — `Willow`, `Ginkgo`, `Ash` and `photosynthesize`. Extraction now produces instances that
nothing consumes, and that is a shape nothing has checked.
