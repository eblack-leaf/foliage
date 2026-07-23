# Pagination

`Pagination` shares [`PageIndex`](../composites-overview.md)/`PageCount` with
[Tabs](./tabs.md) and [Carousel](./carousel.md) -- one paging vocabulary, so any of the
three can drive (or be driven by) the others by writing the same component type.

```rust
// foliage_proper/src/composite/pagination.rs
pub struct Pagination {}
pub enum PaginationMode {
    Dots,     // one dot per page -- the carousel indicator
    Numbered, // at most 5 numbered pips, a sliding window centered on the current page
}
```

## Structure vs. patch: two reactions, two very different costs

```rust
// foliage_proper/src/composite/pagination.rs
tree.react_any::<(PageCount, PaginationStyle), _>(this, move |trigger, counts, styles, pages, mut handles, mut tree| {
    // STRUCTURE: tear down and rebuild every indicator slot
});
tree.react::<PageIndex, _>(this, move |trigger, pages, counts, styles, handles, mut tree| {
    // PATCH: recolor/retext the stable slots the structure reaction already built
});
```

The doc comment is explicit about why this split exists: indicator entities are built
once per *structural* change (page count or style) and only ever patched -- recolored,
relabeled -- on a page change. This matters for exactly the case
[Carousel](./carousel.md) creates: it keeps an embedded `Pagination` in sync by writing
`PageIndex` back to it on every swipe, and if that write triggered a full rebuild, the
resulting teardown could re-enter and fight the very page change that triggered it.
Because a page change only ever patches, that event cycle is always safe.

## `Numbered` mode: a sliding window, computed the same way everywhere

```rust
// foliage_proper/src/composite/pagination.rs
fn window_start(current: usize, count: usize, shown: usize) -> usize {
    current.saturating_sub(shown / 2).min(count.saturating_sub(shown))
}
```

A pure function, deliberately -- the doc comment notes it exists so the build pass, the
patch pass, and every individual pip's click handler all agree on which page a given
slot number represents *by construction*, rather than three separately-maintained
calculations that could drift out of sync.

## Both entities per slot are tracked, and both get removed

`PaginationHandle` stores `(Entity, Entity)` per slot -- for `Dots` these are two
different entities: an invisible, generously-sized hit region, and the small visible pip
parented inside it, because a 4px-tall bar is a real bar to look at but not a real click
target. A structural rebuild removes the hit-region root for every slot, which cascades
to the pip parented inside it (see [Lifecycle](../lifecycle.md)'s `Remove`) -- tracking
just the recolor target instead would leave the hit-region itself, and its
`InteractionListener`, behind.
