# Ops: Named, Keyring, Update, Write

Two small, unrelated-looking tools that turn up constantly once you've read a few
composites: generic targeted markers, and two string-keyed lookup tables.

## `Write<W>` / `Update<U>`: broadcast, then translate -- not duplicates

```rust
// foliage_proper/src/ops.rs
pub struct Write<W: Send + Sync + 'static> { entity: Entity, _phantom: PhantomData<W> }
pub struct Update<U: Send + Sync + 'static> { entity: Entity, _phantom: PhantomData<U> }
```

Identical shape, but they sit at two different ends of the same pipeline, and the
`Section<Logical>` → `Panel`/`Text`/clip fan-out shows exactly why both exist.
`Section<Logical>::on_insert` fires one broadcast every time a `Section` is written:

```rust
// foliage_proper/src/coordinate/section.rs
fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
    ...
    world.trigger_targets(Write::<Self>::new(), this); // one Write<Section<Logical>>
}
```

*Three unrelated* systems each independently observe that same `Write<Section<Logical>>`
and translate it into their *own* type's update signal:

```rust
// foliage_proper/src/panel/mod.rs
fn update_from_section(trigger: Trigger<Write<Section<Logical>>>, mut tree: Tree) {
    tree.trigger_targets(Update::<Panel>::new(), trigger.event_target());
}
// foliage_proper/src/text/mod.rs has its own update_from_section -> Update::<Text>
// foliage_proper/src/ash/clip.rs has its own -> reacts directly, no Update::<Self> needed
```

So: **`Write<T>` is "raw data `T` was just written, broadcast"** -- any number of
unrelated types can subscribe to the same one and each decide independently whether/how
to react. **`Update<U>` is a specific type's own "recompute yourself now" signal** --
observed only by `U`'s own update system, and can be triggered from more than one
source: `Panel::update` fires on `Update<Panel>` regardless of whether that came from a
`Write<Section<Logical>>` (via the bridge above), or directly from `Panel`'s own
`on_insert` when `Color`/`Rounding`/`Outline` change (`panel/mod.rs`'s own `on_insert`
calls `Update::<Panel>::new()` straight away, no `Write` involved). The small bridge
observers (`update_from_section` and friends) are what translate "something raw changed"
into "this specific thing needs recomputing" -- collapsing them into one type would mean
every `Write<Section<Logical>>` subscriber either shares one `Update` type it doesn't
actually own, or `Panel`/`Text`/clip resolution all fire on each other's writes with no
way to opt out selectively.

`Disable`'s cascade (see [Lifecycle](./lifecycle.md)) uses `Write<Disable>` the same way
-- broadcasting "`Disable` was written on this entity" before separately triggering
`InheritDisable` to propagate to children, keeping "notify" and "cascade" as two
distinct steps.

## `Named` / `Keyring`: string keys for entities and assets

```rust
// foliage_proper/src/ops.rs
pub struct Named { map: HashMap<String, Entity> }
pub struct Keyring { map: HashMap<String, AssetKey> }
```

`tree.name(e, "close-button")` / `tree.store(key, "logo")` (both
[`EcsExtension`](./tree.md) methods) register a string alias; `Named::get("close-button")`
/ `Keyring::get("logo")` resolve it back. Both are `Resource`s populated by a private
event (`Name`/`StoredKey`) rather than direct map mutation, so registering a name
follows the same deferred-command path as everything else spawned through
[`Sprout`](./spawning.md) -- an alias set during `build()` is available the moment the
surrounding spawn command actually applies, not before. This is a convenience for
app-level code that wants to look an entity or asset key back up by a human-readable
name instead of threading the raw `Entity`/`AssetKey` through wherever it's needed --
not something the library's own composites use internally.
