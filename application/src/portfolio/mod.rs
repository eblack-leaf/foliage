mod music_player;

use crate::icons::IconHandles;
use crate::portfolio::music_player::MusicPlayer;
use foliage::{
    bevy_ecs, stack, Animation, Attachment, Button, Color, Ease, EcsExtension, Elevation, Event,
    Foliage, FontSize, Grid, GridExt, IconValue, Image, ImageView, InteractionListener, Keyring,
    Location, MemoryId, Named, OnClick, OnEnd, Opacity, Panel, Primary, Res, Rounding, Secondary,
    Stack, Stem, Text, Tree, Trigger,
};

impl Attachment for Portfolio {
    fn attach(foliage: &mut Foliage) {
        foliage.define(Portfolio::init);
    }
}
#[derive(Copy, Clone, Event)]
pub(crate) struct Portfolio {}
impl Portfolio {
    pub(crate) fn init(
        trigger: Trigger<Self>,
        mut tree: Tree,
        named: Res<Named>,
        keyring: Res<Keyring>,
    ) {
        let home = named.get("home");
        let row_size = 400;
        let root = tree.leaf((
            Grid::new(12.col().gap(24), row_size.px().gap(36)),
            Location::new().xs(
                0.pct().left().with(100.pct().right()),
                100.pct().top().with(200.pct().bottom()),
            ),
            InteractionListener::new().scroll(true).pass_through(true),
            Elevation::abs(0),
            Stem::none(),
        ));
        let seq = tree.sequence();
        tree.animate(
            Animation::new(Location::new().xs(
                0.pct().left().with(100.pct().right()),
                0.pct().top().with(100.pct().bottom()),
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
                0.pct().left().with(100.pct().right()),
                (-100).pct().top().with(0.pct().bottom()),
            ))
            .start(0)
            .finish(1000)
            .targeting(home)
            .eased(Ease::EMPHASIS)
            .during(seq),
        );
        let back = tree.leaf((
            Button::new(),
            Rounding::Full,
            IconValue(IconHandles::ArrowUp.value()),
            Primary(Color::gray(300)),
            Secondary(Color::gray(700)),
            Location::new().xs(
                50.pct().center_x().with(48.px().width()),
                12.px().top().with(48.px().height()),
            ),
            Elevation::abs(95),
            Stem::none(),
        ));
        let mut last = 0;
        let mut card_roots = vec![];
        let mut card_interactive = vec![];
        for (i, item) in ITEMS.iter().enumerate() {
            let card_shadow = tree.leaf((
                Panel::new(),
                Color::gray(500),
                Opacity::new(0.25),
                Location::new().xs(
                    1.col()
                        .left()
                        .adjust(12)
                        .with(12.col().right().adjust(12))
                        .max(450.0),
                    (i + 1)
                        .row()
                        .top()
                        .adjust(12)
                        .with((i + 1).row().bottom().adjust(12)),
                ),
                Elevation::up(0),
                Stem::some(root),
            ));
            let card_root = tree.leaf((
                Stem::some(root),
                Elevation::up(1),
                Panel::new(),
                Opacity::new(0.0),
                Color::gray(800),
                Grid::default(),
                Location::new().xs(
                    1.col().left().with(12.col().right()).max(450.0),
                    (i + 1).row().top().with((i + 1).row().bottom()),
                ),
            ));
            card_roots.push(card_root);
            let display = tree.leaf((
                Image::new(i as MemoryId, keyring.get(item.key)),
                ImageView::Crop,
                Location::new().xs(
                    1.col().left().with(1.col().right()),
                    0.pct().top().with(70.pct().bottom()),
                ),
                Elevation::up(1),
                Stem::some(card_root),
            ));
            let info = tree.leaf((
                Stem::some(card_root),
                Elevation::up(1),
                Panel::new(),
                Opacity::new(1.0),
                Color::gray(800),
                Grid::new(1.col().gap(8), 3.row().gap(8)),
                Location::new().xs(
                    1.col().left().with(1.col().right()),
                    70.pct().top().with(100.pct().bottom()),
                ),
            ));
            let title = tree.leaf((
                Text::new(&item.title),
                FontSize::new(16),
                Stem::some(info),
                Elevation::up(1),
                Location::new().xs(
                    1.col().left().with(1.col().right()),
                    1.row().top().with(1.row().bottom()),
                ),
                Color::gray(200),
            ));
            let desc = tree.leaf((
                Text::new(&item.desc),
                FontSize::new(14),
                Stem::some(info),
                Elevation::up(1),
                Location::new().xs(
                    1.col().left().with(1.col().right()),
                    2.row().top().with(3.row().bottom()),
                ),
                Color::gray(500),
            ));
            let launch = tree.leaf((
                Stem::some(info),
                Elevation::up(1),
                Button::new(),
                IconValue(IconHandles::Box.value()),
                Rounding::Full,
                Primary(Color::gray(900)),
                Secondary(Color::orange(800)),
                Location::new().xs(
                    100.pct().right().adjust(-8).with(44.px().width()),
                    100.pct().bottom().adjust(-8).with(44.px().height()),
                ),
            ));
            card_interactive.push((card_root, i, launch));
            card_interactive.push((card_root, i, display));
            last = i + 2;
        }
        for (r, i, ci) in card_interactive.clone() {
            tree.on_click(ci, move |trigger: Trigger<OnClick>, mut tree: Tree| {
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
                let backdrop = tree.leaf((
                    Location::new().xs(
                        stack().left().left().with(stack().right().right()),
                        stack().top().top().with(stack().bottom().bottom()),
                    ),
                    Stack::new(r),
                    Panel::new(),
                    Color::gray(800),
                    Opacity::new(0.0),
                    Elevation::abs(80),
                    Grid::default(),
                    Stem::none(),
                ));
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
                                .left()
                                .adjust(24)
                                .with(100.pct().right().adjust(-24))
                                .max(450.0),
                            0.pct()
                                .top()
                                .adjust(36)
                                .with(100.pct().bottom().adjust(-36)),
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
                        0.pct().left().with(100.pct().right()),
                        0.pct().top().with(100.pct().bottom()),
                    ))
                    .targeting(backdrop)
                    .start(1000)
                    .finish(1500)
                    .during(seq),
                );
                let terminate = tree.leaf((
                    Button::new(),
                    Rounding::Full,
                    IconValue(IconHandles::X.value()),
                    Primary(Color::gray(200)),
                    Secondary(Color::orange(800)),
                    Location::new().xs(
                        16.px().left().with(40.px().width()),
                        16.px().top().with(40.px().height()),
                    ),
                    Elevation::abs(95),
                    Stem::none(),
                ));
                let app = tree.leaf((
                    Stem::some(backdrop),
                    Location::new().xs(
                        0.pct().left().with(100.pct().right()),
                        0.pct().top().with(100.pct().bottom()),
                    ),
                ));
                match i {
                    0 => tree.send_to(MusicPlayer {}, app),
                    _ => println!("unimplemented"),
                }
                tree.on_click(
                    terminate,
                    move |trigger: Trigger<OnClick>, mut tree: Tree| {
                        let seq = tree.sequence();
                        tree.animate(
                            Animation::new(Opacity::new(0.0))
                                .targeting(app)
                                .during(seq)
                                .start(0)
                                .finish(500),
                        );
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
                                        .left()
                                        .adjust(24)
                                        .with(100.pct().right().adjust(-24))
                                        .max(450.0),
                                    0.pct()
                                        .top()
                                        .adjust(36)
                                        .with(100.pct().bottom().adjust(-36)),
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
                                stack().left().left().with(stack().right().right()),
                                stack().top().top().with(stack().bottom().bottom()),
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
                    0.pct().left().with(100.pct().right()),
                    0.pct().top().with(100.pct().bottom()),
                ))
                .start(0)
                .finish(1000)
                .targeting(home)
                .eased(Ease::EMPHASIS)
                .during(s),
            );
            tree.animate(
                Animation::new(Location::new().xs(
                    0.pct().left().with(100.pct().right()),
                    100.pct().top().with(200.pct().bottom()),
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
                0.pct().left().with(100.pct().right()),
                last.row().top().with(100.px().height()),
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
