//! Placeholder sections. Each one is a real page in the shell -- title, prose, and the
//! rail entry that reaches it -- so the structure is walkable before the content exists.

use foliage::{Entity, Tree};

use crate::site::Column;

fn placeholder(tree: &mut Tree, slot: Entity, title: &str, summary: &str) {
    let container = crate::site::shell::content_area(tree, slot);
    let content = crate::site::shell::measured_column(tree, container, None);
    let mut column = Column::new(tree, content);
    column.display(tree, title);
    column.prose(tree, summary);
    column.prose(
        tree,
        "Not written yet. Demos here are inlined beside the prose that explains them, \
         rather than being a page of their own.",
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
