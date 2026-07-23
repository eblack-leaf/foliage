# RadioGroup

There is deliberately no standalone single-radio-button widget in this crate. The doc
comment on `RadioGroup` says why directly: exclusivity ("picking one deselects the rest")
has to be owned by *something*, and a lone radio button with no group to be exclusive
within isn't a meaningful thing to build in the first place -- so the group, not the
button, is the composite.

```rust
// foliage_proper/src/composite/radio_group.rs
pub struct RadioGroup {}
pub struct RadioOptions(pub Vec<String>);  // the label set, rewritten as one unit
pub struct RadioSelected(pub usize);       // the one public value channel
```

## Structure and patch, kept deliberately separate

`RadioGroup` follows the same two-reaction split [Pagination](./pagination.md) and
[SegmentedControl](./segmented-control.md) both use: one reaction rebuilds row entities,
a *different* one just recolors them, and only the first kind of change ever spawns or
despawns anything:

```rust
// foliage_proper/src/composite/radio_group.rs
tree.react_any::<(RadioOptions, RadioStyle), _>(this, move |trigger, options, styles, selected, mut handles, mut tree| {
    // tear down every (circle, label) pair, rebuild the full row set from scratch
});
```

Changing *which* option is selected never runs this -- only changing the option set
itself or restyling does. A row is two entities, a circle and a label, each independently
clickable (`tree.on_click` on both, same handler: `tree.write_to(e, RadioSelected(i))`)
-- the label is included on purpose, as the doc comment on the click wiring notes, since
it's the bigger, more natural tap target than the circle alone.

## Selection is a plain write, same door as every other value channel

```rust
tree.on_click(circle, move |_: Trigger<OnClick>, mut tree: Tree| {
    tree.write_to(e, RadioSelected(i));
});
```

Clicking any row just writes `RadioSelected(i)` to the group root -- exclusivity itself
isn't computed by comparing against the old value; it falls out for free because
whichever row's index matches the new `RadioSelected` reads as active and every other row
reads as inactive, the next time colors are derived. `RadioChanged` fires from its own
`react::<RadioSelected, _>`, the same "state change gets its own event door, separate
from the restyle reaction" pattern every other composite with a persistent value channel
follows -- see [Checkbox](./checkbox.md)'s `Checked` for the identical shape.
