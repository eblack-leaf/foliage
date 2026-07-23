# Dropdown

`Dropdown`'s options are deliberately just strings -- the doc comment calls this "the 90%
case, kept to one honest builder call." Authors needing rich option rows compose their
own trigger + a real [`List`](./list.md) directly; both halves of `Dropdown` are public
exactly so that's a supported path, not a fork of library internals.

```rust
// foliage_proper/src/composite/dropdown.rs
pub struct Dropdown {}
pub struct DropdownOptions(pub Vec<String>);
pub struct Selected(pub usize);   // the public value channel
pub struct Expanded(pub bool);    // open/closed, Button's Engagement pattern
```

## The option surface is a real `List`, floated above everything

```rust
// foliage_proper/src/composite/dropdown.rs (abridged)
let surface = tree.branch(e, List::new()
    .items(ListItems::new(row_options.len(), move |tree, slot, i| { .. }))
    .row_height(config.row_height).gap(config.row_gap)
    .at(..)
    .elevate(Elevation::up(3))
    .with((Anchor::new(e), ClipToViewport, style.background, Panel::default())));
```

Not a bespoke list reimplementation -- a genuine [`List`](./list.md), given its own rows
via the same `ListItems` closure convention every `List` uses, sized to the trigger's
own real width via [`Anchor::new(e)`](../grid.md) (the same real anchor-to-trigger
relationship [Popover](./popover.md) uses for its own content surface). `ClipToViewport`
is what lets the option list render past the trigger's own small rect without escaping
the tree structurally -- see [Coordinates](../coordinate.md)'s `StackKey` `FRONT_TIER`
for why that's a real dedicated stacking tier, not just a large `Elevation` number.

## Click-away: real `Stem`-descendants, walked back to the root

```rust
// foliage_proper/src/composite/dropdown.rs (abridged)
if f == e || Some(f) == handle.options || Stem::ascend_to::<Dropdown>(f, &stems, &dropdowns) == e {
    return; // focus landed inside our own trigger or option surface -- don't collapse
}
```

The option rows are genuine `Stem`-descendants of the dropdown root (through the nested
`List`), so resolving "did focus land somewhere inside us" is a direct
[`Stem::ascend_to`](../composites-overview.md) walk, derived straight from that real
structure.

## Structural config, separate from style, forwarded to the `List` underneath

`DropdownConfig` (chevron icon, `max_visible`, row height/gap) is a separate component
from `DropdownStyle` (colors) for the same reason every other composite in this catalog
splits the two: restyling shouldn't re-evaluate structure, and `max_visible`/row sizing
are forwarded straight through to the `List`'s own config, not reimplemented.
