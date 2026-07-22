# Spawning: Sprout and EcsExtension

If any code could call `world.spawn((Panel::new_marker(), Color::default(), ...))`
directly, two things could go wrong silently: the author could forget `Leaf` (and get an
entity nothing can position or draw), or forget to set an explicit draw order. Neither
should be possible to get wrong by omission -- so foliage funnels every widget spawn
through one narrow door.

## `Sprout`: config now, entity later

A widget's public API is a config *builder*, not a spawn call. `Button::new()` doesn't
create an entity -- it returns a `ButtonSprout`, a plain struct you configure with
`.text(..)`, `.colors(..)`, and so on, which only becomes a real entity once handed to
`tree.leaf(..)` or `tree.branch(..)`:

```rust
// foliage_proper/src/author.rs
pub trait Sprout: Sized {
    fn seed(&mut self) -> &mut LeafSprout;
    /// config -> components on the root entity. This IS the widget's public API.
    fn root(self) -> impl Bundle;
    /// private static skeleton + reaction registration. Default empty == a primitive.
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) {}
    fn at(mut self, location: Location) -> Self { .. }
    fn elevate(mut self, e: Elevation) -> Self { .. }
    fn with<X: Bundle>(self, extra: X) -> With<Self, X> { .. }
}
```

Every `Sprout` embeds a `LeafSprout` -- position, parent, and elevation:

```rust
// foliage_proper/src/author.rs
pub struct LeafSprout {
    pub(crate) location: Location,
    pub(crate) stem: Stem,
    pub(crate) elevation: Option<Elevation>,
}
```

`elevation` is deliberately `Option` with **no default**, per its own doc comment:

> an unset `Elevation` silently picking *some* layer (front, back, or "one above
> whatever the parent is this moment") is exactly the kind of surprise that's cost real
> debugging time in this codebase... Leaving it unset is a hard requirement to specify
> it, not a soft default to fall back on.

`root()` and `build()` split a widget's spawn into two concerns: `root()` is *what
this entity is* -- every config field folded into one bundle, inserted once, at spawn.
`build()` is *what this entity needs around it* -- a config-independent skeleton of
child entities and reactions, grown via `EcsExtension::branch`. A primitive like `Panel`
has nothing to build (the default empty `build` is correct); a composite like `Button`
uses it to spawn its `Panel`/`Text`/`Icon` children.

## `EcsExtension::leaf`/`branch`: the only way in

`Sprout` alone can't spawn anything -- `root`/`build` are just data and a private
`Sow::grow` is the only thing that turns them into an entity, and `Sow` is
`pub(crate)` on purpose:

```rust
// foliage_proper/src/tree.rs
pub(crate) trait Sow {
    fn sow<B: Bundle>(&mut self, b: B) -> Entity;
    fn grow<S: Sprout>(&mut self, mut spec: S, parent: Option<Entity>) -> Entity
    where
        Self: EcsExtension + Sized,
    {
        if let Some(parent) = parent {
            spec.seed().stem = Stem::some(parent);
        }
        let leaf = core::mem::take(spec.seed());
        let this = self.sow((
            leaf.location,
            leaf.stem,
            leaf.elevation
                .expect("elevation not set -- call .elevate(...) before spawning"),
            spec.root(),
        ));
        S::build(this, self);
        this
    }
}
```

That `.expect(...)` is where an un-elevated widget actually fails -- loudly, at spawn
time, rather than silently picking a layer. `leaf`/`branch` are the only public entry
points that reach `grow`:

```rust
// foliage_proper/src/tree.rs (EcsExtension)
fn leaf<S: Sprout>(&mut self, spec: S) -> Entity { self.grow(spec, None) }
fn branch<S: Sprout>(&mut self, parent: Entity, spec: S) -> Entity { self.grow(spec, Some(parent)) }
```

`leaf` spawns a root (no parent -- a screen, a one-off widget); `branch` spawns a child,
with `parent` as a required argument so a child can't be spawned orphaned by forgetting a
chained call. Both are implemented once for `Tree` (`Commands`), `DeferredWorld`, and
`World`, so the same calls work whether you're in a system, an observer, or app setup.

An external crate can implement `Sprout` for its own widget types (as
`application`-style code does), but it can never call `Sow::grow` directly -- there's no
path from a `Sprout` impl to a spawned entity that skips the mandatory `.elevate(...)` or
hand-rolls a parent link. That's the whole point of keeping `Sow` private.

`LeafSprout` itself implements `Sprout` -- the "no named marker component" case, for a
child with no reusable type of its own (a bare interaction hit-area). Its `root()`
returns an empty bundle; any marker it needs is attached via `.with(...)`, the same as
every other extra component -- which is what makes `branch` uniform for *every* child
instead of needing a raw, un-typed spawn path.

Once an entity exists, acting on it further -- `on_click`, `animate`, more components --
is [Tree and Graft's](./tree.md) job, not `Sprout`'s.
