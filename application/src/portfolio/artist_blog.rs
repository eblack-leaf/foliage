//! The "Artist Blog" portfolio item -- styled after the home page's language (lowercase
//! mono, green/orange accents on dark, sparse centered composition) and laid out per
//! breakpoint: `xs` is a stacked single column, `md`+ splits hero/feed left, comment right.
//! Exercises the library composites together: `Carousel` hero (real artwork images),
//! `Dropdown` sort, captions `Toggle`, scrolling `List` feed, `Numbered` `Pagination`
//! archive, multiline `TextInput` comment box (the hint-text regression surface).

use crate::icons::IconHandles;
use foliage::bevy_ecs;
use foliage::{
    Animation, AssetKey, Carousel, CarouselPages, Color, Component, Dropdown, EcsExtension,
    Elevation, Entity, FontSize, GlyphColors, GridExt, HorizontalAlignment, Image, ImageView,
    LineConstraint, List, ListItems, Location, Opacity, PageChanged, Pagination, PaginationMode,
    Panel, Query, Rounding, SelectionChanged, Sequence, Sprout, Text, TextInput, Toggled, Tree,
    Trigger, VerticalAlignment,
};

const ARCHIVE_PAGES: usize = 9;
const POSTS_PER_PAGE: usize = 12;
const CAPTIONS: [&str; 3] = ["gallery wall", "moonlight", "interface study"];

/// The feed's inputs, riding on the blog's root entity (the music player `Playing`
/// pattern) -- three composites' events each funnel their piece here, then rebuild the
/// feed from the whole.
#[derive(Component, Copy, Clone)]
struct BlogState {
    page: usize,
    oldest_first: bool,
    captions: bool,
}

fn posts_for(page: usize) -> Vec<(String, String)> {
    (0..POSTS_PER_PAGE)
        .map(|i| {
            let n = page * POSTS_PER_PAGE + i + 1;
            (
                format!("study no. {n}"),
                format!("2026-{:02}-{:02}", (n % 12) + 1, (n % 27) + 1),
            )
        })
        .collect()
}

fn swatch_color(n: usize) -> Color {
    match n % 3 {
        0 => Color::orange(500),
        1 => Color::green(300),
        _ => Color::gray(400),
    }
}

/// One feed's worth of rows from the current state; spawn and every rebuild share this.
/// Rows are px-split (fixed date column) so nothing wraps at narrow widths.
fn feed_items(state: BlogState) -> ListItems {
    let mut posts = posts_for(state.page);
    if state.oldest_first {
        posts.reverse();
    }
    ListItems::new(posts.len(), move |tree: &mut Tree, slot: Entity, i| {
        let (title, date) = posts[i].clone();
        tree.branch(
            slot,
            Panel::new()
                .rounding(Rounding::Sm)
                .color(swatch_color(state.page * POSTS_PER_PAGE + i))
                .at(Location::new().xs(
                    0.pct().as_left().with(32.px().as_width()),
                    50.pct().as_center_y().with(32.px().as_height()),
                ))
                .elevate(Elevation::up(1)),
        );
        tree.branch(
            slot,
            Text::new(title)
                .size(FontSize::new(16))
                .color(Color::gray(200))
                .at(Location::new().xs(
                    44.px().as_left().with(100.pct().as_right().adjust(-92)),
                    0.pct().as_top().with(100.pct().as_bottom()),
                ))
                .elevate(Elevation::up(1))
                .with(VerticalAlignment::Middle),
        );
        if state.captions {
            tree.branch(
                slot,
                Text::new(date)
                    .size(FontSize::new(14))
                    .color(Color::gray(500))
                    .at(Location::new().xs(
                        100.pct().as_right().with(88.px().as_width()),
                        0.pct().as_top().with(100.pct().as_bottom()),
                    ))
                    .elevate(Elevation::up(1))
                    .with((VerticalAlignment::Middle, HorizontalAlignment::Right)),
            );
        }
    })
}

pub(crate) fn build(tree: &mut Tree, app: Entity, artwork: [AssetKey; 3]) {
    Sequence::new(tree).animate(
        Animation::new(Opacity::new(1.0))
            .start(1000)
            .finish(1500)
            .targeting(app),
    );
    let state = BlogState {
        page: 0,
        oldest_first: false,
        captions: true,
    };
    tree.write_to(app, state);
    // centered mono title, ".blog" accented -- the home page's "foliage.rs" treatment;
    // centering also keeps it clear of the modal's close button at every width
    tree.branch(
        app,
        Text::new("artist.blog")
            .size(FontSize::new(24))
            .color(Color::gray(200))
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                1.row().as_top().with(1.row().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .with((
                HorizontalAlignment::Center,
                VerticalAlignment::Middle,
                GlyphColors::new().add(6..11, Color::green(300)),
            )),
    );
    tree.branch(
        app,
        Text::new("ink + gouache + pixels")
            .size(FontSize::new(16))
            .color(Color::gray(500))
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                2.row().as_top().with(2.row().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .with((
                HorizontalAlignment::Center,
                VerticalAlignment::Middle,
                GlyphColors::new()
                    .add(4..5, Color::orange(500))
                    .add(14..15, Color::orange(500)),
            )),
    );
    // hero: real artwork in a Carousel, caption strip over the top edge, dots embedded
    tree.branch(
        app,
        Carousel::new()
            .pages(CarouselPages::new(
                artwork.len(),
                move |tree: &mut Tree, slot: Entity, i| {
                    tree.branch(
                        slot,
                        Image::new(artwork[i])
                            .view(ImageView::Crop)
                            .at(Location::new().xs(
                                0.pct().as_left().with(100.pct().as_right()),
                                0.pct().as_top().with(100.pct().as_bottom()),
                            ))
                            .elevate(Elevation::up(1)),
                    );
                    let mut scrim = Color::gray(900);
                    scrim.set_alpha(0.65);
                    tree.branch(
                        slot,
                        Panel::new()
                            .color(scrim)
                            .at(Location::new().xs(
                                0.pct().as_left().with(100.pct().as_right()),
                                0.pct().as_top().with(28.px().as_height()),
                            ))
                            .elevate(Elevation::up(2)),
                    );
                    tree.branch(
                        slot,
                        Text::new(CAPTIONS[i])
                            .size(FontSize::new(14))
                            .color(Color::gray(200))
                            .at(Location::new().xs(
                                8.px().as_left().with(100.pct().as_right()),
                                0.pct().as_top().with(28.px().as_height()),
                            ))
                            .elevate(Elevation::up(3))
                            .with(VerticalAlignment::Middle),
                    );
                },
            ))
            .pagination(PaginationMode::Dots)
            .colors(Color::green(300), Color::gray(600))
            .at(Location::new()
                .xs(
                    8.px().as_left().with(100.pct().as_right().adjust(-8)),
                    3.row().as_top().with(6.row().as_bottom()),
                )
                .md(
                    1.col().as_left().with(7.col().as_right()),
                    3.row().as_top().with(7.row().as_bottom()),
                ))
            .elevate(Elevation::up(1)),
    );
    let feed = tree.branch(
        app,
        List::new()
            .items(feed_items(state))
            .row_height(48)
            .gap(8)
            .at(Location::new()
                .xs(
                    8.px().as_left().with(100.pct().as_right().adjust(-8)),
                    8.row().as_top().with(12.row().as_bottom()),
                )
                .md(
                    1.col().as_left().with(7.col().as_right()),
                    9.row().as_top().with(14.row().as_bottom()),
                ))
            .elevate(Elevation::up(1)),
    );
    let sort = tree.branch(
        app,
        Dropdown::new()
            .options([
                "newest",
                "oldest",
                "most liked",
                "alphabetical",
                "random",
                "by series",
                "archived",
            ])
            .chevron(IconHandles::Layers.into())
            .max_visible(5)
            .colors(Color::gray(200), Color::gray(900), Color::green(600))
            .at(Location::new()
                .xs(
                    8.px().as_left().with(45.pct().as_right()),
                    7.row().as_center_y().with(36.px().as_height()),
                )
                .md(
                    1.col().as_left().with(3.col().as_right()),
                    8.row().as_center_y().with(36.px().as_height()),
                ))
            .elevate(Elevation::up(1)),
    );
    tree.subscribe(
        sort,
        move |trigger: Trigger<SelectionChanged>, states: Query<&BlogState>, mut tree: Tree| {
            let mut state = *states.get(app).unwrap();
            state.oldest_first = trigger.event().index == 1;
            tree.write_to(app, state);
            tree.write_to(feed, feed_items(state));
        },
    );
    tree.branch(
        app,
        Text::new("captions")
            .size(FontSize::new(14))
            .color(Color::gray(500))
            .at(Location::new()
                .xs(
                    100.pct().as_right().adjust(-68).with(80.px().as_width()),
                    7.row().as_center_y().with(24.px().as_height()),
                )
                .md(
                    4.col().as_left().with(6.col().as_right()),
                    8.row().as_center_y().with(24.px().as_height()),
                ))
            .elevate(Elevation::up(1))
            .with((VerticalAlignment::Middle, HorizontalAlignment::Right)),
    );
    let captions = tree.branch(
        app,
        foliage::Toggle::new()
            .on(true)
            .colors(Color::green(500), Color::gray(700), Color::gray(200))
            .at(Location::new()
                .xs(
                    100.pct().as_right().adjust(-8).with(52.px().as_width()),
                    7.row().as_center_y().with(28.px().as_height()),
                )
                .md(
                    7.col().as_right().with(52.px().as_width()),
                    8.row().as_center_y().with(28.px().as_height()),
                ))
            .elevate(Elevation::up(1)),
    );
    tree.subscribe(
        captions,
        move |trigger: Trigger<Toggled>, states: Query<&BlogState>, mut tree: Tree| {
            let mut state = *states.get(app).unwrap();
            state.captions = trigger.event().on;
            tree.write_to(app, state);
            tree.write_to(feed, feed_items(state));
        },
    );
    // fixed-width centered strip: the step buttons' insets are fixed, so the pip window
    // must never split a column-relative remainder
    let archive = tree.branch(
        app,
        Pagination::new(ARCHIVE_PAGES)
            .mode(PaginationMode::Numbered)
            .step_icons(IconHandles::SkipLeft.into(), IconHandles::SkipRight.into())
            .step_colors(Color::gray(200), Color::gray(700))
            .colors(Color::green(300), Color::gray(600))
            .at(Location::new()
                .xs(
                    50.pct().as_center_x().with(320.px().as_width()),
                    13.row().as_top().with(13.row().as_bottom()),
                )
                .md(
                    4.col().as_center_x().with(360.px().as_width()),
                    15.row().as_top().with(15.row().as_bottom()),
                ))
            .elevate(Elevation::up(1)),
    );
    tree.subscribe(
        archive,
        move |trigger: Trigger<PageChanged>, states: Query<&BlogState>, mut tree: Tree| {
            let mut state = *states.get(app).unwrap();
            state.page = trigger.event().index;
            tree.write_to(app, state);
            tree.write_to(feed, feed_items(state));
        },
    );
    tree.branch(
        app,
        TextInput::new()
            .line_constraint(LineConstraint::Multiple)
            .hint_text("leave a comment...")
            .foreground(Color::gray(200))
            .background(Color::gray(900))
            .accent(Color::green(600))
            .font_size(FontSize::new(16))
            .at(Location::new()
                .xs(
                    8.px().as_left().with(100.pct().as_right().adjust(-8)),
                    14.row().as_top().with(16.row().as_bottom()),
                )
                .md(
                    8.col().as_left().with(12.col().as_right()),
                    3.row().as_top().with(7.row().as_bottom()),
                ))
            .elevate(Elevation::up(1)),
    );
}
