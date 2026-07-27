use foliage::{
    Animation, Color, Ease, EcsExtension, Elevation, Entity, FontSize, GridExt,
    HorizontalAlignment, Location, OnEnd, Opacity, Polygon, Query, Sprout, Text, TextValue, Tree,
    Trigger, VerticalAlignment,
};

const POLY_LEFT_PCT: f32 = 18.0; // same proportions `location.rs`'s own hepta uses
const POLY_TOP_PCT: f32 = 35.0;
const POLY_WIDTH_PCT: f32 = 26.0;
const POLY_HEIGHT_PCT: f32 = 30.0;
const POLY_ROUNDING: f32 = 0.15;
const MORPH_DURATION: u64 = 500;

// green is both the rest color and where the last stage returns to -- three stages,
// not two, so the loop actually reads as "a value moving through several states", not
// just "a value and its opposite".
const REST_COLOR: i32 = 400; // green
const STAGE_COLOR: i32 = 400; // shared shade for the purple/amber stages in between
const STAGE_DURATION: u64 = 1400; // slow enough that the interpolation itself is the point
const STAGE_PAUSE: u64 = 900; // a real waiting step between stages -- see `wait_before_stage2`

const LABEL_COLOR: i32 = 500; // slate, same blueprint tone the other chapters use
const LABEL_FONT_SIZE: u32 = 13;
const LABEL_GAP_PCT: f32 = 5.0; // below `poly`'s own bottom edge
const LABEL_TOP_PCT: f32 = POLY_TOP_PCT + POLY_HEIGHT_PCT + LABEL_GAP_PCT;
const LABEL_HEIGHT_PCT: f32 = 10.0;
const LABEL_FADE: u64 = 300;
const LABEL_DELAY: u64 = 150; // after `poly` finishes morphing in

const TAGLINE_TEXT: &str = "interpolate between values for smooth transitions";
const TAGLINE_FONT_SIZE: u32 = 13;
const TAGLINE_COLOR: i32 = 400; // slate, muted subtitle -- orients without competing with the demo
const TAGLINE_XS_TOP_PCT: f32 = 6.0;
const TAGLINE_XS_HEIGHT_PCT: f32 = 40.0;
const TAGLINE_XS_LEFT_PCT: f32 = 8.0;
const TAGLINE_XS_WIDTH_PCT: f32 = 84.0;
const TAGLINE_MD_LEFT_PCT: f32 = 78.0;
const TAGLINE_MD_WIDTH_PCT: f32 = 14.0; // ends at 92%, flush with `window_frame`'s own margin
const TAGLINE_MD_TOP_PCT: f32 = 15.0;
const TAGLINE_MD_HEIGHT_PCT: f32 = 65.0;

/// One heptagon, cycling through three color stages (green -> purple -> amber -> green) --
/// nothing else about it ever changes, so the interpolation itself is the whole point, not
/// a chain of different properties (that's `sequence.rs`'s own job). The label underneath
/// always names the color the shape is *currently moving towards*, snapping to the next
/// name right as each transition actually starts.
pub fn build(tree: &mut Tree, slot: Entity) {
    let frame = crate::chapters::window_frame(tree, slot);

    let poly = tree.branch(
        frame,
        Polygon::new()
            .sides(3.0)
            .rounding(0.0)
            .rotation(0.0)
            .color(Color::green(REST_COLOR))
            .at(poly_location())
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

    // names the color the very first stage moves *towards* -- already correct by the time
    // it fades in, so unlike the later transitions, no `wait_before_*` sync step is needed
    // here: the first stage's tween starts the instant this label has already appeared.
    let label = tree.branch(
        frame,
        Text::new("purple")
            .size(FontSize::new(LABEL_FONT_SIZE))
            .color(Color::slate(LABEL_COLOR))
            .at(Location::new().xs(
                POLY_LEFT_PCT
                    .pct()
                    .as_left()
                    .with(POLY_WIDTH_PCT.pct().as_width()),
                LABEL_TOP_PCT
                    .pct()
                    .as_top()
                    .with(LABEL_HEIGHT_PCT.pct().as_height()),
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
            .finish(MORPH_DURATION)
            .eased(Ease::Linear),
    );

    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        stage_to_purple(&mut tree, poly, label);
    });
}

fn poly_location() -> Location {
    Location::new().xs(
        POLY_LEFT_PCT
            .pct()
            .as_left()
            .with(POLY_WIDTH_PCT.pct().as_width()),
        POLY_TOP_PCT
            .pct()
            .as_top()
            .with(POLY_HEIGHT_PCT.pct().as_height()),
    )
}

fn stage_to_purple(tree: &mut Tree, poly: Entity, label: Entity) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(Color::purple(STAGE_COLOR))
            .targeting(poly)
            .during(seq)
            .start(0)
            .finish(STAGE_DURATION)
            .eased(Ease::Linear),
    );
    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        wait_before_stage2(&mut tree, poly, label);
    });
}

// a real waiting step, not a delay folded into `stage_to_amber`'s own animation -- same
// reasoning `sequence.rs`'s own `pause_before_color` uses: the label switch and the
// animation's actual start need to land at the same moment. A no-op `Color` tween (`poly`'s
// already at purple) just gives this a timed `sequence_end`.
fn wait_before_stage2(tree: &mut Tree, poly: Entity, label: Entity) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(Color::purple(STAGE_COLOR))
            .targeting(poly)
            .during(seq)
            .start(0)
            .finish(STAGE_PAUSE)
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
                tree.write_to(label, TextValue("amber".to_string()));
            }
            stage_to_amber(&mut tree, poly, label);
        },
    );
}

fn stage_to_amber(tree: &mut Tree, poly: Entity, label: Entity) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(Color::amber(STAGE_COLOR))
            .targeting(poly)
            .during(seq)
            .start(0)
            .finish(STAGE_DURATION)
            .eased(Ease::Linear),
    );
    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        wait_before_stage3(&mut tree, poly, label);
    });
}

fn wait_before_stage3(tree: &mut Tree, poly: Entity, label: Entity) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(Color::amber(STAGE_COLOR))
            .targeting(poly)
            .during(seq)
            .start(0)
            .finish(STAGE_PAUSE)
            .eased(Ease::Linear),
    );
    tree.sequence_end(
        seq,
        move |_: Trigger<OnEnd>, mut tree: Tree, existing: Query<Entity>| {
            if existing.contains(label) {
                tree.write_to(label, TextValue("green".to_string()));
            }
            stage_to_green(&mut tree, poly);
        },
    );
}

// final phase -- nothing needs to react once this finishes, so no `sequence_end` at all.
fn stage_to_green(tree: &mut Tree, poly: Entity) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(Color::green(REST_COLOR))
            .targeting(poly)
            .during(seq)
            .start(0)
            .finish(STAGE_DURATION)
            .eased(Ease::Linear),
    );
}
