# Tabs

`Tabs` is composed almost entirely from two other composites rather than implementing
its own header or paging logic: the header row *is* a real [`SegmentedControl`](./segmented-control.md),
branched in as a child, and the paging vocabulary is the same `PageIndex`/`PageCount`/
`PageChanged` shape [Carousel](./carousel.md) and [Pagination](./pagination.md) share.

```rust
// foliage_proper/src/composite/tabs.rs
pub struct Tabs {}
pub struct TabsPages { pub labels: Vec<String>, pub(crate) builder: IndexedSlotFn }
```

## Built once, shown by `Visibility` -- never rebuilt on switch

```rust
// foliage_proper/src/composite/tabs.rs (abridged)
for i in 0..tabs_pages.labels.len() {
    let slot = tree.branch(e, Leaf::sprout().at(..).elevate(Elevation::up(1))
        .with((Grid::default(), Visibility::new(i == current))));
    (tabs_pages.builder)(&mut tree, slot, i);
    handle.content_slots.push(slot);
}
```

Every tab's content slot is built exactly once, when the label set or style changes; the
author's builder closure runs once per tab, full stop. Switching tabs later is a separate,
much cheaper reaction that only ever toggles `Visibility` on the already-existing slots
-- it never re-runs the author's content builder. The module doc comment draws the
contrast directly against `Carousel`'s own lifecycle: `Carousel` builds every page once
too, but *repositions* them to slide; `Tabs` builds every page once and *hides* all but
one. Both are "build once" composites, just with a different visual effect layered on
top of the same underlying shape -- neither is `List`'s or `Router`'s
destroy-and-rebuild-on-change lifecycle.

## Element-level, not engine-level

Per the module doc comment: like `List`/`Carousel`, tab content lifecycle is the
author's own to manage -- each tab's builder closure owns whatever state that tab needs,
and `Tabs` itself only ever owns *which one is currently shown*. Nothing here decides
when a tab's own internal state resets or persists; that's entirely a property of what
the author's closure actually does when it's called.

## The header forwards color/rounding, drives `PageIndex` directly

```rust
// TabsStyle forwards active/inactive/foreground/rounding straight to the header SegmentedControl
```

Selecting a header segment writes the same `PageIndex` a swipe or a programmatic write
would -- the header doesn't have its own separate "which tab is active" state to keep in
sync with the content area's; there's only ever one `PageIndex`, and both the header and
the content-visibility reaction read it.
