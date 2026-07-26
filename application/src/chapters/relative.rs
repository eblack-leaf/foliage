use foliage::{
    Anchor, Animation, Color, Ease, EcsExtension, Elevation, Entity, FontSize, Grid, GridExt,
    HorizontalAlignment, Line, Location, OnEnd, Opacity, Panel, Polygon, Query, Rounding, Sprout,
    Text, TextValue, Tree, Trigger, VerticalAlignment, anchor,
};

// matches `location.rs`'s own hepta proportions (`HEPTA_LEFT_PCT`/`WIDTH_PCT`) exactly --
// already proven to leave enough room for the line + label's fixed `LINE_START_GAP_PX +
// LINE_LENGTH_PX + LABEL_GAP_PX + LABEL_WIDTH_PX` (170px) past the right edge without
// running past the frame at narrow widths. `width` stays static (unlike `height` below) --
// growing it would push that same right-anchored line/label even further right, fighting
// the exact overflow this is fixing.
const PANEL_LEFT_PCT: f32 = 18.0;
const PANEL_WIDTH_PCT: f32 = 26.0;
const PANEL_TOP_PCT: f32 = 20.0;
const PANEL_HEIGHT_PCT: f32 = 40.0; // was 55 -- too tall
const PANEL_COLOR: i32 = 600; // slate -- a visible, but muted, "this is the parent" backdrop
const PANEL_FADE: u64 = 400;

// after the intro settles, `panel` grows, pauses, shrinks past rest, pauses, then
// returns -- `child` (positioned entirely in percent of `panel`'s own box) visibly
// rescales in lockstep, live, proving its `Location` genuinely resolves against the
// parent's current box rather than a one-time snapshot of it. `height`, not `width`
// (see `PANEL_WIDTH_PCT`'s own doc) -- `top` stays fixed, so this only ever moves
// `panel`'s *bottom* edge, never its right one, leaving the line/label's own horizontal
// anchor point undisturbed.
const HEIGHT_DELTA_PCT: f32 = 10.0;
const GROW_HEIGHT_PCT: f32 = PANEL_HEIGHT_PCT + HEIGHT_DELTA_PCT;
const SHRINK_HEIGHT_PCT: f32 = PANEL_HEIGHT_PCT - HEIGHT_DELTA_PCT;
const RESIZE_DURATION: u64 = 500;
const RESIZE_PAUSE: u64 = 400;
const RESIZE_STEPS: u32 = 20;
const RESIZE_STEP_MS: u64 = RESIZE_DURATION / RESIZE_STEPS as u64;

// child's own percent values are relative to `panel`'s box, not the window frame's --
// the whole point of this page, made visible by giving the parent an actual backdrop
// instead of leaving it implicit (as `location.rs`'s own window frame is).
const CHILD_LEFT_PCT: f32 = 22.0;
const CHILD_TOP_PCT: f32 = 28.0;
const CHILD_WIDTH_PCT: f32 = 45.0;
const CHILD_HEIGHT_PCT: f32 = 40.0;
const CHILD_ROUNDING: f32 = 0.15;
const MORPH_DELAY: u64 = 200; // after the panel starts fading in
const MORPH_DURATION: u64 = 500;

// off the panel's own right edge, at its own vertical center -- anchored to `panel`
// itself (not `child`, the way `location.rs`'s line anchors to its own polygon), so the
// line and label visibly come off the *parent*, not the thing placed inside it.
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

const TAGLINE_TEXT: &str = "child position resolves against its parent";
const TAGLINE_FONT_SIZE: u32 = 13;
const TAGLINE_COLOR: i32 = 400; // slate, muted subtitle -- orients without competing with the demo
/// Same reasoning as `location.rs`'s own tagline, mirrored for this page's own layout:
/// `Xs` (portrait, narrow but tall) puts it below the demo instead of above -- unlike
/// `location.rs`'s hepta (which sits low in its frame, room to spare above it), `panel`
/// sits high (`PANEL_TOP_PCT` = 20), so the frame's own empty band is below it
/// (`PANEL_TOP_PCT + GROW_HEIGHT_PCT` = 70% at most, even mid-animation), not above.
/// `Md`+ (landscape's width to spare instead) still goes beside it, anchored past the
/// label's own right edge -- stable, since `panel` only ever animates `height`, never
/// `left`/`width`, so this never drifts during the grow/shrink move. Its own vertical
/// band uses `panel`'s *rest* position, not an `Anchor` to its live one, so it stays
/// put through the animation instead of resizing along with `panel`.
const TAGLINE_XS_TOP_PCT: f32 = 74.0;
const TAGLINE_XS_HEIGHT_PCT: f32 = 25.0;
const TAGLINE_XS_LEFT_PCT: f32 = 8.0;
const TAGLINE_XS_WIDTH_PCT: f32 = 84.0;
const TAGLINE_MD_GAP_PX: i32 =
    LINE_START_GAP_PX + LINE_LENGTH_PX + LABEL_GAP_PX + LABEL_WIDTH_PX + 24;
const TAGLINE_MD_WIDTH_PX: i32 = 150;
// dedicated, not `PANEL_HEIGHT_PCT` reused -- that's `panel`'s own visual size (and it
// animates), not how much room this column actually needs to wrap into.
const TAGLINE_MD_HEIGHT_PCT: f32 = 60.0;

/// A child's `Location` percent resolves relative to its own parent's box, not the
/// window frame -- `location.rs` already teaches this, but its "parent" (the window
/// frame) is invisible, so the relationship itself is never actually seen. Here the
/// parent gets a real, visible backdrop (`panel`): a child heptagon morphs in *inside*
/// it, positioned entirely in percent of `panel`'s own box, then a blueprint-style line
/// ticks out from `panel`'s own right edge (not the child's) into a two-row label
/// naming `panel`'s own real height -- both anchored to the parent, so they visibly come
/// off it instead of off the shape placed inside it. Once that settles, `panel` grows,
/// pauses, shrinks past rest, pauses, then returns -- `child` rescales in lockstep the
/// whole time, live proof its `Location` tracks the parent's *current* box, not a
/// snapshot of it taken once at spawn.
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
            .with((Grid::new(1.col().gap(0), 1.row().gap(0)), Opacity::new(0.0))),
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

    let child = tree.branch(
        panel,
        Polygon::new()
            .sides(3.0)
            .rounding(0.0)
            .rotation(0.0)
            .color(Color::orange(400))
            .at(Location::new().xs(
                CHILD_LEFT_PCT
                    .pct()
                    .as_left()
                    .with(CHILD_WIDTH_PCT.pct().as_width()),
                CHILD_TOP_PCT
                    .pct()
                    .as_top()
                    .with(CHILD_HEIGHT_PCT.pct().as_height()),
            ))
            .elevate(Elevation::up(3))
            .with(Opacity::new(0.0)),
    );
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(child)
            .during(seq)
            .start(MORPH_DELAY)
            .finish(MORPH_DELAY + MORPH_DURATION)
            .eased(Ease::Linear),
    );
    tree.animate(
        Animation::new(Polygon {
            sides: 7.0,
            rounding: CHILD_ROUNDING,
            rotation: 0.0,
        })
        .targeting(child)
        .during(seq)
        .start(MORPH_DELAY)
        .finish(MORPH_DELAY + MORPH_DURATION)
        .eased(Ease::DECELERATE),
    );

    // line: panel's own right edge -> `LINE_START_GAP_PX` + `LINE_LENGTH_PX` further
    // right, at its own vertical center -- `Anchor::new(panel)` is what makes both ends
    // resolve off the parent's *live* box, not the window frame or the child inside it.
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
            .start(MORPH_DELAY + MORPH_DURATION)
            .finish(MORPH_DELAY + MORPH_DURATION + LINE_FADE)
            .eased(Ease::Linear),
    );

    // plausible `Location` syntax naming `panel`'s own real height -- not the child's,
    // since the line/label come off the parent this time, and `height` (not `width`) is
    // what the grow/shrink/rest move below actually animates. Split across two `Text`
    // entities at the natural `.` boundary, same reasoning `location.rs`'s own label
    // uses: this renderer is a fixed monospace character grid (see `MonospacedFont`),
    // not word-aware text shaping, so one entity wide enough for the whole string would
    // wrap mid-word wherever the box happens to end rather than at a sensible break.
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
    let label_line_1 = tree.branch(frame, label_row(&format!("{PANEL_HEIGHT_PCT}.pct()"), 0));
    let label_line_2 = tree.branch(frame, label_row(".as_height()", 1));
    for label in [label_line_1, label_line_2] {
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .targeting(label)
                .during(seq)
                .start(MORPH_DELAY + MORPH_DURATION + LINE_FADE + LABEL_DELAY)
                .finish(MORPH_DELAY + MORPH_DURATION + LINE_FADE + LABEL_DELAY + LABEL_FADE)
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

    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        grow(&mut tree, panel, label_line_1);
    });
}

fn panel_location(height_pct: f32) -> Location {
    Location::new().xs(
        PANEL_LEFT_PCT
            .pct()
            .as_left()
            .with(PANEL_WIDTH_PCT.pct().as_width()),
        PANEL_TOP_PCT
            .pct()
            .as_top()
            .with(height_pct.pct().as_height()),
    )
}

fn grow(tree: &mut Tree, panel: Entity, label_line_1: Entity) {
    resize_step(
        tree,
        panel,
        label_line_1,
        PANEL_HEIGHT_PCT,
        GROW_HEIGHT_PCT,
        0,
        0,
        Some(shrink),
    );
}

fn shrink(tree: &mut Tree, panel: Entity, label_line_1: Entity) {
    resize_step(
        tree,
        panel,
        label_line_1,
        GROW_HEIGHT_PCT,
        SHRINK_HEIGHT_PCT,
        0,
        RESIZE_PAUSE,
        Some(rest),
    );
}

fn rest(tree: &mut Tree, panel: Entity, label_line_1: Entity) {
    resize_step(
        tree,
        panel,
        label_line_1,
        SHRINK_HEIGHT_PCT,
        PANEL_HEIGHT_PCT,
        0,
        RESIZE_PAUSE,
        None,
    );
}

/// One `RESIZE_STEP_MS`-long slice of a `from_pct -> to_pct` resize -- chaining
/// `RESIZE_STEPS` of these (via each one's own `sequence_end`, same technique
/// `location.rs`'s own `step` uses for its up/down move) is what makes both `panel`'s
/// resize and the label's own number read as one continuous change instead of
/// `RESIZE_STEPS` separate snaps.
fn resize_step(
    tree: &mut Tree,
    panel: Entity,
    label_line_1: Entity,
    from_pct: f32,
    to_pct: f32,
    i: u32,
    pause_before: u64,
    next: Option<fn(&mut Tree, Entity, Entity)>,
) {
    let target_pct = from_pct + (to_pct - from_pct) * (i + 1) as f32 / RESIZE_STEPS as f32;
    let seq = tree.sequence();
    let start = if i == 0 { pause_before } else { 0 };
    tree.animate(
        Animation::new(panel_location(target_pct))
            .targeting(panel)
            .during(seq)
            .start(start)
            .finish(start + RESIZE_STEP_MS)
            .eased(Ease::Linear),
    );
    tree.sequence_end(
        seq,
        move |_: Trigger<OnEnd>, mut tree: Tree, existing: Query<Entity>| {
            // same reasoning as `location.rs`'s own guard: `write_to` is a raw `.insert()`
            // by entity ID with no existence check, unsafe only here because this is the
            // one chain in this file that can still be pending after the page is gone
            // (navigating away despawns `label_line_1` along with the rest of the page).
            if existing.contains(label_line_1) {
                tree.write_to(
                    label_line_1,
                    TextValue(format!("{}.pct()", target_pct.round() as i32)),
                );
            }
            if i + 1 < RESIZE_STEPS {
                resize_step(
                    &mut tree,
                    panel,
                    label_line_1,
                    from_pct,
                    to_pct,
                    i + 1,
                    pause_before,
                    next,
                );
            } else if let Some(next_fn) = next {
                next_fn(&mut tree, panel, label_line_1);
            }
        },
    );
}
