# Carousel

`Carousel` shares [`PageIndex`](../composites-overview.md)/`PageCount` with
[Tabs](./tabs.md) and [Pagination](./pagination.md) -- swipe, an optional embedded
`Pagination` strip, and a programmatic write all move the same value:

```rust
// foliage_proper/src/composite/carousel.rs
pub struct Carousel {}
pub struct CarouselPages { pub count: usize, pub(crate) builder: IndexedSlotFn }
```

Pages are author content via the [slot convention](../composites-overview.md): every
page gets a viewport-sized slot laid out side-by-side, and the whole strip slides on a
page change.

## Every page exists at once; sliding just repositions

```rust
// foliage_proper/src/composite/carousel.rs
fn slot_location(slot_index: usize, current: usize) -> Location {
    let offset = (slot_index as i64 - current as i64) * 100;
    Location::new().xs((offset as i32).pct().as_left().with(((offset + 100) as i32).pct().as_right()), ...)
}
```

`slot_location` is a pure function of a slot's index *relative to the current page* --
every page slot is spawned once (in the structure reaction, on `CarouselPages`/count
change) and simply repositioned by percentage offset when `PageIndex` changes, not torn
down and rebuilt. This is the opposite lifecycle from [List](./list.md) (which destroys
and rebuilds rows on every `ListItems` write) and [Router](../composites/router.md)
(destroys and rebuilds the whole scene on navigate) -- `Carousel` is closer to
[Tabs](./tabs.md)'s "build once, then just toggle/reposition" shape, just repositioning
instead of toggling `Visibility`.

## The viewport blocks its own drag-panning on purpose

```rust
// foliage_proper/src/composite/carousel.rs
let viewport = tree.branch(this, Leaf::sprout().at(..).elevate(Elevation::up(1))
    .with((Grid::default(), InteractionPropagation::grab().disable_drag())));
```

`disable_drag()` (see [Interaction](../interaction.md)) stops the viewport's own implicit
`View` from accepting a raw drag-pan directly -- paging is driven entirely by a
`Disengaged` subscribe that measures the drag distance and decides whether to advance,
not by the generic view-panning every `Grid`-bearing entity gets by default. Without
this, a stray per-tick drag delta landing on the viewport's own offset would fight the
animated `slot_location` transition and desync the strip from where paging thinks it put
it.

## The embedded `Pagination` is optional, and genuinely just `Pagination`

`.pagination(mode)` embeds a real [`Pagination`](./pagination.md) pinned bottom-center --
not a carousel-specific reimplementation of dots. `Carousel` keeps it in sync purely by
writing `PageIndex` back to it whenever its own page changes, relying on the fact that
`Pagination`'s own `PageIndex` reaction only ever *patches* stable indicator entities
(see [Pagination](./pagination.md)), never rebuilds -- which is exactly what makes that
write-back safe to do on every single page change without risking a rebuild loop.
