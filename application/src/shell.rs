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
    Area, Axes, Boxed, Cap, Color, Corners, Divide, Elevation, Escape, Fit, FontSize, Grid, Grove,
    Grow, Icon, Image, Keypad, Leaf, Line, Location, Palette, Panel, Place, Point, Polygon,
    Rounding, Scheme, Scroll, Shape, Side, Source, Stem, Text, TextInput, anchor, bottom, center_x,
    center_y, content, left, right, top,
};

/// The space between tracks, and the rhythm every other measurement is stated in.
const GUTTER: f32 = 16.0;

/// The widest a column of prose is allowed to get, whatever the viewport does.
const MEASURE: f32 = 680.0;

/// The rail's height while it lies across the top, before there is width enough to stand it up.
const RAIL: f32 = 56.0;

/// How many sections the rail carries.
const SECTIONS: i32 = 4;

/// What the last card's menu offers, in the order it is drawn and read back.
pub(crate) const OPTIONS: [&str; 4] = ["copy", "paste", "open", "save"];

/// Where `open` goes, and what `save` asks the host for.
pub(crate) const REPOSITORY: &str = "https://github.com/eblack-leaf/foliage";
pub(crate) const ARCHIVE: &str =
    "https://github.com/eblack-leaf/foliage/archive/refs/heads/main.zip";

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

/// How much of the map the pane shows at once, and how much map there is to pan over. The map is
/// larger on both axes, which is what gives the pane somewhere to go in either direction.
const PANE: f32 = 160.0;
const MAP_WIDTH: f32 = 600.0;
const MAP_HEIGHT: f32 = 320.0;

/// How many blocks the map is divided into, and how wide the streets between them run.
const BLOCKS_ACROSS: i32 = 6;
const BLOCKS_DOWN: i32 = 4;
const STREET: f32 = 10.0;

/// How large the mark at the middle of the map is.
const PIN: f32 = 14.0;

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
    /// Its options, in [`OPTIONS`] order.
    pub(crate) options: Vec<Leaf>,
    /// The cards in the scrolling column.
    pub(crate) cards: Vec<Leaf>,
    /// The slider: a track, and the knob that takes drags along it.
    pub(crate) track: Leaf,
    pub(crate) knob: Leaf,
    /// The figure at the foot of the column.
    pub(crate) figure: Figure,
    pub(crate) drawer: Drawer,
}

/// The parts of the figure the app writes to again.
pub(crate) struct Figure {
    /// The legend's dot, reshaped as the tour moves.
    pub(crate) legend: Leaf,
    /// The caption, whose tinted range follows the section being read.
    pub(crate) caption: Leaf,
}

/// The drawer, which is grown outside the page so that disabling the page leaves it alone.
pub(crate) struct Drawer {
    pub(crate) sheet: Leaf,
    /// The grounds the fields are set into. A field draws no chrome of its own, so what a reader
    /// sees as the box is the app's, and it is what the focus mark is painted on.
    pub(crate) grounds: Vec<Leaf>,
    /// The fields themselves.
    pub(crate) fields: Vec<Leaf>,
    /// Steps focus to the next field, which is what `Tab` does now that there is a keyboard.
    pub(crate) advance: Leaf,
    pub(crate) close: Leaf,
    /// What the close button says, which follows whether the form has anything in it.
    pub(crate) verb: Leaf,
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
    // A mark rather than a word, so what the header does is legible in what is drawn: the strip
    // holds still while the reading slides under it, and it carries this with it, because the
    // children of a pinned element travel with the element.
    grove.branch(
        header,
        Panel::new()
            .color(Palette::Accent)
            .rounding(Rounding::Full)
            .pass_through()
            .at(Location::new().xs(
                left(GUTTER.px()).width(48.px()),
                center_y(50.pct()).height(4.px()),
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
            .grid(Grid::new().xs(1.columns(), (OPTIONS.len() as i32).rows()))
            .at(Location::new().xs(
                left(anchor().left()).width(anchor().width()),
                top(anchor().bottom() + 4.px()).height(96.px()),
            )),
    );

    // What the menu offers, and the whole of what the page does with the host: the first two reach
    // the clipboard and the last two hand a URL over. Each is a row of the menu's own grid, so the
    // menu is as tall as it was declared and the options divide it.
    let options = OPTIONS
        .iter()
        .enumerate()
        .map(|(row, name)| {
            let option = grove.branch(
                menu,
                Panel::new()
                    .color(Palette::Muted)
                    .interactive()
                    .at(Location::new().xs(
                        left(0.pct()).right(100.pct()),
                        top((row as i32 + 1).row()).bottom((row as i32 + 1).row()),
                    )),
            );
            grove.branch(
                option,
                Text::new(*name)
                    .color(Palette::Ink)
                    .font_size(FontSize::new().xs(12))
                    .pass_through()
                    .at(Location::new().xs(
                        left(GUTTER.px()).width(content()),
                        center_y(50.pct()).height(content()),
                    )),
            );
            option
        })
        .collect::<Vec<_>>();

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
                top(anchor().bottom() + GUTTER.px()).height(PANE.px()),
            )),
    );

    // The ground the pane is a window onto. Its own grid divides it into blocks and the streets
    // between them are that grid's gaps, so a pan reads as movement across somewhere rather than as
    // one rectangle sliding about behind a hole.
    let ground = grove.branch(
        pane,
        Panel::new()
            .color(Palette::Muted)
            .grid(Grid::new().xs(
                BLOCKS_ACROSS.columns().gap(STREET),
                BLOCKS_DOWN.rows().gap(STREET),
            ))
            .at(Location::new().xs(
                left(0.px()).width(MAP_WIDTH.px()),
                top(0.px()).height(MAP_HEIGHT.px()),
            )),
    );
    for down in 1..=BLOCKS_DOWN {
        for across in 1..=BLOCKS_ACROSS {
            grove.branch(
                ground,
                Panel::new()
                    .color(Palette::Raised)
                    .rounding(Rounding::Sm)
                    .at(Location::new().xs(
                        left(across.col()).right(across.col()),
                        top(down.row()).bottom(down.row()),
                    )),
            );
        }
    }

    // A mark at the middle of the map, which the pane does not show until it has been panned to.
    // Somewhere to arrive at is what tells the reader they moved rather than that something did.
    grove.branch(
        ground,
        Panel::new()
            .color(Palette::Accent)
            .rounding(Rounding::Full)
            .elevate(Elevation::up(1))
            .at(Location::new().xs(
                left(((MAP_WIDTH - PIN) / 2.0).px()).width(PIN.px()),
                top(((MAP_HEIGHT - PIN) / 2.0).px()).height(PIN.px()),
            )),
    );

    let figure = figure(grove, column, pane);

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
        options,
        cards,
        track,
        knob,
        figure,
        drawer: drawer(grove),
    }
}

/// How tall the figure is, and how much room its axes leave for the plot inside them.
const FIGURE: f32 = 200.0;
const AXIS: f32 = 28.0;

/// How much of the figure the header takes before the plot begins.
///
/// The plot is measured from here down rather than from the top of the card, so the series cannot
/// reach up into the legend -- a reading of 1.0 lands on the header's underside and no higher.
const HEAD: f32 = 52.0;

/// How large the legend's dot, the figure's mark and its thumbnail are.
const DOT: f32 = 12.0;
const MARK: f32 = 20.0;
const THUMBNAIL: f32 = 28.0;

/// The readings the series is drawn from, as fractions of the plot's height.
const SERIES: [f32; 6] = [0.15, 0.42, 0.30, 0.68, 0.55, 0.92];

/// A figure at the foot of the column: two rules for axes, a series drawn as strokes, a legend, a
/// mark and a thumbnail.
///
/// Every one of the renderers this slice adds, on one card, and each of them doing the thing it
/// exists for rather than standing in for a panel. It is also where the claim that a path needs
/// nothing of its own is tested: the series below is a chain of [`Line`]s between neighbouring
/// points, written here in a loop, and the engine has no concept of it.
fn figure(grove: &mut Grove, column: Leaf, above: Leaf) -> Figure {
    let card = grove.branch(
        column,
        Panel::new()
            .color(Palette::Raised)
            .rounding(Rounding::Md)
            .anchored(above)
            .at(Location::new().xs(
                left(anchor().left()).width(anchor().width()),
                top(anchor().bottom() + GUTTER.px()).height(FIGURE.px()),
            )),
    );

    // The two axes. A rule is two ends that share a coordinate, which is a box of no height until
    // the weight says otherwise -- so this is the case a stroke exists for rather than a thin panel.
    for (from, to) in [
        // Along the bottom.
        (
            (AXIS.px(), 100.pct() - AXIS.px()),
            (100.pct() - GUTTER.px(), 100.pct() - AXIS.px()),
        ),
        // Up the left, from under the header rather than from the top of the card.
        (
            (AXIS.px(), HEAD.px()),
            (AXIS.px(), 100.pct() - AXIS.px()),
        ),
    ] {
        grove.branch(
            card,
            Line::new()
                .weight(1.0)
                .color(Palette::Muted)
                .between(
                    Point::new(from.0.clone(), from.1.clone()),
                    Point::new(to.0.clone(), to.1.clone()),
                ),
        );
    }

    // The series. Each reading is a point in the grammar, so the whole figure stretches with the
    // column rather than being redrawn when it moves -- and the strokes between them follow.
    let plot = |n: usize| {
        let across = AXIS + 8.0;
        Point::new(
            across.px()
                + (100.pct() - (across + GUTTER).px()) * (n as f32 / (SERIES.len() - 1) as f32),
            (100.pct() - AXIS.px()) - (100.pct() - (AXIS + HEAD).px()) * SERIES[n],
        )
    };
    // A chain of strokes meeting end to end, with square ends rather than round ones. Two strokes
    // are two elements and therefore two blends, so a pixel both of them only partly cover is
    // painted twice and reads heavier than the shape they make between them. Round ends are that
    // case at its worst, because they put the same half-disc in the same place twice; square ends
    // abut, and what they leave open on the outside of a turn is a fraction of a pixel at the angles
    // a reading turns through. foliage has no `Polyline`, so the chain is written here.
    for n in 0..SERIES.len() - 1 {
        grove.branch(
            card,
            Line::new()
                .weight(2.0)
                .cap(Cap::Butt)
                .color(Palette::Accent)
                .elevate(Elevation::up(1))
                .between(plot(n), plot(n + 1)),
        );
    }

    // The legend's dot, which the tour reshapes as it moves: sides and rounding are numbers, so a
    // hexagon becomes a circle by passing through the shapes between them.
    let legend = grove.branch(
        card,
        Polygon::new()
            .sides(6.0)
            .color(Palette::Accent)
            .elevate(Elevation::up(1))
            .at(Location::new().xs(
                left(GUTTER.px()).width(DOT.px()),
                center_y(GUTTER.px() + (THUMBNAIL / 2.0).px()).height(DOT.px()),
            )),
    );

    // A mark, filled by the element rather than by the artwork -- so it repaints with the scheme
    // exactly as the label beside it does.
    let field = grove.icon(&mark_field(), MARK_FIELD, MARK_RANGE);
    grove.branch(
        card,
        Icon::new(field)
            .color(Palette::Ink)
            .elevate(Elevation::up(1))
            .at(Location::new().xs(
                left(GUTTER.px() + DOT.px() + 8.px()).width(MARK.px()),
                center_y(GUTTER.px() + (THUMBNAIL / 2.0).px()).height(MARK.px()),
            )),
    );

    // A thumbnail, in pixels the page makes for itself -- which is why it states its own size. A
    // picture read from a path or a URL is `image`, and states none: the decode answers that.
    let plate = grove.pixels(thumbnail(), Area::new(PLATE as f32, PLATE as f32));
    grove.branch(
        card,
        Image::new(plate)
            .fit(Fit::Crop)
            .rounding(Rounding::Sm)
            .elevate(Elevation::up(1))
            .at(Location::new().xs(
                right(100.pct() - GUTTER.px()).width(THUMBNAIL.px()),
                top(GUTTER.px()).height(THUMBNAIL.px()),
            )),
    );

    // The caption. Part of it is filled differently from the rest, over the run's own index space,
    // which is what a tint is -- and the tinted part follows the scheme like everything else.
    let caption = grove.branch(
        card,
        Text::new(CAPTION)
            .color(Palette::Muted)
            .font_size(FontSize::new().xs(11).md(12))
            .tint(0..7, Palette::Ink)
            .pass_through()
            .at(Location::new().xs(
                left((AXIS + 8.0).px()).right(100.pct() - GUTTER.px()),
                bottom(100.pct() - 6.px()).height(16.px()),
            )),
    );

    Figure { legend, caption }
}

/// What the figure's caption says. Part of it is tinted, so the run carries two fills.
const CAPTION: &str = "reading, over the last six intervals";

/// The shape the legend's dot takes for section `n`: fewer sides and rounder as the tour moves on,
/// ending on a true circle.
///
/// Three numbers, which is the whole reason a polygon is animatable without any of the machinery a
/// placement needs -- there is nothing to re-resolve, so there is nothing to keep consistent.
pub(crate) fn legend(n: usize) -> Shape {
    let steps = SECTIONS as f32 - 1.0;
    let at = (n as f32 / steps).clamp(0.0, 1.0);
    Shape {
        sides: 6.0 - 3.0 * at,
        rounding: at,
        rotation: at * core::f32::consts::FRAC_PI_4,
    }
}

/// Which part of the caption is picked out for section `n`, in characters of the value.
///
/// Indices into `CAPTION` rather than into what is drawn: a space leaves no glyph, so counting
/// glyphs would put every range after one somewhere other than the word it names.
pub(crate) fn emphasis(n: usize) -> (core::ops::Range<usize>, Palette) {
    let range = match n {
        // "reading"
        0 => 0..7,
        // "the last six"
        1 => 19..31,
        // "intervals"
        2 => 32..41,
        // the whole of it
        _ => 0..CAPTION.len(),
    };
    (range, Palette::Ink)
}

/// How wide the mark's distance field is, and how many texels its distance range spans.
const MARK_FIELD: u32 = 32;
const MARK_RANGE: f32 = 4.0;

/// A distance field for the mark: a ring, baked here rather than loaded.
///
/// A real one comes out of a tool that traces an outline. This is the same thing computed in closed
/// form, because a ring's distance is one subtraction -- which is enough to prove the pipeline draws
/// a field rather than a bitmap: it is 32 texels across and stays sharp at any size the layout gives
/// it.
///
/// The three colour channels are equal, which is a degenerate multi-channel field: the median of
/// three equal values is that value, so it reconstructs exactly and only gives up the corner
/// sharpness a true bake would keep. A ring has no corners.
fn mark_field() -> Vec<u8> {
    let side = MARK_FIELD as f32;
    let mut field = Vec::with_capacity((MARK_FIELD * MARK_FIELD * 4) as usize);
    for y in 0..MARK_FIELD {
        for x in 0..MARK_FIELD {
            let to = |v: u32| (v as f32 + 0.5) / side - 0.5;
            let (dx, dy) = (to(x), to(y));
            // Distance to the ring's own edge: outside the outer radius or inside the inner one is
            // out, and between them is in.
            let radius = (dx * dx + dy * dy).sqrt();
            let inside = (0.38 - radius).min(radius - 0.22);
            // Zero distance sits at half, and the baked range is what one texel of distance is
            // worth -- which is what the shader converts into a screen-space edge.
            let encoded = (inside * side / MARK_RANGE + 0.5).clamp(0.0, 1.0);
            let byte = (encoded * 255.0) as u8;
            field.extend_from_slice(&[byte, byte, byte, 255]);
        }
    }
    field
}

/// How wide the thumbnail is.
const PLATE: u32 = 24;

/// A picture, generated rather than decoded.
///
/// foliage decodes nothing -- what a PNG turns into is an app's business and an app's crate -- so
/// what the engine takes is pixels, and these are as good a set as any for proving the pipeline.
fn thumbnail() -> Vec<u8> {
    let mut pixels = Vec::with_capacity((PLATE * PLATE * 4) as usize);
    for y in 0..PLATE {
        for x in 0..PLATE {
            let across = (x * 255 / (PLATE - 1)) as u8;
            let down = (y * 255 / (PLATE - 1)) as u8;
            pixels.extend_from_slice(&[across, 90, down, 255]);
        }
    }
    pixels
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

    // The ground and the field are two elements because they are two things: the ground is a box
    // an app chose the colour and the rounding of, and the field is what can be typed into. A field
    // draws no chrome, which is what leaves the first of those entirely to the app.
    let grounds = (1..=2)
        .map(|n| {
            grove.branch(
                sheet,
                Panel::new()
                    .color(Palette::Muted)
                    .rounding(Rounding::Sm)
                    .at(Location::new().xs(
                        left(GUTTER.px()).right(100.pct() - GUTTER.px()),
                        top(n.row()).bottom(n.row()),
                    )),
            )
        })
        .collect::<Vec<_>>();

    // A keypad is the one thing a field says about a keyboard, and it is only ever a hint about
    // what is easy to type: the second field still takes whatever is pasted into it, and what a
    // number is allowed to be is this app's to check either way.
    let fields = grounds
        .iter()
        .zip([("name", Keypad::Text), ("phone", Keypad::Telephone)])
        .map(|(&ground, (placeholder, keypad))| {
            grove.branch(
                ground,
                TextInput::new()
                    .placeholder(placeholder)
                    .keypad(keypad)
                    .color(Palette::Ink)
                    .hint(Palette::Surface)
                    // Read against the ground, which the focus mark paints `Accent` -- and a caret
                    // is only ever drawn while the field is focused, so an accent one would be
                    // invisible for the whole of its life.
                    .caret(Palette::Ink)
                    .selection(Palette::Surface)
                    .font_size(FontSize::new().xs(14))
                    .at(Location::new().xs(
                        left(FIELD.px()).right(100.pct() - FIELD.px()),
                        top(0.px()).bottom(100.pct()),
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

    let labels = [advance, close].map(|button| {
        grove.branch(
            button,
            Text::new("")
                .color(Palette::Ink)
                .font_size(FontSize::new().xs(14))
                .pass_through()
                .at(Location::new().xs(
                    center_x(50.pct()).width(content()),
                    center_y(50.pct()).height(content()),
                )),
        )
    });
    grove.text(labels[0], "next");
    grove.text(labels[1], CLOSE);

    Drawer {
        sheet,
        grounds,
        fields,
        advance,
        close,
        verb: labels[1],
    }
}

/// How far a field is set into its ground, in logical pixels.
const FIELD: f32 = 10.0;

/// What the drawer's second button says while the form is empty, and once it is not.
pub(crate) const CLOSE: &str = "close";
pub(crate) const SAVE: &str = "save";

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
