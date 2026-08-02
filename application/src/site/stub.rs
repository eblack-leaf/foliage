//! Placeholder sections. Each one is a real page in the shell -- title, prose, and the
//! rail entry that reaches it -- so the structure is walkable before the content exists.

use foliage::Leaf;

use crate::site::{Column, Grow};

fn placeholder(g: &mut Grow, slot: Leaf, title: &str, summary: &str) {
    let container = crate::site::shell::content_area(g.canopy, slot);
    g.page.container = Some(container);
    let mut column = Column::new(g.canopy, container);
    column.display(g.canopy, title);
    column.lead(g.canopy, summary);
    column.prose(
        g.canopy,
        "Not written yet. Demos here are inlined beside the prose that explains them, \
         rather than being a page of their own.",
    );
    column.tail(g.canopy, crate::site::SCROLL_TAIL);
}

pub(crate) fn leaf(g: &mut Grow, slot: Leaf) {
    placeholder(
        g,
        slot,
        "leaf",
        "Stem, branch and stem -- what a thing on screen is, how one gets under another, and \
         what the link between them decides.",
    );
}

pub(crate) fn layout(g: &mut Grow, slot: Leaf) {
    placeholder(
        g,
        slot,
        "layout",
        "Location, Grid and Anchor -- where things sit, and what they sit against.",
    );
}

pub(crate) fn motion(g: &mut Grow, slot: Leaf) {
    placeholder(
        g,
        slot,
        "motion",
        "Sequenced animation: easing, staggering, looping, and shape morphs.",
    );
}

pub(crate) fn composites(g: &mut Grow, slot: Leaf) {
    placeholder(
        g,
        slot,
        "composites",
        "Button, Card, TextInput, Slider and Router -- driven by component writes.",
    );
}

pub(crate) fn text(g: &mut Grow, slot: Leaf) {
    placeholder(
        g,
        slot,
        "text",
        "A monospace grid: sizes per breakpoint, per-glyph color, and registered fonts.",
    );
}
