//! `Modal` -- no `.anchor_to(..)`, so it fades/scales in from a small centered box. No
//! `.close_icon(..)` either (optional, no registered icon bytes in this example) -- closing
//! goes through a plain clickable "Close" label inside the content instead, the same pattern
//! `.close_icon(..)` uses internally: give it its own `InteractionListener`, and resolve back
//! to the modal root via `Root` (the slot it's under carries `Root(modal)`, same as every
//! other composite's descendant-to-root lookup).
//! Run with `cargo run --example modal -p foliage`, click "Open Modal" then "Close".

use foliage::{
    Button, CloseModal, Color, EcsExtension, Elevation, Entity, Foliage, FontSize,
    HorizontalAlignment, GridExt, InteractionListener, InteractionPropagation, Location, Modal,
    OnClick, Query, Root, Rounding, Sprout, Stem, Text, Tree, Trigger, VerticalAlignment,
};

fn close_on_click(trigger: Trigger<OnClick>, stems: Query<&Stem>, roots: Query<&Root>, mut tree: Tree) {
    let slot = stems.get(trigger.event_target()).unwrap().id.unwrap();
    let modal = Root::resolve(slot, &roots);
    tree.trigger_targets(CloseModal::new(), modal);
}

fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((300, 200));

    let open_button = foliage.world.leaf(
        Button::new()
            .text("Open Modal")
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
                Modal::new()
                    .content(|tree: &mut Tree, slot: Entity| {
                        tree.branch(
                            slot,
                            Text::new("modal content")
                                .size(FontSize::new(16))
                                .color(Color::gray(200))
                                .at(Location::new().xs(
                                    16.px().as_left().with(100.pct().as_right().adjust(-16)),
                                    35.pct().as_top().with(55.pct().as_bottom()),
                                ))
                                .elevate(Elevation::up(1))
                                .with((
                                    HorizontalAlignment::Center,
                                    VerticalAlignment::Middle,
                                    InteractionPropagation::pass_through(),
                                )),
                        );
                        let close = tree.branch(
                            slot,
                            Text::new("Close")
                                .size(FontSize::new(16))
                                .color(Color::orange(400))
                                .at(Location::new().xs(
                                    0.pct().as_left().with(100.pct().as_right()),
                                    65.pct().as_top().with(85.pct().as_bottom()),
                                ))
                                .elevate(Elevation::up(1))
                                .with((
                                    HorizontalAlignment::Center,
                                    VerticalAlignment::Middle,
                                    InteractionListener::new(),
                                )),
                        );
                        tree.on_click(close, close_on_click);
                        close
                    })
                    .colors(Color::gray(800), Color::gray(200), Color::orange(800))
                    .elevate(Elevation::abs(50)),
            );
        });

    foliage.photosynthesize();
}
