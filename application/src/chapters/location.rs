use foliage::{
    Anchor, Animation, Color, Ease, EcsExtension, Elevation, Entity, FontSize, GridExt,
    HorizontalAlignment, Line, Location, OnEnd, Opacity, Polygon, Query, Sprout, Text, TextValue,
    Tree, Trigger, VerticalAlignment, anchor,
};

const HEPTA_LEFT_PCT: f32 = 18.0;
const HEPTA_TOP_PCT: f32 = 35.0;
const HEPTA_WIDTH_PCT: f32 = 26.0;
const HEPTA_HEIGHT_PCT: f32 = 30.0;
const HEPTA_ROUNDING: f32 = 0.15;
const MORPH_DURATION: u64 = 500;

// after the intro settles, `hepta` (carrying `line`/the label along with it, both
// `Anchor`ed to it) moves up, pauses, moves down past rest, pauses, then returns -- a
// live demonstration that a `Location` value is just a number that can change, not a
// one-time placement. The label's own number is the *real* `top` percent driving
// `hepta`'s own position (35 at rest), not an invented one -- a bigger number pushes
// further from the frame's own top edge (down), a smaller one pulls toward it (up), same
// as the real `.as_top()` semantics everywhere else in this app. Each move is chopped
// into `STEPS` small sub-tweens (not one big jump) so the label's own number counts
// smoothly in step with the visible motion, rather than snapping once per phase.
const UP_DOWN_DELTA_PCT: f32 = 10.0;
const UP_TOP_PCT: f32 = HEPTA_TOP_PCT - UP_DOWN_DELTA_PCT; // 25 -- smaller top% -> higher up
const DOWN_TOP_PCT: f32 = HEPTA_TOP_PCT + UP_DOWN_DELTA_PCT; // 45 -- bigger top% -> further down
const MOVE_DURATION: u64 = 500;
const MOVE_PAUSE: u64 = 400;
const STEPS: u32 = 20;
const STEP_MS: u64 = MOVE_DURATION / STEPS as u64;

// off the heptagon's own right edge, at its own vertical center -- anchored to `hepta`
// itself, not the window frame, so the line and label visibly come off the shape.
const LINE_START_GAP_PX: i32 = 10; // clearance off the heptagon's edge before the line starts
const LINE_LENGTH_PX: i32 = 40;
const LINE_FADE: u64 = 250;

const BLUEPRINT_COLOR: i32 = 500; // slate

const LABEL_GAP_PX: i32 = 10; // between the line's outer end and the label
const LABEL_FONT_SIZE: u32 = 13;
const LABEL_WIDTH_PX: i32 = 110;
const LABEL_ROW_HEIGHT_PX: i32 = 23; // each of the label's two manually-split rows
const LABEL_FADE: u64 = 300;
const LABEL_DELAY: u64 = 150; // after the line finishes drawing in

const TAGLINE_TEXT: &str = "use location to place on screen";
const TAGLINE_FONT_SIZE: u32 = 13;
const TAGLINE_COLOR: i32 = 400; // slate, muted subtitle -- orients without competing with the demo
/// `Xs` (portrait, narrow but tall) stacks this above the demo, in the frame's own empty
/// band before `HEPTA_TOP_PCT` -- there's no room to its side that narrow. `Md`+
/// (landscape phones included -- rotating a phone widens it well past the `Md` threshold,
/// same reasoning `toc.rs`'s own breakpoints lean on) has the opposite shape: width to
/// spare, height scarce, so it goes beside the demo instead, anchored past the label's own
/// right edge (stable -- `hepta` only ever animates its own `top`, never `left`/`width`,
/// so this never drifts during the up/down move) rather than stacked above it, which would
/// eat into the little vertical room landscape actually has.
const TAGLINE_XS_TOP_PCT: f32 = 6.0;
const TAGLINE_XS_HEIGHT_PCT: f32 = 24.0;
const TAGLINE_XS_LEFT_PCT: f32 = 8.0;
const TAGLINE_XS_WIDTH_PCT: f32 = 84.0;
const TAGLINE_MD_GAP_PX: i32 =
    LINE_START_GAP_PX + LINE_LENGTH_PX + LABEL_GAP_PX + LABEL_WIDTH_PX + 24;
const TAGLINE_MD_WIDTH_PX: i32 = 150;

/// Where an entity sits -- percent/px/point values as a live, changeable number, not a
/// one-time placement (`relative.rs` is the one that makes *what* those percentages
/// resolve against visible). A heptagon morphs in (green, the triangle -> heptagon
/// technique used throughout this app), then a short blueprint-style line ticks out
/// `LINE_LENGTH_PX` from its own right edge, then a two-row `Location` label fades in past
/// its end -- both anchored to the heptagon itself, so they visibly come off the shape
/// rather than the window frame around it. A muted tagline orients the reader on what
/// they're looking at -- stacked above the demo on `Xs` (portrait's vertical room to
/// spare), beside it past the label on `Md`+ (landscape's horizontal room to spare
/// instead).
pub fn build(tree: &mut Tree, slot: Entity) {
    let frame = crate::chapters::window_frame(tree, slot);

    let hepta = tree.branch(
        frame,
        Polygon::new()
            .sides(3.0)
            .rounding(0.0)
            .rotation(0.0)
            .color(Color::green(400))
            .at(Location::new().xs(
                HEPTA_LEFT_PCT
                    .pct()
                    .as_left()
                    .with(HEPTA_WIDTH_PCT.pct().as_width()),
                HEPTA_TOP_PCT
                    .pct()
                    .as_top()
                    .with(HEPTA_HEIGHT_PCT.pct().as_height()),
            ))
            .elevate(Elevation::up(2))
            .with(Opacity::new(0.0)),
    );

    let seq = tree.sequence();
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(hepta)
            .during(seq)
            .start(0)
            .finish(MORPH_DURATION)
            .eased(Ease::Linear),
    );
    tree.animate(
        Animation::new(Polygon {
            sides: 7.0,
            rounding: HEPTA_ROUNDING,
            rotation: 0.0,
        })
        .targeting(hepta)
        .during(seq)
        .start(0)
        .finish(MORPH_DURATION)
        .eased(Ease::DECELERATE),
    );

    // line: heptagon's own right edge -> `LINE_START_GAP_PX` + `LINE_LENGTH_PX` further
    // right, at its own vertical center -- `Anchor::new(hepta)` is what makes both ends
    // resolve off the heptagon's *live* box, not the window frame.
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
            .with((Anchor::new(hepta), Opacity::new(0.0))),
    );
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(line)
            .during(seq)
            .start(MORPH_DURATION)
            .finish(MORPH_DURATION + LINE_FADE)
            .eased(Ease::Linear),
    );

    // plausible `Location` syntax, not the literal call above -- `.as_top()`, not
    // `.as_left()`, since what's actually being demonstrated (the up/pause/down/pause/rest
    // move below) changes `hepta`'s own top position, not a left/right one. Deliberately
    // not `anchor()`: that's a different, later chapter, and this page is teaching
    // `Location`, not `Anchor`. Split across two `Text` entities at the natural
    // `.` boundary instead of one wrapping entity -- this renderer is a fixed monospace
    // character grid (see `MonospacedFont`), not word-aware text shaping, so a single
    // entity wide enough to hold the whole string would wrap mid-word wherever the box
    // happens to end rather than at a sensible break.
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
                Anchor::new(hepta),
                Opacity::new(0.0),
            ))
    };
    let label_line_1 = tree.branch(frame, label_row(&format!("{HEPTA_TOP_PCT}.pct()"), 0));
    let label_line_2 = tree.branch(frame, label_row(".as_top()", 1));
    for label in [label_line_1, label_line_2] {
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .targeting(label)
                .during(seq)
                .start(MORPH_DURATION + LINE_FADE + LABEL_DELAY)
                .finish(MORPH_DURATION + LINE_FADE + LABEL_DELAY + LABEL_FADE)
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
                    HEPTA_TOP_PCT
                        .pct()
                        .as_top()
                        .with(HEPTA_HEIGHT_PCT.pct().as_height()),
                ))
            .elevate(Elevation::up(2))
            .with((
                HorizontalAlignment::Left,
                VerticalAlignment::Middle,
                Anchor::new(hepta),
                Opacity::new(0.0),
            )),
    );
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(tagline)
            .during(seq)
            .start(0)
            .finish(MORPH_DURATION)
            .eased(Ease::Linear),
    );

    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        move_up(&mut tree, hepta, label_line_1);
    });
}

fn hepta_location(top_pct: f32) -> Location {
    Location::new().xs(
        HEPTA_LEFT_PCT
            .pct()
            .as_left()
            .with(HEPTA_WIDTH_PCT.pct().as_width()),
        top_pct
            .pct()
            .as_top()
            .with(HEPTA_HEIGHT_PCT.pct().as_height()),
    )
}

fn move_up(tree: &mut Tree, hepta: Entity, label_line_1: Entity) {
    step(
        tree,
        hepta,
        label_line_1,
        HEPTA_TOP_PCT,
        UP_TOP_PCT,
        0,
        0,
        Some(move_down),
    );
}

fn move_down(tree: &mut Tree, hepta: Entity, label_line_1: Entity) {
    step(
        tree,
        hepta,
        label_line_1,
        UP_TOP_PCT,
        DOWN_TOP_PCT,
        0,
        MOVE_PAUSE,
        Some(move_rest),
    );
}

fn move_rest(tree: &mut Tree, hepta: Entity, label_line_1: Entity) {
    step(
        tree,
        hepta,
        label_line_1,
        DOWN_TOP_PCT,
        HEPTA_TOP_PCT,
        0,
        MOVE_PAUSE,
        None,
    );
}

/// One `STEP_MS`-long slice of a `from_pct -> to_pct` move -- chaining `STEPS` of these
/// (via each one's own `sequence_end`) is what makes both `hepta`'s motion and the
/// label's own number read as one continuous change instead of `STEPS` separate snaps.
/// `pause_before` only ever applies to a phase's first step (every step after that starts
/// immediately once the previous one's `sequence_end` fires).
fn step(
    tree: &mut Tree,
    hepta: Entity,
    label_line_1: Entity,
    from_pct: f32,
    to_pct: f32,
    i: u32,
    pause_before: u64,
    next: Option<fn(&mut Tree, Entity, Entity)>,
) {
    let target_pct = from_pct + (to_pct - from_pct) * (i + 1) as f32 / STEPS as f32;
    let seq = tree.sequence();
    let start = if i == 0 { pause_before } else { 0 };
    tree.animate(
        Animation::new(hepta_location(target_pct))
            .targeting(hepta)
            .during(seq)
            .start(start)
            .finish(start + STEP_MS)
            .eased(Ease::Linear),
    );
    tree.sequence_end(
        seq,
        move |_: Trigger<OnEnd>, mut tree: Tree, existing: Query<Entity>| {
            // `write_to` is a raw `.insert()` by entity ID, no existence check -- safe
            // everywhere else in this app because nothing else calls it from a chain that
            // can still be pending after the page itself is gone (navigating away
            // despawns the whole chapter's entity tree, `label_line_1` included). Every
            // other command below (`animate`, `step`, `next_fn`) is already safe to fire
            // against a despawned target on its own -- the animation system checks its
            // target exists each frame and quietly cleans itself up if not.
            if existing.contains(label_line_1) {
                tree.write_to(
                    label_line_1,
                    TextValue(format!("{}.pct()", target_pct.round() as i32)),
                );
            }
            if i + 1 < STEPS {
                step(
                    &mut tree,
                    hepta,
                    label_line_1,
                    from_pct,
                    to_pct,
                    i + 1,
                    pause_before,
                    next,
                );
            } else if let Some(next_fn) = next {
                next_fn(&mut tree, hepta, label_line_1);
            }
        },
    );
}
