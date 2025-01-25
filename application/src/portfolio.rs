use crate::icons::IconHandles;
use foliage::{
    bevy_ecs, Animation, Attachment, Button, ButtonShape, Color, Ease, EcsExtension, Elevation,
    Event, Foliage, FontSize, Grid, GridExt, IconValue, Image, ImageView, InteractionListener,
    Keyring, Location, MemoryId, Named, Opacity, Panel, Primary, Res, Secondary, Stem, Text, Tree,
    Trigger,
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
            InteractionListener::new().scroll(true),
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
            .eased(Ease::ACCELERATE),
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
            .eased(Ease::ACCELERATE)
            .during(seq),
        );
        let mut last = 0;
        let mut card_roots = vec![];
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
                Color::gray(900),
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
                ButtonShape::Circle,
                Primary(Color::gray(500)),
                Secondary(Color::orange(800)),
                Location::new().xs(
                    100.pct().right().adjust(-8).with(44.px().width()),
                    100.pct().bottom().adjust(-8).with(44.px().height()),
                ),
            ));
            last = i + 2;
        }
        let _spacing = tree.leaf((
            Stem::some(root),
            Location::new().xs(
                0.pct().left().with(100.pct().right()),
                last.row().top().with(100.px().height()),
            ),
        ));
        for cr in card_roots {
            tree.animate(
                Animation::new(Opacity::new(1.0))
                    .start(500)
                    .finish(1000)
                    .targeting(cr)
                    .during(seq),
            );
        }
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
