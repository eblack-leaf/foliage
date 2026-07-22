# Introduction

Foliage is a cross-platform UI framework built on one idea: **a widget is an entity**.
Not a widget tree of custom types with their own lifecycle rules, and not a retained
scene graph bolted onto a game engine's ECS as an afterthought -- literally one
[`bevy_ecs`](https://crates.io/crates/bevy_ecs) `Entity`, carrying plain components,
observed by plain systems. `Button` isn't a special case the framework understands;
it's a config struct that spawns three ordinary entities (its own root, a `Panel`, a
`Text`) and wires a couple of reactions between them, using exactly the same tools an
application author has.

This book builds that idea up from nothing, in the order a from-scratch implementation
would actually need it, rather than starting from a finished API and describing its
shape. Each chapter exists because the previous one runs into a problem it can't solve
alone:

- An `Entity` by itself has no position, no parent, no draw order, and can't receive
  input -- [**Leaf**](./leaf.md) is the fixed set of components every on-screen entity
  needs, and why each one is there.
- Something has to turn a config struct into a spawned entity without letting authors
  forget the required pieces -- [**Sprout**](./spawning.md) is that builder, and
  [**Tree**](./tree.md) is how you act on an entity after it exists.
- ECS state changes every tick; re-uploading everything to the GPU every tick would be
  wasteful -- [**Differential**](./differential.md) is the change-tracking layer that
  makes only-what-changed cheap.
- Something has to own the window, the GPU device, and the frame loop that ties input,
  ECS updates, and rendering together -- [**Foliage**](./app.md),
  [**Ash**](./ash.md), [**Ginkgo**](./ginkgo.md), [**Willow**](./willow.md), and
  [**Photosynthesis**](./photosynthesis.md) are that machinery.
- With the machinery in place, the actual rendering primitives (`Panel`, `Text`,
  `Icon`, ...) and the composites built from them (`Button`, `Dropdown`, `Router`, ...)
  are just ordinary uses of everything above -- the [**Composites**](./composites-overview.md)
  section, ending with [building `Button` from scratch](./composite-button.md), is
  where every earlier chapter's concept gets used together.

If you only want to *use* foliage, the [README](https://github.com/eblack-leaf/foliage)
has a short version of this same arc. This book is the depth version -- for
understanding why the API is shaped the way it is, not just what to call.
