use crate::icons::IconHandles;
use crate::type_in;
use foliage::{
    Anchor, Animation, Color, Ease, EcsExtension, Elevation, Entity, GridExt, Icon, Line,
    Location, Opacity, Polygon, Sprout, Tree, anchor,
};
use std::f32::consts::PI;

const WIDTH: f32 = 14.0;
const HEIGHT: f32 = 11.0;
const START_CENTER_X: f32 = -10.0; // fully offscreen left
const END_CENTER_X: f32 = 200.0 / 3.0; // two-thirds across
const CENTER_Y: f32 = 30.0; // upper third of the page
const BOTTOM_CENTER_Y: f32 = 83.0; // center of the bottom third

const MOVE_END: u64 = 3400; // slower glide to its resting horizontal position
const SPIN_DURATION: u64 = 180; // fast spin into the next shape
const BOUNCE_DURATION: u64 = 140; // quick overcorrect back to rest angle
const SHAPE_PAUSE: u64 = 380; // longer hold once it settles
const STAGE_DURATION: u64 = SPIN_DURATION + BOUNCE_DURATION + SHAPE_PAUSE;
/// Timed so the LAST stage's "spin into" lands exactly when the location move finishes --
/// the shape's final flourish and its arrival read as one beat.
const MORPH_DELAY: u64 = MOVE_END - SPIN_DURATION - (STAGES.len() as u64 - 1) * STAGE_DURATION;
const TURN_DURATION: u64 = 7000;
/// Per stage: spin past the resting angle by this much, then bounce back -- faster and
/// punchier than a single monotonic turn.
const ROTATION_PER_STAGE: f32 = PI / 2.0;
const OVERSHOOT: f32 = PI / 10.0;

const LINE_WEIGHT: i32 = 2;
const LINE_GAP: f32 = 4.0; // clearance from the polygon's own edge
const SCREEN_MARGIN: f32 = 6.0; // clearance from the screen edge
const LINE_DRAW: u64 = 1200;

const DOWN_DURATION: u64 = 900; // polygon + both lines, together, to the bottom third
const ICON_PX: i32 = 20; // plain pct sizing resolves against the parent, not an Anchor --
                          // fixed size is the honest option here, not a guessed percentage
const ICON_FADE: u64 = 500;

/// sides/rounding at each morph stage, continuing on from the starting triangle (3) --
/// stops at a heptagon, well short of circle-ish, so the polygon-ness stays legible
/// throughout.
const STAGES: &[(f32, f32)] = &[
    (4.0, 0.0),  // square
    (5.0, 0.25), // pentagon
    (7.0, 0.55), // heptagon
];

fn box_at(center_x: f32, center_y: f32) -> Location {
    box_of_size(center_x, center_y, WIDTH, HEIGHT)
}

fn box_of_size(center_x: f32, center_y: f32, w: f32, h: f32) -> Location {
    Location::new().xs(
        (center_x - w / 2.0).pct().as_left().with(w.pct().as_width()),
        (center_y - h / 2.0).pct().as_top().with(h.pct().as_height()),
    )
}

pub fn home(tree: &mut Tree, slot: Entity) {
    let polygon = tree.branch(
        slot,
        Polygon::new()
            .sides(3.0)
            .rounding(0.0)
            .rotation(PI) // upside-down triangle
            .color(Color::orange(400))
            .at(box_at(START_CENTER_X, CENTER_Y))
            .elevate(Elevation::up(1)),
    );

    let seq = tree.sequence();

    tree.animate(
        Animation::new(box_at(END_CENTER_X, CENTER_Y))
            .targeting(polygon)
            .during(seq)
            .start(0)
            .finish(MOVE_END)
            .eased(Ease::ACCELERATE), // slow start, fast end
    );

    let mut t = MORPH_DELAY;
    let mut rotation = PI;
    for &(sides, rounding) in STAGES {
        let settle_rotation = rotation + ROTATION_PER_STAGE;
        let overshoot_rotation = settle_rotation + OVERSHOOT;

        // spin into the new shape, sailing past the resting angle
        let spin_finish = t + SPIN_DURATION;
        tree.animate(
            Animation::new(Polygon {
                sides,
                rounding,
                rotation: overshoot_rotation,
            })
            .targeting(polygon)
            .during(seq)
            .start(t)
            .finish(spin_finish)
            .eased(Ease::EMPHASIS),
        );

        // overcorrect bounce-back to the actual resting angle
        let bounce_finish = spin_finish + BOUNCE_DURATION;
        tree.animate(
            Animation::new(Polygon {
                sides,
                rounding,
                rotation: settle_rotation,
            })
            .targeting(polygon)
            .during(seq)
            .start(spin_finish)
            .finish(bounce_finish)
            .eased(Ease::INWARD),
        );

        rotation = settle_rotation;
        t = bounce_finish + SHAPE_PAUSE;
    }
    let morph_end = t - SHAPE_PAUSE;

    // shape and rounding are done changing from here -- only a slow final turn remains
    // before everything comes to rest.
    let (last_sides, last_rounding) = *STAGES.last().unwrap();
    tree.animate(
        Animation::new(Polygon {
            sides: last_sides,
            rounding: last_rounding,
            rotation: rotation + 2.0 * PI,
        })
        .targeting(polygon)
        .during(seq)
        .start(morph_end)
        .finish(morph_end + TURN_DURATION)
        .eased(Ease::Linear),
    );

    // once it's arrived (the shape is still finishing its spin), a blueprint spoke draws
    // out from its center on each side -- gapped off both the polygon's own edge and the
    // screen's, so it never touches either.
    let half_w = WIDTH / 2.0 + LINE_GAP;
    let (cx, cy) = (END_CENTER_X, CENTER_Y);
    let spokes = [
        (cx - half_w, cy, SCREEN_MARGIN, cy),         // left
        (cx + half_w, cy, 100.0 - SCREEN_MARGIN, cy), // right
    ];
    for (anchor_x, anchor_y, tip_x, tip_y) in spokes {
        // the anchor point (nearest the polygon) is fixed for the whole animation --
        // only the tip moves, so it reads as drawing out from a fixed root, not
        // growing from both ends.
        let line = tree.branch(
            slot,
            Line::new(LINE_WEIGHT)
                .color(Color::stone(400))
                .at(Location::new().xs(
                    anchor_x.pct().as_x().with(anchor_y.pct().as_y()),
                    anchor_x.pct().as_x().with(anchor_y.pct().as_y()),
                ))
                .elevate(Elevation::up(1)),
        );
        tree.animate(
            Animation::new(Location::new().xs(
                anchor_x.pct().as_x().with(anchor_y.pct().as_y()),
                tip_x.pct().as_x().with(tip_y.pct().as_y()),
            ))
            .targeting(line)
            .during(seq)
            .start(MOVE_END)
            .finish(MOVE_END + LINE_DRAW)
            .eased(Ease::DECELERATE),
        );

        // once the lines are done drawing, this one rides down to the bottom third
        // together with the polygon and its sibling line -- same x's, only y changes.
        tree.animate(
            Animation::new(Location::new().xs(
                anchor_x.pct().as_x().with(BOTTOM_CENTER_Y.pct().as_y()),
                tip_x.pct().as_x().with(BOTTOM_CENTER_Y.pct().as_y()),
            ))
            .targeting(line)
            .during(seq)
            .start(MOVE_END + LINE_DRAW)
            .finish(MOVE_END + LINE_DRAW + DOWN_DURATION)
            .eased(Ease::DECELERATE),
        );
    }

    let settle_start = MOVE_END + LINE_DRAW;
    let settle_end = settle_start + DOWN_DURATION;
    tree.animate(
        Animation::new(box_at(END_CENTER_X, BOTTOM_CENTER_Y))
            .targeting(polygon)
            .during(seq)
            .start(settle_start)
            .finish(settle_end)
            .eased(Ease::DECELERATE),
    );

    // a terminal icon fades in centered on the polygon itself -- anchored to the polygon
    // entity, so it's dead-center in the polygon's own box wherever that box actually is,
    // no separately-tracked screen coordinates to keep in sync. Plain `.pct()` resolves
    // against the *parent* regardless of `Anchor`, so centering has to go through
    // `anchor().center_x()/.center_y()` (the anchor-relative value) rather than a bare
    // `50.pct()`.
    let icon = tree.branch(
        slot,
        Icon::new(IconHandles::Terminal)
            .color(Color::gray(900))
            .at(Location::new().xs(
                anchor().center_x().as_center_x().with(ICON_PX.px().as_width()),
                anchor().center_y().as_center_y().with(ICON_PX.px().as_height()),
            ))
            .elevate(Elevation::up(2))
            .with((Anchor::new(polygon), Opacity::new(0.0))),
    );
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(icon)
            .during(seq)
            .start(settle_end)
            .finish(settle_end + ICON_FADE)
            .eased(Ease::DECELERATE),
    );

    // once polygon + lines are at rest (still spinning), a terminal-style type-in effect
    // starts in the middle of the screen.
    type_in::type_in(tree, slot, seq, settle_end);
}
