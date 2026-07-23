# Popover

`Popover` generalizes `Dropdown`'s click/expand/click-away shape from a fixed string list
to *arbitrary author content*, via the [slot convention](../composites-overview.md) for
both halves: what's always visible (the trigger) and what shows while expanded (the
content).

```rust
// foliage_proper/src/composite/popover.rs
pub struct Popover {}
pub enum PopoverPlacement { Below, Above, Left, Right }
pub struct PopoverExpanded(pub bool); // Toggle's persistent-state pattern
```

## Placement: the real relationship, not a guess

```rust
// foliage_proper/src/composite/popover.rs
PopoverPlacement::Below => Location::new().xs(
    anchor().left().as_left().with(anchor().right().as_right()), // matches the trigger's real width
    anchor().bottom().as_top().adjust(GAP).with(extent.as_height()), // author's own extent
),
```

The doc comment is precise about what `Popover` actually adds over hand-rolled
`anchor()` math: the axis *across* the trigger (width for Below/Above, height for
Left/Right) always matches the trigger's real, resolved size -- the same relationship
[Dropdown](./dropdown.md)'s option list already uses for its own trigger-width match,
not an independent guess. Only the axis *away* from the trigger is the author's call, via
`.extent(..)`, because nothing in the framework knows arbitrary content's own size but
the author who built it -- there's no general "size to fit children" mechanism (only
`Text`/`TextInput` have a real auto-size path, via font-shaping measurement).

## Content is rebuilt fresh every time it opens

```rust
// foliage_proper/src/composite/popover.rs (abridged)
if let Some(existing) = handle.content.take() { tree.remove(existing); }
if open {
    let surface = tree.branch(e, Panel::new()..elevate(Elevation::up(2))
        .with((Anchor::new(e), ClipToViewport, Grid::default())));
    (cfg.content)(&mut tree, surface);
    handle.content = Some(surface);
}
```

Unlike `Dropdown`'s `List` (patched on selection, only rebuilt on structural change),
`Popover`'s content has no cheaper "just recolor it" patch path in general -- arbitrary
author content can't be assumed to support in-place updates the way a fixed row shape
can. So it's matched to [Modal](./modal.md)'s content lifecycle instead: torn down and
rebuilt fresh every time it opens, not patched.

## Real siblings, not an absolute constant

The content surface is `up(2)` relative to the *trigger slot* (`up(1)`) -- an ordinary
structural comparison against a real sibling, not an absolute `Elevation` value chosen to
"float above everything." See [Coordinates](../coordinate.md)'s `StackKey` chapter for
the mechanism (`FRONT_TIER`) that makes a structural, relative elevation comparison like
this one reliable regardless of how deep either side is nested -- two entities in
different branches only ever compare at the first ancestor level where their branches
actually diverge.

## Click-away, the same `Stem::ascend_to` shape `Dropdown` uses

```rust
if f == e || Some(f) == handle.content || Stem::ascend_to::<Popover>(f, &stems, &popovers) == e {
    return;
}
```

Identical mechanism to [Dropdown](./dropdown.md)'s own click-away check -- the content
surface is a genuine `Stem`-descendant of the popover root, so the walk finds its way
back to `this` from the real structure directly.
