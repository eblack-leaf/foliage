# Authoring a Widget

One trait builds every widget in foliage — primitives (`Panel`, `Text`, `Icon`), library
composites (`Button`, `TextInput`), and your own, with no distinction between them:

```rust
pub trait Sprout: Sized {
    /// the position/hierarchy/elevation seed every spawnable shares
    fn seed(&mut self) -> &mut LeafSprout;
    /// config -> components on the root entity. This IS the widget's public API.
    fn root(self) -> impl Bundle;
    /// private, static skeleton. Default empty == a primitive.
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) {}
    // provided: .at(location) .elevate(elevation) .with(extra_components)
}
```

- **`seed`** exposes the location/stem/elevation every widget carries, so `.at()`/`.elevate()`
  work uniformly across every `Sprout` type without each one re-implementing them.
- **`root`** turns your builder's config into the bundle inserted on the root entity — this
  bundle *is* the widget's public API surface (a `Slider`'s `root()` includes `SliderValue`;
  writing to that component later is how callers update it).
- **`build`** is the *private, config-independent* skeleton: whatever children a widget spawns
  every single time, regardless of what it was configured with. Default is empty, which is
  exactly what makes a primitive (`Panel`, `Text`) a primitive — it has nothing to build.

A spec (anything implementing `Sprout`) becomes a spawned entity through exactly two entry
points, and no others:

```rust
tree.leaf(spec)             // root — no parent
tree.branch(parent, spec)   // child — parent is a required argument, so nothing spawns orphaned
```

Both are on `EcsExtension`, the trait every tree handle (`Tree`, `DeferredWorld`, `World`)
implements. There is no lower-level spawn path available outside the library — the machinery
that actually turns a `Sprout` into an entity is `pub(crate)`, reachable only through these two
methods, so an `.elevate(...)` can't be skipped and a child can't forget its stem.

## Why `build` is static, and everything data-dependent goes through `react`

The rule, stated plainly:

> `build` = the config-independent skeleton, nothing else.
> `react` = everything data-dependent — values **and** structure.

This isn't a style preference — it closes a real bug class that existed in an earlier iteration
of this authoring model. That version built a widget's children from `#[require(...)]`-triggered
component hooks, which meant construction happened lazily, keyed to whichever component the
hook was attached to. A `Dropdown` writing its own `Expanded` state *before* its child rows had
been hook-constructed would silently do nothing — the poke landed before the thing it was meant
to affect existed. The bug tracked hook registration order, which is exactly the kind of thing
that's easy to get right once and quietly break months later.

The current model makes that class of bug unrepresentable rather than merely rare: `build`
spawns every static child and registers every reaction *before* `tree.leaf`/`tree.branch`
returns control to the caller. There is no window where a caller can hold a live `Entity` for a
widget whose reactions aren't registered yet — so "poke before construction" simply can't happen.
The next chapter, [Reacting to Data](./reacting.md), covers the mechanism that makes this true:
reactions fire once immediately at registration (with the value already present), then again on
every later write, so initial state and every subsequent update share one code path.

A dropdown's option rows, a hand's cards, a segmented control's buttons — dynamic children
driven by external data are not a special "dynamic widget" feature. They go through the same
door a button's colors go through: a `react` on whatever component carries the data, spawning
or despawning children as its body sees fit. Reconciliation policy (respawn-everything, pool and
reuse, respawn-by-key) is author code, not library policy — nothing in `Sprout` cares which one
you pick.
