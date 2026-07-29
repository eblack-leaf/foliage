//! The landing section: what foliage is, and where to go next.
//!
//! This is the page that has to work on its own -- an unpublished library's site is mostly
//! a signpost, so the destination row matters more than anything else here.
//!
//! Laid out the way a reference page is: a lead paragraph, a blueprint plate, then a card
//! grid, then a ruled break before the destinations. The plate is the page's own
//! demonstration of its aesthetic -- measured lines and annotations around expressive shapes.

use foliage::{Color, Entity, Tree};

use crate::icons::IconHandles;
use crate::site::cards::CardSpec;
use crate::site::figure::{Label, Node, PlateSpec};
use crate::site::{
    Column, POLY_BUTTON_ROW_H, PolyButton, SCROLL_TAIL, cards, figure, motion, poly_button, role,
    space,
};

const DOCS_HREF: &str = "https://eblack-leaf.github.io/foliage/api/foliage/index.html";
const BOOK_HREF: &str = "https://eblack-leaf.github.io/foliage/book/";
const REPO_HREF: &str = "https://github.com/eblack-leaf/foliage";

/// The opener's figure. Nodes are placed where they compose and where their leaders have room
/// to run, not where a trend line would put them -- this is a drawing, not a plot.
const PLATE: PlateSpec = PlateSpec {
    // Both labelled nodes are at a turn in the path, with their label offset into the open
    // side of that turn -- above the first, below the last. Nothing is placed where the path
    // has to run through it at either breakpoint.
    //
    // The run starts at 0.16 rather than hard left: at `xs` the field is barely 256px, so a
    // node any further left had its leftmost point sitting on the tick at the "16" row.
    nodes: &[
        Node {
            at: (0.16, 0.66),
            sides: 6.0,
            size: 26,
            label: None,
        },
        Node {
            at: (0.31, 0.34),
            sides: 8.0,
            size: 38,
            label: Some(Label {
                text: "resolve 0.4ms",
                offset: (10, -36),
            }),
        },
        Node {
            at: (0.48, 0.72),
            sides: 5.0,
            size: 22,
            label: None,
        },
        Node {
            at: (0.65, 0.30),
            sides: 7.0,
            size: 26,
            label: None,
        },
        Node {
            at: (0.81, 0.74),
            sides: 6.0,
            size: 34,
            label: Some(Label {
                text: "reflow xs -> md",
                offset: (-30, 26),
            }),
        },
        Node {
            at: (0.94, 0.44),
            sides: 5.0,
            size: 22,
            label: None,
        },
    ],
    scale: &[
        (0.06, "32"),
        (0.20, ""),
        (0.34, "24"),
        (0.48, ""),
        (0.62, "16"),
        (0.76, ""),
        (0.90, "8"),
    ],
    caption: "fig. 01  shape resolve",
};
const PLATE_H_XS: i32 = 210;
const PLATE_H_MD: i32 = 248;

const CAPABILITIES: [CardSpec; 5] = [
    CardSpec {
        title: "a leaf is an entity",
        body: "Shape, text, position and behaviour are components on it. Changing one is a \
               write, and the next frame is already different.",
        icon: IconHandles::Box,
        sides: 6.0,
    },
    CardSpec {
        title: "layout that resolves",
        body: "Locations are written per breakpoint and against the stem, not computed once \
               and pinned.",
        icon: IconHandles::Grid,
        sides: 4.0,
    },
    CardSpec {
        title: "motion that belongs",
        body: "Animations are sequenced, easable, and tied to the lifetime of what they \
               animate.",
        icon: IconHandles::Repeat,
        sides: 7.0,
    },
    CardSpec {
        title: "composites you drive",
        body: "Buttons, cards, inputs and routers all react to plain component writes.",
        icon: IconHandles::Layers,
        sides: 5.0,
    },
    CardSpec {
        title: "one codebase",
        body: "Native, web and Android from the same source, with an ECS underneath all \
               three.",
        icon: IconHandles::Terminal,
        sides: 8.0,
    },
];

pub fn build(tree: &mut Tree, slot: Entity) {
    let container = crate::site::shell::content_area(tree, slot);
    let content = crate::site::shell::measured_column(tree, container, None);
    let mut column = Column::new(tree, content);

    column.display(tree, "overview");
    column.lead(
        tree,
        "Everything on screen is a leaf. You branch one under another, and the stem it keeps \
         to its parent is what its position, its clipping and its lifetime all resolve \
         against. That tree is the whole model, and the name.",
    );
    let plate = column.region(tree, (PLATE_H_XS, PLATE_H_MD, PLATE_H_MD), space::LG);
    figure::plate(tree, plate, &PLATE, column.sequence(), motion::STAGGER * 2);

    column.heading(tree, "what it gives you");
    cards::grid(tree, &mut column, &CAPABILITIES);

    column.rule(tree);
    column.heading(tree, "where to go");
    column.prose(
        tree,
        "The reference is generated from the source and is the exhaustive answer. The book is \
         the one that explains why. Everything in the repository runs -- every example is a \
         `cargo run` away.",
    );
    destinations(tree, &mut column);
    column.tail(tree, SCROLL_TAIL);
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
