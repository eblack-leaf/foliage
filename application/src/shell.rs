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
    Axes, Color, Corners, Divide, Elevation, Escape, FontSize, Grid, Grove, Grow, Leaf, Location,
    Palette, Panel, Place, Rounding, Scheme, Scroll, Side, Source, Stem, Text, anchor, bottom,
    center_x, center_y, content, left, right, top,
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

/// How tall the column's pinned header is.
const HEADER: f32 = 32.0;

/// How wide the column's scrollbar is, and how far in from its right edge it sits.
const BAR: f32 = 4.0;
const BAR_INSET: f32 = 8.0;

/// The shortest the scrollbar's thumb is allowed to get, so a very long column still leaves
/// something to see.
const THUMB: f32 = 24.0;

/// What the article says about each section of the rail. Rewritten as the reader moves, and of
/// deliberately different lengths, because the point is that the box follows the words.
const PROSE: [&str; SECTIONS as usize] = [
    "Width flows down and height flows up.",
    "A monospaced run knows how wide it wants to be before any layout has happened, so the pass \
     going down needs nothing measured.",
    "Only the pass coming up measures, and by then it has a width to measure against. Two passes, \
     no iteration.",
    "So a box can be as tall as its text turns out to be, and a container as tall as what is \
     grown under it.",
];

/// The parts of the page the app writes to again after growing it.
pub(crate) struct Shell {
    /// The page itself, which is what goes inert while the drawer is open.
    pub(crate) page: Leaf,
    /// The rail's entries, in order, which is what the marker is pointed at.
    pub(crate) entries: Vec<Leaf>,
    /// The bar that marks the current section.
    pub(crate) marker: Leaf,
    /// The article's prose, which is rewritten as the reader moves down the rail.
    pub(crate) prose: Leaf,
    /// The notice, until it is pruned.
    pub(crate) notice: Leaf,
    /// The round badge that opens the drawer.
    pub(crate) opener: Leaf,
    /// The reading column, which is what scrolls.
    pub(crate) column: Leaf,
    /// The column's own scrollbar: a pinned track, and a thumb the app sizes from what it reads.
    pub(crate) thumb: Leaf,
    /// The menu the last card opens, which floats out of the column rather than sitting in it.
    pub(crate) menu: Leaf,
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
            let entry = grove.branch(
                rail,
                Panel::new()
                    .color(Palette::Muted)
                    .rounding(ends(n))
                    .interactive()
                    .at(Location::new()
                        .xs(left(n.col()).right(n.col()), top(0.px()).bottom(100.pct()))
                        .md(left(1.col()).right(1.col()), top(n.row()).bottom(n.row()))),
            );
            // A label centred on its entry, and as large as the two characters make it. It is drawn
            // over the thing it labels, so it is `pass_through` for the same reason the marker is --
            // otherwise the entry would stop being tappable exactly where the label is.
            grove.branch(
                entry,
                Text::new(format!("0{n}"))
                    .color(Palette::Ink)
                    .font_size(FontSize::new().xs(13).md(15))
                    .pass_through()
                    .at(Location::new().xs(
                        center_x(50.pct()).width(content()),
                        center_y(50.pct()).height(content()),
                    )),
            );
            entry
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
    let column = grove.branch(
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

    // A header that stays at the top of the column while the reading slides under it. One
    // declaration says both halves of that: it takes none of the column's offset, and it gives the
    // column nothing to scroll to. Elevated, because what stays put has to be what is drawn over.
    let header = grove.branch(
        column,
        Panel::new()
            .color(Palette::Muted)
            .rounding(Rounding::Sm)
            .elevate(Elevation::up(3))
            .pinned()
            .at(Location::new().xs(
                left(0.px()).right(100.pct()),
                top(0.px()).height(HEADER.px()),
            )),
    );
    grove.branch(
        header,
        Text::new("reading")
            .color(Palette::Ink)
            .font_size(FontSize::new().xs(13))
            .pass_through()
            .at(Location::new().xs(
                left(GUTTER.px()).width(content()),
                center_y(50.pct()).height(content()),
            )),
    );

    // The scrollbar. Both parts are outside the movement -- the track says so, and the thumb
    // inherits it by being grown under something that did, because a pinned element's children
    // travel with the element and not with the content.
    let bar = grove.branch(
        column,
        Panel::new()
            .color(Palette::Muted)
            .rounding(Rounding::Full)
            .elevate(Elevation::up(3))
            .pinned()
            .at(Location::new().xs(
                right(100.pct() - BAR_INSET.px()).width(BAR.px()),
                top(HEADER.px() + GUTTER.px()).bottom(100.pct() - GUTTER.px()),
            )),
    );
    let thumb = grove.branch(
        bar,
        Panel::new()
            .color(Palette::Accent)
            .rounding(Rounding::Full)
            .pass_through()
            .at(thumb_at(0.0, THUMB)),
    );

    let article = grove.branch(
        column,
        Panel::new()
            .color(Palette::Raised)
            .rounding(Rounding::Lg)
            .at(Location::new().xs(
                // As wide as the content area allows, and never wider than a line anyone can read.
                // The ceiling is on the resolved extent, which is a different statement from the
                // width the layout offered -- so this stays one declaration at every breakpoint.
                center_x(50.pct()).width(100.pct()).at_most(MEASURE.px()),
                // As tall as what is grown inside it, plus the gutter it is inset by. The width is
                // decided above and the height falls out of it, which is the whole of width-down and
                // height-up said in one placement.
                //
                // Below the header rather than under it, because the header is opaque: what slides
                // under a pinned strip is the reading further down, and the top of it starts clear.
                top(1.row() + (HEADER + GUTTER).px()).height(content() + GUTTER.px()),
            )),
    );

    // The prose the article is measured from. It wraps at whatever width the column ended up
    // offering, and how many lines that takes is what the article's height turns out to be -- so a
    // resize, a breakpoint or a rewritten string all move the card and everything anchored below it.
    let prose = grove.branch(
        article,
        Text::new(PROSE[0])
            .color(Palette::Ink)
            .font_size(FontSize::new().xs(14).lg(16))
            .pass_through()
            .at(Location::new().xs(
                left(GUTTER.px()).right(100.pct() - GUTTER.px()),
                top(GUTTER.px()).height(content()),
            )),
    );

    // The slider. The knob is what takes drags along the track; a drag down it is not the knob's,
    // and reaches the column instead without either of them being told about the other.
    let track = grove.branch(
        column,
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
            column,
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

    // The menu the last card opens. Grown under the column and positioned by the card, which is
    // what an anchor is for -- and it **floats**, which is the whole of what that costs it
    // otherwise: without the mark the column would cut the options off at its own bottom edge, and
    // would gain a stretch of scroll range leading to options that are drawn over the reading
    // rather than sitting in it.
    //
    // Out of the column and no further, so it can never paint across the rail beside it.
    let menu = grove.branch(
        column,
        Panel::new()
            .color(Palette::Muted)
            .rounding(Rounding::Sm)
            .elevate(Elevation::up(6))
            .floats(Escape::Region)
            // Hidden until the card is tapped, which also keeps it out of the extent while it is
            // away -- two different reasons for the same absence, and neither relies on the other.
            .visible(false)
            .anchored(above)
            .at(Location::new().xs(
                left(anchor().left()).width(anchor().width()),
                top(anchor().bottom() + 4.px()).height(96.px()),
            )),
    );

    // A map. It **contains** both axes: a drag that runs it to an edge stops there rather than
    // carrying on into the column behind it, which is the difference between panning a map and
    // losing your place on the page.
    let pane = grove.branch(
        column,
        Stem::new()
            .scrolls(Scroll::new(Axes::Both).contain(Axes::Both))
            .anchored(above)
            .at(Location::new().xs(
                left(anchor().left()).width(anchor().width()),
                top(anchor().bottom() + GUTTER.px()).height(120.px()),
            )),
    );
    grove.branch(
        pane,
        Panel::new()
            .color(Palette::Raised)
            .rounding(Rounding::Sm)
            .at(Location::new().xs(
                left(0.px()).width(600.px()),
                top(0.px()).height(320.px()),
            )),
    );

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
        prose,
        notice,
        opener,
        column,
        thumb,
        menu,
        cards,
        track,
        knob,
        drawer: drawer(grove),
    }
}

/// Where the scrollbar's thumb sits along its track, and how much of it there is.
///
/// Both are read off the column rather than kept: how far through its range it is, and how much of
/// its content is on screen. foliage draws no scrollbar of its own, and this is why it does not
/// need to -- the three readings a scrollbar is are all readable.
pub(crate) fn thumb_at(travelled: f32, length: f32) -> Location {
    Location::new().xs(
        left(0.px()).right(100.pct()),
        top(travelled.px()).height(length.px()),
    )
}

/// How long the thumb is and how far down it sits, for a column `seen` tall whose content reaches
/// `extent` and which is `progress` of the way through its range.
pub(crate) fn thumb(seen: f32, extent: f32, progress: f32) -> (f32, f32) {
    // The floor first and the track's own length after it, in that order: a column shorter than the
    // floor has a thumb the length of the track rather than one longer than what holds it.
    let length = (seen / extent.max(1.0) * seen).max(THUMB).min(seen);
    (progress * (seen - length), length)
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

    for (button, word) in [(advance, "next"), (close, "close")] {
        grove.branch(
            button,
            Text::new(word)
                .color(Palette::Ink)
                .font_size(FontSize::new().xs(14))
                .pass_through()
                .at(Location::new().xs(
                    center_x(50.pct()).width(content()),
                    center_y(50.pct()).height(content()),
                )),
        );
    }

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

/// What the article says about section `n`.
pub(crate) fn prose(n: usize) -> &'static str {
    PROSE[n.min(PROSE.len() - 1)]
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
