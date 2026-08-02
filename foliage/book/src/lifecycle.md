# Lifecycle: Disable, Enable, Remove, Visibility, Opacity

Four ways an entity's state can change without changing *what* it is: whether it
responds to input, whether it's drawn at all, how transparent it is, and whether it
exists. All four share one shape: an author-facing value on the entity itself, an
inherited value cascading down from its parent, and a resolved value that's the actual
combination -- so a parent's state always propagates without every descendant needing
its own copy of the same logic.

## Disable/Enable: three independent bits, not one flag

Covered in depth in [Interaction](./interaction.md) -- `InteractionState`'s `ENABLED`/
`AUTO_ENABLED`/`INHERIT_ENABLED` bits are what let a parent's `Disable` cascade to every
descendant (via `InheritDisable`, walking `Children`/`AnchorDeps`) without touching the
author's own `ENABLED` bit or the library's own internal `AUTO_ENABLED` opt-outs. `Enable`
is the exact mirror, setting the same three bits back. Across the boundary this is
[`Grows::enable`/`Grows::disable`](./canopy.md).

## Visibility: own flag, inherited flag, resolved

```rust
// foliage_proper/src/visibility.rs
#[derive(Component)]
#[require(InheritedVisibility, ResolvedVisibility, CachedVisibility, AutoVisibility)]
pub struct Visibility { visible: bool }
```

`Visibility::stem_insert` (an observer on `Insert<Parent>`, so it fires the moment an
entity gets a parent) captures the parent's already-`ResolvedVisibility` into this
entity's own `InheritedVisibility` -- hiding a parent after the fact still needs to
cascade to already-spawned children, which is exactly what
[`Differential`](./differential.md)'s visibility-restore path (`cached_differential`'s
`visible: bool` branch) exists to handle for the *render* side: a value that goes
invisible then visible again gets re-sent to the renderer even though it didn't itself
change, since the renderer may have dropped it while hidden. Across the boundary this is
[`Grows::visible`](./canopy.md).

## Opacity: multiplicative blend, not an override

```rust
// foliage_proper/src/opacity.rs
#[derive(Component)]
#[require(InheritedOpacity, BlendedOpacity)]
pub struct Opacity { pub value: f32 }
fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
    let blended = BlendedOpacity::new(inherited.value * current.value);
    ...
}
```

A 50%-opaque child of a 50%-opaque parent renders at 25%, not 50% -- `BlendedOpacity` is
the product of the whole ancestor chain, propagated to every `Children` descendant on
change, the same cascade shape `Visibility` follows. `Opacity` implements
[`Animate`](./anim.md); across the boundary, fading is
`canopy.animate(leaf, Motion::Opacity(0.0), Timing::over(300))` -- the same
[`Grows::animate`](./canopy.md) call every other animatable value uses.

## Remove: a cascade, not a per-entity despawn

```rust
// foliage_proper/src/remove.rs
fn observer(trigger: Trigger<Self>, mut tree: Tree, branches: Query<&Children>, stack_deps: Query<&AnchorDeps>) {
    tree.despawn(trigger.event_target());
    let mut deps = branches.get(trigger.event_target()).unwrap().ids.clone();
    // + AnchorDeps
    tree.send_to(Remove::new(), deps.drain().collect::<Vec<_>>());
}
```

Despawning an entity re-triggers `Remove` on every `Children` child *and* every
`AnchorDeps` dependent (an entity that anchored itself to this one via
[`Anchor::new`](./grid.md), even if it isn't a structural `Parent`-child) --
recursively, until the whole subtree is gone. Across the boundary this whole cascade is
one call: [`Grows::prune`](./canopy.md), which emits
[`Bloom::Withered`](./canopy.md) for every `Leaf` that goes, with no manual teardown list
to maintain per widget.
