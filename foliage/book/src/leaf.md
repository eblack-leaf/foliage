# Every Widget Is an Entity: Leaf

Start from nothing: a bare `bevy_ecs::Entity` with no components. It has no position, so
nothing can lay it out. It has no parent, so nothing knows it belongs to a screen. It has
no draw order, so a renderer wouldn't know when to draw it relative to anything else. It
can't be hidden, faded, or clipped, and it can't receive a click. None of that is
optional for something that's going to appear on screen -- so `Leaf` bundles exactly
that set of requirements onto every entity that needs them, via `bevy_ecs`'s
`#[require(...)]`:

```rust
// foliage_proper/src/leaf.rs
#[derive(Component)]
#[require(Stem, Branch)]
#[require(Opacity, Visibility, ClipSection)]
#[require(Section<Logical>, Elevation, InteractionShape, InteractionPropagation, FocusBehavior)]
#[component(on_add = Self::on_add)]
#[component(on_remove = Self::on_remove)]
pub struct Leaf {}
```

Each piece solves one of the problems above:

- **`Stem`/`Branch`** -- explicit, ECS-visible parent/child. Not "which entity happens
  to be near this one in a tree structure kept elsewhere" -- an actual component, so any
  system can query "what's my parent" or "what are my children" without a side table.
- **`Opacity`/`Visibility`/`ClipSection`** -- fade, show/hide, and the ancestor-derived
  region an entity is allowed to draw within. All three are things any renderable entity
  needs regardless of what it renders.
- **`Section<Logical>`/`Elevation`** -- where an entity is (position + size, in
  logical/DPI-independent units) and how far forward or back it draws relative to
  siblings.
- **`InteractionShape`/`InteractionPropagation`/`FocusBehavior`** -- the hit-testing
  shape, whether a click passes through to something behind it, and how focus behaves --
  present on every leaf whether or not it ever actually receives input, so the
  interaction system doesn't need to special-case entities that opted out.

Every one of these is required, not optional-with-a-default-if-forgotten: an entity
missing any of them isn't a valid on-screen thing, so `Leaf` doesn't let you spawn one
without them.

## Leaf attaches itself, not the other way around

You might expect a rendering primitive like `Panel` to declare `#[require(Leaf, ...)]`
-- but it doesn't. Grep `foliage_proper/src/panel/mod.rs` and `text/mod.rs` and neither
one mentions `Leaf` at all. Instead, `Leaf` is unioned onto *every* entity at the moment
it's spawned, by the one path that's allowed to spawn anything:

```rust
// foliage_proper/src/tree.rs
impl Sow for Tree<'_, '_> {
    fn sow<B: Bundle>(&mut self, b: B) -> Entity {
        let entity = self.spawn((Leaf::new(), b)).id();
        entity
    }
}
```

So `Panel`, `Text`, `Button`, and every other widget are components that describe *what*
an entity is, while `Leaf` is inserted alongside them, unconditionally, describing what
*any* on-screen entity needs regardless of what it is. This is why `Leaf`'s own
`on_add`/`on_remove` hooks (registering `anim_opacity`/`anim_elevation`/`anim_location`
observers, and cleaning up `CurrentInteraction` state on removal) run for literally every
widget in the framework, without each widget type needing to remember to register them
itself.

The next chapter, [Spawning](./spawning.md), covers exactly how `sow` gets called, and
why an entity can't be spawned any other way.
