# Coordinates: Position, Area, Section, Elevation

Every number that describes where something is or how big it is needs to answer one
question first: which units? A window resize changes a widget's on-screen pixel size
without changing what the author asked for; a HiDPI display scales physical pixels
against logical ones. Mixing these up silently is exactly the kind of bug that's hard to
spot from a screenshot -- so the crate makes the *context* part of the type, not just a
convention to remember.

```rust
// foliage_proper/src/coordinate/mod.rs
pub struct Physical;   // real device pixels
pub struct Logical;    // DPI-independent, what an author writes .px()/.pct() against
pub struct Numerical;  // context-erased, for values not yet tied to either
pub type CoordinateUnit = f32;
pub struct Coordinates(pub [CoordinateUnit; 2]);
```

`Position<Context>`, `Area<Context>`, and `Section<Context>` (position + area) are all
generic over one of these three marker types, so `Position<Logical>` and
`Position<Physical>` are different types -- passing one where the other is expected is a
compile error, not a runtime scaling bug. `Section<Logical>` is the one every on-screen
entity carries -- read it back through [`Forest::section`](./forest.md), by
[`Leaf`](./leaf.md). `ResolvedElevation`/`ScaleFactor` (see [Ginkgo](./ginkgo.md)) are
what convert between contexts when a value actually needs to cross into the render
backend (logical units in, physical pixels out to the GPU).

## Elevation: symbolic order in, `StackKey` underneath

`Elevation::up(n)`/`down(n)`/`abs(n)` is what an author writes; it is **not** the number
the GPU sorts by. Two different widgets both writing `up(1)` relative to different
parents must not collide just because their raw numbers happen to match -- so every
`Elevation` write computes a `StackKey`, not a flat float:

```rust
// foliage_proper/src/coordinate/elevation.rs
#[derive(Copy, Clone, Debug, Component)]
pub(crate) struct StackKey {
    key: u128,   // one packed byte per ancestor level (16 levels)
    depth: u8,
}
```

This gives genuine CSS-stacking-context semantics: comparing two `StackKey`s only looks
at the first level where two entities' ancestry actually diverges, so a widget's own
local sibling ordering can never be numerically overridden by unrelated content elsewhere
in the tree, no matter how deep either branch is nested. This replaces a real class of
bug a flat elevation sum has: two unrelated branches off the same root whose *flat*
elevation sums happen to coincide exactly, so one entity renders behind chrome it should
have been in front of, purely by numeric accident. A dedicated `FRONT_TIER` (the top
byte) also exists specifically so a `ClipToViewport` overlay always outranks *every*
ordinary `abs()`-elevated entity, regardless of how far forward that entity's own
absolute value claims to be -- without it, an overlay nested inside a deeply-elevated
parent could still lose to a shallower entity with a numerically larger `abs()`,
rendering behind (and losing clicks to) chrome it should have floated above.

`StackKey` only decides *ordering* (who's in front of whom); it does not decide the
actual GPU depth value. That conversion -- symbolic order to a real `ResolvedElevation`
float -- is [`Ash::assign_elevations`](./ash.md)'s job, covered there.
