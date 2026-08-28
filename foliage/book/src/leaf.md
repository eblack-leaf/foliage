# A Name for an Element: Leaf

An app never holds an entity -- it holds a `Leaf`:

```rust
// foliage_proper/src/boundary/leaf.rs
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Leaf(pub(crate) Entity);
```

The wrapped `Entity` is `pub(crate)` -- nothing outside the crate can read, construct, or
match on it. A `Leaf` is opaque by construction: the only things you can do with one are
name it as a parent, pass it to a [`Grows`](./forest.md) verb, or compare it for
equality. `Leaf::id()` hands back a stable `u64` for logging or as a map key -- not an
address, nothing to dereference, just a way to tell two elements apart.

## Usable before it exists

A `Leaf` is allocated the moment you ask for one, from a shared allocator
[`Forest`](./forest.md) and [`Sprig`](./forest.md) both draw from -- which is what lets a
name minted on a background thread never collide with one minted in the frame closure.
The element it names doesn't come into being until that frame's commands are applied, but
the name is real immediately: it can be used as a parent for a child grown in the same
breath, or as the target of a write queued right after.

## Presence, not panics

```rust
// foliage_proper/src/boundary/leaf.rs
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Presence {
    Planted,
    Live,
    Withered,
}
```

A `Leaf` naming something that was pruned, or never grew, is inert rather than dangerous.
Every command targeting a withered `Leaf` is silently dropped; every sample of one reads
`None`. Nothing panics, and a name is never reused within its generation, so a stale
handle held past its element's lifetime cannot silently address whatever grew after it.
[`Forest::presence`](./forest.md) reads which of the three states a `Leaf` is currently
in.

## What's underneath

Every element a `Leaf` names is, on the engine's own side of the boundary, a `bevy_ecs`
entity carrying `Node` -- the internal marker that brings in the position, draw order,
and interaction defaults anything on screen needs. None of that is reachable from an app:
`Node`, the entity itself, and the `bevy_ecs` crate it comes from are all `pub(crate)`.
See [Inside the Engine](./tree.md) for what `Node` actually is and why spawning one is
funneled through a single door.
