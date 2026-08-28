# Introduction

Foliage is a cross-platform UI framework. An app built with it does not touch an ECS,
does not hold an entity, and cannot construct one -- it describes what should be on
screen, using a small closed vocabulary, and gets back what happened.

- [**Leaf**](./leaf.md) is that vocabulary's name for a thing on screen: an opaque
  handle, allocated the moment you ask for one, usable immediately even though the
  element it names doesn't exist until the frame's commands are applied.
- [**Forest**](./forest.md) is the surface an app actually holds, once per frame: grow
  and change elements by `Leaf`, read back what a frame produced as
  [`Moss`](./forest.md)s, sample current state directly. [**Sprig**](./forest.md) is
  the same command set from another thread.
- [**Sprout**](./spawning.md) is how an element gets configured before it exists --
  `Panel::new().color(..).at(..).elevate(..)` -- and [**Specs**](./spawning.md) are the
  closed set of things `Forest`/`Sprig` know how to grow: `Panel`, `Text`, `Icon`,
  `Image`, `Line`, `Polygon`, `Polyline`, `TextInput`, and a bare container.

That's the whole surface. Underneath it, the [**Inside the Engine**](./tree.md) section
covers how a `Spec` actually becomes a `bevy_ecs` entity, how per-tick state changes turn
into GPU uploads, and how a window and a render loop come together -- useful for
understanding *why* the API is shaped the way it is, and required reading if you're
extending the engine itself, but nothing an app needs in order to use it. The seam
between the two is deliberate: an app cannot reach the ECS from its side of `Forest`,
which is what lets it run its own `bevy_ecs`, at whatever version it likes, without the
two ever meeting.

This book builds that picture up from nothing, in the order a from-scratch
implementation would actually need it, rather than starting from a finished API and
describing its shape. Each chapter exists because the previous one runs into a problem
it can't solve alone:

- Something has to name an element before it exists, so a `Leaf` handed back immediately
  can still be used as a parent or a write target in the same frame --
  [**Leaf**](./leaf.md) is that name.
- Something has to collect what an app wants done and hand back what happened, once per
  frame -- [**Forest**](./forest.md) is that surface, and [`Moss`](./forest.md) is what
  comes back out.
- Something has to turn a config struct like `Panel::new()` into a spec `Forest` can
  grow, without an app ever assembling one by hand --
  [**Sprout**](./spawning.md) is that builder.
- Underneath all of it, an actual entity has to exist, with a position, a parent, a draw
  order, and a way to receive input -- [**Inside the Engine**](./tree.md) covers how a
  `Spec` crosses into `bevy_ecs`, and the machinery (`Differential`, `Ash`, `Ginkgo`,
  `Willow`, `Photosynthesis`) that turns the result into pixels on a screen, tick after
  tick.
- With the machinery in place, the actual rendering primitives (`Panel`, `Text`, `Icon`,
  ...) covered in [**Core Types**](./coordinate.md) and
  [**Rendering Primitives**](./panel.md) are just ordinary uses of everything above.

If you only want to *use* foliage, the [README](https://github.com/eblack-leaf/foliage)
has a short version of this same arc. This book is the depth version -- for
understanding why the API is shaped the way it is, not just what to call.
