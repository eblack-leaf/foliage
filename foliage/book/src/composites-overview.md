# The Slot Convention

Every composite in the crate -- `Modal`, `List`, `Dropdown`, `Carousel`, and so on --
needs to host arbitrary author content without baking in what that content is. A
`Modal` doesn't know or care whether its body is a form, an image, or another composite;
it only needs a place to put whatever the author gives it, positioned and parented
correctly, and torn down correctly when the modal closes. That's `SlotFn`:

```rust
// foliage_proper/src/composite/mod.rs
pub type SlotFn = Arc<dyn Fn(&mut Tree, Entity) -> Entity + Send + Sync>;
pub type IndexedSlotFn = Arc<dyn Fn(&mut Tree, Entity, usize) + Send + Sync>;
```

The composite calls the closure with a transient **slot** entity -- a private, positioned
`Grid::default()` child it just spawned -- and the author branches whatever they want
under it with full `Location`/`Grid`/`Anchor` freedom, returning their content root (see
`Modal`'s `.content(|tree, slot| { ... })` in the [Router](./composites/router.md) and
[Button](./composite-button.md) chapters' sibling examples, or `examples/modal.rs` for a
complete one). The closure is `Fn`, not `FnOnce`, and it lives inside the widget's own
config component -- so rewriting that config via `tree.write_to` re-runs the closure
against a fresh slot. Config writes *are* the re-render API here, the same door
[`react`](./tree.md) opens for every other value in the crate. `IndexedSlotFn` is the
collection variant (`List` rows, `Carousel` pages) -- called once per index with that
index's own slot, no return value, since collection slots are torn down wholesale by the
composite rather than individually removed the way a `Modal`'s single slot is on close.

Slots carry no interaction opinions of their own -- author content under a slot may be
interactive or not, the composite doesn't decide that for you.

## Shared value channels

A handful of components exist purely as a public write-target, so any composite that
carries the same *kind* of state exposes the same API an author already knows:

```rust
// foliage_proper/src/composite/mod.rs
pub struct TextValue(pub String);   // write it to a Text entity, or a root that forwards it (Button, TextInput)
pub struct IconValue(pub IconId);   // same contract, for icons
pub struct Progress(pub f32);       // 0.0..=1.0, e.g. Slider
pub struct PageIndex(pub usize);    // current page, e.g. Pagination, Carousel
pub struct PageCount(pub usize);    // the other half of PageIndex
```

`tree.write_to(button_entity, TextValue("new label".into()))` works identically whether
`button_entity` is a `Button`, a `TextInput`, or anything else that forwards `TextValue`
onward -- the composite's root is the API, regardless of what's actually rendering it.

## Routing a descendant back to its root: `Stem::ascend_to`

A reactive system running on a composite's *descendant* (a slot's content, an option
row) often can't resolve widget-level state locally and needs to route back up to the
composite's root entity. Every composite already carries its own marker component on
that root (`Modal`, `Dropdown`, `TextInput`, ...), so [`Stem`](./leaf.md) itself can
provide the walk directly, derived from the real structure:

```rust
// foliage_proper/src/leaf.rs
impl Stem {
    pub fn ascend_to<C: Component>(entity: Entity, stems: &Query<&Stem>, markers: &Query<&C>) -> Entity {
        let mut current = entity;
        loop {
            if markers.get(current).is_ok() {
                return current;
            }
            match stems.get(current).ok().and_then(|s| s.id) {
                Some(parent) => current = parent,
                None => return current,
            }
        }
    }
}
```

Used as `Stem::ascend_to::<Modal>(entity, &stems, &modals)` -- for instance, by a
`Modal`'s "Close" button (spawned inside the slot) to find its way back to the modal
root it needs to trigger `CloseModal` against. Walking through a *different* composite
type nested in between is harmless: `C` only matches its own type, so the walk passes
through unrelated ancestors (Dropdown's option rows sit inside a nested `List`, for
instance) and keeps going until it reaches the actual root.

[Building `Button` From Scratch](./composite-button.md) is the next chapter, and puts
all of this -- `Sprout`, `react`, value channels -- to work in one worked example.
