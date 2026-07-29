//! The landing section: what foliage is, and where to go next.
//!
//! This is the page that has to work on its own -- an unpublished library's site is mostly
//! a signpost, so the destination row matters more than anything else here.

use foliage::{Color, EcsExtension, Elevation, Entity, FontSize, Grid, GridExt, HorizontalAlignment, HrefLink, Location, OnClick, Opacity, Panel, Rounding, Sprout, Text, Tree, Trigger, VerticalAlignment};

use crate::site::{ACCENT, Column, cutout_badge, fade_in, role, space, type_scale};

const DOCS_HREF: &str = "https://eblack-leaf.github.io/foliage/api/foliage/index.html";
const BOOK_HREF: &str = "https://eblack-leaf.github.io/foliage/book/";
const REPO_HREF: &str = "https://github.com/eblack-leaf/foliage";

const CARD_H: i32 = 132;
const CARD_GAP: i32 = space::MD;
const BADGE: i32 = 26;

/// Three capability cards, each badged with a cutout shape, then the destination row.
pub fn build(tree: &mut Tree, slot: Entity) {
    let container = crate::site::shell::content_area(tree, slot);
    // full-bleed first screen, then the measured column scrolls up under it
    let hero = crate::site::hero::build(tree, container);
    let content = crate::site::shell::measured_column(tree, container, Some(hero));
    let mut column = Column::new(tree, content);

    column.heading(tree, "what it gives you");
    column.prose(
        tree,
        "Every value below is a component you write at runtime -- there is no separate \
         markup pass, and no rebuild step to see a change.",
    );

    let seq = column.sequence();
    for (i, (title, body, sides)) in [
        (
            "layout that resolves",
            "Locations are expressed per breakpoint and against a parent, not computed once.",
            6.0,
        ),
        (
            "motion that belongs",
            "Animations are sequenced, easable, and tied to the entity's own lifetime.",
            7.0,
        ),
        (
            "composites you can drive",
            "Buttons, cards, inputs and routers all react to plain component writes.",
            5.0,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let card = card(tree, content, &mut column, title, body);
        cutout_badge(
            tree,
            content,
            card,
            sides,
            BADGE,
            seq,
            i as u64 * crate::site::motion::STAGGER,
        );
    }

    column.heading(tree, "where to go");
    column.prose(
        tree,
        "The API reference is generated from the source. The book is the long-form \
         explanation. The examples are runnable.",
    );
    destinations(tree, content, &mut column);
}

/// One capability card: a surface with a title and a line of body copy.
fn card(
    tree: &mut Tree,
    parent: Entity,
    column: &mut Column,
    title: &str,
    body: &str,
) -> Entity {
    let surface = column.surface(tree, CARD_H, CARD_GAP);
    tree.branch(
        surface,
        Text::new(title)
            .size(FontSize::new(type_scale::TITLE))
            .color(Color::slate(role::ON_SURFACE))
            .at(Location::new().xs(
                space::MD
                    .px()
                    .as_left()
                    .with(100.pct().as_right().adjust(-(BADGE + space::LG))),
                space::MD.px().as_top().with(24.px().as_height()),
            ))
            .elevate(Elevation::up(2))
            .with((HorizontalAlignment::Left, VerticalAlignment::Middle)),
    );
    tree.branch(
        surface,
        Text::new(body)
            .size(FontSize::new(type_scale::BODY))
            .color(Color::slate(role::ON_SURFACE_VARIANT))
            .at(Location::new().xs(
                space::MD
                    .px()
                    .as_left()
                    .with(100.pct().as_right().adjust(-space::MD)),
                (space::MD + 28)
                    .px()
                    .as_top()
                    .with(100.pct().as_bottom().adjust(-space::MD)),
            ))
            .elevate(Elevation::up(2))
            .with((HorizontalAlignment::Left, VerticalAlignment::Top)),
    );
    let _ = parent;
    surface
}

/// The row this page exists for.
fn destinations(tree: &mut Tree, parent: Entity, column: &mut Column) {
    let seq = column.sequence();
    let row = column.surface_plain(tree, 56, space::MD);
    let entries = [("docs", DOCS_HREF), ("book", BOOK_HREF), ("github", REPO_HREF)];
    let width_pct = 100.0 / entries.len() as f32;
    for (i, (label, href)) in entries.into_iter().enumerate() {
        let left = width_pct * i as f32;
        let button = tree.branch(
            row,
            Panel::new()
                .color(Color::green(ACCENT))
                .rounding(Rounding::Full)
                .at(Location::new().xs(
                    left.pct()
                        .as_left()
                        .adjust(space::XS)
                        .with((left + width_pct).pct().as_right().adjust(-space::XS)),
                    0.px().as_top().with(100.pct().as_bottom()),
                ))
                .elevate(Elevation::up(2))
                // holds its own label, so it needs a grid for that child to resolve against
                .with((Opacity::new(0.0), Grid::new(1.col().gap(0), 1.row().gap(0)))),
        );
        let text = tree.branch(
            button,
            Text::new(label)
                .size(FontSize::new(type_scale::TITLE))
                .color(Color::gray(950))
                .at(Location::new().xs(
                    0.pct().as_left().with(100.pct().as_right()),
                    0.pct().as_top().with(100.pct().as_bottom()),
                ))
                .elevate(Elevation::up(1))
                .with((
                    HorizontalAlignment::Center,
                    VerticalAlignment::Middle,
                    Opacity::new(0.0),
                )),
        );
        let start = i as u64 * crate::site::motion::STAGGER;
        fade_in(tree, button, seq, start);
        fade_in(tree, text, seq, start);
        tree.on_click(button, move |_: Trigger<OnClick>, _: Tree| {
            HrefLink::new(href).navigate();
        });
    }
    let _ = parent;
}
