# Card

`Card` is the common "primary content, then a title, then a description" shape --
an image or main visual on top, a header and a short description stacked below it. One
entity, itself the backdrop panel.

```rust
// foliage_proper/src/composite/card.rs
pub struct Card {}
fn root(self) -> impl Bundle {
    (
        Card {}, CardConfig { .. }, self.style,
        Panel::default(), // the root entity IS the backdrop panel
        Grid::new(1.col(), 3.row()), // three equal rows: main spans 1-2, header/desc share row 3
    )
}
```

## Three regions: `main`, `header`, `desc`

```rust
// foliage_proper/src/composite/card.rs
pub fn main(mut self, f: impl Fn(&mut Tree, Entity) -> Entity + Send + Sync + 'static) -> Self { .. } // required
pub fn header(mut self, f: impl Fn(&mut Tree, Entity) -> Entity + Send + Sync + 'static) -> Self { .. } // optional
pub fn desc(mut self, f: impl Fn(&mut Tree, Entity) -> Entity + Send + Sync + 'static) -> Self { .. }   // optional
```

`main` fills the top two-thirds of the card (rows 1-2 of the root's own 3-row grid) --
required, since a card with nothing in it isn't meaningful. `header` and `desc` are
optional, share the bottom third as their own two-row grouping (header above desc), and
follow the same [slot convention](../composites-overview.md) every other composite with
arbitrary author content uses. Skipping both means no bottom-third container gets
spawned at all -- not a defunct empty one:

```rust
// foliage_proper/src/composite/card.rs (build, abridged)
let main_slot = tree.branch(e, Leaf::sprout().at(Location::new().xs(
    0.pct().as_left().with(100.pct().as_right()),
    1.row().as_top().with(2.row().as_bottom()), // rows 1-2 = top two-thirds
)).elevate(Elevation::up(1)).with(Grid::default()));
(cfg.main)(&mut tree, main_slot);

if cfg.header.is_some() || cfg.desc.is_some() {
    let bottom_third = tree.branch(e, Leaf::sprout().at(Location::new().xs(
        0.pct().as_left().with(100.pct().as_right()),
        3.row().as_top().with(3.row().as_bottom()), // row 3 = bottom third
    )).elevate(Elevation::up(1)).with(Grid::new(1.col(), 2.row())));
    // header -> bottom_third's own row 1, desc -> bottom_third's own row 2
}
```

Two levels of the same row-count Grid idiom [Grid](../grid.md) covers -- the outer 3-row
grid divides the card into thirds, and the bottom third gets its own nested 2-row grid
for header and desc, rather than one grid with irregular row sizes.

## Close button placement

```rust
// foliage_proper/src/composite/card.rs
.at(Location::new().xs(
    16.px().as_left().with(40.px().as_width()),
    16.px().as_top().with(40.px().as_height()),
))
.elevate(Elevation::up(2)), // real siblings (main_slot/bottom_third) are up(1)
```

Present only when `.close_icon(..)` supplies one -- the library ships none. Fixed at
`(16px, 16px)`, so it overlays whichever corner of `main` its `40px` box falls within,
the same "X in the corner of an image" pattern common to image cards. `up(2)` outranks
its siblings under the same parent, which is all [`StackKey`](../coordinate.md) compares.

## One close path, one immediate removal

```rust
// foliage_proper/src/composite/card.rs
tree.subscribe(this, move |trigger: Trigger<CloseCard>, mut tree: Tree| {
    let e = trigger.event_target();
    tree.trigger_targets(Closed::new(), e);
    tree.remove(e); // children (slot content, terminate button) are Stem-parented -- one remove cascades the lot
});
```

The close button and a programmatic `tree.trigger_targets(CloseCard::new(), card)` both
land on this exact handler. `Closed` fires, then the whole card subtree is gone in the
same command batch.

## Using it

```rust
// foliage/examples/card.rs
tree.leaf(
    Card::new()
        .main(|tree: &mut Tree, slot: Entity| {
            let close = tree.branch(
                slot,
                centered_text("Close", Color::orange(400), 16)
                    .at(Location::new().xs(
                        0.pct().as_left().with(100.pct().as_right()),
                        40.pct().as_top().with(60.pct().as_bottom()),
                    ))
                    .elevate(Elevation::up(1))
                    .with(InteractionListener::new()),
            );
            tree.on_click(close, close_on_click);
            close
        })
        .header(|tree: &mut Tree, slot: Entity| {
            tree.branch(slot, centered_text("Card Title", Color::gray(200), 16)
                .at(Location::new().xs(0.pct().as_left().with(100.pct().as_right()), 0.pct().as_top().with(100.pct().as_bottom())))
                .elevate(Elevation::up(1)))
        })
        .desc(|tree: &mut Tree, slot: Entity| {
            tree.branch(slot, centered_text("A short description of this card.", Color::gray(400), 12)
                .at(Location::new().xs(0.pct().as_left().with(100.pct().as_right()), 0.pct().as_top().with(100.pct().as_bottom())))
                .elevate(Elevation::up(1)))
        })
        .colors(Color::gray(800), Color::gray(200), Color::orange(800))
        .at(Location::new().xs(
            50.pct().as_center_x().with(60.pct().as_width()),
            50.pct().as_center_y().with(60.pct().as_height()),
        ))
        .elevate(Elevation::abs(50)),
);
```

No `.close_icon(..)` here (the example has no registered icon bytes to give it), so
closing goes through a plain clickable "Close" label inside `main` instead -- the same
pattern `.close_icon(..)` uses internally: give it its own `InteractionListener`, and
resolve back to the card root via `Stem::ascend_to::<Card>(..)` (see
[composites-overview](../composites-overview.md)), which walks the real `Stem` chain up
to whichever ancestor carries `Card`:

```rust
// foliage/examples/card.rs
fn close_on_click(trigger: Trigger<OnClick>, stems: Query<&Stem>, cards: Query<&Card>, mut tree: Tree) {
    let card = Stem::ascend_to::<Card>(trigger.event_target(), &stems, &cards);
    tree.trigger_targets(CloseCard::new(), card);
}
```

Run with `cargo run --example card -p foliage`.
