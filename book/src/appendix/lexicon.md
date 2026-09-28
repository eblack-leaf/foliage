# Lexicon

foliage names things after trees, by one rule: **structure words name what an app builds, and species
names name the machinery.** `Leaf`, `branch`, `trunk` and `Root` are parts of the app's tree. `Rowan`,
`Elm` and `Ash` are parts of the engine. Reading a name tells you which side of the seam it is on
before you know what it does.

The names are chosen to be distinct and to carry the right connotation, not to be botanically exact.
A `Leaf` names any element, including one with children, because "leaf" reads as "a named node in a
tree" and `branch` was already the verb that makes one.

## What an app builds

| Name | Is |
|---|---|
| `Foliage` | The engine. Built at boot, told what to grow, and run with `photosynthesize` |
| `Root` | The app, as the engine sees it: `take_root` once, then `frame` every frame |
| `Grove` | The surface a frame is lent: every write, every read, and the frame-wide facts |
| `Leaf` | A name for one element. Handed out before the element exists, never reused |
| `Seed` | An element described before it exists: what `plant` and `branch` take |
| `Stem` | An element that draws nothing: a box, a trunk, something to hide or disable at once |
| `Panel`, `Text`, `Icon`, `Image`, `Polygon`, `Line` | The six elements that draw |
| `TextInput`, `TextArea` | The fields: one leaf each, several elements underneath |
| `Grow` | The trait carrying every write. Implemented by `Grove` and `Sprig`, sealed |
| `Place`, `Boxed` | The traits a seed states itself with: behaviour, and (for a box) `at` |
| `Pollen` | What the tree put out since the last frame: a set to interrogate, not a list to walk |
| `Vein` | What a read asks for |
| `Sap` | What a read returns |
| `Sprig` | The engine reached from off the frame: every write, and reads pushed to it |
| `Location` | Where a box sits, per breakpoint |
| `Trace`, `Point` | Where a line's two ends are, in the same grammar |
| `Grid` | How an element divides its box for its children |
| `Palette`, `Scheme`, `Fill` | A tone, the table of tones, and a tone or a literal color |
| `Motion`, `Timing`, `Ease`, `Tween`, `Sequence` | What moves, how long, what shape, its name, a group's name |
| `Font`, `Field`, `Plate` | Names for a registered face, mark and picture |
| `Marks` | A set of marks an app declares and registers together |

### Verbs

| Verb | Does |
|---|---|
| `plant(seed)` | grows a top-level element and hands back its `Leaf` |
| `branch(under, seed)` | grows an element under another |
| `prune(leaf)` | takes an element and everything under it down; each one `withered` |
| `tap(leaf, vein)` | reads one property, as a copy |

You plant a tree, it grows branches, branches carry leaves. The verbs follow the structure rather
than describing it from outside.

## What the engine is made of

| Name | Is | Lives in |
|---|---|---|
| `Fern` | The frame: one function that runs every phase in order | `fern.rs` |
| `photosynthesize` | The loop: light in, growth out, and it does not return | `photosynthesize.rs` |
| `Willow` | The window: what it opens as, its size and density | `willow.rs` |
| `Tree` | The one door onto `bevy_ecs`. A structure word, because it is the tree seen from inside | `tree.rs` |
| `Rowan` | Resolution: declarations into boxes, clips, ranks and products | `rowan.rs`, `placement/` |
| `Aspen` | Motion: tweens, channels, timers and sequences on one clock | `aspen/` |
| `Elm` | Extraction: what changed, as render instances | `elm.rs` |
| `Ash` | The render backend: six renderers and one stack | `ash/` |
| `Ginkgo` | The GPU: device, surface, and where the scale factor stops | `ginkgo/` |
| `Frond` | A leaf divided into leaflets that is still one leaf: the fields | `frond.rs` |
| `Lichen` | Opinionated parts built on foliage, in a crate of their own | `lichen/` |

`Fern` is a subsystem in name only. It has no state, because everything a frame touches lives in the
`Grove`; it is a module and a function.

## Words used inside

Most of these are crate-private, but they appear in the source, in the trace, and throughout the
architecture half of the book.

| Word | Means |
|---|---|
| `Op` | One queued change. What every verb builds |
| `Bud` | An element formed and not yet grown: what `plant` and `branch` queue |
| `Queue` | The one op queue, shared by the frame and every `Sprig` |
| drain | Step 4: every queued op applied, in arrival order |
| dropped | What happens to an op that names something withered, or something without the property written |
| `Drift` | What a frame collects to report, before it is sealed into a `Pollen` |
| `Presence` | What a name is right now: `Planted`, `Live` or `Withered` |
| `Growth` | An element's place in allocation order, which settles every tie |
| `Chlorophyll` | Which renderer an element carries, fixed when it is grown. A stem carries none |
| pigment | What an element's renderer is told: its fill, rounding, shape, mark or picture |
| `Lettering` | What a run of text says |
| touched, restyled | The tree's records of a write that can move a box, and of one that cannot |
| `Elements` | Resolution's columns, kept between frames: the order and everything resolved |
| `Standing` | What an element declared that the later passes read, held beside the order |
| dirty | An element resolution has to resolve again this frame |
| moved | An element whose drawn state changed, so extraction visits it |
| placed, drawn | An element's box as laid out, and the same box moved by scrolling |
| extent, offset, clip | How far a region's content reaches, how far it is scrolled, what its regions leave visible |
| rank | Where an element sits in the stack: accumulated elevation, then allocation order |
| inherited | The visible, opacity and disabled products over an element's ancestry |
| box stack | Every present element, front-most first, as the next frame's dispatch reads it |
| `Region` | One element as the hit test sees it |
| chain | The scrolling ancestors of where a gesture landed, innermost first |
| claim | The moment a gesture becomes a drag, and who takes it |
| hold | A press still for long enough to be a statement of its own |
| coast | A region still moving after a release |
| `Departed` | Where a motion started from: `Declared` (re-resolved) or `Snapshot` (a box) |
| `Shaped` | A run of text, shaped once per string, font and size |
| `Cut` | A glyph at a face, a size and a density |
| atlas, sheet | The texture glyphs are cut onto, and the one marks are packed onto |
| slot, span | An instance's place in a renderer's buffer, and a run of the stack drawn in one call |
| recut | Dropping every cache of what the backend holds, after a density change or a rebuilt backend |
| owed | Whether a frame has to run |
| again | An app asking for one more frame |
| wake, rouse | How a sleeping loop is woken when something arrives from off the frame |
| `Conditions` | The frame-wide reads a `Sprig` is pushed every frame |
| `Sprouts` | What a frond carries to grow its leaflets |
| `Parts` | A field's six leaflets, named on the field |

## The laws

| | Law |
|---|---|
| F1 | One queue, one drain |
| F2 | The drain is total, and in arrival order |
| F3 | Reads do not change inside `frame` |
| F4 | An app reads what is on screen, and writes what will be |
| F5 | Hit-testing runs against what was drawn |
| F6 | One clock |
| F7 | The collection window is "since your last frame" |
| F8 | One writer per property |
| F9 | A frame runs only when one is owed |

[The frame](../architecture/frame.md) states each in full.

## Passes

| | Pass |
|---|---|
| R1 | measure |
| R2a, R2m, R2b | the horizontal axis, the wrap, the vertical axis |
| R3 | extent |
| R4 | scroll |
| R5 | clip |
| R6 | rank |
| R7 | inherit |
| R8 | regions |

[Resolution](../architecture/resolution.md) describes each.
