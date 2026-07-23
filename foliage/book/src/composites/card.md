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

## Removal

`Card` has no close/remove mechanism of its own -- `tree.remove(card)` is the same
general removal every entity uses (see [Lifecycle](../lifecycle.md)'s `Remove` cascade
through `Stem` children), which reaches `main`/`header`/`desc` and everything the
author's own closures branched underneath them.

## Using it

```rust
// foliage/examples/card.rs
foliage.world.leaf(
    Card::new()
        .main(|tree: &mut Tree, slot: Entity| {
            tree.branch(slot, Panel::new().color(Color::gray(700))
                .at(Location::new().xs(0.pct().as_left().with(100.pct().as_right()), 0.pct().as_top().with(100.pct().as_bottom())))
                .elevate(Elevation::up(1)))
        })
        .header(|tree: &mut Tree, slot: Entity| {
            tree.branch(slot, centered_text("Card Title", Color::gray(200), 16)
                .at(Location::new().xs(0.pct().as_left().with(100.pct().as_right()), 0.pct().as_top().with(100.pct().as_bottom()))))
        })
        .desc(|tree: &mut Tree, slot: Entity| {
            tree.branch(slot, centered_text("A short description of this card.", Color::gray(400), 12)
                .at(Location::new().xs(0.pct().as_left().with(100.pct().as_right()), 0.pct().as_top().with(100.pct().as_bottom()))))
        })
        .colors(Color::gray(800))
        .at(Location::new().xs(
            10.pct().as_left().with(90.pct().as_right()),
            10.pct().as_top().with(90.pct().as_bottom()),
        ))
        .elevate(Elevation::up(1)),
);
```

Run with `cargo run --example card -p foliage`.
