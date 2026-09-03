//! The tree the site plants, and every placement in it.
//!
//! One page: a rail of sections, a marker that tracks whichever of them is current, a scrolling
//! reading column beside them, a notice that comes down after a while, and a drawer that opens over
//! the lot.
//!
//! A placement is a chain of breakpoints, each link stating both axes, so what an element looks
//! like at a given width is read in one place. A link that is never written falls back to the
//! nearest smaller one that was, and a chain that runs out fills the trunk.

use foliage::{
    Axes, Color, Corners, Divide, Elevation, Grid, Grove, Grow, Leaf, Location, Palette, Panel,
    Place, Rounding, Scheme, Side, Source, Stem, anchor, bottom, center_x, left, right, top,
};

/// The space between tracks, and the rhythm every other measurement is stated in.
const GUTTER: f32 = 16.0;

/// The widest a column of prose is allowed to get, whatever the viewport does.
const MEASURE: f32 = 680.0;

/// The rail's height while it lies across the top, before there is width enough to stand it up.
const RAIL: f32 = 56.0;

/// How many sections the rail carries.
const SECTIONS: i32 = 4;

/// How many cards sit under the article, which is what makes the column worth scrolling.
const CARDS: i32 = 6;

/// How tall one card is, and how far apart they sit.
const CARD: f32 = 72.0;

/// How wide the knob is, and so how much of the track it cannot reach.
const KNOB: f32 = 28.0;

/// The parts of the page the app writes to again after growing it.
pub(crate) struct Shell {
    /// The page itself, which is what goes inert while the drawer is open.
    pub(crate) page: Leaf,
    /// The rail's entries, in order, which is what the marker is pointed at.
    pub(crate) entries: Vec<Leaf>,
    /// The bar that marks the current section.
    pub(crate) marker: Leaf,
    /// The notice, until it is pruned.
    pub(crate) notice: Leaf,
    /// The round badge that opens the drawer.
    pub(crate) opener: Leaf,
    /// The cards in the scrolling column.
    pub(crate) cards: Vec<Leaf>,
    /// The slider: a track, and the knob that takes drags along it.
    pub(crate) track: Leaf,
    pub(crate) knob: Leaf,
    pub(crate) drawer: Drawer,
}

/// The drawer, which is grown outside the page so that disabling the page leaves it alone.
pub(crate) struct Drawer {
    pub(crate) sheet: Leaf,
    pub(crate) fields: Vec<Leaf>,
    /// Steps focus to the next field, which is what a keyboard will do when there is one.
    pub(crate) advance: Leaf,
    pub(crate) close: Leaf,
}

/// Plants the page.
pub(crate) fn grow(grove: &mut Grove) -> Shell {
    // No placement: an element that says nothing fills its trunk, and the trunk of a planted one is
    // the surface.
    let page = grove.plant(
        Panel::new()
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
        page,
        Panel::new()
            .color(Palette::Raised)
            .rounding(Rounding::Md)
            .at(Location::new()
                .xs(left(1.col()).right(4.col()), top(0.px()).height(RAIL.px()))
                .md(left(1.col()).right(3.col()), top(0.px()).bottom(100.pct())))
            // A grid of its own, nested inside the page's. It is the whole of what turns the
            // rail from a strip into a column: the entries address `col` and `row` and follow.
            .grid(
                Grid::new()
                    .xs(SECTIONS.columns().gap(GUTTER), 1.rows())
                    .md(1.columns(), SECTIONS.rows().gap(8.0)),
            ),
    );

    // A segmented control: the ends round outward and the joins stay square, so the row reads as
    // one shape rather than as a line of separate pills. Each entry receives, so a tap on one
    // selects it; nothing else in the rail does.
    let entries = (1..=SECTIONS)
        .map(|n| {
            grove.branch(
                rail,
                Panel::new()
                    .color(Palette::Muted)
                    .rounding(ends(n))
                    .interactive()
                    .at(Location::new()
                        .xs(left(n.col()).right(n.col()), top(0.px()).bottom(100.pct()))
                        .md(left(1.col()).right(1.col()), top(n.row()).bottom(n.row()))),
            )
        })
        .collect::<Vec<_>>();

    // Branched under the page rather than under the rail, because it belongs to the page and not
    // to any one entry. Its position does not depend on that: an anchor's edges are absolute
    // positions, so the trunk decides what takes it down and what it sits among, and nothing else.
    //
    // It is drawn over whichever entry is current, so it is exactly the case `pass_through` is
    // for: without the mark it would be the top of the stack there, and the entry under it would
    // stop being tappable the moment it became the selected one.
    let marker = grove.branch(
        page,
        Panel::new()
            .color(Palette::Accent)
            .rounding(Rounding::Full)
            .elevate(Elevation::up(1))
            .pass_through()
            .anchored(entries[0])
            .at(Location::new()
                .xs(
                    left(anchor().left()).width(anchor().width()),
                    bottom(anchor().bottom()).height(2.px()),
                )
                .md(
                    left(anchor().left()).width(2.px()),
                    top(anchor().top()).bottom(anchor().bottom()),
                )),
    );

    // Grown beside the rail rather than inside it, so it draws over the rail's own elevation
    // instead of accumulating from it -- and anchored back, so it goes on addressing the rail's
    // grid exactly as an entry does. Leaving a trunk costs the vocabulary nothing.
    //
    // Round, and its hit area is round with it, so it does not take presses in the corners of a
    // box it never draws.
    let opener = grove.branch(
        page,
        Panel::new()
            .color(Palette::Accent)
            .rounding(Rounding::Full)
            .elevate(Elevation::up(2))
            .interactive()
            .round_hit_area()
            .anchored(rail)
            .at(Location::new()
                .xs(
                    left(anchor().col(SECTIONS)).width(28.px()),
                    top(anchor().top()).height(28.px()),
                )
                .md(
                    left(anchor().right()).width(28.px()),
                    top(anchor().row(SECTIONS)).height(28.px()),
                )),
    );

    // The reading column scrolls, and it scrolls because it said so: dividing it with a grid says
    // nothing about scrolling. A drag anywhere inside it moves it, including on the cards, which
    // asked only to be tappable.
    let content = grove.branch(
        page,
        Stem::new()
            .scrolls(Axes::Vertical)
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
        Panel::new()
            .color(Palette::Raised)
            .rounding(Rounding::Lg)
            .at(Location::new().xs(
                // As wide as the content area allows, and never wider than a line anyone can read.
                // The ceiling is on the resolved extent, which is a different statement from the
                // width the layout offered -- so this stays one declaration at every breakpoint.
                center_x(50.pct()).width(100.pct()).at_most(MEASURE.px()),
                top(1.row()).bottom(2.row()),
            )),
    );

    // The slider. The knob is what takes drags along the track; a drag down it is not the knob's,
    // and reaches the column instead without either of them being told about the other.
    let track = grove.branch(
        content,
        Panel::new()
            .color(Palette::Muted)
            .rounding(Rounding::Full)
            .anchored(article)
            .at(Location::new().xs(
                left(anchor().left()).width(anchor().width()),
                top(anchor().bottom() + GUTTER.px()).height(KNOB.px()),
            )),
    );

    let knob = grove.branch(
        track,
        Panel::new()
            .color(Palette::Accent)
            .rounding(Rounding::Full)
            .interactive()
            .round_hit_area()
            .drags(Axes::Horizontal)
            .at(knob_at(0.0)),
    );

    // Sits under the slider wherever the slider ended up, which is what an anchor is for. Each card
    // hangs off the one before it, so the column is as long as the cards make it and the region's
    // extent follows.
    let mut cards = Vec::new();
    let mut above = track;
    for _ in 0..CARDS {
        above = grove.branch(
            content,
            Panel::new()
                .color(Palette::Raised)
                .rounding(Rounding::Sm)
                .interactive()
                .anchored(above)
                .at(Location::new().xs(
                    left(anchor().left()).width(anchor().width()),
                    top(anchor().bottom() + GUTTER.px()).height(CARD.px()),
                )),
        );
        cards.push(above);
    }

    let notice = grove.branch(
        page,
        Panel::new()
            .color(Palette::Raised)
            .rounding(Rounding::Md)
            // Above the page it covers, and above it by saying so rather than by being grown late.
            .elevate(Elevation::up(4))
            .at(Location::new().xs(
                center_x(50.pct())
                    .width(100.pct() - GUTTER.px() * 2.0)
                    .at_most(420.px()),
                bottom(100.pct() - GUTTER.px()).height(48.px()),
            )),
    );

    Shell {
        page,
        entries,
        marker,
        notice,
        opener,
        cards,
        track,
        knob,
        drawer: drawer(grove),
    }
}

/// The drawer, planted at top level rather than under the page.
///
/// That is what lets the page be disabled as one write when the drawer opens: everything under the
/// page goes inert and swallows, with no scrim to arrange, and the drawer is not under the page so
/// it stays live. It declares itself a focus scope, so stepping through it cycles inside it instead
/// of walking off into the page behind.
fn drawer(grove: &mut Grove) -> Drawer {
    let sheet = grove.plant(
        Panel::new()
            .color(Palette::Raised)
            .rounding(Rounding::Lg)
            .elevate(Elevation::up(8))
            .focus_scope()
            // Hidden while it is away rather than merely off the bottom of the surface, so nothing
            // in it can be reached by focus while it is closed. Showing it and sliding it in are
            // separate statements, and the second is what the reader sees.
            .visible(false)
            .grid(Grid::new().xs(1.columns(), 4.rows().gap(GUTTER)))
            .at(sheet_at(false)),
    );

    let fields = (1..=2)
        .map(|n| {
            grove.branch(
                sheet,
                Panel::new()
                    .color(Palette::Muted)
                    .rounding(Rounding::Sm)
                    .interactive()
                    .at(Location::new().xs(
                        left(GUTTER.px()).right(100.pct() - GUTTER.px()),
                        top(n.row()).bottom(n.row()),
                    )),
            )
        })
        .collect::<Vec<_>>();

    let advance = grove.branch(
        sheet,
        Panel::new()
            .color(Palette::Muted)
            .rounding(Rounding::Full)
            .interactive()
            // Last in the cycle whatever the layout does with it, which is what an override is for.
            .focus_order(1)
            .at(Location::new().xs(
                left(GUTTER.px()).width(120.px()),
                top(3.row()).bottom(3.row()),
            )),
    );

    let close = grove.branch(
        sheet,
        Panel::new()
            .color(Palette::Accent)
            .rounding(Rounding::Full)
            .interactive()
            .at(Location::new().xs(
                right(100.pct() - GUTTER.px()).width(120.px()),
                top(3.row()).bottom(3.row()),
            )),
    );

    Drawer {
        sheet,
        fields,
        advance,
        close,
    }
}

/// Where the drawer sits: over the lower half of the page, or below the surface entirely.
///
/// Two placements and nothing between them. What puts the sheet between them is the motion, and it
/// resolves both of these every frame -- so a resize part way through moves the drawer with it and
/// it still arrives flush against the bottom.
pub(crate) fn sheet_at(open: bool) -> Location {
    match open {
        true => Location::new().xs(
            left(0.px()).right(100.pct()),
            top(45.pct()).bottom(100.pct()),
        ),
        false => Location::new().xs(
            left(0.px()).right(100.pct()),
            top(100.pct()).height(55.pct()),
        ),
    }
}

/// The site's scheme, `dim` of the way toward the ground it takes while the drawer is over it.
///
/// A scheme is the app's own value, and foliage has no concept of one -- so there is no `Motion`
/// that could move it, and there should not be. Moving it is what a `tween` is for: the engine's
/// clock and easing, handed to a value it does not know, with the write staying on this side.
pub(crate) fn scheme(dim: f32) -> Scheme {
    let base = Scheme::new();
    let toward = 1.0 - dim.clamp(0.0, 1.0) * 0.6;
    let shaded = |role: Palette| {
        let color = base.color(role);
        Color::rgb(
            color.red * toward,
            color.green * toward,
            color.blue * toward,
        )
    };
    base.set(Palette::Accent, Color::rgb(0.42, 0.68, 0.96))
        .set(Palette::Surface, shaded(Palette::Surface))
        .set(Palette::Raised, shaded(Palette::Raised))
}

/// Where the knob sits along its track, `travelled` pixels from the near end.
pub(crate) fn knob_at(travelled: f32) -> Location {
    Location::new().xs(
        left(travelled.px()).width(KNOB.px()),
        top(0.px()).height(KNOB.px()),
    )
}

/// How far the knob can travel along a track of this width.
pub(crate) fn knob_room(track: f32) -> f32 {
    (track - KNOB).max(0.0)
}

/// The rounding for entry `n` of the rail: the first and last round outward, the rest stay square.
fn ends(n: i32) -> Corners {
    let mut corners = Corners::none();
    if n == 1 {
        corners = corners.side(Side::Left, Rounding::Sm);
    }
    if n == SECTIONS {
        corners = corners.side(Side::Right, Rounding::Sm);
    }
    corners
}
