# List

`List` owns exactly two things: the scroll viewport, and uniform row layout. What a row
*is* -- visuals, interaction, selection semantics -- belongs entirely to the author's own
closure, via the [slot convention](../composites-overview.md):

```rust
// foliage_proper/src/composite/list.rs
pub struct List {}
```

```rust
// foliage_proper/src/composite/list.rs (build, abridged)
tree.react_any::<(ListItems, ListLayout), _>(this, move |trigger, items, layouts, mut tree| {
    for slot in slots.drain(..) { tree.remove(slot); }
    tree.write_to(e, Grid::new(1.col().gap(layout.gap), layout.row_height.px().gap(layout.gap)));
    for i in 0..items.count {
        let slot = tree.branch(e, Leaf::sprout().at(..).elevate(Elevation::up(1)).with(Grid::default()));
        (items.builder)(&mut tree, slot, i);
        slots.push(slot);
    }
});
```

Rewriting `ListItems` via `tree.write_to` is the entire re-render API: every prior row is
torn down and the closure runs again per index, tracked by a captured `Vec<Entity>` (the
"ProjectCard-image pattern" this book's other composite chapters refer back to). There's
no virtualization -- the doc comment is explicit that `count` is expected to be O(dozens),
not thousands; a list that size doesn't need windowing to stay cheap.

## `View` + a listener: scrolling and ancestor clipping, for free

`List`'s own root carries a real `InteractionListener` and requires `View`
(via [Grid](../grid.md), since `Grid` requires `View`) -- which is what gives the whole
widget wheel/touch scrolling and ancestor clipping (content past the visible rows simply
doesn't render) without `List` implementing either one itself. This is the same `View`
mechanism [Interaction](../interaction.md)'s ancestor-walk targets when a drag or scroll
needs to find the nearest scrollable `View` above whatever was actually grabbed.

## Teardown is one `Remove`

Removal cascades through each row's own `Stem` children automatically (see
[Lifecycle](../lifecycle.md)'s `Remove` cascade) -- so `List` doesn't need special
teardown logic of its own for author content, however deep a row's own subtree goes.
[Dropdown](./dropdown.md)'s option surface is a real `List` underneath, branched in as a
child rather than a second implementation of row-building -- see
[Dropdown](./dropdown.md) for how that composition actually works.
