use foliage::{
    Anchor, Animation, Color, Ease, EcsExtension, Elevation, Entity, FontSize, Grid, GridExt,
    HorizontalAlignment, Leaf, Location, OnEnd, Opacity, Panel, Query, Rounding, Sprout, Text,
    TextValue, Tree, Trigger, VerticalAlignment, anchor,
};

// Three cells, re-placed three times. The grid underneath never changes: six columns is
// the common multiple of the 1/2/3 splits below, so every arrangement is expressible as
// column spans and only each cell's own `Location` is ever rewritten.
const CELLS: usize = 3;
const GRID_COLS: i32 = 6;
const GRID_ROWS: i32 = 3;
const CELL_GAP_PX: i32 = 8;

const PANEL_LEFT_PCT: f32 = 16.0;
const PANEL_WIDTH_PCT: f32 = 52.0;
const PANEL_TOP_PCT: f32 = 20.0;
const PANEL_HEIGHT_PCT: f32 = 42.0;
const PANEL_COLOR: i32 = 500; // slate, same muted outline the other chapters' parents use
const PANEL_PAD_PX: i32 = 10;

const CELL_COLOR: i32 = 400; // sky -- one hue for all three; this chapter is about placement
const CELL_ROUNDING: Rounding = Rounding::Sm;

// Each phase names a breakpoint and the column span each cell takes at it. Spans are
// `(first_col, last_col, row)`, 1-based and inclusive on both ends -- the same convention
// `.col()`/`.row()` themselves use.
struct Phase {
    name: &'static str,
    note: &'static str,
    spans: [(i32, i32, i32); CELLS],
}
const PHASES: [Phase; 3] = [
    Phase {
        name: "xs",
        note: "one column",
        spans: [(1, 6, 1), (1, 6, 2), (1, 6, 3)],
    },
    Phase {
        name: "md",
        note: "two columns",
        spans: [(1, 3, 1), (4, 6, 1), (1, 3, 2)],
    },
    Phase {
        name: "lg",
        note: "three columns",
        spans: [(1, 2, 1), (3, 4, 1), (5, 6, 1)],
    },
];

const REVEAL: u64 = 300;
const CELL_STAGGER: u64 = 120;
const PHASE_HOLD: u64 = 1100; // how long each arrangement rests before the next
const REFLOW: u64 = 450; // the `Location` tween between arrangements

const LABEL_COLOR: i32 = 300; // slate
const LABEL_FONT_SIZE: u32 = 15;
const LABEL_WIDTH_PX: i32 = 90;
const LABEL_HEIGHT_PX: i32 = 22;
const LABEL_GAP_PX: i32 = 14;

const NOTE_COLOR: i32 = 500;
const NOTE_FONT_SIZE: u32 = 12;
const NOTE_WIDTH_PX: i32 = 130;
const NOTE_HEIGHT_PX: i32 = 18;

const TAGLINE_TEXT: &str = "one location, resolved per breakpoint";
const TAGLINE_FONT_SIZE: u32 = 13;
const TAGLINE_COLOR: i32 = 400;
const TAGLINE_XS_TOP_PCT: f32 = 72.0;
const TAGLINE_XS_HEIGHT_PCT: f32 = 24.0;
const TAGLINE_XS_LEFT_PCT: f32 = 8.0;
const TAGLINE_XS_WIDTH_PCT: f32 = 84.0;
const TAGLINE_MD_LEFT_PCT: f32 = 74.0;
const TAGLINE_MD_WIDTH_PCT: f32 = 18.0; // ends at 92%, flush with `window_frame`'s margin
const TAGLINE_MD_TOP_PCT: f32 = 22.0;
const TAGLINE_MD_HEIGHT_PCT: f32 = 60.0;

/// Three cells inside one parent, stepping through the arrangements `xs`, `md` and `lg`
/// would each resolve to -- with the breakpoint's own name and the column count labelled
/// as each lands.
///
/// The point is what *doesn't* happen: no cell is ever despawned or respawned. The same
/// three entities carry the same `Location` component throughout, and only the values in
/// it differ per breakpoint -- which is exactly what `.xs(..)/.md(..)/.lg(..)` expresses
/// in one declaration.
///
/// Stepped by hand rather than driven by the real [`Layout`] resource. `Layout` is global
/// and derived from the window, so honoring it for real would reflow the chrome and the
/// navigator along with this page -- every other chapter is self-contained inside its own
/// frame, and this one stays that way. The arrangements themselves are the ones a real
/// `.xs(1.col())/.md(2.col())/.lg(3.col())` produces.
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
            // `Panel` doesn't carry a `Grid` of its own, and `field` below is a child that
            // positions itself -- so `panel` needs one for that child to resolve against,
            // even though nothing here addresses a cell of it.
            .with(Grid::new(1.col().gap(0), 1.row().gap(0))),
    );
    // The grid lives on an inset child rather than on `panel` itself so the cells sit
    // inside the outline instead of flush against it.
    let field = tree.branch(
        panel,
        Leaf::sprout()
            .at(Location::new().xs(
                PANEL_PAD_PX
                    .px()
                    .as_left()
                    .with(100.pct().as_right().adjust(-PANEL_PAD_PX)),
                PANEL_PAD_PX
                    .px()
                    .as_top()
                    .with(100.pct().as_bottom().adjust(-PANEL_PAD_PX)),
            ))
            .elevate(Elevation::up(1))
            .with(Grid::new(
                GRID_COLS.col().gap(CELL_GAP_PX),
                GRID_ROWS.row().gap(CELL_GAP_PX),
            )),
    );

    let seq = tree.sequence();
    let mut cells = [field; CELLS]; // placeholder -- overwritten below before any read
    for i in 0..CELLS {
        let cell = tree.branch(
            field,
            Panel::new()
                .color(Color::sky(CELL_COLOR))
                .rounding(CELL_ROUNDING)
                .at(cell_location(PHASES[0].spans[i]))
                .elevate(Elevation::up(2))
                .with(Opacity::new(0.0)),
        );
        cells[i] = cell;
        let start = i as u64 * CELL_STAGGER;
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .targeting(cell)
                .during(seq)
                .start(start)
                .finish(start + REVEAL)
                .eased(Ease::Linear),
        );
    }

    // Both labels are anchored to `panel`, not to any one cell -- they describe the whole
    // arrangement, and a cell moves out from under them on every phase.
    let name_label = tree.branch(
        frame,
        Text::new(PHASES[0].name)
            .size(FontSize::new(LABEL_FONT_SIZE))
            .color(Color::slate(LABEL_COLOR))
            .at(Location::new().xs(
                anchor()
                    .center_x()
                    .as_center_x()
                    .with(LABEL_WIDTH_PX.px().as_width()),
                anchor()
                    .bottom()
                    .as_top()
                    .adjust(LABEL_GAP_PX)
                    .with(LABEL_HEIGHT_PX.px().as_height()),
            ))
            .elevate(Elevation::up(2))
            .with((
                HorizontalAlignment::Center,
                VerticalAlignment::Middle,
                Anchor::new(panel),
                Opacity::new(0.0),
            )),
    );
    let note_label = tree.branch(
        frame,
        Text::new(PHASES[0].note)
            .size(FontSize::new(NOTE_FONT_SIZE))
            .color(Color::slate(NOTE_COLOR))
            .at(Location::new().xs(
                anchor()
                    .center_x()
                    .as_center_x()
                    .with(NOTE_WIDTH_PX.px().as_width()),
                anchor()
                    .bottom()
                    .as_top()
                    .adjust(LABEL_GAP_PX + LABEL_HEIGHT_PX)
                    .with(NOTE_HEIGHT_PX.px().as_height()),
            ))
            .elevate(Elevation::up(2))
            .with((
                HorizontalAlignment::Center,
                VerticalAlignment::Middle,
                Anchor::new(panel),
                Opacity::new(0.0),
            )),
    );
    for label in [name_label, note_label] {
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .targeting(label)
                .during(seq)
                .start(CELLS as u64 * CELL_STAGGER)
                .finish(CELLS as u64 * CELL_STAGGER + REVEAL)
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
                    TAGLINE_MD_LEFT_PCT
                        .pct()
                        .as_left()
                        .with(TAGLINE_MD_WIDTH_PCT.pct().as_width()),
                    TAGLINE_MD_TOP_PCT
                        .pct()
                        .as_top()
                        .with(TAGLINE_MD_HEIGHT_PCT.pct().as_height()),
                ))
            .elevate(Elevation::up(2))
            .with((
                HorizontalAlignment::Center,
                VerticalAlignment::Middle,
                Opacity::new(0.0),
            )),
    );
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(tagline)
            .during(seq)
            .start(0)
            .finish(REVEAL)
            .eased(Ease::Linear),
    );

    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        hold_then_advance(&mut tree, cells, name_label, note_label, 1);
    });
}

// `(first_col, last_col, row)` -> the `Location` that spans it. `.col()` is exclusive as a
// left edge and inclusive as a right edge, so a cell occupying columns 1..3 is written
// `1.col().as_left().with(3.col().as_right())`.
fn cell_location((first, last, row): (i32, i32, i32)) -> Location {
    Location::new().xs(
        first.col().as_left().with(last.col().as_right()),
        row.row().as_top().with(row.row().as_bottom()),
    )
}

// One phase per call, looping back to `xs` after `lg` so the page keeps demonstrating
// itself. `Location` is `Animate`, so the reflow is a real tween between the two resolved
// boxes rather than a jump -- which is what makes "the same entity moved" legible instead
// of looking like a respawn.
//
// Same guard `text.rs`'s own grow step uses: `write_to` inserts by entity id with no
// existence check, and this chain can still be pending after the page is gone.
fn hold_then_advance(
    tree: &mut Tree,
    cells: [Entity; CELLS],
    name_label: Entity,
    note_label: Entity,
    next: usize,
) {
    let seq = tree.sequence();
    // A no-op tween purely to get a timed `sequence_end` -- the same waiting-step pattern
    // `text.rs`/`sequence.rs` use, since there's nothing to animate during the hold.
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(cells[0])
            .during(seq)
            .start(0)
            .finish(PHASE_HOLD)
            .eased(Ease::Linear),
    );
    tree.sequence_end(
        seq,
        move |_: Trigger<OnEnd>, mut tree: Tree, existing: Query<Entity>| {
            if !existing.contains(cells[0]) {
                return;
            }
            let phase = &PHASES[next % PHASES.len()];
            tree.write_to(name_label, TextValue(phase.name.to_string()));
            tree.write_to(note_label, TextValue(phase.note.to_string()));
            let reflow = tree.sequence();
            for (i, cell) in cells.iter().copied().enumerate() {
                tree.animate(
                    Animation::new(cell_location(phase.spans[i]))
                        .targeting(cell)
                        .during(reflow)
                        .start(0)
                        .finish(REFLOW)
                        .eased(Ease::DECELERATE),
                );
            }
            tree.sequence_end(reflow, move |_: Trigger<OnEnd>, mut tree: Tree| {
                hold_then_advance(&mut tree, cells, name_label, note_label, next + 1);
            });
        },
    );
}
