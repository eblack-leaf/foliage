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
descendant (via `InheritDisable`, walking `Branch`/`AnchorDeps`) without touching the
author's own `ENABLED` bit or the library's own internal `AUTO_ENABLED` opt-outs. `Enable`
is the exact mirror, setting the same three bits back.

## Visibility: own flag, inherited flag, resolved

```rust
// foliage_proper/src/visibility.rs
#[derive(Component)]
#[require(InheritedVisibility, ResolvedVisibility, CachedVisibility, AutoVisibility)]
pub struct Visibility { visible: bool }
```

`Visibility::stem_insert` (an observer on `Insert<Stem>`, so it fires the moment an
entity gets a parent) captures the parent's already-`ResolvedVisibility` into this
entity's own `InheritedVisibility` -- hiding a parent after the fact still needs to
cascade to already-spawned children, which is exactly what
[`Differential`](./differential.md)'s visibility-restore path (`cached_differential`'s
`visible: bool` branch) exists to handle for the *render* side: a value that goes
invisible then visible again gets re-sent to the renderer even though it didn't itself
change, since the renderer may have dropped it while hidden.

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
the product of the whole ancestor chain, propagated to every `Branch` descendant on
change, the same cascade shape `Visibility` follows. This is what
`examples/opacity_and_elevation.rs` demonstrates directly: three nested panels, each less
opaque than the last, blending as a soft stack rather than three independent flat values.
`Opacity` implements [`Animate`](./anim.md), so fading is `tree.animate(Animation::new(Opacity::new(0.0)).targeting(e))`,
the same call every other animatable component uses.

## Remove: a cascade, not a per-entity despawn

```rust
// foliage_proper/src/remove.rs
fn observer(trigger: Trigger<Self>, mut tree: Tree, branches: Query<&Branch>, stack_deps: Query<&AnchorDeps>) {
    tree.entity(trigger.event_target()).despawn();
    let mut deps = branches.get(trigger.event_target()).unwrap().ids.clone();
    // + AnchorDeps
    tree.trigger_targets(Remove::new(), deps.drain().collect::<Vec<_>>());
}
```

`tree.remove(entity)` despawns that entity, then re-triggers `Remove` on every `Branch`
child *and* every `AnchorDeps` dependent (an entity that anchored itself to this one via
[`Anchor::new`](./grid.md), even if it isn't a structural `Stem`-child) -- recursively,
until the whole subtree is gone. This is why [Router](./composites/router.md) can say
"navigating tears the current scene down" as a single `tree.remove(old_root)` call: one
`Remove` event reaches everything the old scene ever spawned, structural or anchored,
with no manual teardown list to maintain per composite.
