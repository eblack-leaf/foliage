//! The tree the site plants, and every placement in it.
//!
//! One page: a rail of sections, a marker that tracks whichever of them is current, a reading
//! column beside them, and a notice that comes down after a while. It is all `Stem`, because
//! nothing draws yet -- what is being exercised is the grammar, not the pixels.
//!
//! A placement is a chain of breakpoints, each link stating both axes, so what an element looks
//! like at a given width is read in one place. A link that is never written falls back to the
//! nearest smaller one that was, and a chain that runs out fills the parent.

use foliage::{
    Divide, Grid, Grove, Grow, Leaf, Location, Place, Source, Stem, anchor, bottom, center_x, left,
    top,
};

/// The space between tracks, and the rhythm every other measurement is stated in.
const GUTTER: f32 = 16.0;

/// The widest a column of prose is allowed to get, whatever the viewport does.
const MEASURE: f32 = 680.0;

/// The rail's height while it lies across the top, before there is width enough to stand it up.
const RAIL: f32 = 56.0;

/// How many sections the rail carries.
const SECTIONS: usize = 4;

/// The parts of the page the app writes to again after growing it.
pub(crate) struct Shell {
    /// The rail's entries, in order, which is what the marker is pointed at.
    pub(crate) entries: Vec<Leaf>,
    /// The bar that marks the current section.
    pub(crate) marker: Leaf,
    /// The notice, until it is pruned.
    pub(crate) notice: Leaf,
}

/// Plants the page.
pub(crate) fn grow(grove: &mut Grove) -> Shell {
    // No placement: an element that says nothing fills its parent, and the parent of a planted one
    // is the surface.
    let shell = grove.plant(
        Stem::new()
            // Four columns on a phone, twelve once there is room for a rail beside the content.
            // The children address columns rather than pixels, so widening the grid moves them
            // and none of them is rewritten.
            .grid(
                Grid::new()
                    .xs(4.columns().gap(GUTTER), 1.rows())
                    .md(12.columns().gap(GUTTER), 1.rows()),
            ),
    );

    let rail = grove.branch(
        shell,
        Stem::new()
            .at(Location::new()
                .xs(left(1.col()).right(4.col()), top(0.px()).height(RAIL.px()))
                .md(left(1.col()).right(3.col()), top(0.px()).bottom(100.pct())))
            // A grid of its own, nested inside the shell's. It is the whole of what turns the
            // rail from a strip into a column: the entries address `col` and `row` and follow.
            .grid(
                Grid::new()
                    .xs(SECTIONS.columns().gap(GUTTER), 1.rows())
                    .md(1.columns(), SECTIONS.rows().gap(8.0)),
            ),
    );

    let entries = (1..=SECTIONS as i32)
        .map(|n| {
            grove.branch(
                rail,
                Stem::new().at(Location::new()
                    .xs(left(n.col()).right(n.col()), top(0.px()).bottom(100.pct()))
                    .md(left(1.col()).right(1.col()), top(n.row()).bottom(n.row()))),
            )
        })
        .collect::<Vec<_>>();

    // Branched under the shell rather than under the rail, because it belongs to the page and not
    // to any one entry. Its position does not depend on that: an anchor's edges are absolute
    // positions, so the trunk decides what takes it down and what it sits among, and nothing else.
    let marker = grove.branch(
        shell,
        Stem::new().anchored(entries[0]).at(Location::new()
            .xs(
                left(anchor().left()).width(anchor().width()),
                bottom(anchor().bottom()).height(2.px()),
            )
            .md(
                left(anchor().left()).width(2.px()),
                top(anchor().top()).bottom(anchor().bottom()),
            )),
    );

    let content = grove.branch(
        shell,
        Stem::new()
            .at(Location::new()
                .xs(
                    left(1.col()).right(4.col()),
                    top(RAIL.px() + GUTTER.px()).bottom(100.pct()),
                )
                .md(left(4.col()).right(12.col()), top(0.px()).bottom(100.pct())))
            .grid(Grid::new().xs(1.columns(), 3.rows().gap(GUTTER))),
    );

    let article = grove.branch(
        content,
        Stem::new().at(Location::new().xs(
            // As wide as the content area allows, and never wider than a line anyone can read.
            // The ceiling is on the resolved extent, which is a different statement from the
            // width the layout offered -- so this stays one declaration at every breakpoint.
            center_x(50.pct()).width(100.pct()).at_most(MEASURE.px()),
            top(1.row()).bottom(2.row()),
        )),
    );

    // Sits under the article wherever the article ended up, which is what an anchor is for: the
    // article's height comes from the content grid, so a fixed offset would be a guess.
    //
    // Its name is not kept. A `Leaf` is worth holding for as long as there is something left to
    // say to it, and there is nothing left to say to this one.
    grove.branch(
        content,
        Stem::new().anchored(article).at(Location::new().xs(
            left(anchor().left()).width(anchor().width()),
            top(anchor().bottom() + GUTTER.px()).height(40.px()),
        )),
    );

    let notice = grove.branch(
        shell,
        Stem::new().at(Location::new().xs(
            center_x(50.pct())
                .width(100.pct() - GUTTER.px() * 2.0)
                .at_most(420.px()),
            bottom(100.pct() - GUTTER.px()).height(48.px()),
        )),
    );

    Shell {
        entries,
        marker,
        notice,
    }
}
