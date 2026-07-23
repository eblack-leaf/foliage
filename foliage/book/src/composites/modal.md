# Modal

Unlike every other composite in this catalog, `Modal`'s root entity *is* its own visual
surface -- the doc comment is explicit: "the modal owns its root `Location` (it IS the
overlay rect...) -- `.at()` on the sprout is meaningless here."

```rust
// foliage_proper/src/composite/modal.rs
fn root(self) -> impl Bundle {
    (
        Modal {}, ModalConfig { .. }, self.style,
        Panel::default(), // the root entity IS the backdrop panel
        Grid::default(),
    )
}
```

Making the one public entity the visual overlay itself (rather than a container holding
a separate backdrop child) means teardown is a single `tree.remove(this)`, and opening is
a single write of final state:

```rust
// foliage_proper/src/composite/modal.rs (config reaction, abridged)
tree.write_to(e, (full_location(), style.backdrop, Opacity::new(1.0)));
```

`Modal` opens and closes instantly -- there's no animation of its own. `full_location()`
is a full-bleed, edge-to-edge rect; anything padded or centered is the author's own
`.content(..)` closure's job to lay out inside that space.

## One close path, one immediate removal

```rust
// foliage_proper/src/composite/modal.rs
tree.subscribe(this, move |trigger: Trigger<CloseModal>, mut tree: Tree| {
    let e = trigger.event_target();
    tree.trigger_targets(Closed::new(), e);
    tree.remove(e); // children (slot content, terminate button) are Stem-parented -- one remove cascades the lot
});
```

The close button (present only when `.close_icon(..)` supplies one -- the library ships
none) and a programmatic `tree.trigger_targets(CloseModal::new(), modal)` both land on
this exact handler -- there's no separate "close via button" code path to keep in sync
with "close via API." `Closed` fires, then the whole modal subtree is gone in the same
command batch.

## `.anchor_to(..)` currently has no visible effect

`ModalSprout::anchor_to(entity)` sets an `Anchor` component on the modal root. Nothing in
`Modal` currently reads it -- `full_location()` doesn't reference it, and nothing else in
`build()` does either. This is an open item, not a documented feature: an author calling
`.anchor_to(..)` today gets no observable behavior difference from not calling it at all.
