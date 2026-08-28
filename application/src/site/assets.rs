//! The `assets` section.
//!
//! Three boards, two of which stand in for something rather than reproducing it -- and say so
//! in the prose above them.
//!
//! The loader collapses its own interesting moment. A key is minted before the bytes exist, and
//! for an asset compiled into the binary the bytes then arrive in the same frame, so there is
//! no live gap to watch; which of the two sources a build uses is settled at compile time, so
//! there is no live choice either. Neither of those makes the idea unshowable -- it makes the
//! *timing* the part to stand in for. The key board holds the two states apart on a step so the
//! first can be looked at; the sources board draws one path at a time so the two you cannot
//! have at once are still visibly two. Everything else in both is real: real key, real bytes,
//! real calls named on the readout.
//!
//! Where the line is: what a board stands in for is always something the platform or the clock
//! decides, never something the library does. A board that faked a *call* would be lying.

use foliage::{
    AssetSource, Forest, Color, Elevation, FontSize, Grid, GridExt, Grows, HorizontalAlignment,
    Icon, IconId, Image, ImageView, Leaf, Location, Panel, Rounding, Sprout, Text,
    VerticalAlignment,
};

use crate::icons::IconHandles;
use crate::site::blueprint::{self, Blueprint};
use crate::site::copy::{assets as copy, board, headings, reference};
use crate::site::{Column, Demo, Grow, SCROLL_TAIL, role, space, type_scale};

const STAGE_H: (i32, i32, i32) = (150, 165, 190);

pub(crate) fn build(g: &mut Grow, slot: Leaf) {
    let container = crate::site::shell::content_area(g.forest, slot);
    let mut column = Column::new(g.forest, container);

    column.display(g.forest, headings::ASSETS);
    column.lead(g.forest, copy::LEAD);

    column.heading(g.forest, headings::ASSETS_KEY);
    column.prose(g.forest, copy::KEY);
    key(g, &mut column);

    column.heading(g.forest, headings::ASSETS_ARTWORK);
    column.prose(g.forest, copy::ARTWORK);
    artwork(g, &mut column);

    column.heading(g.forest, headings::ASSETS_WHERE);
    column.prose(g.forest, copy::WHERE);
    sources(g, &mut column);

    column.rule(g.forest);
    column.heading(g.forest, headings::ASSETS_FONTS);
    column.prose(g.forest, copy::FONTS);

    column.tail(g.forest, SCROLL_TAIL);
}

// ---- key before bytes ------------------------------------------------------------------------

/// Its own copy of the photograph, asked for by this board rather than reusing the key
/// registered at startup -- so the key on the readout is one this board really was handed.
const SAMPLE: &[u8] = include_bytes!("../assets/images/sample.jpg");

/// Taller than the others: this one holds a picture.
const KEY_STAGE_H: (i32, i32, i32) = (200, 220, 260);

struct Key {
    board: Blueprint,
    /// Grown at build with the real key, and held hidden until the second step.
    ///
    /// Hidden rather than grown on the press, because the two are not the same claim: growing
    /// it late would say the element waits for its bytes, and it does not -- it is grown with a
    /// key that names nothing yet, which is the section's whole point. What the step controls is
    /// only whether the picture is on screen.
    image: Leaf,
    /// The word standing where the picture will be, while it is not.
    slot: Leaf,
    /// How many bytes the key turned out to name. Read once, when the board is built.
    bytes: usize,
}

fn key(g: &mut Grow, column: &mut Column) {
    let mut board = blueprint::board(
        g,
        column,
        KEY_STAGE_H,
        board::KEY_ROWS,
        &board::KEY_STEPS,
        &reference::KEY,
    );
    // The two calls the section is about, in the order it describes and in one breath: the key
    // comes back from the ask, and the element naming it is grown before anything is decoded.
    let key = g.forest.load_asset(AssetSource::Bytes(SAMPLE.to_vec()));
    let frame = blueprint::frame(
        g.forest,
        board.stage,
        Location::new().xs(
            0.pct().as_left().with(100.pct().as_right()),
            0.pct().as_top().with(100.pct().as_bottom()),
        ),
        board::SOURCES_ELEMENT,
        false,
    );
    let slot = g.forest.branch(
        frame.leaf,
        Text::new(board::KEY_SLOT)
            .size(FontSize::new(type_scale::LABEL))
            .color(role::on_surface_variant())
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                50.pct().as_center_y().with(1.letters().as_height()),
            ))
            .elevate(Elevation::up(1))
            .align(HorizontalAlignment::Center, VerticalAlignment::Middle)
            .pass_through(),
    );
    let image = g.forest.branch(
        frame.leaf,
        Image::new(key)
            .view(ImageView::Aspect)
            .at(Location::new().xs(
                space::XS
                    .px()
                    .as_left()
                    .with(100.pct().as_right().adjust(-space::XS)),
                space::XS
                    .px()
                    .as_top()
                    .with(100.pct().as_bottom().adjust(-space::XS)),
            ))
            .elevate(Elevation::up(2))
            .pass_through(),
    );
    g.forest.visible(image, false);
    board.set(g.forest, 0, board::key_digits(key));
    board.set(g.forest, 1, board::KEY_NO_BYTES);
    g.page.demos.push(Box::new(Key {
        board,
        image,
        slot,
        // Same frame the key was issued in, so this reads `None` and the number is taken from
        // what was handed to the loader -- which is the same bytes, and is available now.
        bytes: SAMPLE.len(),
    }));
}

impl Demo for Key {
    fn clicked(&mut self, forest: &mut Forest, leaf: Leaf) -> bool {
        let Some(step) = self.board.pressed(leaf) else {
            return false;
        };
        self.board.select(forest, step);
        let arrived = step == 1;
        forest.visible(self.image, arrived);
        forest.visible(self.slot, !arrived);
        // The key row never changes across the row. That is the claim: one handle, valid from
        // the moment it was issued, naming nothing and then naming a picture.
        self.board.set(
            forest,
            1,
            if arrived {
                board::key_bytes(self.bytes)
            } else {
                board::KEY_NO_BYTES.to_string()
            },
        );
        true
    }
}

// ---- swapping artwork ------------------------------------------------------------------------

/// One per [`board::ARTWORK_STEPS`], in order, and each one already registered at startup --
/// this board adds no artwork of its own, which is the point of it.
const ARTWORK: [IconHandles; 4] = [
    IconHandles::Box,
    IconHandles::Code,
    IconHandles::Grid,
    IconHandles::Play,
];
const ARTWORK_SIZE: i32 = 96;

struct Artwork {
    board: Blueprint,
    mark: Leaf,
}

fn artwork(g: &mut Grow, column: &mut Column) {
    let mut board = blueprint::board(
        g,
        column,
        STAGE_H,
        board::ARTWORK_ROWS,
        &board::ARTWORK_STEPS,
        &reference::ARTWORK,
    );
    let mark = g.forest.branch(
        board.stage,
        Icon::new(IconId::from(ARTWORK[0]))
            .color(role::accent())
            .at(Location::new().xs(
                50.pct().as_center_x().with(ARTWORK_SIZE.px().as_width()),
                50.pct().as_center_y().with(ARTWORK_SIZE.px().as_height()),
            ))
            .elevate(Elevation::up(2))
            .pass_through(),
    );
    board.set(g.forest, 0, board::ARTWORK_IDS[0]);
    board.set(g.forest, 1, board::ARTWORK_ELEMENT);
    g.page.demos.push(Box::new(Artwork { board, mark }));
}

impl Demo for Artwork {
    fn clicked(&mut self, forest: &mut Forest, leaf: Leaf) -> bool {
        let Some(step) = self.board.pressed(leaf) else {
            return false;
        };
        self.board.select(forest, step);
        // One write, no respawn: the element outlives every field it draws.
        forest.icon(self.mark, IconId::from(ARTWORK[step]));
        self.board.set(forest, 0, board::ARTWORK_IDS[step]);
        true
    }
}

// ---- bundled or fetched ----------------------------------------------------------------------

/// The three shapes across the stage, as centre fractions. Source, key, element -- and only the
/// first of them changes with the step, which is what the board is claiming.
const PATH_AT: [f32; 3] = [18.0, 50.0, 82.0];
const PATH_NODE_W: i32 = 30;
const PATH_NODE_H: i32 = 56;
/// The two runs between the three shapes, as (from, to) fractions, clear of the shapes at both
/// ends so a connector never disappears under the thing it connects.
const PATH_RUNS: [(f32, f32); 2] = [(18.0, 50.0), (50.0, 82.0)];

fn path_node_at(center: f32) -> Location {
    Location::new().xs(
        center
            .pct()
            .as_center_x()
            .with(PATH_NODE_W.pct().as_width()),
        50.pct().as_center_y().with(PATH_NODE_H.px().as_height()),
    )
}

struct Sources {
    board: Blueprint,
    /// The first shape's caption. The only thing on the stage a step rewrites -- the key and the
    /// element are the same two whichever build you are looking at.
    origin: Leaf,
    source: Leaf,
}

fn sources(g: &mut Grow, column: &mut Column) {
    let mut board = blueprint::board(
        g,
        column,
        STAGE_H,
        board::SOURCES_ROWS,
        &board::SOURCES_STEPS,
        &reference::SOURCES,
    );
    // Behind the shapes: a hairline from the first to the last, so the three read as one run
    // rather than three things that happen to be in a row. Drawn in two segments so the middle
    // shape sits on the line rather than over a line that passes behind it.
    for (from, to) in PATH_RUNS {
        g.forest.branch(
            board.stage,
            Panel::new()
                .color(role::outline())
                .rounding(Rounding::None)
                .at(Location::new().xs(
                    from.pct().as_left().with(to.pct().as_right()),
                    50.pct().as_center_y().with(1.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .pass_through(),
        );
    }
    // Built out rather than taken from `child_box`, which names its shape on the way in and
    // hands back only the shape -- and this is the one caption on the stage that gets rewritten.
    let source = g.forest.branch(
        board.stage,
        Panel::new()
            .color(role::accent())
            .rounding(Rounding::Xs)
            .at(path_node_at(PATH_AT[0]))
            .elevate(Elevation::up(2))
            .grid(Grid::new(1.col().gap(0), 1.row().gap(0)))
            .pass_through(),
    );
    let origin = blueprint::name(g.forest, source, board::SOURCES_ORIGINS[0]);
    blueprint::child_box(
        g.forest,
        board.stage,
        path_node_at(PATH_AT[1]),
        role::surface(),
        board::SOURCES_KEY,
    );
    blueprint::child_box(
        g.forest,
        board.stage,
        path_node_at(PATH_AT[2]),
        role::surface(),
        board::SOURCES_ELEMENT,
    );
    board.set(g.forest, 0, board::SOURCES_CALLS[0]);
    board.set(g.forest, 1, board::SOURCES_THEN[0]);
    g.page.demos.push(Box::new(Sources {
        board,
        origin,
        source,
    }));
}

impl Demo for Sources {
    fn clicked(&mut self, forest: &mut Forest, leaf: Leaf) -> bool {
        let Some(step) = self.board.pressed(leaf) else {
            return false;
        };
        self.board.select(forest, step);
        forest.text(self.origin, board::SOURCES_ORIGINS[step]);
        // A second tone for the second path, so which one you are looking at is legible from
        // the drawing and not only from the row under it.
        forest.color(
            self.source,
            if step == 0 {
                role::accent()
            } else {
                Color::rose(400)
            },
        );
        self.board.set(forest, 0, board::SOURCES_CALLS[step]);
        self.board.set(forest, 1, board::SOURCES_THEN[step]);
        true
    }
}
