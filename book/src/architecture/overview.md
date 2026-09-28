# The shape of the engine

This half follows a frame through foliage, from the platform's event loop to the pixels. This
chapter is the map: what the pieces are, where each lives, and the one idea they are all arranged
around.

## One seam

An app and the engine meet at a seam, and only three things cross it:

```text
              app                      │                     engine
                                       │
  grove.at(card, ..)  ──── an Op ─────▶│──▶ the queue ──▶ the drain ──▶ the tree
  sprig.color(row, ..) ─── an Op ─────▶│──▶ the queue
                                       │
  pollen.clicked(card) ◀── Pollen ─────│◀── the drift, sealed once a frame
                                       │
  grove.tap(card, Vein::Drawn) ◀─ Sap ─│◀── a copy of one property
```

- **Ops in.** Every write is a verb on the [`Grow`](https://eblack-leaf.github.io/foliage/api/foliage/trait.Grow.html)
  trait, and every verb does the same thing: build an `Op` and push it onto one queue. Nothing is
  applied where it is written.
- **Reports out.** Everything the engine has to tell an app is collected in a `Drift` while the
  frame runs, sealed into a `Pollen` at one point in the next frame, and handed over by value.
- **Reads beside.** `tap` copies one property out of the engine's state and returns it. Nothing the
  app holds afterwards refers back into that state.

No engine type is handed out and nothing an app holds borrows from the world. A
[`Leaf`](https://eblack-leaf.github.io/foliage/api/foliage/struct.Leaf.html) is a name, eight bytes
and `Copy`, and naming something that is gone is harmless: an op that names it is dropped, and a
read of it answers `None`.

That seam is why the rest of the engine can be arranged the way it is. Since nothing outside can
touch the state, the engine decides exactly when every piece of it changes, and it chooses to change
all of it at one place in the frame, in one order. [The frame](frame.md) is that order.

## The parts

The names are trees, by one rule: **structure words name what an app builds** (`Leaf`, `Stem`,
`branch`, `trunk`, `Root`), and **species names name the machinery**. Reading a name tells you which
side it is on.

| Part | Lives in | Owns |
|---|---|---|
| `Foliage` | `foliage.rs` | The boot object: the grove, the app, the window, the device. Consumed by `photosynthesize` |
| the loop | `photosynthesize.rs` | The platform's event loop: translation of events, whether a frame is owed, and drawing |
| `Willow` | `willow.rs` | The window: what it is opened as, its size and density |
| `Fern` | `fern.rs` | The frame: one function that runs every phase in order, and the drain |
| `Grove` | `grove.rs` | All the state a frame touches, and the surface an app's frame is lent |
| `Tree` | `tree.rs` | The one door onto `bevy_ecs`: each element is an entity, and every declaration a component |
| `Rowan` | `rowan.rs`, `placement/` | Resolution: declarations into boxes, extents, clips and ranks |
| `Aspen` | `aspen/` | Motion: tweens, timers and sequences on the frame's one clock |
| interaction | `interaction/` | The box stack, gestures, and focus |
| views | `view.rs` | Scrolling regions: offsets, coasting, and handing a scroll outward |
| text | `text/`, `text_input.rs` | Fonts, shaping, wrapping, and the editable fields |
| `Frond` | `frond.rs` | Elements that are several elements under one name |
| `Elm` | `elm.rs` | Extraction: what changed, as render instances |
| `Ash` | `ash/` | The render backend: one renderer per kind of element, and the draw |
| `Ginkgo` | `ginkgo/` | The GPU: device, surface, and the density everything is scaled by |
| `Sprig` | `sprig.rs` | The engine reached from off the frame |

`Fern` has no state. Everything a frame touches is a field of `Grove`: the tree, the queue, the
clock, the running motion, the fonts and pictures, the focus, the box stack, the drift being
collected. That is also why an app's `frame` is handed a `&mut Grove`: the surface an app writes
through is the same value the engine keeps everything in, with every field private to the crate and
only the verbs and reads public.

## Where the frame runs

```text
photosynthesize ─ the platform's loop ─────────────────────────────────────────┐
│                                                                              │
│  window events ──▶ Incoming          about_to_wait: is a frame owed? (F9)    │
│                                       └── yes: ask the platform to paint     │
│  RedrawRequested ──▶ paint()                                                 │
│       │                                                                      │
│       ├── owed?  advance the clock, then                                     │
│       │     fern::run ─ steps 1 to 8 ──────────────────────────────┐         │
│       │     │  intake · dispatch · root · drain · aspen · rowan ·  │         │
│       │     │  publish · elm                                       │         │
│       │     └──────────────────────────────────────────────────────┘         │
│       │     ash.absorb(the batch elm made)                                   │
│       └── ash.draw ─ step 9                                                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

The split is deliberate. [`fern::run`](frame.md) is steps 1 to 8 and knows nothing about a window
or a GPU, so the headless test suite calls exactly the same function the loop does. What only a
platform can do (turning a window event into input, deciding whether a frame is owed, and drawing)
is in [the loop](loop.md), and is the whole of what the suite cannot reach.

## Where the rest of the workspace sits

| Crate | Is |
|---|---|
| `foliage` | The engine. Everything this half describes |
| `lichen` | Opinionated parts built on foliage's public API: a look, chips, fields, the mosaic |
| `application` | The site, written against nothing but the public APIs of foliage and lichen |
| `foliage-icons` | The icon baker: an SVG in, a distance field out |
| `foliage-android` | The Android tool: generates the Gradle project and drives the build |
| `xtask` | Repo tasks: building the site, the book and the API reference |

`application` doubles as a gate: `cargo check -p application` fails if the public API cannot build a
real page.

## Reading the source

Every module opens with a comment saying what it owns and why it is shaped that way, and most
functions say why as well as what. This half is a route through those comments rather than a
replacement for them. Excerpts of the engine's source are marked with the file they come from and
are sometimes shortened; they are not compiled with the book, and where one disagrees with the
source, the source is right.

The chapters follow the frame:

1. [The loop](loop.md): when a frame runs at all.
2. [The frame](frame.md): the nine steps and the laws that order them.
3. [The tree](tree.md) and [Ops and the drain](drain.md): where elements live and how writes
   reach them.
4. [Resolution](resolution.md) and [the resolver](resolver.md): from declarations to boxes.
5. [Text](text.md), [Motion](motion.md), [Interaction](interaction.md),
   [Scrolling](views.md) and [Fronds](fronds.md): the subsystems resolution calls on or runs beside.
6. [Extraction](extraction.md) and [the backend](backend.md): from boxes to pixels.
7. [Off the frame](sprig.md): the engine reached from a thread.
8. [Proving it](testing.md) and [What a frame costs](performance.md).
