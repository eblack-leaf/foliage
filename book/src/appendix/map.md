# Where to look

Every module opens with a comment saying what it owns and why, so the fastest way into any part of
foliage is to find the file and read its first screen. This page is the index. Paths are relative to
[`foliage/src`](https://github.com/eblack-leaf/foliage/tree/main/foliage/src).

## By question

| If the question is | Look in |
|---|---|
| what happens in a frame, and in what order | `fern.rs` (`run`) |
| whether a frame runs at all | `photosynthesize.rs` (`owed`) |
| how a window event becomes input | `photosynthesize.rs` (`window_event`), `interaction/input.rs` |
| how the window is opened, and its size and density | `willow.rs` |
| what a verb does | `verbs.rs` (the verb), `op.rs` (the op), `fern.rs` (`drain`, its arm) |
| why an op was dropped | `fern.rs` (`drain`: every `dropped(..)` call names its reason) |
| where names come from | `naming.rs`, `leaf.rs` |
| how elements are stored, and what a write records | `tree.rs` |
| what a seed carries | `seed.rs`, `place.rs`, and each element's own file |
| how a placement is written | `placement/role.rs`, `placement/source.rs`, `placement/basis.rs` |
| how a placement becomes a box | `placement/resolve.rs` |
| breakpoints and grids | `layout.rs`, `placement/breakpoints.rs`, `placement/grid.rs` |
| the resolution passes, and what is resolved again | `rowan.rs` |
| fonts, the character cell, shaping and wrapping | `text/font.rs`, `text/shape.rs` |
| text fields: editing, the caret, the selection | `text_input.rs` |
| elements made of several elements | `frond.rs` |
| motion, timers and sequences | `aspen/mod.rs`, `aspen/ease.rs` |
| hit-testing, claiming, holds | `interaction/stack.rs`, `interaction/mod.rs` |
| focus | `interaction/focus.rs` |
| scrolling, coasting, pinning and floating | `view.rs` |
| the three off-states | `lifecycle.rs` |
| elevation and the stack order | `elevation.rs` |
| colors, tones and schemes | `color.rs`, `palette.rs` |
| what a frame reports | `pollen.rs` |
| what can be read | `vein.rs`, `grove.rs` (`tap`) |
| fonts, marks and pictures arriving | `asset.rs`, `icon.rs`, `image.rs` |
| the clipboard, links, the soft keyboard | `clipboard.rs`, `link.rs`, `keyboard.rs` |
| the engine from another thread | `sprig.rs` |
| extraction | `elm.rs` |
| the renderers and the shared stack | `ash/mod.rs`, `ash/quad.rs`, `ash/instances.rs`, `ash/text.rs` |
| glyphs, marks and pictures on the GPU | `ash/atlas.rs`, `ash/sheet.rs`, `ash/picture.rs` |
| the shaders | `ash/*.wgsl` |
| the device, the surface, the depth order | `ginkgo/mod.rs`, `ginkgo/depth.rs`, `ginkgo/viewport.rs` |
| how something is proven | `tests/`, one file per subsystem |

## By subsystem

| Subsystem | Files | Chapter |
|---|---|---|
| the loop | `photosynthesize.rs`, `willow.rs`, `foliage.rs`, `clock.rs`, `queue.rs` (`Wake`) | [The loop](../architecture/loop.md) |
| `Fern` | `fern.rs`, `root.rs` | [The frame](../architecture/frame.md) |
| `Tree` | `tree.rs`, `leaf.rs`, `naming.rs`, `seed.rs`, `place.rs` | [The tree](../architecture/tree.md) |
| ops | `verbs.rs`, `op.rs`, `queue.rs` | [Ops and the drain](../architecture/drain.md) |
| `Rowan` | `rowan.rs`, `lifecycle.rs`, `elevation.rs` | [Resolution](../architecture/resolution.md) |
| placement | `placement/` | [The resolver](../architecture/resolver.md) |
| text | `text/`, `ash/atlas.rs`, `ash/text.rs` | [Text, shaped and cut](../architecture/text.md) |
| `Aspen` | `aspen/` | [Motion](../architecture/motion.md) |
| interaction | `interaction/` | [Interaction](../architecture/interaction.md) |
| views | `view.rs` | [Scrolling](../architecture/views.md) |
| fronds | `frond.rs`, `text_input.rs` | [Fronds](../architecture/fronds.md) |
| `Elm` | `elm.rs` | [Extraction](../architecture/extraction.md) |
| `Ash`, `Ginkgo` | `ash/`, `ginkgo/` | [The backend](../architecture/backend.md) |
| `Sprig` | `sprig.rs`, `asset.rs` | [Off the frame](../architecture/sprig.md) |
| the suite | `tests/` | [Proving it](../architecture/testing.md) |

## Beyond the crate

| Where | What |
|---|---|
| [`README.md`](https://github.com/eblack-leaf/foliage/blob/main/README.md) | Getting started, icons, Android, the repo layout and tasks |
| [`USAGE.md`](https://github.com/eblack-leaf/foliage/blob/main/USAGE.md) | The concept reference: every rule, as a lookup |
| [`OPTIMIZATION.md`](https://github.com/eblack-leaf/foliage/blob/main/OPTIMIZATION.md) | How to measure a frame, what it costs today, and the open candidates |
| [`TODO.md`](https://github.com/eblack-leaf/foliage/blob/main/TODO.md) | Known gaps, each with where the change would go |
| [`foliage/examples`](https://github.com/eblack-leaf/foliage/tree/main/foliage/examples) | Runnable examples, and `stress`, which measures under a real surface |
| [`lichen`](https://github.com/eblack-leaf/foliage/tree/main/lichen) | Opinionated parts built on the public API |
| [`application`](https://github.com/eblack-leaf/foliage/tree/main/application) | The site, built on nothing but the public APIs |
| [`foliage-icons`](https://github.com/eblack-leaf/foliage/tree/main/foliage-icons) | The icon baker |
| [`foliage-android`](https://github.com/eblack-leaf/foliage/tree/main/foliage-android) | The Android project generator and build driver |
| [`xtask`](https://github.com/eblack-leaf/foliage/tree/main/xtask) | `cargo xtask site`, `book`, `api`, `docs`, `web` |
