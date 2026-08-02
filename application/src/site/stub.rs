//! Placeholder sections. Each one is a real page in the shell -- title, prose, and the
//! rail entry that reaches it -- so the structure is walkable before the content exists.

use foliage::{Entity, Tree};

use crate::site::Column;

fn placeholder(tree: &mut Tree, slot: Entity, title: &str, summary: &str) {
    let container = crate::site::shell::content_area(tree, slot);
    let mut column = Column::new(tree, container);
    column.display(tree, title);
    column.lead(tree, summary);
    column.prose(
        tree,
        "Not written yet. Demos here are inlined beside the prose that explains them, \
         rather than being a page of their own.",
    );
    column.tail(tree, crate::site::SCROLL_TAIL);
}

pub fn leaf(tree: &mut Tree, slot: Entity) {
    placeholder(
        tree,
        slot,
        "leaf",
        "Stem, branch and stem -- what a thing on screen is, how one gets under another, and \
         what the link between them decides.",
    );
}

pub fn layout(tree: &mut Tree, slot: Entity) {
    placeholder(
        tree,
        slot,
        "layout",
        "Location, Grid and Anchor -- where things sit, and what they sit against.",
    );
}

pub fn motion(tree: &mut Tree, slot: Entity) {
    placeholder(
        tree,
        slot,
        "motion",
        "Sequenced animation: easing, staggering, looping, and shape morphs.",
    );
}

pub fn composites(tree: &mut Tree, slot: Entity) {
    placeholder(
        tree,
        slot,
        "composites",
        "Button, Card, TextInput, Slider and Router -- driven by component writes.",
    );
}

pub fn text(tree: &mut Tree, slot: Entity) {
    placeholder(
        tree,
        slot,
        "text",
        "A monospace grid: sizes per breakpoint, per-glyph color, and registered fonts.",
    );
}
