use foliage::{
    Animation, Color, Ease, EcsExtension, Elevation, Entity, FontSize, GridExt,
    HorizontalAlignment, Location, OnEnd, Opacity, Panel, Query, Rounding, Sprout, Text, Tree,
    Trigger, VerticalAlignment,
};

// two solid, overlapping panels -- same geometry `interact.rs` uses (one offset
// down+right from the other), but both filled this time: the point here isn't who
// listens for a click, it's which one *renders in front* in their shared corner, and
// that's only legible if both have real color to show through it.
const PANEL_WIDTH_PCT: f32 = 32.0;
const PANEL_HEIGHT_PCT: f32 = 28.0;
const FRONT_LEFT_PCT: f32 = 20.0;
const FRONT_TOP_PCT: f32 = 22.0;
const OFFSET_PCT: f32 = 14.0; // how far down+right `back` sits from `front`
const BACK_LEFT_PCT: f32 = FRONT_LEFT_PCT + OFFSET_PCT;
const BACK_TOP_PCT: f32 = FRONT_TOP_PCT + OFFSET_PCT;
const PANEL_FADE: u64 = 700; // was 400 -- slower, easier to actually watch

// blue/cyan -- adjacent, not complementary (the old blue/orange pairing was too
// high-contrast) -- still distinguishable in the overlap, just softer.
const FRONT_COLOR: i32 = 500; // blue
const BACK_COLOR: i32 = 500; // cyan -- `back`'s own elevation never changes, only `front`'s does

// `back`'s own elevation never changes -- only `front`'s does, stepping below it then
// back above it. Both start at a real, different `Elevation::up(N)` (not just spawn
// order, which this engine never uses for stacking) so there's an actual value to change.
const BACK_ELEVATION: i32 = 2;
const FRONT_ABOVE_ELEVATION: i32 = 3;
const FRONT_BELOW_ELEVATION: i32 = 1;

const SWAP_FADE_OUT: u64 = 650; // was 350 -- slower, easier to actually watch
const SWAP_FADE_IN: u64 = 650; // was 350
const SWAP_WAIT: u64 = 900; // between settling underneath and swapping back on top
const INTRO_PAUSE: u64 = 800; // between the intro fade-in settling and the first swap starting

const TAGLINE_TEXT: &str = "elevation changes render order";
const TAGLINE_FONT_SIZE: u32 = 13;
const TAGLINE_COLOR: i32 = 400; // slate, muted subtitle -- orients without competing with the demo
const TAGLINE_XS_TOP_PCT: f32 = 2.0; // above `front` -- same empty band `location.rs`'s tagline uses
const TAGLINE_XS_HEIGHT_PCT: f32 = 19.0;
const TAGLINE_XS_LEFT_PCT: f32 = 8.0;
const TAGLINE_XS_WIDTH_PCT: f32 = 84.0;
const TAGLINE_MD_LEFT_PCT: f32 = 78.0;
const TAGLINE_MD_WIDTH_PCT: f32 = 14.0; // ends at 92%, flush with `window_frame`'s own margin
const TAGLINE_MD_TOP_PCT: f32 = FRONT_TOP_PCT;
const TAGLINE_MD_HEIGHT_PCT: f32 = 70.0; // more room than the panels' own combined extent gave it

/// Two solid, overlapping panels -- `front` starts above `back` in their shared corner.
/// `front` fades out, and only *while it's invisible* does its own `Elevation` actually
/// change (below `back`'s) -- there's nothing to see in that swap itself, it's a value
/// changing on a hidden entity, which is the point: elevation is just a component like
/// any other, not something that has to visibly animate to take effect. `front` fades
/// back in already underneath, waits, fades out again, swaps back above, and fades in on
/// top once more.
pub fn build(tree: &mut Tree, slot: Entity) {
    let frame = crate::chapters::window_frame(tree, slot);

    let back = tree.branch(
        frame,
        Panel::new()
            .color(Color::cyan(BACK_COLOR))
            .rounding(Rounding::None)
            .at(Location::new().xs(
                BACK_LEFT_PCT
                    .pct()
                    .as_left()
                    .with(PANEL_WIDTH_PCT.pct().as_width()),
                BACK_TOP_PCT
                    .pct()
                    .as_top()
                    .with(PANEL_HEIGHT_PCT.pct().as_height()),
            ))
            .elevate(Elevation::up(BACK_ELEVATION))
            .with(Opacity::new(0.0)),
    );

    let front = tree.branch(
        frame,
        Panel::new()
            .color(Color::blue(FRONT_COLOR))
            .rounding(Rounding::None)
            .at(Location::new().xs(
                FRONT_LEFT_PCT
                    .pct()
                    .as_left()
                    .with(PANEL_WIDTH_PCT.pct().as_width()),
                FRONT_TOP_PCT
                    .pct()
                    .as_top()
                    .with(PANEL_HEIGHT_PCT.pct().as_height()),
            ))
            .elevate(Elevation::up(FRONT_ABOVE_ELEVATION))
            .with(Opacity::new(0.0)),
    );

    let seq = tree.sequence();
    for panel in [back, front] {
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .targeting(panel)
                .during(seq)
                .start(0)
                .finish(PANEL_FADE)
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
            .finish(PANEL_FADE)
            .eased(Ease::Linear),
    );

    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        wait_before_swap(&mut tree, front);
    });
}

// a real waiting step, not folded into the intro fade-in's own animation -- same
// reasoning `wait_then_swap_back` uses below: a no-op `Opacity` tween (`front`'s
// already at `1.0`) just gives this a timed `sequence_end` before the first swap starts.
fn wait_before_swap(tree: &mut Tree, front: Entity) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(front)
            .during(seq)
            .start(0)
            .finish(INTRO_PAUSE)
            .eased(Ease::Linear),
    );
    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        fade_out_to_below(&mut tree, front);
    });
}

fn fade_out_to_below(tree: &mut Tree, front: Entity) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(Opacity::new(0.0))
            .targeting(front)
            .during(seq)
            .start(0)
            .finish(SWAP_FADE_OUT)
            .eased(Ease::Linear),
    );
    tree.sequence_end(
        seq,
        move |_: Trigger<OnEnd>, mut tree: Tree, existing: Query<Entity>| {
            // `write_to` is a raw `.insert()` by entity ID, no existence check -- safe
            // everywhere else in this app because nothing else calls it from a chain
            // that can still be pending after the page itself is gone (navigating away
            // despawns `front` along with the rest of the page).
            if existing.contains(front) {
                tree.write_to(front, Elevation::up(FRONT_BELOW_ELEVATION));
            }
            fade_in_below(&mut tree, front);
        },
    );
}

fn fade_in_below(tree: &mut Tree, front: Entity) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(front)
            .during(seq)
            .start(0)
            .finish(SWAP_FADE_IN)
            .eased(Ease::Linear),
    );
    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        wait_then_swap_back(&mut tree, front);
    });
}

// a real waiting step, not a delay folded into the next fade's own animation -- same
// reasoning `animate.rs`'s/`sequence.rs`'s own `pause_before_color` uses: a no-op
// `Opacity` tween (`front`'s already at `1.0`) just gives this a timed `sequence_end`.
fn wait_then_swap_back(tree: &mut Tree, front: Entity) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(front)
            .during(seq)
            .start(0)
            .finish(SWAP_WAIT)
            .eased(Ease::Linear),
    );
    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        fade_out_to_above(&mut tree, front);
    });
}

fn fade_out_to_above(tree: &mut Tree, front: Entity) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(Opacity::new(0.0))
            .targeting(front)
            .during(seq)
            .start(0)
            .finish(SWAP_FADE_OUT)
            .eased(Ease::Linear),
    );
    tree.sequence_end(
        seq,
        move |_: Trigger<OnEnd>, mut tree: Tree, existing: Query<Entity>| {
            if existing.contains(front) {
                tree.write_to(front, Elevation::up(FRONT_ABOVE_ELEVATION));
            }
            fade_in_above(&mut tree, front);
        },
    );
}

// final phase -- nothing needs to react once this finishes, so no `sequence_end` at all.
fn fade_in_above(tree: &mut Tree, front: Entity) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(front)
            .during(seq)
            .start(0)
            .finish(SWAP_FADE_IN)
            .eased(Ease::Linear),
    );
}
