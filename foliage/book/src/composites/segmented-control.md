# SegmentedControl

The same exclusive-option shape as [RadioGroup](./radio-group.md), rendered as one
inline joined strip instead of a stacked list of circle+label rows -- "the same shape as
`Pagination`'s `Numbered` mode," per its own doc comment, "just labeled by the author
instead of page numbers, and with no paging concept at all."

```rust
// foliage_proper/src/composite/segmented_control.rs
pub struct SegmentedControl {}
pub struct SegmentedOptions(pub Vec<String>);
pub struct SegmentedSelected(pub usize);
pub struct SegmentedStyle {
    pub active: Color, pub inactive: Color, pub foreground: Color, pub rounding: Rounding,
    pub first_end: Side,  // defaults Side::left() -- a full capsule end
    pub last_end: Side,   // defaults Side::right()
}
```

## One strip, not a row of separate pills

```rust
// foliage_proper/src/composite/segmented_control.rs (abridged)
let side = if last == 0 { solo } else if i == 0 { first_end } else if i == last { last_end } else { Side::none() };
```

Only the two outer segments ever round, each independently shaped via `first_end`/
`last_end` ([Panel](../panel.md)'s `Side`, see [Panel](../panel.md) for how `Side`
composes independently of `Rounding`'s amount); every seam in between stays flat
(`Side::none()`), so the whole thing reads as one continuous bar rather than a row of
touching rounded rectangles. A single-segment control unions `first_end`/`last_end`
together (`solo`), so the default pair still produces a full capsule rather than two
half-capsules fighting for the same segment.

## Structure/patch split -- identical shape to RadioGroup and Pagination

```rust
// foliage_proper/src/composite/segmented_control.rs
tree.react_any::<(SegmentedOptions, SegmentedStyle), _>(this, move |trigger, options, styles, selected, mut handles, mut tree| {
    // full rebuild -- option set or style changed
});
```

Same reasoning as [RadioGroup](./radio-group.md) and [Pagination](./pagination.md):
changing which segment is selected never touches this reaction, only the option set or
style does, so a plain selection change (`tree.write_to(control, SegmentedSelected(i))`)
is a cheap recolor against stable entities, not a teardown-and-rebuild.
[Tabs](./tabs.md)'s header row is a real `SegmentedControl`, branched in as a child, not
a second implementation of this same row-of-exclusive-options behavior.
