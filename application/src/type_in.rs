use foliage::{
    Animation, Color, Ease, EcsExtension, Elevation, Entity, FontSize, Grid, GridExt,
    HorizontalAlignment, Leaf, Location, Opacity, Panel, Sprout, Text, Tree,
};

const TEXT: &str = "foliage.rs";
const FONT_SIZE: u32 = 34;
/// Index (0-based) where the ".rs" extension starts -- colored differently from the rest.
const EXTENSION_START: usize = 7;
const CELL_GAP_PX: i32 = 10; // spacing between letter cells (must exceed 2*CELL_PAD)
/// One cell per character, plus one trailing cell for the cursor's resting spot past the
/// last letter.
const TOTAL_COLS: i32 = TEXT.len() as i32 + 1;

const REVEAL_SNAP: u64 = 40; // a pop, not a fade -- appears/moves near-instantly
const BLINK_INTERVAL: u64 = 500;
const BLINK_SNAP: u64 = 20;
const TRAILING_BLINKS: u64 = 16; // keeps blinking a while after typing finishes

/// Delay before each letter of `TEXT` lands, one per character -- asymmetric, like
/// someone hunting for the right key: mostly quick, with a couple of real hesitations
/// (before "g", and again before the period).
const DELAYS: [u64; 10] = [350, 130, 150, 420, 140, 600, 160, 480, 150, 200];

/// `N.col().as_left().with(N.col().as_right())`, the SAME index on both edges, is cell N
/// (1-indexed): `Left` is exclusive (`(N-1)*pitch`), `Right` is inclusive (`N*pitch`), so
/// matching indices give exactly one cell's span. Mismatched indices (`i`/`i+1`) double
/// the width instead -- the bug the hand-rolled version sidestepped by not using `.col()`
/// at all.
///
/// The pitch itself is `character_block('a', FONT_SIZE).a()` -- ONE reference glyph's
/// advance width, measured from `'a'` specifically. `HorizontalAlignment::Center` then
/// centers each *actual* glyph inside that exact-pitch cell; any character whose real
/// rendered width is even a pixel wider than `'a'`'s overflows symmetrically past both of
/// its own cell edges, into whichever neighbor is closest -- which reads as one letter
/// "clipping" another, with the overflow (and so the apparent gap) differing per
/// character rather than being uniform. `CELL_PAD` pads the cell wider than the raw pitch
/// on both sides (without moving its center, so column spacing stays even) so a glyph a
/// few px off from `'a'` still fits.
const CELL_PAD: i32 = 3;
fn cell_location(i: i32) -> Location {
    let n = i + 1;
    Location::new().xs(
        n.col()
            .as_left()
            .adjust(-CELL_PAD)
            .with(n.col().as_right().adjust(CELL_PAD)),
        1.row().as_top().with(1.row().as_bottom()),
    )
}

fn cursor_location(i: i32) -> Location {
    let n = i + 1;
    Location::new().xs(
        n.col().as_left().with(n.col().as_right()),
        100.pct().as_bottom().adjust(-4).with(3.px().as_height()),
    )
}

/// A terminal-style type-in: a blinking `_` sits where the first letter will go, then
/// "foliage.rs" is typed out one character at a time with uneven, human timing, the
/// cursor hopping to just past each new letter as it lands. `start` is when this whole
/// effect should begin (already-settled, still-spinning content upstream of it).
pub fn type_in(tree: &mut Tree, parent: Entity, seq: Entity, start: u64) {
    // `Grid::new(1.letters().gap(N), 1.letters())` makes one column exactly one
    // character of *this entity's own* `FontSize`, with `N` logical px between cells --
    // the same mechanism `TextInput`'s field/cursor grid uses, now with a real
    // regression test behind it (see `text::monospaced::tests::col_against_a_letters_grid_...`).
    let grid = Grid::new(1.letters().gap(CELL_GAP_PX), 1.letters());

    // Sized off its own content, not a guessed screen percentage: `TOTAL_COLS.letters()`
    // is the *other* `.letters()` mechanism (a value, off this entity's own `FontSize`,
    // covered by `letters_resolves_...`) -- exactly as wide as `TOTAL_COLS` pitches, plus
    // an explicit `.adjust()` for the `TOTAL_COLS - 1` gaps the grid will actually place
    // between them. Bump `FONT_SIZE` (or make it responsive per breakpoint later) and
    // this box follows without anything here needing to be re-tuned by hand.
    let field = tree.branch(
        parent,
        Leaf::sprout()
            .at(Location::new().xs(
                50.pct().as_center_x().with(
                    TOTAL_COLS
                        .letters()
                        .as_width()
                        .adjust((TOTAL_COLS - 1) * CELL_GAP_PX),
                ),
                50.pct().as_center_y().with(1.letters().as_height()),
            ))
            .elevate(Elevation::up(1))
            .with((grid, FontSize::new(FONT_SIZE))),
    );

    let cursor = tree.branch(
        field,
        Panel::new()
            .color(Color::gray(50))
            .at(cursor_location(0))
            .elevate(Elevation::up(1))
            .with(Opacity::new(0.0)),
    );
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(cursor)
            .during(seq)
            .start(start)
            .finish(start + REVEAL_SNAP)
            .eased(Ease::Linear),
    );

    let mut letter_time = start;
    for (i, ch) in TEXT.chars().enumerate() {
        letter_time += DELAYS[i];
        let col = i as i32;

        let color = if i >= EXTENSION_START {
            Color::orange(600) // ".rs" -- darker than the polygon's own orange
        } else {
            Color::gray(50)
        };
        let letter = tree.branch(
            field,
            Text::new(ch.to_string())
                .size(FontSize::new(FONT_SIZE))
                .color(color)
                .at(cell_location(col))
                .elevate(Elevation::up(1))
                .with((HorizontalAlignment::Center, Opacity::new(0.0))),
        );
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .targeting(letter)
                .during(seq)
                .start(letter_time)
                .finish(letter_time + REVEAL_SNAP)
                .eased(Ease::Linear),
        );

        // the cursor hops to just past the letter that just landed.
        tree.animate(
            Animation::new(cursor_location(col + 1))
                .targeting(cursor)
                .during(seq)
                .start(letter_time)
                .finish(letter_time + REVEAL_SNAP)
                .eased(Ease::Linear),
        );
    }

    // cursor blink: a snapped on/off toggle at a steady cadence, running through the
    // whole typing pass and a while after it settles.
    let blink_end = letter_time + TRAILING_BLINKS * BLINK_INTERVAL;
    let mut blink_time = start;
    let mut on = true;
    while blink_time < blink_end {
        blink_time += BLINK_INTERVAL;
        on = !on;
        tree.animate(
            Animation::new(Opacity::new(if on { 1.0 } else { 0.0 }))
                .targeting(cursor)
                .during(seq)
                .start(blink_time)
                .finish(blink_time + BLINK_SNAP)
                .eased(Ease::Linear),
        );
    }
}
