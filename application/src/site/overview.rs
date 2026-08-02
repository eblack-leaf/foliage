//! The landing section: what foliage is, and where to go next.
//!
//! This is the page that has to work on its own -- an unpublished library's site is mostly
//! a signpost, so the destination row matters more than anything else here.
//!
//! Laid out the way a reference page is, in two movements. The lead and the plate are the
//! opener -- what the name means and what the thing feels like. Then a heading, a plain
//! statement of what the library is, and cards breaking that into parts. A ruled break, and
//! the destinations.
//!
//! The split matters: the lead is thematic and the cards are specifics, and with nothing
//! between them the page went from a metaphor straight to a list with no sentence anywhere
//! saying what foliage actually is.
//!
//! The plate is the page's own demonstration of its aesthetic -- measured lines and
//! annotations around expressive shapes.

use foliage::{Color, Leaf};

use crate::icons::IconHandles;
use crate::site::cards::CardSpec;
use crate::site::figure::{Label, Node, PlateSpec, Side};
use crate::site::{
    Column, Grow, POLY_BUTTON_ROW_H, PolyButton, SCROLL_TAIL, cards, figure, motion, poly_button,
    role, space,
};

const DOCS_HREF: &str = "https://eblack-leaf.github.io/foliage/api/foliage/index.html";
const BOOK_HREF: &str = "https://eblack-leaf.github.io/foliage/book/";
const REPO_HREF: &str = "https://github.com/eblack-leaf/foliage";

/// The opener's figure. Nodes are placed where they compose and where their labels have room,
/// not where a trend line would put them -- this is a drawing, not a plot.
const PLATE: PlateSpec = PlateSpec {
    // Labels are centred on their node and clear of it, so the only placement decision left is
    // the *node's*: each labelled one is a turning point, with its label thrown into the open
    // side of the turn. The peak near the top takes its label above, the trough near the bottom
    // takes its under, and the path enters neither band at either breakpoint.
    //
    // The run starts at 0.14 rather than hard left: at `xs` the field is barely 300px, so a
    // node any further left had its leftmost point sitting on the tick at the "16" row.
    nodes: &[
        Node {
            at: (0.14, 0.62),
            sides: 6.0,
            size: 24,
            label: None,
        },
        Node {
            at: (0.28, 0.26),
            sides: 8.0,
            size: 38,
            label: Some(Label {
                text: "resolve 0.4ms",
                side: Side::Above,
            }),
        },
        Node {
            at: (0.45, 0.66),
            sides: 5.0,
            size: 26,
            label: None,
        },
        Node {
            at: (0.59, 0.40),
            sides: 7.0,
            size: 20,
            label: None,
        },
        Node {
            at: (0.75, 0.72),
            sides: 6.0,
            size: 34,
            label: Some(Label {
                text: "reflow xs -> md",
                side: Side::Under,
            }),
        },
        Node {
            at: (0.91, 0.34),
            sides: 5.0,
            size: 24,
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
/// The plate grows in both directions with the window: it runs on the figure measure, so past
/// the reading cap it keeps widening while the prose stops, and it gets taller to match. A
/// drawing that only widened would go letterboxed on a big screen, which wastes exactly the
/// room it was given.
const PLATE_H_XS: i32 = 210;
const PLATE_H_MD: i32 = 248;
const PLATE_H_LG: i32 = 340;

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

pub(crate) fn build(g: &mut Grow, slot: Leaf) {
    // straight into the scroll container -- elements carry the measure themselves, so the
    // side gutters are part of the same scrollable box as the text
    let container = crate::site::shell::content_area(g.canopy, slot);
    g.page.container = Some(container);
    let mut column = Column::new(g.canopy, container);

    column.display(g.canopy, "overview");
    column.lead(
        g.canopy,
        "Everything on screen is a leaf. You branch one under another, and the stem it keeps \
         to its parent is what its position, its clipping and its lifetime all resolve \
         against. That tree is the whole model, and the name.",
    );
    let plate = column.figure(g.canopy, (PLATE_H_XS, PLATE_H_MD, PLATE_H_LG), space::LG);
    let seq = column.sequence();
    figure::plate(g, plate, &PLATE, seq, motion::STAGGER * 2);

    // The lead is thematic -- it explains the name and the shape of the model, which is what
    // an opener should do and not what an overview is. This is the overview: what the thing
    // actually is, stated plainly, with the cards below breaking it into parts.
    column.heading(g.canopy, "the library");
    column.prose(
        g.canopy,
        "foliage is a UI framework for Rust. It renders through wgpu and runs the same source \
         on desktop, on the web and on Android. State lives in an ECS world rather than a \
         component tree, layout resolves per breakpoint against the stem, and there is no \
         markup language or build step standing between a change and the frame that shows it.",
    );
    cards::grid(g, &mut column, &CAPABILITIES);

    column.rule(g.canopy);
    column.heading(g.canopy, "where to go");
    column.prose(
        g.canopy,
        "The reference is generated from the source and is the exhaustive answer. The book is \
         the one that explains why. Everything in the repository runs -- every example is a \
         `cargo run` away.",
    );
    destinations(g, &mut column);
    column.tail(g.canopy, SCROLL_TAIL);
}

/// The row this page exists for -- the same poly buttons the hero uses, so the two pages
/// speak with one vocabulary rather than a rounded rectangle here and a polygon there.
fn destinations(g: &mut Grow, column: &mut Column) {
    let seq = column.sequence();
    let row = column.surface_plain(g.canopy, POLY_BUTTON_ROW_H, space::MD);
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
            g,
            row,
            spec,
            center,
            seq,
            i as u64 * crate::site::motion::STAGGER * 2,
        );
    }
}
