pub(crate) mod demo;
pub(crate) mod music_player;

use crate::icons::IconHandles;
use crate::widgets::{Launch, ProjectCard};
use foliage::{
    anchor, Anchor, Animation, Button, Color, Ease, EcsExtension, Elevation, Entity, Grid, GridExt,
    Keyring, Leaf, Location, OnClick, OnEnd, Opacity, Panel, Res, Rounding, Sequence, Sprout, Tree,
    Trigger,
};

pub(crate) fn build(tree: &mut Tree, home: Entity, keyring: &Keyring) {
    let row_size = 400;
    let root = tree.leaf(
        Leaf::sprout()
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                100.pct().as_top().with(200.pct().as_bottom()),
            ))
            .elevate(Elevation::abs(0))
            .with(Grid::new(12.col().gap(24), row_size.px().gap(36))),
    );
    let seq = Sequence::new(tree)
        .animate(
            Animation::new(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .start(0)
            .finish(1000)
            .targeting(root)
            .eased(Ease::EMPHASIS),
        )
        .animate(
            Animation::new(Opacity::new(0.0))
                .start(500)
                .finish(1000)
                .targeting(home),
        )
        .animate(
            Animation::new(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                (-100).pct().as_top().with(0.pct().as_bottom()),
            ))
            .start(0)
            .finish(1000)
            .targeting(home)
            .eased(Ease::EMPHASIS),
        )
        .id();
    let back = tree.leaf(
        Button::new()
            .rounding(Rounding::Full)
            .icon(IconHandles::ArrowUp.into())
            .colors(Color::gray(300), Color::gray(700))
            .at(Location::new().xs(
                50.pct().as_center_x().with(48.px().as_width()),
                12.px().as_top().with(48.px().as_height()),
            ))
            .elevate(Elevation::abs(95)),
    );
    let mut last = 0;
    let card_roots: Vec<Entity> = ITEMS
        .iter()
        .enumerate()
        .map(|(i, item)| {
            tree.branch(
                root,
                Panel::new()
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
                    .with(Opacity::new(0.25)),
            );
            // the whole card interior (image + title + desc + launch button + their click
            // wiring) is one widget spawn -- ProjectInfo in, Launch out.
            let card_root = tree.branch(
                root,
                ProjectCard::new()
                    .title(item.title)
                    .desc(item.desc)
                    .image(keyring.get(item.key))
                    .at(Location::new().xs(
                        1.col().as_left().with(12.col().as_right()).max(450.0),
                        (i + 1).row().as_top().with((i + 1).row().as_bottom()),
                    ))
                    .elevate(Elevation::up(1))
                    .with(Opacity::new(0.0)),
            );
            last = i + 2;
            let open_modal =
                move |trigger: Trigger<Launch>, mut tree: Tree, keyring: Res<Keyring>| {
                    tree.disable([root, back]);
                    // spawn everything first -- animating happens once every entity involved
                    // already exists, so it's one uninterrupted Sequence chain below instead of
                    // animate calls threaded between spawns.
                    let backdrop = tree.leaf(
                        Panel::new()
                            .color(Color::gray(800))
                            .at(Location::new().xs(
                                anchor().left().as_left().with(anchor().right().as_right()),
                                anchor().top().as_top().with(anchor().bottom().as_bottom()),
                            ))
                            .elevate(Elevation::abs(50))
                            .with((Anchor::new(card_root), Opacity::new(0.0), Grid::default())),
                    );
                    let terminate = tree.leaf(
                        Button::new()
                            .rounding(Rounding::Full)
                            .icon(IconHandles::X.into())
                            .colors(Color::gray(200), Color::orange(800))
                            .at(Location::new().xs(
                                16.px().as_left().with(40.px().as_width()),
                                16.px().as_top().with(40.px().as_height()),
                            ))
                            .elevate(Elevation::abs(95)),
                    );
                    let app_base = Leaf::sprout()
                        .at(Location::new().xs(
                            0.pct().as_left().with(100.pct().as_right()),
                            0.pct().as_top().with(100.pct().as_bottom()),
                        ))
                        .elevate(Elevation::up(1))
                        .with(Opacity::new(0.0));
                    let app = match i {
                        0 => tree.branch(
                            backdrop,
                            app_base.with((
                                Panel::default(),
                                Grid::new(12.col().gap(8), 40.px().gap(8)),
                                Color::gray(900),
                            )),
                        ),
                        _ => tree.branch(
                            backdrop,
                            app_base.with(Grid::new(12.col().gap(8), 40.px().gap(8))),
                        ),
                    };
                    match i {
                        0 => music_player::build(&mut tree, app, &keyring),
                        _ => demo::build(&mut tree, app),
                    }
                    tree.on_click(
                        terminate,
                        move |trigger: Trigger<OnClick>, mut tree: Tree| {
                            Sequence::new(&mut tree)
                                .animate(
                                    Animation::new(Opacity::new(0.0))
                                        .targeting(terminate)
                                        .start(0)
                                        .finish(500),
                                )
                                .animate(
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
                                    .eased(Ease::INWARD),
                                )
                                .animate(
                                    Animation::new(Location::new().xs(
                                        anchor().left().as_left().with(anchor().right().as_right()),
                                        anchor().top().as_top().with(anchor().bottom().as_bottom()),
                                    ))
                                    .targeting(backdrop)
                                    .start(750)
                                    .finish(1250),
                                )
                                .animate(
                                    Animation::new(Opacity::new(1.0))
                                        .targeting(root)
                                        .start(1000)
                                        .finish(1500),
                                )
                                .animate(
                                    Animation::new(Opacity::new(1.0))
                                        .targeting(back)
                                        .start(1000)
                                        .finish(1500),
                                )
                                .end(move |trigger: Trigger<OnEnd>, mut tree: Tree| {
                                    tree.remove([terminate, backdrop]);
                                    tree.enable([root, back]);
                                });
                            tree.disable(terminate);
                            tree.remove(app);
                        },
                    );
                    Sequence::new(&mut tree)
                        .animate(
                            Animation::new(Opacity::new(0.0))
                                .targeting(root)
                                .start(0)
                                .finish(500),
                        )
                        .animate(
                            Animation::new(Opacity::new(0.0))
                                .targeting(back)
                                .start(0)
                                .finish(500),
                        )
                        .animate(
                            Animation::new(Opacity::new(1.0))
                                .targeting(backdrop)
                                .start(0)
                                .finish(200),
                        )
                        .animate(
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
                            .eased(Ease::INWARD),
                        )
                        .animate(
                            Animation::new(Location::new().xs(
                                0.pct().as_left().with(100.pct().as_right()),
                                0.pct().as_top().with(100.pct().as_bottom()),
                            ))
                            .targeting(backdrop)
                            .start(1000)
                            .finish(1500),
                        );
                };
            tree.subscribe(card_root, open_modal);
            card_root
        })
        .collect();
    tree.graft(back)
        .disable()
        .on_click(move |trigger: Trigger<OnClick>, mut tree: Tree| {
            tree.disable([back, root]);
            Sequence::new(&mut tree)
                .animate(
                    Animation::new(Opacity::new(0.0))
                        .start(0)
                        .finish(500)
                        .targeting(root),
                )
                .animate(
                    Animation::new(Opacity::new(0.0))
                        .start(0)
                        .finish(500)
                        .targeting(back),
                )
                .animate(
                    Animation::new(Opacity::new(1.0))
                        .start(500)
                        .finish(1000)
                        .targeting(home),
                )
                .animate(
                    Animation::new(Location::new().xs(
                        0.pct().as_left().with(100.pct().as_right()),
                        0.pct().as_top().with(100.pct().as_bottom()),
                    ))
                    .start(0)
                    .finish(1000)
                    .targeting(home)
                    .eased(Ease::EMPHASIS),
                )
                .animate(
                    Animation::new(Location::new().xs(
                        0.pct().as_left().with(100.pct().as_right()),
                        100.pct().as_top().with(200.pct().as_bottom()),
                    ))
                    .start(0)
                    .finish(1000)
                    .targeting(root)
                    .eased(Ease::EMPHASIS),
                )
                .end(move |trigger: Trigger<OnEnd>, mut tree: Tree| {
                    tree.remove([root, back]);
                    tree.enable(home);
                });
        });
    let _spacing = tree.branch(
        root,
        Leaf::sprout()
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                last.row().as_top().with(100.px().as_height()),
            ))
            .elevate(Elevation::abs(0)),
    );
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
