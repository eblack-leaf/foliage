//! Placeholder sections. Each one is a real page in the shell -- title, prose, and the
//! rail entry that reaches it -- so the structure is walkable before the content exists.

use foliage::Leaf;

use crate::site::copy::stub;
use crate::site::{Column, Grow};

fn placeholder(g: &mut Grow, slot: Leaf, title: &str, summary: &str) {
    let container = crate::site::shell::content_area(g.canopy, slot);
    let mut column = Column::new(g.canopy, container);
    column.display(g.canopy, title);
    column.lead(g.canopy, summary);
    column.prose(g.canopy, stub::NOT_WRITTEN);
    column.tail(g.canopy, crate::site::SCROLL_TAIL);
}

pub(crate) fn input(g: &mut Grow, slot: Leaf) {
    placeholder(g, slot, stub::INPUT_TITLE, stub::INPUT_LEAD);
}

pub(crate) fn text(g: &mut Grow, slot: Leaf) {
    placeholder(g, slot, stub::TEXT_TITLE, stub::TEXT_LEAD);
}
