# Introduction

foliage is a cross-platform UI library for Rust. One element on screen is one entity, held by an
ECS ([`bevy_ecs`](https://crates.io/crates/bevy_ecs)) that stays an implementation detail: no engine
type is ever handed out, and nothing an app holds borrows from the world. Rendering is
[`wgpu`](https://wgpu.rs), and windowing and input are [`winit`](https://docs.rs/winit). The same
code runs on Linux, Windows, macOS, the web and Android.

Everything that crosses the seam between an app and the engine is plain data:

- **ops in.** An app never touches an element directly. It writes verbs (`plant`, `at`, `color`,
  `animate`) against a name, and each one is queued.
- **reports out.** Once a frame, the engine hands the app a `Pollen`: what was clicked, what
  finished, what arrived, what withered.
- **reads beside.** At any point in its frame an app can `tap` one property of one element and
  get a copy of it back.

An app says what an element *is* (where it sits, what fills it, whether it takes a gesture) and
then only writes verbs against the name it got back. There is no render call to make: the next
frame is already different.

## How this book is laid out

It has two halves, and they can be read in either order.

**[Using foliage](usage/first-app.md)** is a set of walkthroughs. Each chapter builds something
small and runnable (a counter, a card that sizes to its text, a scrolling list with a pinned
header, a field that filters it) and explains the part of the engine it leans on while it goes. The
last chapter puts most of it together in one app.

**[How foliage works](architecture/overview.md)** follows one frame through the engine, from the
platform's event loop to the pixels. Each chapter takes one subsystem: what it owns, what it reads,
what it hands the next one, and why it is shaped that way. It names the module each piece lives in,
because the modules carry their own reasoning in their comments, and this half is a guided route
through them rather than a replacement for them.

The two halves meet in the middle. The usage chapters say *what* happens when an app writes a
verb. The architecture chapters say *where* in the frame it happens and *what it costs*, which is
what explains the rules the usage chapters state.

## Conventions

- **Every distance is in logical pixels**, with the origin at the top-left and `y` growing
  downward. Device pixels exist only inside the render backend, and nothing an app writes or reads
  is in them.
- **Code samples compile.** Each one is checked against the crate as it stands. Where a sample is a
  fragment, the lines that make it compile (imports, the function around it) are hidden; the
  button at the top right of a code block shows them. The architecture half also quotes the
  engine's own source; those excerpts name the file they come from, are sometimes shortened, and
  are not compiled with the book.
- **"An app"** is the value implementing `Root`, and the code written around it. "The engine" is
  everything behind `Foliage`.
- Names of subsystems are trees: `Fern` is the frame, `Rowan` resolves, `Elm` extracts, `Ash`
  renders. The [lexicon](appendix/lexicon.md) lists all of them.

## Elsewhere

- The [API reference](https://eblack-leaf.github.io/foliage/api/foliage/) states every public type
  and method, and the crate root is a map of the whole surface.
- [`USAGE.md`](https://github.com/eblack-leaf/foliage/blob/main/USAGE.md) is the concept reference:
  shorter than this book, and organised as a lookup rather than as a walkthrough.
- [`foliage/examples`](https://github.com/eblack-leaf/foliage/tree/main/foliage/examples) holds
  runnable examples, each doing one thing.
- [The site](https://eblack-leaf.github.io/foliage/) is itself a foliage app, built from
  [`application/`](https://github.com/eblack-leaf/foliage/tree/main/application) against nothing
  but the public surfaces of foliage and [`lichen`](https://github.com/eblack-leaf/foliage/tree/main/lichen).
