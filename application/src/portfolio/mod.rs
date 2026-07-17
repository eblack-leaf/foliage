pub(crate) mod artist_blog;
pub(crate) mod music_player;

use crate::icons::IconHandles;
use crate::widgets::{icon_button, Launch, ProjectCard};
use foliage::{
    Animation, Closed, Color, Ease, EcsExtension, Elevation, Entity, Grid, GridExt, Keyring,
    Leaf, Location, Modal, OnClick, OnEnd, Opacity, Panel, Res, Rounding, Sequence, Sprout, Tree,
    Trigger,
};

/// The panel each modal's content lives in, plus injecting that content -- isolated here so
/// the `music_player`-vs-`artist_blog` branch reads as one named step instead of sitting
/// mid-closure.
/// `album_cover` is pre-resolved from the `Keyring` at the call site rather than threading a
/// whole `Keyring` reference into `Modal`'s content closure, which needs to be `'static`.
fn spawn_modal_content(
    tree: &mut Tree,
    slot: Entity,
    i: usize,
    album_cover: Option<foliage::AssetKey>,
    artwork: [foliage::AssetKey; 3],
) -> Entity {
    let app_base = Leaf::sprout()
        .at(Location::new().xs(
            0.pct().as_left().with(100.pct().as_right()),
            0.pct().as_top().with(100.pct().as_bottom()),
        ))
        .elevate(Elevation::up(1))
        .with(Opacity::new(0.0));
    let app = match i {
        0 => tree.branch(
            slot,
            app_base.with((
                Panel::default(),
                Grid::new(12.col().gap(8), 40.px().gap(8)),
                Color::gray(900),
            )),
        ),
        _ => tree.branch(
            slot,
            app_base.with(Grid::new(12.col().gap(8), 40.px().gap(8))),
        ),
    };
    match i {
        0 => music_player::build(tree, app, album_cover.expect("album cover key")),
        _ => artist_blog::build(tree, app, artwork),
    }
    app
}

/// `root`/`back` fade out together whenever a card opens into a modal -- separate from
/// `Modal`'s own open animation since `root`/`back` are page-level, not the modal's concern.
fn fade_page_chrome_out(tree: &mut Tree, root: Entity, back: Entity) {
    Sequence::new(tree)
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
        );
}

/// The mirror of `fade_page_chrome_out`, timed to land near the end of `Modal`'s own close
/// animation, re-enabling both once fully faded back in.
fn fade_page_chrome_in(tree: &mut Tree, root: Entity, back: Entity) {
    Sequence::new(tree)
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
        .end(move |_: Trigger<OnEnd>, mut tree: Tree| {
            tree.enable([root, back]);
        });
}

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
        icon_button(IconHandles::ArrowUp, Color::gray(300), Color::gray(700))
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
            let album_cover = (i == 0).then(|| keyring.get("album-cover"));
            let artwork = [
                keyring.get("artist-blog"),
                keyring.get("album-cover"),
                keyring.get("music-player"),
            ];
            let open_modal = move |trigger: Trigger<Launch>, mut tree: Tree| {
                tree.disable([root, back]);
                fade_page_chrome_out(&mut tree, root, back);
                let modal = tree.leaf(
                    Modal::new()
                        .anchor_to(card_root)
                        .content(move |tree: &mut Tree, slot: Entity| {
                            spawn_modal_content(tree, slot, i, album_cover, artwork)
                        })
                        .close_icon(IconHandles::X.into())
                        .colors(Color::gray(800), Color::gray(200), Color::orange(800))
                        .elevate(Elevation::abs(50)),
                );
                tree.subscribe(modal, move |_: Trigger<Closed>, mut tree: Tree| {
                    fade_page_chrome_in(&mut tree, root, back);
                });
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
