//! `Card` -- positioned with a plain centered `.at(..)`, the same way any entity
//! positions itself. `main` (the top two-thirds) holds a plain close instruction;
//! `header`/`desc` share the bottom third, header above desc. No `.close_icon(..)` (optional, no
//! registered icon bytes in this example) -- closing goes through a plain clickable
//! "Close" label instead, the same pattern `.close_icon(..)` uses internally: give it its
//! own `InteractionListener`, and resolve back to the card root via
//! `Stem::ascend_to::<Card>(..)`, which walks the real `Stem` chain up to whichever
//! ancestor carries the `Card` component.
//! Run with `cargo run --example card -p foliage`, click "Open Card" then "Close".

use foliage::{
    Button, Card, CloseCard, Color, EcsExtension, Elevation, Entity, Foliage, FontSize,
    GridExt, HorizontalAlignment, InteractionListener, InteractionPropagation, Location,
    OnClick, Query, Rounding, Sprout, Stem, Text, Tree, Trigger, VerticalAlignment,
};

fn close_on_click(trigger: Trigger<OnClick>, stems: Query<&Stem>, cards: Query<&Card>, mut tree: Tree) {
    let card = Stem::ascend_to::<Card>(trigger.event_target(), &stems, &cards);
    tree.trigger_targets(CloseCard::new(), card);
}

fn centered_text(text: &str, color: Color, size: u32) -> impl Sprout {
    Text::new(text)
        .size(FontSize::new(size))
        .color(color)
        .with((
            HorizontalAlignment::Center,
            VerticalAlignment::Middle,
            InteractionPropagation::pass_through(),
        ))
}

fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((300, 200));

    let open_button = foliage.world.leaf(
        Button::new()
            .text("Open Card")
            .rounding(Rounding::Sm)
            .colors(Color::gray(900), Color::green(500))
            .at(Location::new().xs(
                20.px().as_left().with(160.px().as_width()),
                20.px().as_top().with(64.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );
    // `leaf(..)` spawns via a deferred command -- `open_button` is only a reserved id until
    // a flush actually materializes it; `.entity_mut(..)` on an unflushed id panics.
    foliage.world.flush();

    foliage
        .world
        .entity_mut(open_button)
        .observe(|_: Trigger<OnClick>, mut tree: Tree| {
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
                        tree.branch(
                            slot,
                            centered_text("Card Title", Color::gray(200), 16)
                                .at(Location::new().xs(
                                    0.pct().as_left().with(100.pct().as_right()),
                                    0.pct().as_top().with(100.pct().as_bottom()),
                                ))
                                .elevate(Elevation::up(1)),
                        )
                    })
                    .desc(|tree: &mut Tree, slot: Entity| {
                        tree.branch(
                            slot,
                            centered_text("A short description of this card.", Color::gray(400), 12)
                                .at(Location::new().xs(
                                    0.pct().as_left().with(100.pct().as_right()),
                                    0.pct().as_top().with(100.pct().as_bottom()),
                                ))
                                .elevate(Elevation::up(1)),
                        )
                    })
                    .colors(Color::gray(800), Color::gray(200), Color::orange(800))
                    .at(Location::new().xs(
                        50.pct().as_center_x().with(60.pct().as_width()),
                        50.pct().as_center_y().with(60.pct().as_height()),
                    ))
                    .elevate(Elevation::abs(50)),
            );
        });

    foliage.photosynthesize();
}
