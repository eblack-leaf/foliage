//! Placeholder sections. Each one is a real page in the shell -- title, prose, and the
//! rail entry that reaches it -- so the structure is walkable before the content exists.

use foliage::Leaf;

use crate::site::{Column, Grow};

fn placeholder(g: &mut Grow, slot: Leaf, title: &str, summary: &str) {
    let container = crate::site::shell::content_area(g.canopy, slot);
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

pub(crate) fn renderers(g: &mut Grow, slot: Leaf) {
    placeholder(
        g,
        slot,
        "renderers",
        "Panel, Icon, Image, Polygon and LineQuad -- the types with a pipeline of their own. \
         Text has a section to itself.",
    );
}

pub(crate) fn input(g: &mut Grow, slot: Leaf) {
    placeholder(
        g,
        slot,
        "input",
        "Hit shapes, click and drag, focus, and the one assembled control foliage ships: \
         TextInput.",
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
