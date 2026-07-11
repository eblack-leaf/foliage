pub(crate) mod music_player;
pub(crate) mod demo;

use crate::icons::IconHandles;
use foliage::{
    anchor, Anchor, Animation, Button, Color, Ease, EcsExtension, Elevation, Entity, FontSize,
    Grid, GridExt, Image, ImageView, InteractionListener, Keyring, LeafBuilder, LeafSpec, Location,
    MemoryId, OnClick, OnEnd, Opacity, Panel, Res, Rounding, Stem, Text, Tree, Trigger,
};

pub(crate) fn build(tree: &mut Tree, home: Entity, keyring: &Keyring) {
        let row_size = 400;
        let root = tree.leaf((
            Grid::new(12.col().gap(24), row_size.px().gap(36)),
            Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                100.pct().as_top().with(200.pct().as_bottom()),
            ),
            Elevation::abs(0),
            Stem::none(),
        ));
        let seq = tree.sequence();
        tree.animate(
            Animation::new(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .start(0)
            .finish(1000)
            .targeting(root)
            .during(seq)
            .eased(Ease::EMPHASIS),
        );
        tree.animate(
            Animation::new(Opacity::new(0.0))
                .start(500)
                .finish(1000)
                .targeting(home)
                .during(seq),
        );
        tree.animate(
            Animation::new(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                (-100).pct().as_top().with(0.pct().as_bottom()),
            ))
            .start(0)
            .finish(1000)
            .targeting(home)
            .eased(Ease::EMPHASIS)
            .during(seq),
        );
        let back = Button::new()
            .rounding(Rounding::Full)
            .icon(IconHandles::ArrowUp.value())
            .colors(Color::gray(300), Color::gray(700))
            .at(Location::new().xs(
                50.pct().as_center_x().with(48.px().as_width()),
                12.px().as_top().with(48.px().as_height()),
            ))
            .elevate(Elevation::abs(95))
            .spawn(tree);
        let mut last = 0;
        let mut card_roots = vec![];
        for (i, item) in ITEMS.iter().enumerate() {
            let card_shadow = Panel::new()
                .color(Color::gray(500))
                .at(Location::new().xs(
                    1.col()
                        .as_left()
                        .adjust(12)
                        .with(12.col().as_right().adjust(12))
                        .max(450.0),
                    (i + 1)
                        .row()
                        .as_top()
                        .adjust(12)
                        .with((i + 1).row().as_bottom().adjust(12)),
                ))
                .elevate(Elevation::up(0))
                .stem(root)
                .spawn(tree);
            tree.write_to(card_shadow, Opacity::new(0.25));
            let card_root = Panel::new()
                .color(Color::gray(800))
                .at(Location::new().xs(
                    1.col().as_left().with(12.col().as_right()).max(450.0),
                    (i + 1).row().as_top().with((i + 1).row().as_bottom()),
                ))
                .elevate(Elevation::up(1))
                .stem(root)
                .spawn(tree);
            tree.write_to(card_root, (Opacity::new(0.0), Grid::default()));
            card_roots.push(card_root);
            let display = tree.leaf((
                Image::new(i as MemoryId, keyring.get(item.key)),
                ImageView::Crop,
                Location::new().xs(
                    1.col().as_left().with(1.col().as_right()),
                    0.pct().as_top().with(70.pct().as_bottom()),
                ),
                Elevation::up(1),
                Stem::some(card_root),
                InteractionListener::new(),
            ));
            let info = Panel::new()
                .color(Color::gray(800))
                .at(Location::new().xs(
                    1.col().as_left().with(1.col().as_right()),
                    70.pct().as_top().with(100.pct().as_bottom()),
                ))
                .elevate(Elevation::up(1))
                .stem(card_root)
                .spawn(tree);
            tree.write_to(info, (Opacity::new(1.0), Grid::new(1.col().gap(8), 3.row().gap(8))));
            let title = Text::new(item.title)
                .size(FontSize::new(16))
                .color(Color::gray(200))
                .at(Location::new().xs(
                    1.col().as_left().with(1.col().as_right()),
                    1.row().as_top().with(1.row().as_bottom()),
                ))
                .elevate(Elevation::up(1))
                .stem(info)
                .spawn(tree);
            let desc = Text::new(item.desc)
                .size(FontSize::new(14))
                .color(Color::gray(500))
                .at(Location::new().xs(
                    1.col().as_left().with(1.col().as_right()),
                    2.row().as_top().with(3.row().as_bottom()),
                ))
                .elevate(Elevation::up(1))
                .stem(info)
                .spawn(tree);
            let launch = Button::new()
                .icon(IconHandles::Box.value())
                .rounding(Rounding::Full)
                .colors(Color::gray(900), Color::orange(800))
                .at(Location::new().xs(
                    100.pct().as_right().adjust(-8).with(44.px().as_width()),
                    100.pct().as_bottom().adjust(-8).with(44.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .stem(info)
                .spawn(tree);
            card_interactive.push((card_root, i, launch));
            card_interactive.push((card_root, i, display));
            last = i + 2;
        }
        for (r, i, ci) in card_interactive.clone() {
            tree.on_click(ci, move |trigger: Trigger<OnClick>, mut tree: Tree, keyring: Res<Keyring>| {
                tree.disable([root, back]);
                let seq = tree.sequence();
                tree.animate(
                    Animation::new(Opacity::new(0.0))
                        .targeting(root)
                        .start(0)
                        .finish(500)
                        .during(seq),
                );
                tree.animate(
                    Animation::new(Opacity::new(0.0))
                        .targeting(back)
                        .start(0)
                        .finish(500)
                        .during(seq),
                );
                let backdrop = Panel::new()
                    .color(Color::gray(800))
                    .at(Location::new().xs(
                        anchor().left().as_left().with(anchor().right().as_right()),
                        anchor().top().as_top().with(anchor().bottom().as_bottom()),
                    ))
                    .elevate(Elevation::abs(50))
                    .with((Anchor::new(r), Opacity::new(0.0), Grid::default()))
                    .spawn(&mut tree);
                tree.animate(
                    Animation::new(Opacity::new(1.0))
                        .targeting(backdrop)
                        .start(0)
                        .finish(200)
                        .during(seq),
                );
                tree.animate(
                    Animation::new(
                        Location::new().xs(
                            0.pct()
                                .as_left()
                                .adjust(24)
                                .with(100.pct().as_right().adjust(-24))
                                .max(450.0),
                            0.pct()
                                .as_top()
                                .adjust(36)
                                .with(100.pct().as_bottom().adjust(-36)),
                        ),
                    )
                    .targeting(backdrop)
                    .start(0)
                    .finish(750)
                    .eased(Ease::INWARD)
                    .during(seq),
                );
                tree.animate(
                    Animation::new(Location::new().xs(
                        0.pct().as_left().with(100.pct().as_right()),
                        0.pct().as_top().with(100.pct().as_bottom()),
                    ))
                    .targeting(backdrop)
                    .start(1000)
                    .finish(1500)
                    .during(seq),
                );
                let terminate = Button::new()
                    .rounding(Rounding::Full)
                    .icon(IconHandles::X.value())
                    .colors(Color::gray(200), Color::orange(800))
                    .at(Location::new().xs(
                        16.px().as_left().with(40.px().as_width()),
                        16.px().as_top().with(40.px().as_height()),
                    ))
                    .elevate(Elevation::abs(95))
                    .spawn(&mut tree);
                let app_base = LeafSpec::new()
                    .stem(backdrop)
                    .at(Location::new().xs(
                        0.pct().as_left().with(100.pct().as_right()),
                        0.pct().as_top().with(100.pct().as_bottom()),
                    ))
                    .elevate(Elevation::up(1))
                    .with(Opacity::new(0.0));
                let app = match i {
                    0 => app_base
                        .with((
                            Panel::default(),
                            Grid::new(12.col().gap(8), 40.px().gap(8)),
                            Color::gray(900),
                        ))
                        .spawn(&mut tree),
                    _ => app_base
                        .with(Grid::new(12.col().gap(8), 40.px().gap(8)))
                        .spawn(&mut tree),
                };
                match i {
                    0 => music_player::build(&mut tree, app, &keyring),
                    _ => demo::build(&mut tree, app),
                }
                tree.on_click(
                    terminate,
                    move |trigger: Trigger<OnClick>, mut tree: Tree| {
                        let seq = tree.sequence();
                        tree.animate(
                            Animation::new(Opacity::new(0.0))
                                .targeting(terminate)
                                .during(seq)
                                .start(0)
                                .finish(500),
                        );
                        tree.animate(
                            Animation::new(
                                Location::new().xs(
                                    0.pct()
                                        .as_left()
                                        .adjust(24)
                                        .with(100.pct().as_right().adjust(-24))
                                        .max(450.0),
                                    0.pct()
                                        .as_top()
                                        .adjust(36)
                                        .with(100.pct().as_bottom().adjust(-36)),
                                ),
                            )
                            .targeting(backdrop)
                            .start(0)
                            .finish(500)
                            .eased(Ease::INWARD)
                            .during(seq),
                        );
                        tree.animate(
                            Animation::new(Location::new().xs(
                                anchor().left().as_left().with(anchor().right().as_right()),
                                anchor().top().as_top().with(anchor().bottom().as_bottom()),
                            ))
                            .targeting(backdrop)
                            .start(750)
                            .finish(1250)
                            .during(seq),
                        );
                        tree.animate(
                            Animation::new(Opacity::new(1.0))
                                .targeting(root)
                                .start(1000)
                                .finish(1500)
                                .during(seq),
                        );
                        tree.animate(
                            Animation::new(Opacity::new(1.0))
                                .targeting(back)
                                .start(1000)
                                .finish(1500)
                                .during(seq),
                        );
                        tree.disable(terminate);
                        tree.remove(app);
                        tree.sequence_end(seq, move |trigger: Trigger<OnEnd>, mut tree: Tree| {
                            tree.remove([terminate, backdrop]);
                            tree.enable([root, back]);
                        });
                    },
                )
            });
        }
        tree.disable(back);
        tree.on_click(back, move |trigger: Trigger<OnClick>, mut tree: Tree| {
            tree.disable([back, root]);
            let s = tree.sequence();
            tree.animate(
                Animation::new(Opacity::new(0.0))
                    .start(0)
                    .finish(500)
                    .during(s)
                    .targeting(root),
            );
            tree.animate(
                Animation::new(Opacity::new(0.0))
                    .start(0)
                    .finish(500)
                    .during(s)
                    .targeting(back),
            );
            tree.animate(
                Animation::new(Opacity::new(1.0))
                    .start(500)
                    .finish(1000)
                    .during(s)
                    .targeting(home),
            );
            tree.animate(
                Animation::new(Location::new().xs(
                    0.pct().as_left().with(100.pct().as_right()),
                    0.pct().as_top().with(100.pct().as_bottom()),
                ))
                .start(0)
                .finish(1000)
                .targeting(home)
                .eased(Ease::EMPHASIS)
                .during(s),
            );
            tree.animate(
                Animation::new(Location::new().xs(
                    0.pct().as_left().with(100.pct().as_right()),
                    100.pct().as_top().with(200.pct().as_bottom()),
                ))
                .start(0)
                .finish(1000)
                .targeting(root)
                .eased(Ease::EMPHASIS)
                .during(s),
            );
            tree.sequence_end(s, move |trigger: Trigger<OnEnd>, mut tree: Tree| {
                tree.remove([root, back]);
                tree.enable(home);
            });
        });
        let _spacing = tree.leaf((
            Stem::some(root),
            Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                last.row().as_top().with(100.px().as_height()),
            ),
        ));
        for (i, cr) in card_roots.iter().enumerate() {
            let i = i as u64;
            tree.animate(
                Animation::new(Opacity::new(1.0))
                    .start(i * 500 + 750)
                    .finish(i * 500 + 1250)
                    .targeting(*cr)
                    .during(seq),
            );
        }
        tree.timer(1000, move |trigger: Trigger<OnEnd>, mut tree: Tree| {
            tree.enable(back);
        });
}
pub(crate) struct PortfolioItem {
    title: &'static str,
    desc: &'static str,
    key: &'static str,
}
impl PortfolioItem {
    const fn new(text: &'static str, desc: &'static str, key: &'static str) -> Self {
        Self {
            title: text,
            desc,
            key,
        }
    }
}
pub(crate) const ITEMS: [PortfolioItem; 2] = [
    PortfolioItem::new(
        "Music Player",
        "Listen to tunes with this nifty music playing app.",
        "music-player",
    ),
    PortfolioItem::new(
        "Artist Blog",
        "Showcase your artwork with a scrolling feed.",
        "artist-blog",
    ),
];
