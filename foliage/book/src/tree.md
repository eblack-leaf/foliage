# Acting on Entities: Tree and Graft

[Spawning](./spawning.md) covers how an entity comes into existence. Most of what a
widget actually *does* -- restyling on state changes, running an animation, responding
to a click -- happens after that, against an entity that already exists. `Tree` (a type
alias for `bevy_ecs::system::Commands`) is where that work goes.

## `react`/`react_any`/`forward`: one door for values and structure

A composite's appearance almost always depends on more than one input at once (`Button`
restyles on `ButtonStyle`, `Engagement`, *and* `IconValue` together). Registering a
separate observer per field, and hand-calling each one once at spawn to establish
initial state, would mean two code paths for "value changed" and "value set for the
first time." `EcsExtension::react` collapses that into one:

```rust
// foliage_proper/src/tree.rs
fn react<C: Component + Clone, M>(&mut self, entity: Entity, observer: impl IntoEntityObserver<M>) {
    self.subscribe(entity, observer);
    self.refire::<(C,)>(entity);
}
```

It subscribes the observer to `Trigger<Insert, C>`, then immediately re-inserts the
entity's current value of `C` so the observer fires once against real data -- the same
code path handles "just spawned" and "written to later." `react_any` is the same idea
over a tuple of components (fires if *any* member changes); `forward` is the pure-copy
special case (`source`'s value of `C` copied to `target`, verbatim, on every write) --
anything that isn't a pure copy (like `Button`'s text, whose *position* also depends on
the string's length) stays an explicit `react` instead, so the distinction between "just
copied" and "actually computed" stays visible in the code rather than hidden behind one
generic mechanism.

## `Graft`: chaining onto a just-spawned entity

Rather than a separate pass over previously-spawned entities elsewhere in a function,
`Graft` lets you attach behavior right where an entity was created:

```rust
// foliage_proper/src/tree.rs
pub struct Graft<'t, T: EcsExtension + ?Sized> { entity: Entity, tree: &'t mut T }
impl<'t, T: EcsExtension + ?Sized> Graft<'t, T> {
    pub fn on_click<M>(self, o: impl IntoEntityObserver<M>) -> Self { .. }
    pub fn animate<A: Animate>(self, anim: Animation<A>) -> Self { .. }
    pub fn write<B: Bundle>(self, b: B) -> Self { .. }
    pub fn enable(self) -> Self { .. }
    pub fn disable(self) -> Self { .. }
}
```

Used as `tree.graft(e).on_click(...).animate(...)`. Each method returns `Self`, so calls
chain, and `Graft` converts back to `Entity` via `From` when you need the id back out.

## Sequencing animations: `sequence`/`sequence_end`, and `Sequence` as sugar over them

The actual primitive -- what every composite in this codebase reaches for directly (60+
call sites; the `Sequence` sugar below is a much rarer 2nd choice) -- is three
`EcsExtension` methods:

```rust
// foliage_proper/src/tree.rs
fn sequence(&mut self) -> Entity;
fn sequence_end<M>(&mut self, seq: Entity, end: impl IntoEntityObserver<M>);
```

```rust
let seq = tree.sequence();
tree.animate(Animation::new(end_value).targeting(e1).during(seq)./* .. */);
tree.animate(Animation::new(other_value).targeting(e2).during(seq)./* .. */);
tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| { /* .. */ });
```

`tree.sequence()` just spawns a bare entity carrying a `SequenceMarker { animations_to_finish: i32 }`
(starting at 0). Every `Animation` with `.during(seq)` increments that counter the moment
its `AnimationRunner` is inserted (an `on_insert` hook, so it's automatic regardless of
call order -- animations can be added to a sequence incrementally, no need to know the
final count up front); each one decrements it again on finishing. Once the count reaches
zero, the sequence entity fires `OnEnd` at itself (the same event type [`Timer`](./time.md)
uses) and **despawns itself** -- `sequence_end`'s closure is a plain `Trigger<OnEnd>`
observer registered on that entity (`tree.entity(seq).observe(end)`), so it fires exactly
once, after every animation in the group has actually finished, never before. Don't hold
onto or reuse a `seq` `Entity` past that point -- it's gone.

`Sequence::new(tree).animate(a1).animate(a2).end(on_finish)` is a thin builder over the
exact same three calls (`Sequence::new` calls `tree.sequence()`, `.animate` calls
`tree.animate(anim.during(seq))`, `.end` calls `tree.sequence_end`) -- convenient when a
sequence's animations are all known up front in one chain, but most real usage reaches
for the raw calls instead, either because the animations are built across a loop/branch,
or because keeping `seq` around as a plain `Entity` (to pass into a closure, thread
through a helper function) is simpler than threading a `Sequence<'t, T>` borrow through
the same code.

## `IntoTargets` and `TargetedEvent`

`send_to`/`remove`/`enable`/`disable` all accept anything implementing `IntoTargets`
(`Entity`, `Vec<Entity>`, `[Entity; N]`) so a single entity or a batch can be targeted
without allocating for the common single-entity case. `TargetedEvent` is how an event
carries its own destination: it stores an `entity` field that `send_to` rewrites per
target immediately before triggering, so the same event value can be aimed at several
entities in turn. The `#[targeted_event]` macro attribute generates this scaffolding
(the field, the `Event`/`EntityEvent` impls, a `new(..)` constructor) so authors never
write `Entity::PLACEHOLDER` by hand.

All of this -- `leaf`/`branch` from the [previous chapter](./spawning.md), plus `react`/
`graft`/`sequence` here -- is implemented once on `Tree`, `DeferredWorld`, and `World`
via the shared `EcsExtension` trait, so the same calls work in a system, an observer, or
app setup code.
