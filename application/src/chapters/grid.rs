use foliage::{
    Anchor, Animation, AspectRatio, Color, Ease, EcsExtension, Elevation, Entity, FontSize, Grid,
    GridExt, HorizontalAlignment, Line, Location, OnEnd, Opacity, Panel, Polygon, Query, Rounding,
    Sprout, Text, TextValue, Tree, Trigger, VerticalAlignment, anchor,
};

const PANEL_LEFT_PCT: f32 = 18.0; // same left/top `relative.rs`'s own panel uses
const PANEL_TOP_PCT: f32 = 20.0;
// `width`/`height` don't need to actually match for a square result -- `AspectRatio::new()
// .xs(1.0)` (the same mechanism every `Polygon` in this app already carries, see
// `chapters/toc.rs`'s own `CARD_HEIGHT_PX` doc for the full story) clamps whichever comes
// out smaller in real px and centers the square within the declared box, so this only
// needs to be *at least* as big on both axes as the eventual square, not an exact match.
const PANEL_WIDTH_PCT: f32 = 26.0;
const PANEL_HEIGHT_PCT: f32 = 40.0;
const PANEL_COLOR: i32 = 600; // slate -- same muted parent backdrop `relative.rs` uses
const PANEL_FADE: u64 = 400;
const GRID_GAP: i32 = 0; // flush cells -- lets the dashed dividers sit at a clean 50%/50%

const HIGHLIGHT_COLOR: i32 = 400; // amber
const HIGHLIGHT_OPACITY: f32 = 0.28; // translucent -- reads as "region in use", not a solid fill

const DASH_DELAY: u64 = 200; // after the panel starts fading in
const DASH_FADE: u64 = 300;
const DASH_COLOR: i32 = 500; // slate, same blueprint tone `location.rs`/`relative.rs` use
const DASH_WEIGHT: i32 = 2;
const DASH_COUNT: i32 = 6; // dashes per divider
const DASH_ON_FRACTION: f32 = 0.55; // portion of each dash's own slot that's actually drawn

const CELL_MORPH_DELAY: u64 = 500; // after the dashes finish fading in
const CELL_MORPH_DURATION: u64 = 500;
const CELL_ROUNDING: f32 = 0.15;
const MOVE_DURATION: u64 = 500;
const MOVE_PAUSE: u64 = 400;

// off the panel's own right edge, same blueprint-line technique `location.rs`/
// `relative.rs` both use -- anchored to `panel` (not `cell`), since the label just needs
// to *show* the current cell, not visually track a shape that's also moving.
const LINE_START_GAP_PX: i32 = 10;
const LINE_LENGTH_PX: i32 = 40;
const LINE_FADE: u64 = 250;

const BLUEPRINT_COLOR: i32 = 500; // slate

const LABEL_GAP_PX: i32 = 10;
const LABEL_FONT_SIZE: u32 = 13;
const LABEL_WIDTH_PX: i32 = 110;
const LABEL_ROW_HEIGHT_PX: i32 = 23;
const LABEL_FADE: u64 = 300;
const LABEL_DELAY: u64 = 150; // after the line finishes drawing in

const TAGLINE_TEXT: &str = "a grid divides a parent into addressable cells";
const TAGLINE_FONT_SIZE: u32 = 13;
const TAGLINE_COLOR: i32 = 400; // slate, muted subtitle -- orients without competing with the demo
// same reasoning as `relative.rs`'s own tagline: `panel` sits high in the frame
// (`PANEL_TOP_PCT` = 20), so the frame's own empty band is below it, not above --
// `Xs` goes there; `Md`+ goes beside it instead, anchored past the label's own right
// edge, using `panel`'s rest band (not a live `Anchor`) so it doesn't need to track
// anything moving.
const TAGLINE_XS_TOP_PCT: f32 = 70.0;
const TAGLINE_XS_HEIGHT_PCT: f32 = 30.0;
const TAGLINE_XS_LEFT_PCT: f32 = 8.0;
const TAGLINE_XS_WIDTH_PCT: f32 = 84.0;
const TAGLINE_MD_GAP_PX: i32 =
    LINE_START_GAP_PX + LINE_LENGTH_PX + LABEL_GAP_PX + LABEL_WIDTH_PX + 24;
const TAGLINE_MD_WIDTH_PX: i32 = 150;
const TAGLINE_MD_HEIGHT_PCT: f32 = 65.0; // more room than `PANEL_HEIGHT_PCT` alone gave it

/// Every moving/labeled piece of the demo, threaded through the cell-to-cell chain as one
/// value instead of a growing parameter list -- same reasoning `navigator.rs`'s own `Nav`
/// struct gives for bundling its own live entities together.
#[derive(Copy, Clone)]
struct CellDemo {
    cell: Entity,
    highlight: Entity,
    label_line_1: Entity,
    label_line_2: Entity,
}

/// A parent (square, via `AspectRatio` -- the same mechanism every `Polygon` in this app
/// already carries) divides into a real 2x2 `Grid` (dashed lines marking the column/row
/// split -- decorative only, plain percent-of-`panel` children, not tied to the grid's
/// own cell math), then a heptagon morphs in at cell `(1, 1)`, a translucent panel right
/// behind it tracking the exact same cell, and both step through what a `Grid` actually
/// buys a child over raw percent: a plain cell address (`1/1` -> `2/2`), then a span
/// across multiple rows *or* columns -- never both at once, since that's just a bigger
/// single cell, not the distinct "spans two of one axis" case this demonstrates -- before
/// returning to rest. The translucent panel exists because the heptagon's own silhouette
/// doesn't fill its bounding box, so the actual rectangular cell being addressed wouldn't
/// otherwise be legible, especially once it spans two cells at once. A two-row label off
/// `panel`'s own right edge names the current cell, snapping to the new address on each
/// move rather than counting continuously -- there's no single number tweening here the
/// way `location.rs`/`relative.rs` have, just a cell address that's simply different.
pub fn build(tree: &mut Tree, slot: Entity) {
    let frame = crate::chapters::window_frame(tree, slot);

    let panel = tree.branch(
        frame,
        Panel::new()
            .color(Color::slate(PANEL_COLOR))
            .outline(2)
            .rounding(Rounding::None)
            .at(Location::new().xs(
                PANEL_LEFT_PCT
                    .pct()
                    .as_left()
                    .with(PANEL_WIDTH_PCT.pct().as_width()),
                PANEL_TOP_PCT
                    .pct()
                    .as_top()
                    .with(PANEL_HEIGHT_PCT.pct().as_height()),
            ))
            .elevate(Elevation::up(2))
            .with((
                Grid::new(2.col().gap(GRID_GAP), 2.row().gap(GRID_GAP)),
                AspectRatio::new().xs(1.0),
                Opacity::new(0.0),
            )),
    );

    let seq = tree.sequence();
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(panel)
            .during(seq)
            .start(0)
            .finish(PANEL_FADE)
            .eased(Ease::Linear),
    );

    // translucent -- shows the *rectangular region* `cell` is actually addressing, since
    // a heptagon's own silhouette doesn't fill its bounding box and wouldn't otherwise
    // make the cell's real extent legible, especially once it spans two cells at once.
    // Sits behind both the dashes and `cell` itself (elevation-wise), tracking the exact
    // same `cell_location` moves in lockstep -- see `move_cell`.
    let highlight = tree.branch(
        panel,
        Panel::new()
            .color(Color::amber(HIGHLIGHT_COLOR))
            .rounding(Rounding::None)
            .at(cell_location(1, 1, 1, 1))
            .elevate(Elevation::up(3))
            .with(Opacity::new(0.0)),
    );
    tree.animate(
        Animation::new(Opacity::new(HIGHLIGHT_OPACITY))
            .targeting(highlight)
            .during(seq)
            .start(CELL_MORPH_DELAY)
            .finish(CELL_MORPH_DELAY + CELL_MORPH_DURATION)
            .eased(Ease::Linear),
    );

    // dashed 50%/50% dividers -- plain percent-of-`panel` children (no `Anchor` needed,
    // the same way `relative.rs`'s own `child` resolves against `panel` directly), each
    // divider built from `DASH_COUNT` short `Line` segments with a gap between them
    // rather than one continuous line, since `Line` itself has no dash style of its own.
    let dash_slot_pct = 100.0 / DASH_COUNT as f32;
    for i in 0..DASH_COUNT {
        let seg_start = i as f32 * dash_slot_pct;
        let seg_end = seg_start + dash_slot_pct * DASH_ON_FRACTION;

        let vertical = tree.branch(
            panel,
            Line::new(DASH_WEIGHT)
                .color(Color::slate(DASH_COLOR))
                .at(Location::new().xs(
                    50.0.pct().as_x().with(seg_start.pct().as_y()),
                    50.0.pct().as_x().with(seg_end.pct().as_y()),
                ))
                .elevate(Elevation::up(4))
                .with(Opacity::new(0.0)),
        );
        let horizontal = tree.branch(
            panel,
            Line::new(DASH_WEIGHT)
                .color(Color::slate(DASH_COLOR))
                .at(Location::new().xs(
                    seg_start.pct().as_x().with(50.0.pct().as_y()),
                    seg_end.pct().as_x().with(50.0.pct().as_y()),
                ))
                .elevate(Elevation::up(4))
                .with(Opacity::new(0.0)),
        );
        for dash in [vertical, horizontal] {
            tree.animate(
                Animation::new(Opacity::new(1.0))
                    .targeting(dash)
                    .during(seq)
                    .start(DASH_DELAY)
                    .finish(DASH_DELAY + DASH_FADE)
                    .eased(Ease::Linear),
            );
        }
    }

    let cell = tree.branch(
        panel,
        Polygon::new()
            .sides(3.0)
            .rounding(0.0)
            .rotation(0.0)
            .color(Color::orange(400))
            .at(cell_location(1, 1, 1, 1))
            .elevate(Elevation::up(5))
            .with(Opacity::new(0.0)),
    );
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(cell)
            .during(seq)
            .start(CELL_MORPH_DELAY)
            .finish(CELL_MORPH_DELAY + CELL_MORPH_DURATION)
            .eased(Ease::Linear),
    );
    tree.animate(
        Animation::new(Polygon {
            sides: 7.0,
            rounding: CELL_ROUNDING,
            rotation: 0.0,
        })
        .targeting(cell)
        .during(seq)
        .start(CELL_MORPH_DELAY)
        .finish(CELL_MORPH_DELAY + CELL_MORPH_DURATION)
        .eased(Ease::DECELERATE),
    );

    // line: panel's own right edge -> `LINE_START_GAP_PX` + `LINE_LENGTH_PX` further
    // right, at its own vertical center -- `Anchor::new(panel)`, same as `relative.rs`'s
    // own line.
    let line = tree.branch(
        frame,
        Line::new(2)
            .color(Color::slate(BLUEPRINT_COLOR))
            .at(Location::new().xs(
                anchor()
                    .right()
                    .as_x()
                    .adjust(LINE_START_GAP_PX)
                    .with(anchor().center_y().as_y()),
                anchor()
                    .right()
                    .as_x()
                    .adjust(LINE_START_GAP_PX + LINE_LENGTH_PX)
                    .with(anchor().center_y().as_y()),
            ))
            .elevate(Elevation::up(2))
            .with((Anchor::new(panel), Opacity::new(0.0))),
    );
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(line)
            .during(seq)
            .start(CELL_MORPH_DELAY + CELL_MORPH_DURATION)
            .finish(CELL_MORPH_DELAY + CELL_MORPH_DURATION + LINE_FADE)
            .eased(Ease::Linear),
    );

    // plausible `Location` syntax naming the current cell -- split across two `Text`
    // entities at the natural `.` boundary, same reasoning `location.rs`'s/`relative.rs`'s
    // own labels use: this renderer is a fixed monospace character grid (see
    // `MonospacedFont`), not word-aware text shaping, so one entity wide enough for the
    // whole string would wrap mid-word wherever the box happens to end rather than at a
    // sensible break.
    let label_row = |text: &str, row: i32| {
        Text::new(text)
            .size(FontSize::new(LABEL_FONT_SIZE))
            .color(Color::slate(BLUEPRINT_COLOR))
            .at(Location::new().xs(
                anchor()
                    .right()
                    .as_left()
                    .adjust(LINE_START_GAP_PX + LINE_LENGTH_PX + LABEL_GAP_PX)
                    .with(LABEL_WIDTH_PX.px().as_width()),
                anchor()
                    .center_y()
                    .as_top()
                    .adjust(-LABEL_ROW_HEIGHT_PX + row * LABEL_ROW_HEIGHT_PX)
                    .with(LABEL_ROW_HEIGHT_PX.px().as_height()),
            ))
            .elevate(Elevation::up(2))
            .with((
                HorizontalAlignment::Left,
                VerticalAlignment::Middle,
                Anchor::new(panel),
                Opacity::new(0.0),
            ))
    };
    let (col_text, row_text) = cell_label_text(1, 1, 1, 1);
    let label_line_1 = tree.branch(frame, label_row(&col_text, 0));
    let label_line_2 = tree.branch(frame, label_row(&row_text, 1));
    for label in [label_line_1, label_line_2] {
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .targeting(label)
                .during(seq)
                .start(CELL_MORPH_DELAY + CELL_MORPH_DURATION + LINE_FADE + LABEL_DELAY)
                .finish(
                    CELL_MORPH_DELAY + CELL_MORPH_DURATION + LINE_FADE + LABEL_DELAY + LABEL_FADE,
                )
                .eased(Ease::Linear),
        );
    }

    let tagline = tree.branch(
        frame,
        Text::new(TAGLINE_TEXT)
            .size(FontSize::new(TAGLINE_FONT_SIZE))
            .color(Color::slate(TAGLINE_COLOR))
            .at(Location::new()
                .xs(
                    TAGLINE_XS_LEFT_PCT
                        .pct()
                        .as_left()
                        .with(TAGLINE_XS_WIDTH_PCT.pct().as_width()),
                    TAGLINE_XS_TOP_PCT
                        .pct()
                        .as_top()
                        .with(TAGLINE_XS_HEIGHT_PCT.pct().as_height()),
                )
                .md(
                    anchor()
                        .right()
                        .as_left()
                        .adjust(TAGLINE_MD_GAP_PX)
                        .with(TAGLINE_MD_WIDTH_PX.px().as_width()),
                    PANEL_TOP_PCT
                        .pct()
                        .as_top()
                        .with(TAGLINE_MD_HEIGHT_PCT.pct().as_height()),
                ))
            .elevate(Elevation::up(2))
            .with((
                HorizontalAlignment::Left,
                VerticalAlignment::Middle,
                Anchor::new(panel),
                Opacity::new(0.0),
            )),
    );
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(tagline)
            .during(seq)
            .start(0)
            .finish(PANEL_FADE)
            .eased(Ease::Linear),
    );

    let demo = CellDemo {
        cell,
        highlight,
        label_line_1,
        label_line_2,
    };
    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        move_to_2_2(&mut tree, demo);
    });
}

/// `(col_start, col_end, row_start, row_end)` as 1-indexed `Grid` cells -- `col_start ==
/// col_end` (and same for rows) is a single-cell address like `1/1`; different start/end
/// on exactly one axis is a span across that axis only, the case this whole page exists
/// to show off.
fn cell_location(col_start: i32, col_end: i32, row_start: i32, row_end: i32) -> Location {
    Location::new().xs(
        col_start.col().as_left().with(col_end.col().as_right()),
        row_start.row().as_top().with(row_end.row().as_bottom()),
    )
}

/// The same cell address as real `.col()`/`.row()` syntax, split into the label's own two
/// rows -- `col_start == col_end` reads as a single index (`2.col()`), otherwise as the
/// span it actually is (`1-2.row()`).
fn cell_label_text(col_start: i32, col_end: i32, row_start: i32, row_end: i32) -> (String, String) {
    let col_text = if col_start == col_end {
        format!("{col_start}.col()")
    } else {
        format!("{col_start}-{col_end}.col()")
    };
    let row_text = if row_start == row_end {
        format!("{row_start}.row()")
    } else {
        format!("{row_start}-{row_end}.row()")
    };
    (col_text, row_text)
}

fn move_to_2_2(tree: &mut Tree, demo: CellDemo) {
    move_cell(tree, demo, (2, 2, 2, 2), 0, Some(move_to_span));
}

fn move_to_span(tree: &mut Tree, demo: CellDemo) {
    // column 2, spanning both rows -- multiple rows under one column, not both axes.
    move_cell(tree, demo, (2, 2, 1, 2), MOVE_PAUSE, Some(move_to_rest));
}

fn move_to_rest(tree: &mut Tree, demo: CellDemo) {
    move_cell(tree, demo, (1, 1, 1, 1), MOVE_PAUSE, None);
}

fn move_cell(
    tree: &mut Tree,
    demo: CellDemo,
    (col_start, col_end, row_start, row_end): (i32, i32, i32, i32),
    pause_before: u64,
    next: Option<fn(&mut Tree, CellDemo)>,
) {
    let seq = tree.sequence();
    let target = cell_location(col_start, col_end, row_start, row_end);
    for entity in [demo.cell, demo.highlight] {
        tree.animate(
            Animation::new(target.clone())
                .targeting(entity)
                .during(seq)
                .start(pause_before)
                .finish(pause_before + MOVE_DURATION)
                .eased(Ease::EMPHASIS),
        );
    }
    tree.sequence_end(
        seq,
        move |_: Trigger<OnEnd>, mut tree: Tree, existing: Query<Entity>| {
            // same reasoning as `location.rs`'s own guard: `write_to` is a raw `.insert()`
            // by entity ID with no existence check, unsafe only here because this is the
            // one chain in this file that can still be pending after the page is gone
            // (navigating away despawns the label entities along with the rest of it).
            if existing.contains(demo.label_line_1) {
                let (col_text, row_text) = cell_label_text(col_start, col_end, row_start, row_end);
                tree.write_to(demo.label_line_1, TextValue(col_text));
                tree.write_to(demo.label_line_2, TextValue(row_text));
            }
            if let Some(next_fn) = next {
                next_fn(&mut tree, demo);
            }
        },
    );
}
