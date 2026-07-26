use foliage::{
    Anchor, Animation, Color, Ease, EcsExtension, Elevation, Entity, FontSize, GridExt,
    HorizontalAlignment, Location, OnEnd, Opacity, Polygon, Query, Sprout, Text, TextValue, Tree,
    Trigger, VerticalAlignment, anchor,
};

const POLY_LEFT_PCT: f32 = 18.0; // same proportions `location.rs`'s own hepta uses
const POLY_TOP_PCT: f32 = 35.0;
const POLY_WIDTH_PCT: f32 = 26.0;
const POLY_HEIGHT_PCT: f32 = 30.0;
const POLY_ROUNDING: f32 = 0.15;
const POLY_COLOR: i32 = 400; // green
const CHANGED_COLOR: i32 = 400; // purple -- the "color" phase's target, then back to green
const MORPH_DURATION: u64 = 500;

const LABEL_COLOR: i32 = 500; // slate, same blueprint tone the other chapters use
const LABEL_FONT_SIZE: u32 = 13;
const LABEL_GAP_PX: i32 = 10; // below `poly`'s own bottom edge
const LABEL_WIDTH_PX: i32 = 130; // wide enough for "N-Mms" -- longer than a bare duration
const LABEL_HEIGHT_PX: i32 = 20;
const LABEL_FADE: u64 = 300;
const LABEL_DELAY: u64 = 150; // after `poly` finishes morphing in

// the exact same three phases `animate.rs`'s own demo chains -- `location`
// (up/pause/down past rest/pause/rest), `opacity` (1 -> 0 -> 1), then `color`
// (green -> purple -> green) -- because this page's own point isn't "a component is a
// value" (that's `animate.rs`'s job), it's that a `Sequence` is what actually chains
// them: each phase is its own `tree.sequence()`, and the *next* phase only ever starts
// from that sequence's own `sequence_end` firing, not a guessed delay. The label names
// each phase's own *absolute* span (`0-2300ms`, `2300-4300ms`, `4300-6500ms`) rather than
// just a bare duration, snapping to the new range right as its sequence starts -- since
// consecutive phases chain directly off one another's `sequence_end`, each range's start
// is exactly the previous one's end, with no gap.
const LOCATION_DELTA_PCT: f32 = 10.0;
const UP_PCT: f32 = POLY_TOP_PCT - LOCATION_DELTA_PCT;
const DOWN_PCT: f32 = POLY_TOP_PCT + LOCATION_DELTA_PCT;
const LOCATION_MOVE_DURATION: u64 = 500;
const LOCATION_PAUSE: u64 = 400;
// a real waiting step before `opacity` starts too (see `wait_before_opacity`), same
// reasoning `AFTER_OPACITY_PAUSE` uses -- folded into `LOCATION_END` below since nothing
// new starts until it elapses.
const BEFORE_OPACITY_PAUSE: u64 = 800;
// `up` (0..500) + `down` (pause 400 + duration 500) + `rest` (pause 400 + duration 500) +
// the extra `wait_before_opacity` wait -- the absolute moment `wait_before_opacity`'s own
// `sequence_end` fires and writes the next label.
const LOCATION_END: u64 =
    LOCATION_MOVE_DURATION + (LOCATION_PAUSE + LOCATION_MOVE_DURATION) * 2 + BEFORE_OPACITY_PAUSE;

const OPACITY_DURATION: u64 = 400;
const OPACITY_PAUSE: u64 = 400;
const AFTER_OPACITY_PAUSE: u64 = 800;
// `fade_out` + `fade_in` (pause + duration) + the extra `pause_before_color` wait --
// folded into this same range since nothing new starts until it elapses; the moment
// `pause_before_color`'s own `sequence_end` fires and writes the "color" label.
const OPACITY_END: u64 =
    LOCATION_END + OPACITY_DURATION + (OPACITY_PAUSE + OPACITY_DURATION) + AFTER_OPACITY_PAUSE;

const COLOR_DURATION: u64 = 900;
const COLOR_PAUSE: u64 = 400;
// `change_color` + `change_color_back` (pause + duration) -- the absolute moment
// everything finishes; nothing reacts after this, so it's the label's own last range end.
const COLOR_END: u64 = OPACITY_END + COLOR_DURATION + (COLOR_PAUSE + COLOR_DURATION);

const TAGLINE_TEXT: &str = "sequences coordinate many animations";
const TAGLINE_FONT_SIZE: u32 = 13;
const TAGLINE_COLOR: i32 = 400; // slate, muted subtitle -- orients without competing with the demo
/// `Xs` (portrait) stacks this above the demo, in the frame's own empty band before
/// `POLY_TOP_PCT` -- same band `location.rs`'s own tagline uses, since `poly` sits at the
/// same rest position its hepta does. `Md`+ goes beside it instead, anchored past
/// `poly`'s own right edge -- stable, since `poly` only ever animates `top` (during the
/// `location` phase) or `Opacity`/`Color` (never `left`/`width`), so this never drifts.
const TAGLINE_XS_TOP_PCT: f32 = 6.0;
const TAGLINE_XS_HEIGHT_PCT: f32 = 32.0;
const TAGLINE_XS_LEFT_PCT: f32 = 8.0;
const TAGLINE_XS_WIDTH_PCT: f32 = 84.0;
const TAGLINE_MD_GAP_PX: i32 = 56; // was 24, then 40 -- still too close to `poly`'s own right edge
const TAGLINE_MD_WIDTH_PX: i32 = 150;
const TAGLINE_MD_TOP_PCT: f32 = 15.0; // was `POLY_TOP_PCT` (35) -- centered too far down against `poly`
const TAGLINE_MD_HEIGHT_PCT: f32 = 55.0; // dedicated, not `POLY_HEIGHT_PCT` reused -- more room to wrap

/// The same three-phase demo `animate.rs` uses (`Location`, then `Opacity`, then `Color`),
/// but this page's point is the chaining itself, not the properties: each phase is its
/// own `tree.sequence()`, and the next phase's `tree.animate(..)` calls are only ever
/// made from the previous one's `sequence_end` actually firing -- never a fixed delay
/// guessed to land after the last one finishes. The label names each phase's own
/// *absolute* time range (`0-2300ms`, then `2300-4300ms`, then `4300-6500ms`), snapping
/// to the new range the instant its sequence starts -- each one's start is exactly the
/// previous phase's end, with no gap, since that's literally when the chain hands off.
pub fn build(tree: &mut Tree, slot: Entity) {
    let frame = crate::chapters::window_frame(tree, slot);

    let poly = tree.branch(
        frame,
        Polygon::new()
            .sides(3.0)
            .rounding(0.0)
            .rotation(0.0)
            .color(Color::green(POLY_COLOR))
            .at(poly_location(POLY_TOP_PCT))
            .elevate(Elevation::up(2))
            .with(Opacity::new(0.0)),
    );

    let seq = tree.sequence();
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(poly)
            .during(seq)
            .start(0)
            .finish(MORPH_DURATION)
            .eased(Ease::Linear),
    );
    tree.animate(
        Animation::new(Polygon {
            sides: 7.0,
            rounding: POLY_ROUNDING,
            rotation: 0.0,
        })
        .targeting(poly)
        .during(seq)
        .start(0)
        .finish(MORPH_DURATION)
        .eased(Ease::DECELERATE),
    );

    let label = tree.branch(
        frame,
        Text::new(format!("0-{LOCATION_END}ms"))
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
                Anchor::new(poly),
                Opacity::new(0.0),
            )),
    );
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(label)
            .during(seq)
            .start(MORPH_DURATION + LABEL_DELAY)
            .finish(MORPH_DURATION + LABEL_DELAY + LABEL_FADE)
            .eased(Ease::Linear),
    );

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
                    TAGLINE_MD_TOP_PCT
                        .pct()
                        .as_top()
                        .with(TAGLINE_MD_HEIGHT_PCT.pct().as_height()),
                ))
            .elevate(Elevation::up(2))
            .with((
                HorizontalAlignment::Left,
                VerticalAlignment::Middle,
                Anchor::new(poly),
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
        move_up(&mut tree, poly, label);
    });
}

fn poly_location(top_pct: f32) -> Location {
    Location::new().xs(
        POLY_LEFT_PCT
            .pct()
            .as_left()
            .with(POLY_WIDTH_PCT.pct().as_width()),
        top_pct.pct().as_top().with(POLY_HEIGHT_PCT.pct().as_height()),
    )
}

fn move_up(tree: &mut Tree, poly: Entity, label: Entity) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(poly_location(UP_PCT))
            .targeting(poly)
            .during(seq)
            .start(0)
            .finish(LOCATION_MOVE_DURATION)
            .eased(Ease::EMPHASIS),
    );
    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        move_down(&mut tree, poly, label);
    });
}

fn move_down(tree: &mut Tree, poly: Entity, label: Entity) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(poly_location(DOWN_PCT))
            .targeting(poly)
            .during(seq)
            .start(LOCATION_PAUSE)
            .finish(LOCATION_PAUSE + LOCATION_MOVE_DURATION)
            .eased(Ease::EMPHASIS),
    );
    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        move_rest(&mut tree, poly, label);
    });
}

fn move_rest(tree: &mut Tree, poly: Entity, label: Entity) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(poly_location(POLY_TOP_PCT))
            .targeting(poly)
            .during(seq)
            .start(LOCATION_PAUSE)
            .finish(LOCATION_PAUSE + LOCATION_MOVE_DURATION)
            .eased(Ease::EMPHASIS),
    );
    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        wait_before_opacity(&mut tree, poly, label);
    });
}

// a real waiting step, not a delay folded into `fade_out`'s own animation -- same
// reasoning `pause_before_color` uses: the label switch and the animation's actual start
// need to land at the same moment. A no-op `Location` tween (`poly`'s already at
// `POLY_TOP_PCT`) just gives this a timed `sequence_end`.
fn wait_before_opacity(tree: &mut Tree, poly: Entity, label: Entity) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(poly_location(POLY_TOP_PCT))
            .targeting(poly)
            .during(seq)
            .start(0)
            .finish(BEFORE_OPACITY_PAUSE)
            .eased(Ease::Linear),
    );
    tree.sequence_end(
        seq,
        move |_: Trigger<OnEnd>, mut tree: Tree, existing: Query<Entity>| {
            // same reasoning as `location.rs`'s own guard: `write_to` is a raw `.insert()`
            // by entity ID with no existence check, unsafe only here because this chain
            // can still be pending after the page is gone (navigating away despawns
            // `label` along with the rest of it).
            if existing.contains(label) {
                tree.write_to(label, TextValue(format!("{LOCATION_END}-{OPACITY_END}ms")));
            }
            fade_out(&mut tree, poly, label);
        },
    );
}

fn fade_out(tree: &mut Tree, poly: Entity, label: Entity) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(Opacity::new(0.0))
            .targeting(poly)
            .during(seq)
            .start(0)
            .finish(OPACITY_DURATION)
            .eased(Ease::Linear),
    );
    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        fade_in(&mut tree, poly, label);
    });
}

fn fade_in(tree: &mut Tree, poly: Entity, label: Entity) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(poly)
            .during(seq)
            .start(OPACITY_PAUSE)
            .finish(OPACITY_PAUSE + OPACITY_DURATION)
            .eased(Ease::Linear),
    );
    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        pause_before_color(&mut tree, poly, label);
    });
}

// a real waiting step, not just a `.start(delay)` baked into `change_color`'s own
// animation -- `write_to` and the color tween itself need to land at the *same* moment,
// and a bare internal delay only pushes the animation's own start back, leaving the
// label switched to the color phase's duration for the whole pause while the shape
// hasn't changed at all yet. A no-op `Opacity` tween (poly's already at `1.0`) just gives
// this a timed `sequence_end` to hang both off of together.
fn pause_before_color(tree: &mut Tree, poly: Entity, label: Entity) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(poly)
            .during(seq)
            .start(0)
            .finish(AFTER_OPACITY_PAUSE)
            .eased(Ease::Linear),
    );
    tree.sequence_end(
        seq,
        move |_: Trigger<OnEnd>, mut tree: Tree, existing: Query<Entity>| {
            if existing.contains(label) {
                tree.write_to(label, TextValue(format!("{OPACITY_END}-{COLOR_END}ms")));
            }
            change_color(&mut tree, poly);
        },
    );
}

fn change_color(tree: &mut Tree, poly: Entity) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(Color::purple(CHANGED_COLOR))
            .targeting(poly)
            .during(seq)
            .start(0)
            .finish(COLOR_DURATION)
            .eased(Ease::Linear),
    );
    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        change_color_back(&mut tree, poly);
    });
}

// final phase -- nothing needs to react once this finishes, so no `sequence_end` at all.
fn change_color_back(tree: &mut Tree, poly: Entity) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(Color::green(POLY_COLOR))
            .targeting(poly)
            .during(seq)
            .start(COLOR_PAUSE)
            .finish(COLOR_PAUSE + COLOR_DURATION)
            .eased(Ease::Linear),
    );
}
