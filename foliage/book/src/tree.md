# Inside the Engine: Node, Author, and Tree

Everything in [Leaf](./leaf.md), [Forest](./forest.md), and
[Specs and Sprout](./spawning.md) is what an app sees. None of it says how a `Spec`
actually becomes something in `bevy_ecs`. This chapter is engine-internal -- every type
here is `pub(crate)`, reachable only from inside `foliage_proper` itself -- but it's
where the guarantees the earlier chapters describe (a `Leaf` is always well-formed, an
un-elevated element panics instead of misplacing itself) actually get enforced.

## `Node`: what every on-screen entity carries

```rust
// foliage_proper/src/node.rs
#[derive(Component)]
#[require(Parent, Children)]
#[require(Opacity, Visibility, ClipSection)]
#[require(Section<Logical>, LayoutSection, Elevation, InteractionShape, InteractionPropagation)]
#[require(FocusBehavior)]
#[component(on_add = Self::on_add)]
#[component(on_remove = Self::on_remove)]
pub(crate) struct Node {}
```

A bare `bevy_ecs::Entity` has no position, no parent, no draw order, and can't receive a
click. `Node` bundles exactly the set of components that stops all of that being true, via
`#[require(...)]`. Its `on_add` hook registers the observers that let opacity/elevation/
location animate; its `on_remove` hook is where a pruned entity's `Leaf` gets reported as
[`Moss::Withered`](./forest.md) and where stale `CurrentInteraction` state gets cleared
-- both run for every widget, without each one remembering to register them itself.

`Node` attaches itself rather than being declared on `Panel`/`Text`/etc: every entity the
engine's own spawn path produces gets one unconditionally, at the moment it's spawned,
regardless of what else it carries.

## `Author`: config now, entity later

```rust
// foliage_proper/src/author.rs
pub(crate) trait Author: Sized {
    fn seed(&mut self) -> &mut LeafSprout;
    fn root(self) -> impl Bundle;
    fn build(this: Entity, tree: &mut Tree) {}
}
```

Every `Sprout`-implementing builder (`PanelSprout`, `TextSprout`, ...) is also an
`Author`. `root` folds every config field into the one bundle inserted at spawn -- never
a follow-up write after a bare insert, so reactive hooks resolve once against real values
rather than placeholder defaults. `build` is the config-*independent* skeleton (default
empty, which is correct for a primitive): a composite spawns its children here, and
registers the reactions that keep them in sync via `Tree::react`.

`LeafSprout` -- the position/parent/elevation seed every `Author` embeds -- is itself an
`Author`: the "no marker component of its own" case, for a bare hit area or grouping
element with nothing to render.

## `Tree`: the one door onto `bevy_ecs`

```rust
// foliage_proper/src/tree.rs
#[derive(bevy_ecs::system::SystemParam)]
pub(crate) struct Tree<'w, 's> {
    commands: Commands<'w, 's>,
}
```

`Tree` wraps a `Commands`, so everything through it is deferred to the end of the current
step. It's how the engine spawns entities, writes components, starts animations, and
registers handlers -- obtained as a system param, or via `AsTree::tree` from a `World`/
`DeferredWorld` in component hooks and at startup.

`leaf`/`branch` are the only entry points that reach the private `grow`:

```rust
// foliage_proper/src/tree.rs (abridged)
fn grow<S: Author>(&mut self, mut spec: S, parent: Option<Entity>) -> Entity {
    if let Some(parent) = parent {
        spec.seed().stem = Parent::some(parent);
    }
    let seed = core::mem::take(spec.seed());
    let elevation = seed.elevation
        .expect("elevation not set -- call .elevate(...) before spawning");
    let this = self.sow(());
    self.trimmings(this, &seed);
    self.write_to(this, (seed.location, seed.stem, elevation, spec.root()));
    S::build(this, self);
    this
}
```

That `.expect(...)` is where [`Sprout::elevate`](./spawning.md)'s "no default" guarantee
actually fires. `branch` fills the parent from a required argument, so a child can't be
spawned orphaned by a forgotten chained call. `grow_at` is the counterpart used when an
id was already allocated across the boundary -- growing into a `Leaf` an app was handed
the instant it asked, rather than picking a fresh one.

## `react`/`react_any`/`forward`: one door for values and structure

A composite's appearance usually depends on more than one input at once. Registering a
separate observer per field, and hand-calling each one once at spawn for initial state,
would mean two code paths for "value changed" and "value set for the first time":

```rust
// foliage_proper/src/tree.rs (abridged)
pub fn react<C: Component + Clone, M>(&mut self, entity: Entity, observer: impl IntoEntityObserver<M>) {
    self.subscribe(entity, observer);
    self.refire::<(C,)>(entity);
}
```

`react` subscribes the observer to `Trigger<Insert, C>`, then immediately re-inserts the
entity's current value of `C` so the observer fires once against real data -- the same
code path handles "just spawned" and "written to later." `react_any` is the same idea
over a tuple of components, firing when *any* member changes. `forward` is the pure-copy
special case: `source`'s value of `C` copied to `target` verbatim, on every write.
Anything that isn't a pure copy -- a text whose *position* also depends on the string's
length -- stays an explicit `react`, so "just copied" and "actually computed" stay visibly
different in the code.

## Sequencing animations

`Tree::animate` starts an `Animation<A>` and returns its runner entity. Every animation is
counted against a sequence entity (`spawn_sequence` for one an app never named,
`sequence_at` for one already allocated across the boundary): joining increments the
sequence's own counter on insert, finishing decrements it, and once it reaches zero the
sequence fires `OnEnd` at itself and despawns. Nothing outside the crate ever names that
entity directly -- across the boundary this whole mechanism is what
[`Grows::sequence`/`Grows::animate_during`](./forest.md) and
[`Moss::SequenceFinished`](./forest.md) surface.

## What `Sprout` being sealed actually buys

`Author` being `pub(crate)` stops an app from implementing it, and `Tree::leaf`/
`Tree::branch` being `pub(crate)` stops an app from calling them directly even if it
could. Combined, there is no path from outside this crate to a spawned, `Node`-bearing
entity that skips the mandatory `.elevate(...)` or hand-rolls a parent link -- the only
way onto the tree is [`Forest`/`Sprig`](./forest.md) queuing a [`Spec`](./spawning.md) for
the engine to grow on its own side of the boundary.
