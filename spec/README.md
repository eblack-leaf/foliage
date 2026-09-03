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

`Fern::run` is steps 1 through 8. Step 9 belongs to the caller, which is what lets the suite and the
event loop share one frame.

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
the `Scheme`'s answer, written at any frame with `repaint` — the one op that names no element,
because a role belongs to the scheme and not to the elements declaring it. Extraction resolves the
role each frame and compares the result, so a repaint moves exactly the elements painted in a color
that changed, and a scheme that resolves to what it already did sends nothing.

`Color` stays the form a renderer reads and the form a scheme is stated in. Still owed is the *ramp*
behind the roles — a tint scale per role, and a light and dark reading of it — which will change
what a role resolves to and never what an element declares. Five roles is a starting set, and adding
one is a variant rather than a migration, because no callsite names a color.

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

C1 has landed. `Willow`, `Ginkgo`, `Ash` and `photosynthesize`: the page is on screen, and the loop
F9 governs is the one that put it there. The headless suite is unchanged and still passes, which is
the point — C1 adds no test because it is the layer the suite cannot reach.

`Fern::run` is called by the loop exactly as the suite calls it, and step 9 is a separate statement
after it. `Ash::absorb` sits between the two, in the same breath as the extraction that produced the
batch: that is what makes Elm's cache safe to keep, since a batch is applied where it is made and a
paint that fails afterwards costs a picture rather than a change. Painting is therefore its own
question — a compositor asking for a redraw repaints from what the renderers hold without running a
frame nothing owed.

Elevation reaches the GPU as a *position*, not a value. `Ash` sorts its instances back to front,
which alpha blending requires anyway, and takes each depth from where the instance landed in that
order. A rank is an accumulated elevation beside an allocation counter and has no magnitude to
scale; it has an order, and a position is exactly that order. The depth test then holds what the
draw order alone would not, and two identical runs draw identically because the rank is total.

The rank moved out of the renderer's instance and into `Instances`, beside it. Where an element sits
in the one stack is a fact about the element rather than about what it draws, and every renderer
will need it. Taking it out is also what leaves `PanelInstance` free to be `#[repr(C)]` over
`Section`, `Color` and four radii — so what extraction compares and what the vertex buffer holds are
the same bytes, and there is no second, upload-shaped copy of a panel to keep in step. `Position`,
`Area`, `Section` and `Color` state that layout where they are defined, which is the commitment that
buys it.

Logical pixels reach the shader. The projection is built from the logical area and the surface is
configured in physical ones, so the rasteriser applies the scale factor and no instance, radius or
section is ever carried in device pixels. Rounding is a distance field rather than geometry, and it
antialiases from the field's own screen-space derivative — which is what makes one written-once
radius read the same at any display density.

The platform's clock is capped at 100ms per frame. A wall clock is the one time source that can lie:
a slept machine, a backgrounded tab, a window held mid-drag or a breakpoint in the loop all report a
gap that has nothing to do with the app, and reported honestly the frame that wakes the loop hands
every running tween the whole of it at once — what is on screen tears its way to wherever wall time
says it should be.

The gap is not skipped, it is waited on. Everything advances by the capped delta, so the whole engine
is deferred by the same amount rather than any part of it moving; a tween resumes where it was and
takes longer in wall time instead of arriving at its end value having never been drawn. Playing late
is the better failure. It is available only because F6 gives the frame one clock — with a second time
source, a subsystem would have something to fall out of step against. The cap is the loop's and not
`Clock`'s, because the suite advances by hand and has to stay exact.

`application/` exercises the loop rather than describing it: the tour asks for the next frame with
`again()` for as long as it is walking the rail, and stops asking when it is done — after which the
engine idles until the window is touched.

C1 owed one thing: the draw order was the panel buffer's own, because there was one renderer. `B4`
paid it, because clipping needed the same walk — the stack is shared across renderers now, and a
second one is a variant rather than a reordering.

B4 has landed. The box stack, gesture claiming, scrolling handoff, and focus. A hundred and
eighty-four headless tests and thirteen compile-fail doctests.

The hit test is one read: the front-most element at the point that is not `pass_through`, and stop.
It is enforced in a single function, and the suite is written against the failure it prevents — with
that read changed to skip elements that do not receive, ten tests fail, including every one that
proves a drag scrolls the region it started in. That is the second half of the law under test, not
only the first.

`interactive()` and `pass_through()` are the whole of what is declared about targeting. `drags(..)`
is the one thing about a gesture the engine cannot derive; everything else is claimed at the time,
per axis, against thresholds tuned once at boot with `Foliage::tune(Claim { .. })`. The claim is
settled once and only ever travels outward, so a drag that turns is still the drag it was claimed
as, and a region that can no longer consume hands it to the next one out mid-gesture.

Focus is a verb. `focus`, `unfocus`, `focus_next` and `focus_previous`, answered at settle against
the geometry and products of the frame they are asked in — which is what lets an app open a drawer
and focus into it in one turn. Order is reading order over what declared `interactive()`, with
`focus_order(..)` to pull one element out of it and `focus_scope()` to trap the cycle inside a
drawer. A press moves focus nowhere: the app writes that from `clicked` where it wants it.

The slice needed three things it did not own, and each landed at the size the box stack's own
definition requires rather than at its slice's full size:

- **The three ways to be off**, as R7's one inherited product. `visible(false)` and a zero opacity
  leave the stack; `disable` stays in it and swallows, which is what makes `disable(page)` enough on
  its own when a drawer opens and why no scrim is arranged anywhere.
- **Scrolling regions**, structurally: `scrolls(Axes::..)`, extent derived bottom-up in R3, the
  offset clamped and accumulated in R4, and clip rects in R5. What `views.md` still owns is
  untouched by this — `contain`, `pinned`, `.extent(..)`, `ScrollTo`, and momentum.
- **Clipping through to the pixels.** A clip reaches `Ash` beside the instance and is applied to the
  pass as a scissor, never carried in an instance and tested per fragment, so it costs nothing per
  element and every renderer has it without a line of shader.

That last one closed C1's open item. The stack is now shared across renderers: `Ash` merges every
renderer's slots by rank, gives each instance its depth from its position in that one walk, and cuts
the walk into spans — maximal runs sharing a renderer and a clip, each one draw under one scissor. A
renderer draws the slots it is asked for and decides nothing about when it is asked, so the second
renderer adds a variant rather than a reordering. The walk runs only when the stack actually moved.

Still owed by B4:

- `focus(leaf)` on a text input placing a caret is `B9`'s: there is no `TextInput` to focus. What is
  proven now is the half that exists — focus moves by verb with no gesture anywhere near it, and is
  reported through `Pollen`.
- Release velocity. Interaction is to hand a coast its initial velocity and stop; nothing consumes
  one until `views.md`'s momentum, so nothing measures one yet.
- Keys. `focus_next` is the verb a Tab key will call, and there is no key intake until `B10`.

`application/` is pressable. The rail's entries take taps and the tour gives way to the first one;
the marker is `pass_through`, which is the mark earning itself on the page — it is drawn over
whichever entry is current, and without it that entry would stop being tappable the moment it became
the selected one. The reading column scrolls, and the cards in it are tappable without either of
them being told about the other: a drag from a card scrolls the column, a tap on one lights it. The
slider's knob takes drags across and nothing else, so a drag down it scrolls the column instead. The
round badge has a round hit area. The drawer is grown outside the page, declares itself a focus
scope, and opens by disabling the page — one write, no scrim — while stepping focus inside it is a
button standing in for the Tab key that `B10` will bring.

B5 has landed. Tweens, easing, channels and timers, against the resolver that was made pure for
exactly this. Two hundred and twenty-two headless tests and thirteen compile-fail doctests.

`animate(leaf, Motion::_, Timing)` writes the target to the element at once, and the tween carries
what the element left — so what an element declares is already where it is going, from the frame it
was told to go there. Ending is therefore a removal, because the blend at the end is the plain
reading of the declaration; cancelling is the same removal, which is what makes F8 structural.

Which phase applies a blend follows from one line:

> **A blend of the same type as the declaration is written back over it at `animate`. A blend of a
> different type is left to the phase that reads the declaration.**

Opacity blends to a number and is written back, so a tap reads where the motion has reached, which
is what is on screen. A `Location` blends to a box and is applied at `resolve`, which resolves both
endpoints in one context and interpolates the results. A fill blends to a color and is applied at
`extract`, which is where a fill becomes one.

**B3 amended: a fill is a `Fill`, which is a `Palette` role or a `Color` stated outright.** The
amendment is what `Motion::Color(Color)` needs to exist at all, and that is the argument for it.
A motion writes its target to the element the moment it starts — that is what makes ending a removal
and cancelling trivial — so a target has to be something the element can *hold*. With a role as the
only declaration, a literal target had nowhere to land, and the choice was between animating to a
color and keeping the property that makes the whole model work.

Holding both in one type costs nothing and states the difference where it belongs: a role is part of
the scheme and a `repaint` moves it; a literal is an element saying it is not, and a repaint leaves
it alone. `Motion::Color` and `Motion::Palette` are two ways of naming the same property, sharing one
slot and one applier, and a motion may cross between them — the role end follows a repaint while the
literal end does not, which is the correct answer rather than a special case. `Grow::color` and
`Panel::color` take `impl Into<Fill>`, so no existing callsite changed.

The one thing this gives up is the reading in which a repaint mid-motion always moves both ends. It
is not worth what it cost: an app changing its scheme has no way to know which motions are in flight,
and should not have to, while animating to a color it computed is an ordinary thing to want.

`Motion` carries four variants and is `#[non_exhaustive]`. `Polygon`, `Outline` and `DrawProgress`
name types `B8` introduces, and `Scroll` names the `ScrollTo` `views.md` still owns, so each arrives
as a variant beside its renderer rather than as a placeholder now. Adding one is a variant and an
applier, which is the shape `Property` exists to keep: it names the property, never the kind of value
stated about it, so a second way to say where something is going shares the first one's slot.

`Ease` is a cubic bezier and nothing else — `Linear`, three named shapes, and `Curve` for the rest —
read as a function of the *elapsed fraction* rather than of the curve's own parameter, which is the
difference between an ease and a curve that merely looks like one. It is exact at both ends,
whatever the shape, so a landing is never a rounding error. `Timing` is a duration, a delay and a
shape, in milliseconds, and says nothing about what is moving.

A tween takes no time on the frame it starts. A frame's delta is how long the interval *ending* at
that frame took, and charging it to a tween created at that frame's drain would move the element on
the frame it was told to begin, away from where it currently is. It is also what makes a zero-length
timer read as `frame.md` describes it, with nothing special-cased.

`tween` is the other half, and it is what lets `Motion` be closed: a start and an end, reported each
frame as `Pollen` and written nowhere, so the engine's clock and easing are available to values it
has no concept of. A `timer` is one whose value is not read, and `stop` ends one — a channel has no
declaration for a direct write to cancel it through, which is the whole difference between the two.
Both are keyed on the tween rather than on a `Leaf`, against `pollen.md`'s table, because neither is
about an element.

F9 gained the clause it was always missing, and animation is what made it visible. An emission is
produced at steps 4–7 and delivered at step 3 of the *next* frame (F7), so a report made and not yet
delivered owes the frame that delivers it — otherwise the loop can idle holding one. `withered` had
the same hole and was covered by the tour asking for frames anyway; `landed` had nowhere to hide.

Still owed by B5:

- The `text_content()` endpoint among `aspen.md`'s proof obligations. `Tree::intrinsic` is B2's
  outstanding seam and still measures zero, so a test of it would assert nothing. What that
  obligation is *about* — the endpoint a motion left re-resolving each frame, in the target's own
  context, through the one resolver — is discharged against an anchor instead, which reaches
  `resolve` by the same call and the same `Context`.
- Named sequences. The previous engine's is not a mystery — a marker entity carrying a count that
  every animation registers against and decrements on finish, plus a per-animation delay taken from
  the sequence's own time range. B5 has the second half of that and not the first: `Timing::after`
  is a delay on one tween, which is what staggers a group, and `landed(leaf)` reports one element
  arriving. What is missing is the *group* report, and with it the question of what names a group.
  That is a public handle and a decision about who owns the offsets, and neither `aspen.md` nor
  `pollen.md` settles it — `pollen.md` only lists `sequence_finished` in a table of shapes. It
  should be decided rather than inherited.

`application/` is in motion. The rail's fills cross rather than swap; the drawer slides in and is
taken out of the picture once it has finished leaving, which is what `landed` is for; the notice
waits on a timer, fades, and is pruned when there is nothing left to see. The ground under the page
moves on a `tween`, because a `Scheme` is the app's own value and foliage has no concept of one —
the clock is borrowed and the write stays on the app's side. A card lit by a tap and then released
is recolored by a direct write, which cancels the fill still moving on it: F8's case, on the page.

B6 has landed. Fonts, the character cell, wrapping, and `content()` on both axes. Two hundred and
sixty-seven headless tests and thirteen compile-fail doctests.

`Tree::cell` and `Tree::intrinsic` are filled, and with them every arithmetic B2 proved against
fed-in values now has something real to read. Two passes write them, and the split *is* width-down
and height-up:

| | Pass | Direction | Writes |
|---|---|---|---|
| R1 | `measure` | — | the character cell, and max-content width |
| R2m | `wrap` | **bottom-up** | the measured height |

R1 reads no geometry at all, because neither answer has anything to do with where the element ended
up: a monospaced run's max-content width is its longest line's character count times its cell. That
is what leaves the down-pass with nothing to measure and only the up-pass with anything to do — and
by then it has a width to do it against. R2m sits between the two halves of R2 because that is the
one moment when every width is known and no height is. Neither pass iterates, and neither is a
second resolution: R2a and R2b are still the only two.

A font is a fact about the program rather than about any element, so `Foliage::font` registers one at
boot and hands back a `Font`. Registration **refuses a proportional font**, naming the two characters
that disagreed. That is not a nicety — every measurement foliage makes is a count of cells, so a
proportional font does not degrade, it puts every column address somewhere it does not belong.

`font` and `font_size` are on `Place`, not on `Text`. A cell is what `letters()` and a letter-pitched
track are measured in, and neither of those is text's — an element sized in characters carries a
typeface whether or not it draws any. An element that names neither has *no* cell and reads zero,
which keeps a cell a declaration rather than a default nobody asked for. `FontSize` is the same
breakpoint chain `Location` and `Grid` are, for the same reason: what an element looks like at a
given width is one thought.

**`placement.md` amended: `content()` on a container is the reach of what is grown under it.** The
document promised a container that grows to fit stacked children and did not say how, and the
two-pass bound is what makes it answerable: R2m runs bottom-up, so everything inside an element is
measured before the element asks. A run wraps at its own width; anything with children takes the
furthest any of them reaches down. Both are the one question `content()` asks — *how large is what is
inside me* — and an element takes the greater of them.

What that needs, and what is new, is a rule about which children count:

> **A child that reads a vertical box does not decide the measure.**

`100.pct()`, a row of the trunk's grid, an anchor's edge: each is asking how tall something else is,
so none of them can be what answers how tall this is without answering it circularly. Such a child is
left out of the measure and given its real height by R2b like anything else — so a child sized to its
trunk still fills it, it just is not what sized it. Everything that describes an extent in its own
terms counts, and that includes any *horizontal* reading, because the horizontal axis has already
resolved. The rule falls out of the axis asymmetry the grammar already states as types.

**Shaping is the one thing kept between frames**, keyed on `(value, font, size)` and swept back to
what the tree states at the end of every resolve. Wrapping is deliberately not kept: it is a function
of the width the layout produced, which is a different answer every time the layout moves, and
walking an already-shaped run to find its lines is cheap. That is the whole of the exception, and
there is no second cache.

`order()` was leaving elements out. It waits on a trunk *and* an anchor, but the cycle refused at the
op only follows anchors, so an element anchored to something grown under it left both ends waiting
forever — and they were dropped from the frame with no box, no rank, no place in the stack and
nothing said about it. That is not the same contradiction an anchor cycle is: both boxes resolve,
just not both against a settled other. The remainder now resolves in allocation order, so **every
live element is in the order**, which is the property every pass downstream was already assuming.

**Sequences are settled: a `Sequence` is a handle, and the offsets stay on the tween.** `sequence()`
hands out a name and `Timing::within` joins one — from any callsite, at any frame, by a motion, a
channel or a timer alike. That is the whole point of a group: it exists to time together things that
have no reason to be written together, and a form that had to state its members in one call could not
do it. The offsets stay where `Timing::after` already puts them, so there is one place a delay is
stated rather than two that can disagree. `Pollen::sequence_finished` is keyed on the sequence rather
than on a `Leaf`, against `pollen.md`'s table and for the same reason `finished(tween)` is: a group is
not about an element. It reports when nothing is running under it any more, however each member
ended — landed, cancelled by a direct write, or taken down with its element — because a group being
over is one fact.

B5's other owed item is discharged: `aspen.md`'s `text_content()` endpoint, rewritten mid-tween,
re-measures and lands exactly, beside the resize, breakpoint and anchor cases that prove the same
re-resolution through the same resolver.

**A fill is a `Fill` whatever holds it.** A run declares one, `color` writes it, `Motion::Color` and
`Motion::Palette` move it, and `Vein::Color` reads it back — the same words a panel takes. Rounding
stays a panel's, because a run has no box of its own to round.

`application/` reads. The rail's entries are labelled and the labels are `pass_through`, which is the
mark earning itself twice on one page. The article is `height(content() + 16.px())` and the prose
inside it is `height(content())`, so the card is as tall as its own words turn out to be at whatever
width the column offered — and the slider anchored below it and the six cards anchored below that all
follow, none of which is written anywhere. Walking the rail rewrites the prose with `text`, and the
whole column reflows in that frame. The drawer's opening and closing are each one `Sequence`: a
placement on the sheet, an opacity on the page, and a channel driving a value foliage has no concept
of, counted together and waited on as one.

Still owed by B6:

- **The glyph pipeline.** Everything about a run resolves and is readable — its box, its fill, its
  rank, its clip — and nothing turns them into pixels yet. `Instances` is keyed on a `u64` rather
  than on a `Leaf` for it: a run is **one** entry in the one stack, whose renderer holds its glyphs
  under its own numbering, so the stack never learns what a glyph is and R6's tie-break is not
  reopened.
- **Per-character tints.** They are `Fill`s over a range of the run's own index space, under the
  fill a run already declares rather than beside it, and there is nothing to prove about them
  headlessly until there is something drawing them.

Next: B7 — views and scrolling. `contain`, `pinned`, `.extent(..)`, `ScrollTo` and momentum, over the
structure B4 already put under them.

## B4 §3, resolved

A gesture goes to the top of the box stack, and `pass_through` takes an element out of that stack.
That is the previous engine's rule, kept — and kept for a reason the previous engine never stated.

The law behind it is that **the hit test does not search.** It reads the top of the stack and stops
there. Every alternative considered for §3 replaces that single read with a walk down the stack
looking for an element willing to take the gesture, whatever it calls the walk: making stopping
opt-in through `interactive()`, giving blocking its own word and defaulting to transparent, or
attributing a press upward from where it landed. They are one mechanism and they fail one way. At a
point, a composite's decoration over its own target and a backdrop over the page it covers are the
same picture, so a walk decides between them by inference — silently, at a distance from anything
the author wrote, and twice over, because the element a gesture lands on is also where a following
drag looks for its scrolling region.

The marks that model costs are real, and they compound with nesting: a composite states
`pass_through` on each of its decorations. They are not a defect to be designed away. They are the
price of a hit test that never guesses, and the alternative is not fewer marks but a press reaching
something the author never put in its way.

The rest of B4 is unaffected — the five flags collapsing to `drags(..)` alone, tap as an outcome
rather than a retracted click, overscroll as a runtime handoff, focus as a verb with derived order
and trapping scopes, and the per-axis claim threshold under `Foliage::tune`.
