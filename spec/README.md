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
| B8 | `Icon`, `Image`, `Polygon`, `Line` | The remaining renderers |
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

Focus is a verb. `focus`, `unfocus`, `focus_next` and `focus_previous`, applied where they are
decided rather than deferred to a pass — whether a target can take focus is a walk of what has been
declared, so it is current as of the drain and an app can open a drawer and focus into it in one
turn. Order is reading order over what declared `interactive()`, with `focus_order(..)` to pull one
element out of it and `focus_scope()` to trap the cycle inside a drawer. A tap moves focus to what it
landed on, and a tap on nothing that receives takes it away; an app that wants otherwise writes
`focus` from `clicked`, which is the later write and wins.

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

Still owed by B6. The first is delivered by B8 below; the second is still owed there:

- **The glyph pipeline.** Everything about a run resolves and is readable — its box, its fill, its
  rank, its clip — and nothing turns them into pixels yet. `Instances` is keyed on a `u64` rather
  than on a `Leaf` for it: a run is **one** entry in the one stack, whose renderer holds its glyphs
  under its own numbering, so the stack never learns what a glyph is and R6's tie-break is not
  reopened.
- **Per-character tints.** They are `Fill`s over a range of the run's own index space, under the
  fill a run already declares rather than beside it, and there is nothing to prove about them
  headlessly until there is something drawing them.

B7 has landed. `contain`, `pinned`, `floats`, `ScrollTo` and momentum, over the structure B4 put
under them. Three hundred and twenty-three headless tests and thirteen compile-fail doctests.

Two of those tests are corrections the page found once it could be flung by hand, and both were the
same shape — a law that was right about one frame and wrong about a gesture:

- **Release velocity is measured over a trailing window**, not over the frame the release landed in.
  A release carries almost no movement in its own frame — a pointer reports none at all in the frame
  its button came up — so a fling read as a stop and `minimum` threw the whole coast away. See the
  coast section above for what replaced it.
- **A press that catches a running coast is spent on catching it.** Stopping a moving list is a
  thing the gesture did; the element it happened to stop over does not engage and the release is not
  a tap. Invisible while flings barely coasted, and unmissable once they did.

**The glyph pipeline waits for B8, and that is a decision rather than a deferral.** B7 is a
resolution slice: extent, offsets, clips and a coast, every one of which is proven headlessly and
none of which needs a glyph on screen. The pipeline is a *renderer*, and what it needs — the atlas,
the per-renderer numbering behind the `u64` key, the spans the shared walk cuts — is what `Icon`,
`Image`, `Polygon`, `Line` and `Polyline` need too. Building it for one and revisiting it for five
is how a wrong shape survives. Per-character tints go with it, because there is nothing to prove
about them until something draws them.

### B7 — what is content

`rowan.md` asked whether a child parked outside its parent grows the extent, and the answer is
**yes, and that is not where the problem was**. On the axis a region declares, a child past its far
edge and the tenth item of a list are the same picture, and no geometry separates them — so any rule
excluding the far child excludes the list, which is the thing scrolling exists for.

What was accidental was accidental for four other reasons, and all four are closed:

| | before | now |
|---|---|---|
| a child reaching sideways out of a column that scrolls down | grew the extent | no extent is computed on an undeclared axis at all |
| a child the app hid | counted, because culling wrote the same flag | out, and its subtree with it |
| a child behind the content origin | created backward range | clamped at the near side |
| **an overlay grown inside a region** | **grew the extent, and was sliced at the region's edge** | **`floats`** |

The last of those is the one this slice found, and it was a hole in the spec rather than in the code.
`lifecycle.md` sends the question of a dropdown inside a scrolling view to `views.md`, and `views.md`
never answered it: the rewrite deleted the previous engine's `ClipToViewport` and put nothing in its
place. So an overlay grown inside a region was cut off at the region's edge — measured, a
hundred-and-twenty-pixel menu with twenty pixels of it left — *and* fed the region a stretch of
scroll range leading to content drawn over it rather than in it.

Those are one question. An overlay is not content of the thing it hangs off, and saying so once
settles both:

> **`pinned` escapes the movement. `floats` escapes the clip. Both leave the extent, because neither
> is content.**

An element that says nothing is content: clipped by its region and counted. That is what an inline
expander wants — options that push into the flow and are scrolled down to — and it is the default
because it is the one that needs no thought.

**How far a float escapes is stated, not defaulted.** A menu on a row of a table with its own
scrollbar inside a settings panel with another wants out of both; a tooltip on a file in a list
inside a sidebar wants out of the list and not out of the sidebar. Nothing distinguishes them to the
engine, so `Escape::Region`, `Escape::Surface` and `Escape::Within(leaf)` are three answers and the
callsite picks. `Within` names an element rather than counting regions, because a count is a fact
about the tree's current shape and would mean something else the moment a wrapper was added; naming
something not above it falls back to `Region`, since a mistake is not permission to leave everything.
Escape depth is about clipping alone — a region contributes its own box and never its content to
whatever contains it, so an overlay out of its own region has no second one to be excluded from.

### B7 — pinning

`pinned` is one declaration because it answers one question, and R3 and R4 read the same bit. The
rule R4 applies is the *nearest* scrolling ancestor: a pinned header inside an inner region still
travels when the page under it moves, which is the only reading where pinning composes. That needs a
second accumulation carried down beside the first — what a pinned child of each element would
receive — and R5 now carries the matching one for `floats`. Both are three lines, because the answer
at a region is the accumulation as it stood before that region's own contribution.

Nothing under a pinned element needs marking. Offsets accumulate down the tree and a pinned element
adds none, so its children cannot move independently of it. That is the accumulation doing its job,
not a second rule about inheriting a mark.

### B7 — chain, contain, and where a gesture goes

`scrolls` takes an `impl Into<Scroll>`, so `.scrolls(Axes::Vertical)` is unchanged and
`.scrolls(Scroll::new(Axes::Both).contain(Axes::Vertical))` is the other case. Two `Axes`-shaped
declarations that could disagree about which axes are in play is the failure the per-axis split
exists to avoid, and a boundary policy on an axis the region does not scroll describes nothing.

Chain and contain are one function — `outward` — asked by a drag, a wheel notch and a coast alike,
because there is one answer to where a gesture goes when a region can take no more of it.

**`views.md` amended: `ScrollTo` names an axis only where its unit needs one.** Four of the five
framings are stated against the region's own range — the end is the end of each axis, half way is
half of each, `show` is one place in two dimensions — so they mean the same thing on either axis and
need nothing said. `px` is the one absolute distance, and two hundred pixels down and two hundred
across are unrelated distances that share a number: on a region scrolling both it names one with
`.on(..)`, and an op that does not is dropped rather than moving two axes by a coincidence.

**Every destination is answered in R4**, not where it was written. A `scroll` verb, a
`Motion::Scroll` and a coast all need the extent, and the extent is R3's — one pass earlier in the
same frame. So the verb records a destination and the pass answers it, which is what makes
`scroll(column, ScrollTo::end())` land at the end of a list that grew in the same frame.

The read surface a scrollbar needs is three veins and no state: `Vein::Offset` in the pixels it was
written in, `Vein::Extent`, and the derived `Vein::Progress`. All three read `None` on anything that
does not scroll — an element with no scrolling axis has no offset rather than one it can never
leave. `Sap::Progress` is held apart from `Sap::Position` although it carries the same two numbers,
so a read cannot quietly take a fraction for pixels.

### B7 — motion and momentum

**`Motion::Scroll` needed one line of `aspen.md`'s rule extended.** A region holds no *declaration*
of where it is going, because an offset is a resolved value rather than a placement. So the motion
carries both ends — a departed number of pixels and a live `ScrollTo` — and R4 applies the blend,
re-resolving the target every frame like a placement's endpoints. Ending is then not quite a removal:
the destination is written out as the ordinary one-shot on the frame the motion finishes, so it lands
exactly and the frame after is identical. A drag cancels it through the existing cancel path with no
special case, which is `aspen.md`'s claim under test.

**A coast is not a tween**, and is held beside them rather than on elements. There is no target and
no duration; it is an integration that runs until it settles. Interaction measures the release
velocity as the mean over a **trailing window** of the last hundred milliseconds, hands it to the
region holding the claim, and stops. One frame is too noisy a sample to measure it from: a hand
slows as it lifts and a pointer reports nothing at all in the frame its button came up, so a fling
measured that way reads as a stop and is thrown away by the minimum speed. A hand that came to rest
before lifting still leaves no speed, because the frames it rested for are inside the window and are
what bring the mean down — which is what makes holding still before lifting stop a list rather than
fling it. The decay is continuous, so a fling covers the same ground
at thirty frames a second as at a hundred and twenty, and it is stated as a **half-life** because
that is the only form of it a person can read and predict. A coast is not charged the delta of the
frame it starts in, for the same reason a tween is not: that delta is the interval the drag was
still being made in. It is pending work under F9 in its own right, so frames run until it settles.

### B7 — the defect momentum found

A delta of **zero** was being read as a region declining to move, so the claim was handed outward —
and a release re-delivers the last position, whose delta is always zero. Every gesture was ending one
region further out than it was made in. Nothing depended on it until something did: the region
holding the claim at the release is the region handed the coast, so a fling in an inner list coasted
the page behind it. Nothing was asked of a region by a move that went nowhere, and nothing about who
holds the gesture changes now.

The B4 suite could not have caught it. No test asked *who holds the claim when a gesture ends*,
because until momentum nothing read it: the three handoff tests never release, and the tests that do
release have one region and nowhere outward to go. There is a test for it now.

### B7 — deferred

**`.extent(..)` is not in this slice.** `views.md` names it for virtualised content, and it is the
one value in the whole engine that can contradict where the children actually are. There is no
virtualised list yet, so there is nothing to prove it against and nothing that needs it — and a
declared number that can disagree with the layout, with no consumer and no way back to derivation, is
worse shipped than deferred. It comes back with the list that wants it, and with a verb that undoes
it.

`application/` scrolls in earnest. The reading column carries a pinned header the article slides
under and a pinned scrollbar beside it — the thumb is grown under the track and does not move,
because offsets accumulate and the track adds none. Its length and position are written from
`Vein::Extent` and `Vein::Progress` and from nothing the app keeps, so a drag, a wheel notch, a coast
and a `Motion::Scroll` all move it without any of them being known about; it is written when it
changes rather than every frame, because an op queued every frame is a loop that never idles. Tapping
a section of the rail runs the column to the matching card with `Motion::Scroll(ScrollTo::show(..))`,
which knows neither how long the column is nor where the card sits, and which the reader cancels by
taking hold of the column. The last card opens a menu that `floats` out of the column — drawn over
what is below rather than cut off at the edge, and adding no scroll range leading down to it. A map
sits at the foot of the column and `contain`s both axes, so panning it to an edge does not carry on
into the column behind it.

## B8 — the remaining renderers

`Icon`, `Image`, `Polygon`, `Line`, per-character tints, and the glyph pipeline B6 left. Three
hundred and sixty headless tests and fourteen compile-fail doctests. Six renderers now, one stack
over all of them.

`Polyline` is **not built**. `Line` also carries a regression that needs undoing before anything
else here is trusted — both are under *Still owed* below.

What they arrived together for is the **sheet**, which `Icon` wants as much as a run does, and the
spans the one walk cuts. The per-renderer numbering behind the `u64` key is not shared by all of
them: it is what a *run* needs, and among what is left only the glyph pipeline needed it.

**The glyph pipeline landed first.** Text draws.

### A run is one entry in the stack and many quads under it

The decision the other five inherit. Every other renderer draws one instance per element, so its
slots and its instances are one list. A run is not: it is one entry — at one rank, under one clip,
at one depth — holding as many quads as it has characters. So `ash::text` keeps **two numberings**.
Slots are runs, sorted by rank, and are what the shared walk meets and cuts spans on; glyphs are the
renderer's own, laid out in slot order with each run's contiguous, so a span of runs is a range of
glyphs and one draw covers it.

That is what keeps the shared stack the size of the tree rather than the size of the text on it, and
what keeps R6's total order over elements from having to say anything about characters. Every glyph
of a run carries that run's one depth, because they are all at that one place in the stack.

**A run is the exception and not the pattern.** `Line` is one thing and `Polygon` is one thing, so
each is one instance per element exactly as a panel is, and neither wants two numberings. Reaching
for this shape because an element sounds composite is how a renderer ends up with bookkeeping it has
no use for.

### What is diffed is the run entire

`Elm` gained `Runs` beside its `Instances`, because a run's instance is not a fixed-size value: what
is compared is its glyphs, its fill, its face, its size, its rank and its clip, all at once. A run's
glyphs move, refill and restack together, so finding which of them changed would cost more than
rewriting the run. Extraction gathers into a scratch kept between frames; an unchanged frame
allocates nothing and writes nothing.

A run that kept its glyph count and its rank is rewritten where it already sits — a run that moved,
scrolled or was refilled has as many characters as it had — and only a length change rebuilds the
order.

### One walk

`Shaped::lines` and `Shaped::place` are the same `walk`. How tall a run is and where its glyphs go
are one question asked for two reasons, and asking it twice is how a run comes to be measured at one
height and drawn at another. A test asserts across ten wrap cases that no glyph lands on a line the
measure did not count.

### The sheet

A **cut** is a character of a face at a size *for a display*: the density is in the key, because the
bitmap is device pixels and the same glyph on another screen is a different cut. Coverage only —
what a glyph is filled with is the element's, and is carried on the instance.

Two things about how a glyph is made, both carried over from the previous engine and both worth
keeping:

- **Three samples, one channel.** Taken through the rasteriser's subpixel path and averaged back
  into one coverage byte. That is horizontal supersampling and *not* subpixel antialiasing: keeping
  the three as R/G/B needs per-channel alpha to composite — dual-source blending, which WebGL2 does
  not have — and assumes a stripe order that is wrong on a rotated or pentile display. What it buys
  is the case single-sample coverage estimates worst: a vertical stem narrower than a pixel, which
  is small type and is round letters.
- **The ink is snapped to the device pixel grid.** A cut is an exact number of device pixels and its
  quad is that many wide, so landed on a whole pixel the two are one to one and the glyph is the
  bitmap. A baseline is a fractional metric — `ascent` is 14.4 at size 16 — so it never landed on
  one, every sample fell between texels, and each glyph was filtered against the clearance packed
  around it. It read as trimmed along the bottom rather than as half a pixel low, which is what it
  was. The whole ink snaps, not the baseline alone, because the run's own box is fractional too and
  it is their sum that has to fall on the grid.

### The sheet has no eviction, and that is deliberate

2048² of coverage, shelf-packed with a pixel of clearance. Measured against the bundled font, where
an *alphabet* is one `(font, size, density)` set of printable ASCII:

| size | scale | cell | per shelf | shelves | cuts | alphabets |
|---|---|---|---|---|---|---|
| 13 | 1× | 8×18 | 227 | 107 | 24,289 | 107 |
| 16 | 1× | 10×22 | 186 | 89 | 16,554 | 89 |
| 16 | 2× | 10×22 | 97 | 45 | 4,365 | 45 |
| 24 | 2× | 15×32 | 66 | 31 | 2,046 | 15 |
| 40 | 2× | 24×53 | 41 | 19 | 779 | 6 |

Body text cannot reach it: an app has a handful of sizes across breakpoints and one or two
densities, against a budget of forty-five to eighty-nine. Large display type is where it tightens.

**Eviction waits for `Image`**, on the same argument that made the glyph pipeline wait for B8:
`Image` is what actually fills a sheet — arbitrary bitmaps, unbounded, one of them possibly a whole
shelf, and quite possibly wanting a sheet of its own rather than sharing this one. Building
reclamation against glyphs alone is building it twice.

When it lands, the packer already implies the shape: **shelves are the reclaim unit**, since
everything on one is a similar height and dropping one leaves no holes among the others. Track the
pass each shelf was last read in, and a failed pack takes the coldest.

The part that is not free: evicting orphans the runs still pointing at those texels, and
`ash::text::Held` keeps only GPU instances — section, colour, sheet rect — so the backend cannot
re-cut them. It needs the character kept per glyph, four bytes each, and then eviction can mark the
runs it orphaned and rebuild them without crossing back into extraction.

Until then it degrades rather than corrupts: a failed pack is held as nothing, that character draws
blank, and the trace says so once with the count it managed.

### What extraction states and what the backend derives

The question the four new renderers all turned on, and it was already answered — by text, which had
been the only one that needed it.

A `Panel` and a `Polygon` are described entirely in logical pixels, so one value is both what `Elm`
compares and what the vertex buffer holds. The other four are not, and each for the same reason:
turning what the element declares into what the GPU draws needs the display's density, and the
density stops at `Ginkgo`.

| | declares | the backend derives |
|---|---|---|
| `Text` | cells and characters | the cut ink, snapped to device pixels |
| `Line` | two ends and a weight | four corners, axis-aligned ones snapped |
| `Icon` | a box and a field | the sheet rect, and the field's screen-space range |
| `Image` | a box and a plate | which texture to bind |

So `Instances<I>` gained the split `Runs` already had, and extraction stays written in one unit
throughout. **A cut is the sheet's word, not a glyph's** — the atlas is asked for ink and the sheet
for a field, and neither vocabulary reached back into the pass that states boxes.

That closed a hole text had carried since B6. Extraction compares logical values and the backend
holds instances derived from them in device pixels, so a display whose *density* changed leaves
every derived instance correct against a density that is gone — while the logical values they came
from are untouched and compare equal forever. Moving a window between two displays is exactly that
case, since the logical size can be identical on both. `Elm::recut` drops what the backend is held
to, so the next frame writes the tree entire; it is total rather than per renderer, because the
density is not any renderer's.

### A line is placed by its ends

`Line` is the one element with no box to state, so it does not get to be handed one. **`at` moved
off `Place` and onto `Boxed`**, which every rectangle implements and a stroke does not — an `at` it
could be handed and would ignore is the kind of surface that has to be remembered rather than read.
`between` is the other half, and the two verbs refuse each other's elements.

A point is the **ordinary grammar**, not a second one: one `HorizontalCoordinate` and one
`VerticalCoordinate`, resolved through the same pure resolver by the same call a near edge already
made. So an end can sit on a grid track, half way across its trunk, or at an anchor's edge, and the
stroke follows when any of those move. `Point` is constructed rather than opened, because the free
functions in the grammar are all *roles* — each names what a value describes and returns something
the grammar has yet to complete — and a point describes nothing and is already finished.

A stroke still needs a box, for clipping, ranking, the stack and hit-testing. It is the rectangle
around its two ends **grown by half its weight**, which is what makes the weight placement rather
than decoration: a rule's ends share a coordinate, and a box of no height is culled before it is
ever drawn. The ends are settled beside the box rather than recovered from it, because a rectangle
has two diagonals and which one the stroke runs along is a fact about where the ends resolved.

### `Cap`, and the feather the backend states

`Line` takes a `Cap`: `Butt` by default, because a rule that reached half a weight past where it was
told to stop would not line up with anything, and `Round` for a stroke that is a mark or part of a
path. Placement is unaffected — a box already grows by half the weight on every side, which contains
a round end.

**The coverage ramp is exactly one device pixel wide, and the backend says so.** That width is not a
preference: a linear ramp sampled at pixel centres sums to the shape's true area only at one sample
spacing. Wider, and what a column of pixels sums to depends on where the centreline falls between
them — which draws as a stroke thinning and thickening along its own length as it drifts. Measured
off the screen before it was fixed, a stroke asked for 2.0 logical pixels drew between 1.96 and 2.25.

A screen-space derivative cannot state that width, which is why deriving it was the mistake. `fwidth`
is `|dpdx| + |dpdy|`, the **Manhattan** norm, which over a distance field is `|cos| + |sin|` of the
gradient's direction: one only where the edge is axis-aligned and one and a half at forty-five
degrees, so the ramp widened with whatever angle the stroke happened to run at. It is also taken per
2×2 quad, so whatever it reports is quantised across pairs of pixels, and it is discontinuous along a
butt cap where the field's `max` changes which term is nearest.

The density is the whole of the answer, and `LineQuad` is already built where the density is known —
so the feather is one more `f32` on the instance, `0.5 / scale` clamped to half the weight, and the
shader takes no derivatives at all. That is the previous engine's `edge_precision = min(0.5,
half_weight)`, which only looked as though it depended on working in physical pixels; what it
actually was is half a device pixel, and moving to logical pixels lost it by reaching for a
derivative to get it back.

The closed-form segment distance **stays**. It is the better field of the two and it is what makes
`Cap` expressible at all — four-edge coverage is an oriented box and cannot be round. What is kept
beside it: the quad grows past the true edge so the rasteriser is asked about the pixels the feather
covers, the ramp is linear rather than smoothstepped, and an axis-aligned stroke is put on whole
device pixels — thickness snapped first and the centreline placed from it, so a one-pixel rule is one
lit row rather than two half-lit ones.

### The bead at a turn, and why no cap choice removes it

Two elements are two draws and two blends, so a pixel both of them only partly cover is composited
twice: `2α − α²` where the shape they make between them has `α`. That is the whole of the seam at a
chain's turns, and it is not z-ordering — the stack sorts back to front, depths are strictly
decreasing and `LessEqual` rejects nothing, which is exactly why nothing prevents the second blend
either.

`Cap::Round` on both sides of a shared vertex is that case at its worst, because it puts the *same*
half-disc in the same place twice: near the shared point both segments' fields reduce to the identical
radial field, so the two coverages are equal at every pixel of the join and the outer arc is painted
at one and a half times what it should be. Simulated at the page's own geometry, against a nominal
2.00:

| construction | apparent weight at the vertex |
|---|---|
| round caps both sides | 2.37 |
| square caps, segments meeting | 2.15 |
| square caps and a `Polygon` disc over the joint | 2.47 |
| one union field, evaluated once | 2.04 |

Square ends are therefore what a chain wants — the wedge they leave open at a shallow turn is a few
hundredths of a pixel — and the previous engine's butt-plus-disc joint is *worse* than either, so
there was nothing to take back from it. Only evaluating the path's union once removes the bead, and
that is a property of a shader and a tessellation rather than of the stack: **a single entry in the
stack would not fix it**, since a run is already one entry holding many quads and those quads blend
against each other too. That is the answer to the question `Polyline` was holding open, and the
reason a path would want a renderer rather than a reason to build one now.

It generalises past lines: any two elements whose antialiased edges coincide conflate, two panels
sharing an edge included. It is structural to one element, one instance, composited in rank order.

### The two sheets, and the one texture each

Three things sample now, and they do not share:

- **Glyphs** are coverage in one channel, keyed on a character of a face at a size *for a display*.
- **Marks** are a multi-channel distance field, packed once per registered icon and read at any size
  — which is the whole reason an icon is not a glyph. A glyph is cut at a size because text is
  composed at a handful of them; a mark is stretched to whatever box a layout hands it, and a
  distance is what stays sharp at both ends of that. The three channels are not decoration: the
  median of them reconstructs the true distance while keeping the corners a single channel rounds
  off.
- **Pictures** get a texture each, and that is the decision rather than an omission. A picture is
  arbitrarily large, arrives whenever a decode finishes, and there may be any number of them — so
  packing them onto a shared sheet needs eviction, and evicting one orphans every instance already
  pointing at it. **`Image` was what eviction was waiting for, and this is the answer that means it
  is not needed.**

A texture each costs a binding change, so **a span now cuts on renderer, clip and binding**. Only
pictures ever put anything but zero there; everything else binds one thing or nothing and so never
cuts on it.

### Pixels are the app's, and so is decoding

`Foliage::image` and `Grove::image` take **decoded RGBA**. foliage decodes nothing: what a PNG or a
JPEG turns into is an app's own business and an app's own crate, and the engine's business starts at
the pixels. That keeps a decoder out of the dependency list for something `B10` already owns.

A picture is **not boot-only**, and the surface says so in two verbs rather than one flag. `plate`
names a picture whose pixels have not arrived and `load` fills it; `image` is the two together.
A name is valid the moment it is handed out, so elements can be grown against it now and filled when
a fetch finishes — an element drawing a plate with nothing behind it occupies its box, draws nothing,
and appears on the frame its pixels do. It is absent from the batch rather than held as blank, so
there is nothing to undo. There is deliberately **no readback for whether it has loaded**: the
answer only ever changes what is on screen, and the engine already changes that.

`Fit` is three answers because two ratios disagreeing has three readings and no default that is
right for all of them. One of the two moves, never both: fitting inside the box changes the box and
shows the whole picture, filling it keeps the box and shows part of the picture. An image takes no
`color` — it carries its own, and a fill would be a second opinion about it — but it rounds through
the same field a panel does, so a full-bleed picture sits flush inside a rounded card.

`application/` registering its picture inside `take_root` is what found the last gap in the surface:
`Foliage::icon` was boot-only, and an app that takes root inside the first frame had no way to reach
it. `Grove::icon` is the same registration at any frame. It is **not** an op, where loading a picture
is one: a field is written once and never changes, so there is nothing for it to be ordered against,
while a picture's pixels are replaced over the life of the program and when they land relative to
everything else is a real question.

### A shape is three numbers

`Polygon` is the expressive shape beside `Panel`'s rectangle and deliberately not a generalisation
of it: a regular polygon's corners only stay circular while its own bounds are square, so the two do
not collapse into one primitive. It is inscribed in the largest circle its box holds, which is what
lets a composite size one loosely without reasoning about the aspect it lands at.

`Motion::Polygon` needed none of the machinery a placement's endpoints do, and that is the point of
it: sides, rounding and rotation are numbers, so a shape blends to a shape and is **written back
over its own declaration** — the same standing opacity has, and the reason reading one back reports
where the motion has reached. A fractional side count is a distance-field blend rather than a
vertex-matched morph, so a hexagon becomes a triangle by passing through the shapes between them.

### Per-character tints

`Fill`s over a range of the run's own index space, **under** the fill the run already declares rather
than beside it: everything untinted is the run's, so a run with no tints is exactly the run it was
before tints existed. A tint is a `Fill` like any other, so a role follows a `repaint` and a literal
does not.

Ranges are in **characters of the value**, spaces included — the space a caret and a selection are
addressed in, and the space the string was written in. Counting drawn glyphs instead would put every
range after a space somewhere other than the word it names. `Glyph` carries a colour for it, which
costs the comparison it would have cost anyway.

`tint` replaces every tint rather than adding one, for the reason a placement is one value. `untint`
is how they come off rather than handing `tint` an empty set: the fill type of a set with nothing in
it cannot be inferred, and a verb that has to be told the type of what it is not writing is a worse
surface than a second verb.

### Still owed by B8

- **`Polyline`** — not built. `application/` draws its series as a chain of square-capped `Line`s,
  which is enough to exercise the renderer but is not the same thing as a path the engine knows
  about. What a real one owes is a handle, a dash pattern, and `Motion::DrawProgress`. The question
  that was open — whether the segments of one path have to be a single entry in the stack for their
  joins to composite cleanly — is answered above, and the answer is that one entry is not enough:
  what a path needs is one *coverage evaluation*, which is what a renderer of its own would buy.
- **A hazard, not yet a defect.** `LineQuad::new` snaps a stroke whose ends share a coordinate, and
  it moves both ends and the weight. In a chain, one axis-aligned segment would therefore stop
  meeting its neighbours and be a different thickness from them. Nothing on the page reaches it, and
  whatever owns a path has to answer it.
- **`Motion::DrawProgress`** — goes with whatever owns a path. Revealing a prefix by arc length
  needs the resolved positions of every point, which is a pass, and there is no element here yet for
  that pass to belong to.
- **Eviction** — still not built, and now with a reason rather than a deferral: the two things that
  could have filled a shared sheet do not share one. Marks are a bounded set packed once; pictures
  have a texture each. Should a sheet ever fill, the packer already implies the shape — shelves are
  the reclaim unit, and the note above on orphaning the runs still pointing at those texels stands.

`application/` has a figure at the foot of the column, and every renderer on it is doing the thing it
exists for rather than standing in for a panel. The axes are rules — two ends sharing a coordinate,
which is a box of no height until the weight says otherwise. The series is a path drawn as strokes
meeting end to end, its readings stated as points in the grammar, so the whole
figure stretches with the column rather than being redrawn when it moves; the ends are square,
because two round ones at a shared point are the same disc drawn twice. The legend's dot is a
shape the tour animates through every side count between a hexagon and a circle. The mark is a
distance field the element fills, so it repaints with the scheme like the label beside it. The
thumbnail is registered inside `take_root` rather than at boot, which is what found the last gap in
the surface — `Foliage::icon` was boot-only, and an app that takes root inside the first frame had
no way to reach it. The caption is one run with a range of it filled differently, and which range
follows the section being read.

## B9 — the one composite

`TextInput`. Three hundred and ninety-six headless tests and fourteen compile-fail doctests.

### One name, four elements

Everything else foliage grows is one element. A field is four — a run, a placeholder, a caret and a
selection — because each is already something the engine draws and they move independently of one
another. The app holds **one** `Leaf`: every verb and every read is addressed to the field, and
`text`, `select`, `Vein::Text`, `Vein::Selection`, `edited` and `submitted` all reach the part they
are actually about. What a field is made of is not a surface to keep in step with.

The parts are grown **in the drain that grew the field**, from a `Sprout` the bud carries, with names
from the same allocator. They are not four more queued ops because they are not the app's to order
against anything: a field is one thing to plant, and the frame that planted it is the frame the whole
of it is live in. Downstream nothing can tell them from anything else grown that frame — which is
what keeps the composite from being a second kind of element.

### The caret is placed in the ordinary grammar

A caret at character `n` is `anchor().left() + anchor().letters(n)` against the run, and a selection
is the span between two of those. That is the whole of the geometry. No pass measures a caret,
because `letters()` already resolves a character count against the font the run composes in — on
either axis, so the caret's height is `anchor().letters(1.0)` and is one line of that same font.

It is the first real consumer of `letters()`, which B2 introduced against fed-in values and B6 gave a
font to. A caret was the reading it was for: `Shaped::place` already said its index space is "the
space a tint and a caret are both addressed in", and this is what that sentence was reserving.

### A field is a scrolling region, and that is not a convenience

One line, as wide as its own value, inside a box that clips it — which *is* a region, so it is
declared as one rather than given a clip of its own. `views.md` already owns everything that follows
from that: the clip comes from R5, the offset from R4, and keeping the caret in view is
`ScrollTo::show(caret)` pushed at every edit and answered in R4 against the extent the same frame
measured. Typing past the right edge scrolls the field in the frame it was typed in, and `Home`
brings it back, with nothing written for either.

Clipping only ever came from `scrolls(..)`, and a field wanting a clip and wanting to follow its
caret are the same want. Giving it a second mechanism would have been two answers to one question.

### One element that is more than one, stated once

A field is four elements and the drain knows none of that. A `Bud` may carry a `Sprouts`, the drain
grows the element the app named and hands the rest to it, and what those parts are is the seed's own
business — so the next composite is a seed and a trait impl, with no third place to edit. The parts
are grown in the same drain step rather than as more queued ops, because a composite is one thing to
plant and the frame that planted it is the frame the whole of it is live in.

### Focus settles where it is decided

Focus used to be answered at step 7, after resolution. That one fact generated every wart around it:
anything downstream of focus had already missed its pass, so a caret needed patching up afterwards,
and each attempt at that patch was an engine-wide mechanism serving one element — a text-shaped pass
inside resolution, then a component the inherited product had to consult, then an op only one seed
emitted. Three shapes for one problem, and the problem was the timing.

It was at step 7 because it read two things resolution produces: the inherited product, to know
whether a target can take focus, and where elements were drawn, to know what order to step in. But
**dispatch already resolves against what the last frame settled** — that is the law for hit-testing,
and reading order for a keyboard event is the same kind of question about the same kind of event.
And focusability is not really the product: it is *hidden or disabled anywhere in the ancestry*,
which is a walk of what has been declared, and that walk is current as of the drain rather than one
frame stale. It is strictly better than what R7 offered, since it sees a drawer shown this frame.

So focus applies where it is decided — a tap and `Tab` at dispatch, `focus(..)` in the drain — and is
final before resolution runs. Nothing follows it with a pass to miss. A caret's visibility is an
ordinary `visible` write inherited by R7 like anything else, R7 is exactly what it was, and
resolution knows nothing about fields.

What is selected is not cleared with it. A selection is state and focus is not, so leaving a field
and coming back finds it as it was.

The run is drawn **in front of** both marks. A caret sits on the boundary between two character cells
and is as wide as it needs to be seen; drawn over the run it took a bite out of the glyph it stood
before — at 14px, about a quarter of it.

### Keys

`interaction.md` gained §10, and F1's clause about keystrokes being genuinely ordered is finally
about something. A keystroke arrives at intake beside a press, in one queue, and is dispatched
against what the last frame settled.

A key goes to whatever it is about and nothing searches for that — the same law §3 states about a
point, stated about a key. `Tab` and `Escape` are focus's own and are answered wherever focus is,
**including nowhere**, so a keyboard reaches a page that has never been pressed; that discharges
B4's third owed item. Everything else goes to the element holding focus and to nothing at all if
that element has no use for it — dispatch knows which keys steer focus and nothing else about any of
them.

What is *held* travels as its own event in the same stream, so what a key was pressed with is the
order the two arrived in rather than a flag kept beside the queue. That is what makes it engine
state: the headless suite holds a modifier by writing the event a window writes, and there is no
second path for a test to miss. `control` arrived with that move, and `Ctrl+A` with it.

Dispatch decides *which* element a key is about; the drain decides *what* it does. So a keystroke is
queued like every other change, F1 keeps one queue and one drain, and what a key changed is reported
on F7's ordinary footing rather than by a second, faster path.

What a key *produced* is the platform's answer and not the engine's. A layout, a dead key and a
composed sequence are resolved before intake, so a key that produced text is taken as the text it
produced and nothing here maps a scancode or holds a binding table.

### Focus goes to what was tapped

B4 held that a press moves focus nowhere, so an app wrote the line itself. That is not tenable for an
element the engine ships: making an app hand-wire "tapping a field focuses it" is asking it to
assemble a part that came assembled.

**Focus goes to what was tapped**, and nothing declares it. `interactive()` is already the statement
that an element takes input and focus already rests only on what said that, so the target of a tap is
by definition somewhere focus can be — a second flag would have been the same question asked twice,
free to disagree with itself. A tap that reached nothing which receives takes focus away, which is
the same rule rather than a dismissal rule beside it.

The verb keeps the last word without needing to be protected: a tap settles focus at dispatch, the
frame *before* an app is handed the `clicked` it produced, so an app writing `focus` elsewhere is
simply the later write.

**A caret lands on the tap**, under the rule that already denies a drag its click — a gesture that
became a drag was never a statement about what it began on, so it moves no caret and takes no focus
either. And the field reads that gesture rather than being handed it: interaction reports where a tap
landed, `TextInput` asks whether it was tapped and where, and interaction has no idea fields exist.

`interaction.md`'s outstanding obligation — "`focus(leaf)` on a text input places a usable caret with
no click involved" — is discharged, and it is discharged literally: the caret is drawn from focus and
from nothing else, and every editing test in the suite reaches the value without a pointer anywhere
near it.

### What editing is

A pure function: a value, a caret and a keystroke in, a value and a caret out. Every off-by-one in a
text field lives in character arithmetic and none of it needs a tree, so none of it is tested through
one. Indices are **characters** throughout — the space `Shaped` lays a run out in and the space a
`tint` is written in — so a caret, a highlight and a range all mean the same thing by the same number.

Two rules are worth stating because they are the ones a reader notices when they are wrong. Backspace
and delete remove *the span*, with an empty span reaching one character first, so a selection and a
caret are one rule rather than two. And an unshifted arrow against a selection **collapses to the edge
it points at** rather than stepping from the caret, because the selection is the thing being moved
away from.

### Still owed by B9

The first of these is the next work. Both it and the long press below came out of using the thing
rather than reading it, and the long press is the reason this one is worth having.

- **A drag that reaches the edge should keep scrolling.** It does not, and the bug is exact:
  `refresh` asks the field to `show` the caret, but a field only refreshes when a drag *reports
  movement*, and movement is only reported from `Input::Moved`. A pointer held still past the edge
  produces no event, so nothing scrolls and the reader has to jiggle to make progress.

  The fix reads the **open gesture** each frame rather than only the moves: while a field holds a
  drag whose pointer is outside its box, scroll toward it every frame whatever the pointer did.
  `Fronds::gestured` already runs every frame and can see `incoming.gesture`, so there is nowhere
  new to put it.

- **A blinking caret.** Deliberate rather than forgotten: a blink is a frame owed for as long as a
  field holds focus, and F9's idling is worth more than the blink until something says otherwise. The
  caret is solid while focused.
- **Clipboard, and the virtual keyboard.** Both are `B10`'s, and both are platform edges rather than
  anything about what a field is. `Ctrl+C`/`Ctrl+V` wait on the first of those and not on the
  modifier, which is here.
- **Composition.** A key that produced text is taken as the text it produced, which covers a dead key
  and a committed sequence and does not cover an inline preedit. That needs a run drawn in a state it
  does not have yet.
- **More than one line.** A field is one line and a text *area* is not the same element: wrapping puts
  the caret back into the walk, and the walk answers a cell for every character rather than a column
  for one. It is a second element when there is something that wants it.
- **Word-wise motion.** `Left`/`Right`, `Home`/`End`, shift over both, and `Ctrl+A` are what is
  here. Stepping by word is a decision about where a word ends, which is a different question from
  which modifier was held.
- **Recolouring a field after it is grown.** Its four fills are stated when it is planted and
  `color` reaches none of them, because which part a fill written to a field means is a real
  question and picking the run silently would be an answer nobody asked for. What an app usually
  wants — an error state, a focused border — is the ground it put the field in, which is its own
  panel and already writable.

`application/` has a form in it. The drawer's two stand-in panels are real fields now: the ground and
the field are two elements because they are two things — the ground is a box the app chose the colour
and rounding of, and the field is what can be typed into, since a field draws no chrome of its own.
Nothing in the app focuses a field and nothing in it places a caret: a tap does both, because that is
what a tap does. What the app writes is the one thing that is genuinely its own — the ground it
painted, which is where a focus mark goes when the engine draws none. `Tab` moves between the fields
and `Escape` leaves, so the button standing in for a keyboard is standing in for nothing any more.
Enter closes the drawer, and what either field holds is read back rather than kept — the second
button says `save` once the form has anything in it and `close` while it does not.

## The long press

`interaction.md` §6's owed item, and the first work after B9. Four hundred and fifteen headless tests
and fourteen compile-fail doctests.

A gesture had one threshold and it was a distance, so a press that was sitting still and a press that
had not moved yet were the same state. `resolving` has a second way out now — held past `Hold::after`
and reported as `Pollen::held`, which is a gesture fact of its own and not a text feature, and which
nothing declares to receive.

```
opened ──▶ resolving ──▶ claimed ──▶ ended
               └──────▶ held ──▶ claimed ──▶ ended
```

**A hold is measured against the frame's own clock.** It is the one transition nothing arrives to
make — every other one is a move that crossed a threshold, a release or a cancel — so it is read at
the top of every dispatch, before that frame's input, because by the time any of that input arrived
the press had already been down that long. It is also the only thing the primitive needed from
outside interaction: an open gesture that could still become a hold is **pending work under F9**, or
the loop idles under a finger and the duration passes unremarked. That clause is under test, because
the headless suite advances the clock by hand and so is the one place it cannot be missed.

**A press that was held is not a tap.** The clause that already denied a drag its click, read against
the other way out of resolving: it earns no tap, moves no caret and takes no focus. It does not ask
whether anything was listening, because an app wanting a slow press to be a tap anyway is asking for
the two to stay indistinguishable, which is the whole of what the hold ends.

**A drag out of a hold belongs to whoever took the hold** — whatever that element declared with
`drags(..)`, and whichever way it goes. The hold has already settled that the gesture is not a tap and
which element is holding it, so there is nothing left for an axis to decide and no second distance to
cross: the first movement out of a hold is the drag. `drags(..)` is untouched and is still the one
thing about a gesture the engine cannot derive; what a hold adds is a second way to claim one, and it
is the way an element declaring no drags at all takes one.

**`TextInput` declares no drags now**, which is the whole of what this was built for and one line off
its seed. A field was declaring both `drags(Horizontal)` and `scrolls(Horizontal)`, and a target's
claim beats the region's — so the field's own region could never be reached by a drag, only by the
caret-follow and a wheel. That is exactly "a drag across a field can never scroll it". With the
declaration gone, a drag across a field is the region's, the region is the field, and the value moves
under its box; selection is a press that was held and then dragged, and the field reads `held` for
where the caret goes and writes its own `focus` there, because a hold is not a tap and the engine
moves focus on nothing else.

One pointer answers for all three devices (`input.rs`), so this is the mouse's behaviour too: a
click-drag across a field scrolls it rather than selecting. That is `interaction.md` §6's answer taken
literally — the two motions are identical until a hold separates them, and there is nothing else in a
gesture to separate them by.

**A hold needs a holder.** Where the press landed on nothing that receives, there is nobody for it to
be a fact about, so the gesture stays resolving and ends as the tap it would always have been — which
is what keeps holding still on a backdrop and lifting a dismissal rather than nothing at all.

`Hold` is a tuning value beside `Claim` and `Momentum` rather than a field of `Claim`. Distances that
compete per axis and a duration are two statements, and `Claim`'s entire argument is about the two
axes competing at different scales, which a duration has no part in. Half a second by default, and
one value for the whole app for the reason every other tuning value is one.

`application/` opens the last card's menu on a hold rather than on a tap of that card, which is the
affordance the primitive exists for: the card is not lit by it, nothing has to be undone when the menu
arrives, and a tap puts it away — the two never arriving together being the law rather than the page's
own care.

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
