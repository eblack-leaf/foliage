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
use crate::site::copy::{self, CAPABILITIES, PLATE};
use crate::site::{
    Column, Grow, POLY_BUTTON_ROW_H, PolyButton, SCROLL_TAIL, cards, figure, motion, poly_button,
    role, space,
};

const DOCS_HREF: &str = "https://eblack-leaf.github.io/foliage/api/foliage/index.html";
const BOOK_HREF: &str = "https://eblack-leaf.github.io/foliage/book/";
const REPO_HREF: &str = "https://github.com/eblack-leaf/foliage";

/// The plate grows in both directions with the window: it runs on the figure measure, so past
/// the reading cap it keeps widening while the prose stops, and it gets taller to match. A
/// drawing that only widened would go letterboxed on a big screen, which wastes exactly the
/// room it was given.
const PLATE_H_XS: i32 = 210;
const PLATE_H_MD: i32 = 248;
const PLATE_H_LG: i32 = 340;

pub(crate) fn build(g: &mut Grow, slot: Leaf) {
    // straight into the scroll container -- elements carry the measure themselves, so the
    // side gutters are part of the same scrollable box as the text
    let container = crate::site::shell::content_area(g.canopy, slot);
    let mut column = Column::new(g.canopy, container);

    column.display(g.canopy, copy::headings::OVERVIEW);
    column.lead(g.canopy, copy::overview::LEAD);
    let plate = column.figure(g.canopy, (PLATE_H_XS, PLATE_H_MD, PLATE_H_LG), space::LG);
    let seq = column.sequence();
    figure::plate(g, plate, &PLATE, seq, motion::STAGGER * 2);

    // The lead is thematic -- it explains the name and the shape of the model, which is what
    // an opener should do and not what an overview is. This is the overview: what the thing
    // actually is, stated plainly, with the cards below breaking it into parts.
    column.heading(g.canopy, copy::headings::OVERVIEW_LIBRARY);
    column.prose(g.canopy, copy::overview::LIBRARY);
    cards::grid(g, &mut column, &CAPABILITIES);

    column.rule(g.canopy);
    column.heading(g.canopy, copy::headings::OVERVIEW_WHERE);
    column.prose(g.canopy, copy::overview::WHERE);
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
            label: copy::overview::DOCS,
            icon: IconHandles::Code,
            href: DOCS_HREF,
            sides: 7.0,
            face: Color::amber(400),
        },
        PolyButton {
            label: copy::overview::BOOK,
            icon: IconHandles::BookOpen,
            href: BOOK_HREF,
            sides: 6.0,
            face: role::accent(),
        },
        PolyButton {
            label: copy::overview::GITHUB,
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
