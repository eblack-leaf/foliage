//! The landing section: what foliage is, and where to go next.
//!
//! This is the page that has to work on its own -- an unpublished library's site is mostly
//! a signpost, so the destination row matters more than anything else here.

use foliage::{
    Color, EcsExtension, Elevation, Entity, FontSize, GridExt, HorizontalAlignment, Location,
    Sprout, Text, Tree, VerticalAlignment,
};

use crate::icons::IconHandles;
use crate::site::{
    Column, POLY_BUTTON_ROW_H, PolyButton, cutout_badge, poly_button, role, space, type_scale,
};

const DOCS_HREF: &str = "https://eblack-leaf.github.io/foliage/api/foliage/index.html";
const BOOK_HREF: &str = "https://eblack-leaf.github.io/foliage/book/";
const REPO_HREF: &str = "https://github.com/eblack-leaf/foliage";

const CARD_H: i32 = 132;
const CARD_GAP: i32 = space::MD;
const BADGE: i32 = 26;

/// Three capability cards, each badged with a cutout shape, then the destination row.
pub fn build(tree: &mut Tree, slot: Entity) {
    let container = crate::site::shell::content_area(tree, slot);
    let content = crate::site::shell::measured_column(tree, container, None);
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
        // parented to the full-width scroll container, not the measured column -- the badge
        // overhangs the card's edge and the column's own box would slice it
        cutout_badge(
            tree,
            container,
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
    destinations(tree, &mut column);
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
            .color(role::on_surface())
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
            .color(role::on_surface_variant())
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

/// The row this page exists for -- the same poly buttons the hero uses, so the two pages
/// speak with one vocabulary rather than a rounded rectangle here and a polygon there.
fn destinations(tree: &mut Tree, column: &mut Column) {
    let seq = column.sequence();
    let row = column.surface_plain(tree, POLY_BUTTON_ROW_H, space::MD);
    let entries = [
        PolyButton {
            label: "docs",
            icon: IconHandles::Code,
            href: DOCS_HREF,
            sides: 7.0,
            face: Color::amber(400),
        },
        PolyButton {
            label: "book",
            icon: IconHandles::BookOpen,
            href: BOOK_HREF,
            sides: 6.0,
            face: role::accent(),
        },
        PolyButton {
            label: "github",
            icon: IconHandles::Github,
            href: REPO_HREF,
            sides: 5.0,
            face: Color::rose(400),
        },
    ];
    let third = 100.0 / entries.len() as f32;
    for (i, spec) in entries.iter().enumerate() {
        let center = third * i as f32 + third / 2.0;
        poly_button(
            tree,
            row,
            spec,
            center,
            seq,
            i as u64 * crate::site::motion::STAGGER * 2,
        );
    }
}
