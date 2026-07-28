use foliage::{
    Anchor, Animation, Color, Ease, EcsExtension, Elevation, Entity, FontSize, Grid, GridExt,
    HorizontalAlignment, Leaf, Location, OnEnd, Opacity, Query, Sprout, Text, Tree, Trigger,
    VerticalAlignment, anchor,
};

// three glyphs, not a real word -- each one exists purely to carry its own annotation.
const GLYPHS: [char; 3] = ['a', 'w', 'g'];
const ANNOTATIONS: [&str; 3] = ["kerning", "char-width", "char-color"];
// above/below/above -- the trailing `font-size` annotation (see `wait_before_grow`)
// continues the alternation to `below`.
const ANNOTATION_ABOVE: [bool; 3] = [true, false, true];
const CHAR_COLOR_INDEX: usize = 2; // 'g' -- rendered in its own accent color, not the others' shared slate
const CHAR_COLOR_ACCENT: i32 = 400; // amber
const GLYPH_COLOR: i32 = 300; // gray -- the other two glyphs' shared, undifferentiated color

const WORD_CENTER_X_PCT: f32 = 40.0;
const WORD_TOP_PCT: f32 = 42.0;
const INITIAL_FONT_SIZE: u32 = 40;
const GROWN_FONT_SIZE: u32 = 64;
const CELL_GAP_PX: i32 = 34; // generous -- leaves real room for the kerning nudge below
const CELL_PAD: i32 = 4; // same reasoning as `type_in.rs`'s own `CELL_PAD`: pitch is measured
// off one reference glyph's own advance width, so anything wider needs a margin to not overflow
const TOTAL_COLS: i32 = GLYPHS.len() as i32;
// pseudo, not a real per-pair kerning feature (this engine's text is a fixed monospace
// grid, see `type_in.rs`'s own `CELL_PAD` doc) -- just a small positional nudge on the
// first glyph's own cell, closing its gap to the second glyph enough to *read* as kerned.
const KERNING_NUDGE_PX: i32 = 10;

const LETTER_REVEAL: u64 = 350;
const LETTER_STAGGER: u64 = 650; // gap between successive glyphs' own start
const LABEL_DELAY: u64 = 150; // after its own glyph finishes revealing
const LABEL_FADE: u64 = 300;

const LABEL_COLOR: i32 = 500; // slate, same blueprint tone the other chapters use
const LABEL_FONT_SIZE: u32 = 13;
const LABEL_GAP_PX: i32 = 10;
const LABEL_WIDTH_PX: i32 = 120;
const LABEL_HEIGHT_PX: i32 = 20;

const GROW_PAUSE: u64 = 900; // real waiting step before the font-size bump -- see `wait_before_grow`
const FONT_SIZE_LABEL_TEXT: &str = "font-size";
const FONT_SIZE_LABEL_FADE: u64 = 300;
// a full extra row below `annotation_location(false)`'s own gap, not the same one
// `char-width` uses -- both anchors (`field` and glyph `w`) sit at roughly the same
// vertical position and grow together when `FontSize` bumps, so reusing the plain
// "below" gap put them in the same band, overlapping once the glyphs grew.
const FONT_SIZE_LABEL_GAP_PX: i32 = LABEL_GAP_PX + LABEL_HEIGHT_PX + LABEL_GAP_PX;

const TAGLINE_TEXT: &str = "text configuration";
const TAGLINE_FONT_SIZE: u32 = 13;
const TAGLINE_COLOR: i32 = 400; // slate, muted subtitle -- orients without competing with the demo
const TAGLINE_XS_TOP_PCT: f32 = 70.0;
const TAGLINE_XS_HEIGHT_PCT: f32 = 25.0;
const TAGLINE_XS_LEFT_PCT: f32 = 8.0;
const TAGLINE_XS_WIDTH_PCT: f32 = 84.0;
const TAGLINE_MD_LEFT_PCT: f32 = 78.0;
const TAGLINE_MD_WIDTH_PCT: f32 = 14.0; // ends at 92%, flush with `window_frame`'s own margin
const TAGLINE_MD_TOP_PCT: f32 = 20.0;
const TAGLINE_MD_HEIGHT_PCT: f32 = 65.0;

/// Three glyphs, typed in one at a time, each landing with its own annotation: `kerning`
/// (pseudo -- this engine's text is a fixed monospace grid, so it's just a cosmetic
/// nudge, not a real per-pair feature), `char-width` (real -- a glyph wider than the
/// grid's own reference pitch, same story `type_in.rs`'s own `CELL_PAD` tells), and
/// `char-color` (real -- one glyph rendered in its own distinct `Color`). Once all three
/// have landed, a pause, then every glyph's own `FontSize` jumps to a larger value
/// together (not an `Animation` -- `FontSize` isn't `Animate`, see `wait_before_grow`'s
/// own doc) while a fourth, final label ("font-size") fades in alongside it.
pub fn build(tree: &mut Tree, slot: Entity) {
    let frame = crate::chapters::window_frame(tree, slot);

    let field = tree.branch(
        frame,
        Leaf::sprout()
            .at(Location::new().xs(
                WORD_CENTER_X_PCT.pct().as_center_x().with(
                    TOTAL_COLS
                        .letters()
                        .as_width()
                        .adjust((TOTAL_COLS - 1) * CELL_GAP_PX + KERNING_NUDGE_PX),
                ),
                WORD_TOP_PCT.pct().as_top().with(1.letters().as_height()),
            ))
            .elevate(Elevation::up(2))
            .with((
                Grid::new(1.letters().gap(CELL_GAP_PX), 1.letters()),
                FontSize::new(INITIAL_FONT_SIZE),
            )),
    );

    let seq = tree.sequence();
    let mut letters = [field; 3]; // placeholder -- overwritten below before any read
    for (i, &ch) in GLYPHS.iter().enumerate() {
        let col = i as i32;
        let color = if i == CHAR_COLOR_INDEX {
            Color::amber(CHAR_COLOR_ACCENT)
        } else {
            Color::gray(GLYPH_COLOR)
        };
        let letter = tree.branch(
            field,
            Text::new(ch.to_string())
                .size(FontSize::new(INITIAL_FONT_SIZE))
                .color(color)
                .at(cell_location(col))
                .elevate(Elevation::up(2))
                .with((HorizontalAlignment::Center, Opacity::new(0.0))),
        );
        letters[i] = letter;

        let letter_start = i as u64 * LETTER_STAGGER;
        let letter_finish = letter_start + LETTER_REVEAL;
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .targeting(letter)
                .during(seq)
                .start(letter_start)
                .finish(letter_finish)
                .eased(Ease::Linear),
        );

        let label_start = letter_finish + LABEL_DELAY;
        let label_finish = label_start + LABEL_FADE;
        let label = tree.branch(
            frame,
            Text::new(ANNOTATIONS[i])
                .size(FontSize::new(LABEL_FONT_SIZE))
                .color(Color::slate(LABEL_COLOR))
                .at(annotation_location(ANNOTATION_ABOVE[i]))
                .elevate(Elevation::up(2))
                .with((
                    HorizontalAlignment::Center,
                    VerticalAlignment::Middle,
                    Anchor::new(letter),
                    Opacity::new(0.0),
                )),
        );
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .targeting(label)
                .during(seq)
                .start(label_start)
                .finish(label_finish)
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
            .finish(LETTER_REVEAL)
            .eased(Ease::Linear),
    );

    tree.sequence_end(
        seq,
        move |_: Trigger<OnEnd>, mut tree: Tree, existing: Query<Entity>| {
            if !existing.contains(field) {
                return;
            }
            wait_before_grow(&mut tree, frame, field, letters);
        },
    );
}

fn cell_location(i: i32) -> Location {
    let n = i + 1;
    let nudge = if i == 0 { KERNING_NUDGE_PX } else { 0 };
    Location::new().xs(
        n.col().as_left().adjust(-CELL_PAD + nudge).with(n.col().as_right().adjust(CELL_PAD + nudge)),
        1.row().as_top().with(1.row().as_bottom()),
    )
}

fn annotation_location(above: bool) -> Location {
    let vertical = if above {
        anchor()
            .top()
            .as_bottom()
            .adjust(-LABEL_GAP_PX - LABEL_HEIGHT_PX)
            .with(LABEL_HEIGHT_PX.px().as_height())
    } else {
        anchor()
            .bottom()
            .as_top()
            .adjust(LABEL_GAP_PX)
            .with(LABEL_HEIGHT_PX.px().as_height())
    };
    Location::new().xs(
        anchor()
            .center_x()
            .as_center_x()
            .with(LABEL_WIDTH_PX.px().as_width()),
        vertical,
    )
}

// a real waiting step, not a delay folded into the grow itself -- same reasoning
// `animate.rs`'s/`sequence.rs`'s own `wait_before_opacity` uses: the `font-size` label's
// own fade-in and the actual `FontSize` change need to land at the same moment. A no-op
// `Opacity` tween (`letters[2]`'s already at `1.0`) just gives this a timed `sequence_end`.
// There's no `Animation<FontSize>` in this engine (`FontSize` isn't `Animate` -- only
// `Location`/`Opacity`/`Color`/`Polygon`/`Outline`/`Elevation` are), so the size change
// itself is a plain, instant `write_to`, not a smooth tween.
fn wait_before_grow(tree: &mut Tree, frame: Entity, field: Entity, letters: [Entity; 3]) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(letters[2])
            .during(seq)
            .start(0)
            .finish(GROW_PAUSE)
            .eased(Ease::Linear),
    );
    tree.sequence_end(
        seq,
        move |_: Trigger<OnEnd>, mut tree: Tree, existing: Query<Entity>| {
            // same reasoning as `location.rs`'s own guard: `write_to` is a raw `.insert()`
            // by entity ID with no existence check, unsafe only here because this chain
            // can still be pending after the page is gone (navigating away despawns
            // `field`/`letters` along with the rest of it). An early return rather than a
            // wrapping `if`, so `grow_label` is covered too -- it branches onto `frame`
            // and anchors to `field`, both just as gone as the entities written above.
            if !existing.contains(field) {
                return;
            }
            tree.write_to(field, FontSize::new(GROWN_FONT_SIZE));
            for letter in letters {
                tree.write_to(letter, FontSize::new(GROWN_FONT_SIZE));
            }
            grow_label(&mut tree, frame, field);
        },
    );
}

// final phase -- nothing needs to react once this finishes, so no `sequence_end` at all.
// Anchored to `field` (the whole word), not any one glyph -- every glyph grows together,
// so this names the change as a whole rather than pointing at a single letter.
fn grow_label(tree: &mut Tree, frame: Entity, field: Entity) {
    let label = tree.branch(
        frame,
        Text::new(FONT_SIZE_LABEL_TEXT)
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
                    .adjust(FONT_SIZE_LABEL_GAP_PX)
                    .with(LABEL_HEIGHT_PX.px().as_height()),
            ))
            .elevate(Elevation::up(2))
            .with((
                HorizontalAlignment::Center,
                VerticalAlignment::Middle,
                Anchor::new(field),
                Opacity::new(0.0),
            )),
    );
    let seq = tree.sequence();
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(label)
            .during(seq)
            .start(0)
            .finish(FONT_SIZE_LABEL_FADE)
            .eased(Ease::Linear),
    );
}
