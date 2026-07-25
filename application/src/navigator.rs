use crate::icons::IconHandles;
use foliage::bevy_ecs::query::With;
use foliage::{
    Anchor, Animation, Branch, Color, Ease, EcsExtension, Elevation, Entity, GridExt, Icon,
    IconValue, InteractionListener, InteractionPropagation, InteractionShape, Line, Location,
    OnClick, OnEnd, Opacity, PageChanged, PageCount, PageIndex, Polygon, Query, Router,
    RouterHandle, Sprout, Tree, Trigger, ValueDescriptor, anchor, component, targeted_event,
};
use std::f32::consts::PI;

/// Fired at the router's current slot the moment home is ready to start: either the
/// first time the navigator's intro actually lands, or -- since `home` can be revisited
/// -- every later return to it once the navigator has already landed once. Targeted
/// (not global), because "whoever's listening" changes on every visit: `home` registers
/// a fresh subscription on its own fresh `slot` each time it builds, and a global,
/// fire-once event would only ever reach the first of those.
#[targeted_event]
#[derive(Copy)]
pub struct NavigatorLanded {}

/// Marker: the navigator has landed at least once. Distinguishes "this is the very
/// first `PageChanged` back to home, before the intro has even started" from "home is
/// being revisited" -- the former is handled directly by the intro's own landing step,
/// the latter needs this to know it should resend `NavigatorLanded`.
#[component]
struct Landed;

const WIDTH: f32 = 14.0;
const HEIGHT: f32 = 11.0;
/// Both polygons spawn and travel through their whole intro at this size -- much bigger
/// than their `WIDTH`/`HEIGHT` resting size, so the intro reads as prominent -- then shrink
/// down to resting size during `build_down_move`, simply by animating from this size to
/// `WIDTH`/`HEIGHT` (the `Animation<Location>` interpolates size exactly like position).
const INTRO_SCALE: f32 = 3.0;
const INTRO_WIDTH: f32 = WIDTH * INTRO_SCALE;
const INTRO_HEIGHT: f32 = HEIGHT * INTRO_SCALE;

/// Two decorative "shadow" copies of `forward`, offset down+left and progressively
/// smaller/lighter -- a layered-card depth effect, not interactive, never clicked. Index 0
/// is the nearer/larger/darker one. `Anchor::new(forward)` + `anchor().width() * scale`
/// (root-level, not a child -- no clipping concerns) rather than a fixed screen-percentage
/// offset: a root-level offset expressed as one percent of screen *width* and another of
/// screen *height* produced visibly different pixel shifts on each axis depending on the
/// screen's own aspect ratio (reading as almost purely vertical on a portrait screen,
/// almost purely horizontal on a wide one). Anchoring to `forward` and scaling *its* own
/// resolved size means the offset is always relative to a real, already-square (thanks to
/// `AspectRatio`) reference, so it means the same thing on both axes regardless of the
/// screen's shape. It also means the shadows automatically track every one of `forward`'s
/// own position/size changes (fade-in, settle, down-move) for free, continuously, the same
/// way an anchored icon already tracks its own target -- no separate position animation
/// needed for the shadows at all.
const SHADOW_SCALE: [f32; 2] = [0.78, 0.58];
/// Each shadow's own fade-in/morph starts this much later than the previous layer's (and
/// `forward`'s own, for layer 0) -- reads as the shadows "catching up" to `forward`'s shape
/// with a lag, rather than moving in perfect lockstep with it.
const SHADOW_LAG: u64 = 220;
/// Kept short enough that even the farthest layer (`2 * SHADOW_LAG + SHADOW_FADE_IN`)
/// finishes before `FORWARD_FADE_IN` does -- `fade_seq`'s own completion (which triggers
/// `forward`'s morph and `back`'s whole intro) is driven by `forward`'s fade, not stalled
/// waiting on the shadows.
const SHADOW_FADE_IN: u64 = 400;
const START_CENTER_X: f32 = -10.0; // fully offscreen left
const END_CENTER_X: f32 = 200.0 / 3.0; // `forward`'s fixed x, always
/// `back`'s fixed x, always -- enough clearance from the screen edge that the whole
/// polygon (accounting for its own half-width) actually sits on-screen, not partially
/// clipped off it.
const BACK_X: f32 = 15.0;
const CENTER_Y: f32 = 30.0; // upper third of the page, during the intro
/// `back` holds lower than `forward` during the intro -- at `INTRO_HEIGHT`, the two boxes
/// would otherwise overlap outright (both start from the same off-screen spawn point, and
/// `back`'s own arrive is staggered, so there's a real window where `forward` has started
/// peeling away but `back` hasn't moved yet). `CENTER_Y + INTRO_HEIGHT` puts `back`'s own
/// top edge exactly level with `forward`'s bottom edge (`CENTER_Y + INTRO_HEIGHT / 2`,
/// plus another `INTRO_HEIGHT / 2` for `back`'s own half-height); `+ 3.0` is a small
/// margin so they don't sit flush against each other.
const BACK_CENTER_Y: f32 = CENTER_Y + INTRO_HEIGHT + 3.0;
/// Resting spot once settled -- low enough to read as a nav control, leaving the rest of
/// the screen for whatever scene is showing above it.
const REST_CENTER_Y: f32 = 91.0;

const MOVE_END: u64 = 3400; // `back`'s slower glide to its resting horizontal position
const FORWARD_FADE_IN: u64 = 900; // `forward`'s in-place fade-in (it never slides)
/// `back` becomes visible over this long, starting exactly when its own arrive begins --
/// it isn't actually fully off-screen at intro size (`START_CENTER_X` still leaves a
/// corner of it on-screen at `INTRO_WIDTH`), so it needs to fade in rather than just be
/// visible from spawn, or that corner sits there as a stray artifact before it starts
/// moving.
const BACK_ARRIVE_FADE_IN: u64 = 400;
const SETTLE_DURATION: u64 = 400; // in-place shrink from intro size to resting size
const SPIN_DURATION: u64 = 180; // fast spin into the next shape
const BOUNCE_DURATION: u64 = 140; // quick overcorrect back to rest angle
const SHAPE_PAUSE: u64 = 380; // longer hold once it settles
const STAGE_DURATION: u64 = SPIN_DURATION + BOUNCE_DURATION + SHAPE_PAUSE;
/// `back`'s own morph delay: timed so its LAST stage's "spin into" lands exactly when
/// `back`'s own glide (`MOVE_END`) finishes. `forward` doesn't glide anywhere, so its
/// morph just starts right away once it's done fading in -- see its own call to
/// `build_morph` in `build`.
const MORPH_DELAY: u64 = MOVE_END - SPIN_DURATION - (STAGES.len() as u64 - 1) * STAGE_DURATION;
const TURN_DURATION: u64 = 11000;
/// Per stage: spin past the resting angle by this much, then bounce back.
const ROTATION_PER_STAGE: f32 = PI / 2.0;
const OVERSHOOT: f32 = PI / 10.0;

const LINE_WEIGHT: i32 = 2;
const LINE_GAP: f32 = 4.0; // clearance from each polygon's own edge
const SCREEN_MARGIN: f32 = 6.0; // clearance from the screen edge
const LINE_DRAW: u64 = 1200;
/// Real pixels, not a percentage -- the drop-preview line's start needs clearance from
/// `back`'s bounding-box edge regardless of screen size, since it's compensating for the
/// polygon's own rotation not the screen's aspect ratio.
const DROP_LINE_MARGIN: i32 = 6;
/// Same reasoning as `DROP_LINE_MARGIN`, applied to `back_line`/`edge_line`'s end that sits
/// against `forward`'s own edge -- without it the lines visually touch/overlap `forward`
/// rather than leaving a small gap.
const LINE_MARGIN: i32 = 16;

const DOWN_DURATION: u64 = 900; // `forward` + both lines, together, to the resting spot
const ICON_PX: i32 = 20;
const ICON_FADE: u64 = 500;

/// `back`'s icon fades in this long after `forward`'s icon-fade sequence ends -- a
/// staggered second entrance for the icon layer specifically (the polygon itself is
/// already fully visible by then, from its own intro).
const BACK_STAGGER: u64 = 400;
const BACK_FADE: u64 = 500;

/// Both polygons exist the whole time; a boundary (index 0 or the last page) mutes
/// whichever one isn't currently usable rather than hiding it outright -- reads as
/// "disabled," not "gone," and means the lines connecting them never have to change.
const MUTED_OPACITY: f32 = 0.3;

/// sides/rounding at each morph stage, continuing on from the starting triangle (3) --
/// stops at a heptagon, well short of circle-ish, so the polygon-ness stays legible.
const STAGES: &[(f32, f32)] = &[
    (4.0, 0.0),  // square
    (5.0, 0.25), // pentagon
    (7.0, 0.55), // heptagon
];

// -- click transition: fade current scene, pull lines in, spin + hop, redraw, advance --
const CONTENT_FADE_OUT: u64 = 90;
const LINES_PULL_IN: u64 = 170;
const SPIN_TRANSITION: u64 = 420;
const HOP_UP: u64 = 150;
const HOP_PAUSE: u64 = 120;
const HOP_DOWN: u64 = 150; // HOP_UP + HOP_PAUSE + HOP_DOWN == SPIN_TRANSITION: hop and
// spin read as one beat, not two staggered motions.
const HOP_HEIGHT_PCT: f32 = 4.0;
const REDRAW_LINES: u64 = 300;
const REVOLUTION: f32 = 2.0 * PI;
const BOUNDARY_FADE: u64 = 120; // muting/unmuting at the ends
/// `build_continuous_spin`'s own duration -- deliberately separate from `TURN_DURATION`
/// (the intro's own finishing spin): the two read the same, but aren't meant to be tied
/// to the same value going forward.
const POST_CLICK_SPIN_DURATION: u64 = 3000;

fn box_at(center_x: f32, center_y: f32) -> Location {
    box_of_size(center_x, center_y, WIDTH, HEIGHT)
}

fn box_of_size(center_x: f32, center_y: f32, w: f32, h: f32) -> Location {
    Location::new().xs(
        (center_x - w / 2.0)
            .pct()
            .as_left()
            .with(w.pct().as_width()),
        (center_y - h / 2.0)
            .pct()
            .as_top()
            .with(h.pct().as_height()),
    )
}

/// `layer`-th shadow's box (1 or 2) -- entirely `Anchor`-derived (the shadow entity needs
/// `Anchor::new(forward)`), continuously live against whatever `forward`'s own current
/// box actually is, at any stage. Built via the `(Width, Right)`/`(Top, Height)` edge
/// pairings (not `(Left, Width)`/center-plus-delta): the shadow's right edge is pinned to
/// a reference point on `forward` and it extends left/down from there by its own scaled
/// size -- no arithmetic between two independently-resolved anchor values needed (only
/// `Mul<f32>` on a single one, which is all `LocationValue::Anchor` actually supports).
/// Layer 1 pins to `forward`'s center (a moderate, mostly-overlapping peek); layer 2 pins
/// to `forward`'s far corner (left edge / bottom edge) for a more separated, further-back
/// read.
fn shadow_box(layer: usize) -> Location {
    let scale = SHADOW_SCALE[layer - 1];
    let (right_ref, top_ref) = if layer == 1 {
        (anchor().center_x(), anchor().center_y())
    } else {
        (anchor().left(), anchor().bottom())
    };
    Location::new().xs(
        (anchor().width() * scale)
            .as_width()
            .with(right_ref.as_right()),
        top_ref
            .as_top()
            .with((anchor().height() * scale).as_height()),
    )
}

fn shadow_color(layer: usize) -> Color {
    if layer == 1 {
        Color::stone(600)
    } else {
        Color::stone(500)
    }
}

/// A rough estimate of where `back`'s bottom edge ends up once it's fully landed at
/// `REST_CENTER_Y` -- used only as the drop-preview line's *fixed* target during its
/// initial draw-in (`build_lines`). Doesn't need to be exact: the line's other end tracks
/// `back`'s actual live bottom edge via `Anchor` (see `build_lines`), so this only
/// determines how close to fully "eaten" the line gets, not whether it connects correctly.
fn back_rest_bottom_y_estimate() -> f32 {
    REST_CENTER_Y + HEIGHT / 2.0
}

/// A rough estimate of `back`'s own right edge (with `LINE_GAP` clearance) -- used only
/// as `back_line`'s far tip. `back_line` is anchored to `forward` (both lines are --
/// "forward is the sole anchor," per the original design, is a real invariant, not just a
/// convenience: a `Line` only gets one `Anchor` target, and `forward`'s edge is the one
/// every draw/pull-in/redraw treats as immobile), so `back`'s own edge can't *also* be
/// anchored on the same entity -- this stays a percentage estimate, same class of
/// imprecision `WIDTH` always had, just no longer worth chasing given the constraint.
fn back_r_estimate() -> f32 {
    BACK_X + WIDTH / 2.0 + LINE_GAP
}

/// `forward`'s left/right edge, each pushed `LINE_MARGIN` real pixels further away from
/// `forward` -- every line touching `forward` (`back_line`/`edge_line`'s near end, in every
/// phase: initial draw, pull-in, down-move, redraw) should go through these rather than a
/// bare `anchor().left()/.right()`, so the breathing room stays consistent everywhere
/// instead of only wherever someone remembered to add it.
fn fwd_left_x() -> ValueDescriptor {
    anchor().left().as_x().adjust(-LINE_MARGIN)
}
fn fwd_right_x() -> ValueDescriptor {
    anchor().right().as_x().adjust(LINE_MARGIN)
}

/// The whole navigator's live entities, threaded through the click-transition chain as
/// one value instead of a growing parameter list.
#[derive(Copy, Clone)]
struct Nav {
    router: Entity,
    forward: Entity,
    forward_icon: Entity,
    shadows: [Entity; 2],
    back: Entity,
    back_icon: Entity,
    lines: [Entity; 2], // [left, right]
}

fn icon_bundle(target: Entity) -> impl foliage::Sprout {
    Icon::new(IconHandles::Terminal)
        .color(Color::gray(900))
        .at(Location::new().xs(
            anchor()
                .center_x()
                .as_center_x()
                .with(ICON_PX.px().as_width()),
            anchor()
                .center_y()
                .as_center_y()
                .with(ICON_PX.px().as_height()),
        ))
        .elevate(Elevation::up(11))
        .with((
            Anchor::new(target),
            Opacity::new(0.0),
            InteractionPropagation::pass_through(),
        ))
}

/// Persistent, built once, outside any route: the polygon pair + their connecting lines
/// double as a "focal navigator" for the app's sections, surviving every route switch
/// (the router tears its own scene subtree down completely on navigation -- this lives
/// entirely outside that subtree). Plays its intro, then waits to be clicked.
pub fn build(tree: &mut Tree, router: Entity) {
    // elevated well above anything any route's own content will ever reach, so it
    // always renders as chrome on top of whatever scene is showing.
    let forward = tree.leaf(
        Polygon::new()
            .sides(3.0)
            .rounding(0.0)
            .rotation(0.0) // upside-down triangle (the shader's un-rotated triangle already points down)
            .color(Color::orange(400))
            .at(box_of_size(
                END_CENTER_X,
                CENTER_Y,
                INTRO_WIDTH,
                INTRO_HEIGHT,
            ))
            .elevate(Elevation::up(10))
            .with((
                InteractionListener::new(),
                InteractionShape::Circle,
                Opacity::new(0.0),
            )),
    );
    tree.disable(forward); // not clickable until the intro finishes

    // decorative shadows, offset down+left and smaller/lighter per layer -- purely a
    // depth backdrop, never interactive, elevated behind `forward` (and each other).
    // Anchored to `forward`, not children of it -- see `shadow_box`.
    let mut shadows = [Entity::PLACEHOLDER; 2];
    for (i, shadow) in shadows.iter_mut().enumerate() {
        let layer = i + 1;
        *shadow = tree.leaf(
            Polygon::new()
                .sides(3.0)
                .rounding(0.0)
                .rotation(0.0)
                .color(shadow_color(layer))
                .at(shadow_box(layer))
                .elevate(Elevation::up(10 - layer as i32))
                .with((Opacity::new(0.0), Anchor::new(forward))),
        );
    }

    // `back` runs its own intro alongside `forward`'s -- same big size, same morph
    // stages, fully visible throughout -- staggered a beat later and travelling a much
    // shorter horizontal distance (it's heading for `BACK_X`, already close to where it
    // starts), so it reads as "in place" next to forward's longer glide. Stays disabled
    // the whole time -- there's nothing to go back to at index 0 regardless of the intro.
    let back = tree.leaf(
        Polygon::new()
            .sides(3.0)
            .rounding(0.0)
            .rotation(0.0) // upside-down, matching `forward`'s starting orientation
            .color(Color::red(400))
            .at(box_of_size(
                START_CENTER_X,
                BACK_CENTER_Y,
                INTRO_WIDTH,
                INTRO_HEIGHT,
            ))
            .elevate(Elevation::up(10))
            .with((
                InteractionListener::new(),
                InteractionShape::Circle,
                Opacity::new(0.0),
            )),
    );
    tree.disable(back);

    // `forward` doesn't travel at all -- it's already sitting at its permanent horizontal
    // (`END_CENTER_X`, roughly mid-screen), just invisible, and fades in in place, slowly,
    // as the precursor beat. Once it's visible, it starts morphing immediately -- still at
    // full intro size, same as it always did -- and `back`'s own intro (the one that
    // actually slides in from off-screen) only starts once that fade is done, not racing
    // it on a guessed timer.
    let fade_seq = tree.sequence();
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(forward)
            .during(fade_seq)
            .start(0)
            .finish(FORWARD_FADE_IN)
            // DECELERATE (fast-start, slow-end) made this look like it just popped in --
            // most of the opacity gain happened in the first fraction of the duration.
            // ACCELERATE (slow-start, fast-end) actually reads as a slow build.
            .eased(Ease::ACCELERATE),
    );
    // each shadow fades in a beat behind the previous layer (and `forward`'s own, at
    // "layer 0"), so they read as trailing reactions rather than moving in lockstep.
    for (i, &shadow) in shadows.iter().enumerate() {
        let lag = (i + 1) as u64 * SHADOW_LAG;
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .targeting(shadow)
                .during(fade_seq)
                .start(lag)
                .finish(lag + SHADOW_FADE_IN)
                .eased(Ease::ACCELERATE),
        );
    }
    tree.sequence_end(fade_seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        build_morph(&mut tree, forward, 0); // starts morphing right away, still big
        for (i, &shadow) in shadows.iter().enumerate() {
            build_morph(&mut tree, shadow, (i + 1) as u64 * SHADOW_LAG);
        }
        build_back_arrive(&mut tree, router, forward, back, shadows);
    });
}

/// `back`'s whole intro -- slide in from off-screen, morphing the same as `forward` did --
/// only starts once `forward` has finished fading in. Once `back` finishes arriving, both
/// polygons (and both shadows, tracking `forward`) settle down to normal size together
/// (same duration, same beat, in the same shared sequence -- not several separate shrinks
/// that merely happen to line up), and only then do the lines draw.
fn build_back_arrive(
    tree: &mut Tree,
    router: Entity,
    forward: Entity,
    back: Entity,
    shadows: [Entity; 2],
) {
    let back_arrive_seq = tree.sequence();
    tree.animate(
        Animation::new(box_of_size(
            BACK_X,
            BACK_CENTER_Y,
            INTRO_WIDTH,
            INTRO_HEIGHT,
        ))
        .targeting(back)
        .during(back_arrive_seq)
        .start(0)
        .finish(MOVE_END)
        .eased(Ease::ACCELERATE),
    );
    // fades in right as it starts moving -- it spawned invisible specifically so the
    // corner of it that sits on-screen at its off-screen spawn spot (its intro size is
    // wider than `START_CENTER_X` alone hides) doesn't show as a stray artifact while it
    // was waiting for `forward`'s own fade to finish.
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(back)
            .during(back_arrive_seq)
            .start(0)
            .finish(BACK_ARRIVE_FADE_IN)
            .eased(Ease::ACCELERATE),
    );
    build_morph(tree, back, MORPH_DELAY);

    tree.sequence_end(back_arrive_seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        let settle_seq = tree.sequence();
        build_settle(
            &mut tree,
            settle_seq,
            forward,
            box_at(END_CENTER_X, CENTER_Y),
        );
        build_settle(&mut tree, settle_seq, back, box_at(BACK_X, BACK_CENTER_Y));
        // no shadow entry here: their box is entirely `Anchor`-derived (see
        // `shadow_box`/`build`), so they track `forward`'s own settle automatically,
        // continuously, the moment it happens -- same as an anchored icon always does.
        tree.sequence_end(settle_seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
            build_lines(&mut tree, router, forward, back, shadows);
        });
    });
}

/// Quick, in-place shrink from the big intro size down to normal resting size, without
/// moving row/column. Adds to `seq` rather than creating its own, so multiple polygons
/// (see `build_back_arrive`, which settles `forward`, `back`, and both shadows this way)
/// shrink together in one genuinely shared beat, not several separately-timed animations
/// that happen to overlap. Takes the target `Location` directly rather than a plain
/// `(x, y)` pair so shadows -- which settle to their own offset+scaled box, not a plain
/// `box_at` -- can use it too.
fn build_settle(tree: &mut Tree, seq: Entity, target: Entity, location: Location) {
    tree.animate(
        Animation::new(location)
            .targeting(target)
            .during(seq)
            .start(0)
            .finish(SETTLE_DURATION)
            .eased(Ease::DECELERATE),
    );
}

/// The shape-morph sequence -- square -> pentagon -> heptagon, spin-and-bounce into each
/// -- shared by both `forward` and `back` so they run through the identical beats, offset
/// only by `delay` (each polygon's own intro start time). Self-contained: nothing
/// downstream depends on when this finishes, including the final slow spin that keeps
/// going well past both polygons landing at rest.
fn build_morph(tree: &mut Tree, target: Entity, delay: u64) {
    let morph_seq = tree.sequence();
    let mut t = delay;
    let mut rotation = 0.0; // matches both polygons' actual starting (upside-down) rotation
    for &(sides, rounding) in STAGES {
        let settle_rotation = rotation + ROTATION_PER_STAGE;
        let overshoot_rotation = settle_rotation + OVERSHOOT;

        let spin_finish = t + SPIN_DURATION;
        tree.animate(
            Animation::new(Polygon {
                sides,
                rounding,
                rotation: overshoot_rotation,
            })
            .targeting(target)
            .during(morph_seq)
            .start(t)
            .finish(spin_finish)
            .eased(Ease::EMPHASIS),
        );

        let bounce_finish = spin_finish + BOUNCE_DURATION;
        tree.animate(
            Animation::new(Polygon {
                sides,
                rounding,
                rotation: settle_rotation,
            })
            .targeting(target)
            .during(morph_seq)
            .start(spin_finish)
            .finish(bounce_finish)
            .eased(Ease::INWARD),
        );

        rotation = settle_rotation;
        t = bounce_finish + SHAPE_PAUSE;
    }
    let morph_end = t - SHAPE_PAUSE;
    let (last_sides, last_rounding) = *STAGES.last().unwrap();
    tree.animate(
        Animation::new(Polygon {
            sides: last_sides,
            rounding: last_rounding,
            rotation: rotation + REVOLUTION,
        })
        .targeting(target)
        .during(morph_seq)
        .start(morph_end)
        .finish(morph_end + TURN_DURATION)
        .eased(Ease::Linear),
    );
}

/// Fires once `arrive` truly ends: a blueprint line draws out from `forward`'s edge on
/// each side -- one toward `back`, one out to the right edge -- plus a transient third
/// line hinting at `back`'s upcoming drop. The line reaching toward `back`, and the
/// drop-preview line, are both anchored directly to it (`Anchor::new(back)`) for their
/// *X* edge (`anchor().right()`/`.center_x()`) -- no compile-time constant can predict
/// where `back`'s real edge ends up once `AspectRatio` constrains it, without knowing the
/// screen's own aspect ratio. Y stays a plain row constant throughout, never anchored:
/// `back`'s own Y-center isn't an aspect-distorted quantity, it's just a different
/// (perfectly known) row than `forward`'s until they drop together later -- anchoring Y
/// too made the line diagonal instead of the intended horizontal connector. `build_down_move`
/// eats the drop line back to nothing in step with `back`'s own descent, rather than
/// leaving a stray line hanging around after.
fn build_lines(
    tree: &mut Tree,
    router: Entity,
    forward: Entity,
    back: Entity,
    shadows: [Entity; 2],
) {
    let cy = CENTER_Y;
    let lines_seq = tree.sequence();

    // both lines anchor to `forward` -- it's the one true immobile reference every
    // draw/pull-in/redraw treats as fixed. `back`'s own edge (a `Line` only gets one
    // `Anchor` target, already spent on `forward` here) stays a percentage estimate.
    let fwd_left = fwd_left_x();
    let fwd_right = fwd_right_x();
    let back_line = tree.leaf(
        Line::new(LINE_WEIGHT)
            .color(Color::stone(400))
            .at(Location::new().xs(
                fwd_left.with(cy.pct().as_y()),
                fwd_left.with(cy.pct().as_y()),
            ))
            .elevate(Elevation::up(10))
            .with(Anchor::new(forward)),
    );
    tree.animate(
        Animation::new(Location::new().xs(
            fwd_left.with(cy.pct().as_y()),
            back_r_estimate().pct().as_x().with(cy.pct().as_y()),
        ))
        .targeting(back_line)
        .during(lines_seq)
        .start(0)
        .finish(LINE_DRAW)
        .eased(Ease::DECELERATE),
    );

    let edge_line = tree.leaf(
        Line::new(LINE_WEIGHT)
            .color(Color::stone(400))
            .at(Location::new().xs(
                fwd_right.with(cy.pct().as_y()),
                fwd_right.with(cy.pct().as_y()),
            ))
            .elevate(Elevation::up(10))
            .with(Anchor::new(forward)),
    );
    tree.animate(
        Animation::new(Location::new().xs(
            fwd_right.with(cy.pct().as_y()),
            (100.0 - SCREEN_MARGIN).pct().as_x().with(cy.pct().as_y()),
        ))
        .targeting(edge_line)
        .during(lines_seq)
        .start(0)
        .finish(LINE_DRAW)
        .eased(Ease::DECELERATE),
    );
    let lines = [back_line, edge_line];

    // top always tracks `back`'s own actual bottom edge live, via `Anchor` -- no
    // animation needed for that end at all, in this or any later phase. `back`'s morph
    // keeps rotating independently (including its long final spin), so its visual shape
    // at any given moment often doesn't actually reach the bounding box's exact
    // bottom-center (that only holds for a perfect circle, or a vertex pointing straight
    // down) -- `DROP_LINE_MARGIN` pushes the start a few real pixels further down so it
    // clears the shape regardless of whatever rotation it's currently at, rather than
    // visibly starting inside it. The bottom is a fixed landing estimate; as `back`
    // actually descends in `build_down_move`, the top catches up to it on its own and the
    // line "eats" itself with no further code.
    let drop_top = anchor()
        .center_x()
        .as_x()
        .with(anchor().bottom().as_y().adjust(DROP_LINE_MARGIN));
    let drop_line = tree.leaf(
        Line::new(LINE_WEIGHT)
            .color(Color::stone(400))
            .at(Location::new().xs(drop_top, drop_top))
            .elevate(Elevation::up(10))
            .with(Anchor::new(back)),
    );
    tree.animate(
        Animation::new(
            Location::new().xs(
                drop_top,
                BACK_X
                    .pct()
                    .as_x()
                    .with(back_rest_bottom_y_estimate().pct().as_y()),
            ),
        )
        .targeting(drop_line)
        .during(lines_seq)
        .start(0)
        .finish(LINE_DRAW)
        .eased(Ease::DECELERATE),
    );

    tree.sequence_end(lines_seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        build_down_move(&mut tree, router, forward, back, shadows, lines, drop_line);
    });
}

/// Fires once the lines are truly done drawing: `forward` + `back` (both already at
/// resting size, courtesy of `build_settle`) + both permanent lines ride down together to
/// the resting spot -- and both shadows ride down with `forward`, tracking its offset, so
/// they end up sitting as a fixed backdrop just behind/below it at rest, never touched
/// again after this. The transient drop-preview line shrinks in the same window, at the
/// same easing, its top edge tracking `back` down so it's fully consumed exactly when
/// `back` lands -- then it's removed outright, rather than left sitting at zero length.
fn build_down_move(
    tree: &mut Tree,
    router: Entity,
    forward: Entity,
    back: Entity,
    shadows: [Entity; 2],
    lines: [Entity; 2],
    drop_line: Entity,
) {
    let down_seq = tree.sequence();
    tree.animate(
        Animation::new(box_at(END_CENTER_X, REST_CENTER_Y))
            .targeting(forward)
            .during(down_seq)
            .start(0)
            .finish(DOWN_DURATION)
            .eased(Ease::DECELERATE),
    );
    tree.animate(
        Animation::new(box_at(BACK_X, REST_CENTER_Y))
            .targeting(back)
            .during(down_seq)
            .start(0)
            .finish(DOWN_DURATION)
            .eased(Ease::DECELERATE),
    );
    // no shadow entries here either, for the same reason as `build_settle` above -- their
    // `Anchor`-derived box already tracks `forward`'s descent automatically.
    tree.animate(
        Animation::new(
            Location::new().xs(
                fwd_left_x().with(REST_CENTER_Y.pct().as_y()),
                back_r_estimate()
                    .pct()
                    .as_x()
                    .with(REST_CENTER_Y.pct().as_y()),
            ),
        )
        .targeting(lines[0])
        .during(down_seq)
        .start(0)
        .finish(DOWN_DURATION)
        .eased(Ease::DECELERATE),
    );
    tree.animate(
        Animation::new(
            Location::new().xs(
                fwd_right_x().with(REST_CENTER_Y.pct().as_y()),
                (100.0 - SCREEN_MARGIN)
                    .pct()
                    .as_x()
                    .with(REST_CENTER_Y.pct().as_y()),
            ),
        )
        .targeting(lines[1])
        .during(down_seq)
        .start(0)
        .finish(DOWN_DURATION)
        .eased(Ease::DECELERATE),
    );
    // the drop line needs no animation here at all: its "top" already tracks `back`'s
    // live bottom edge via `Anchor` (see `build_lines`), so it shrinks to nothing on its
    // own as `back` (animated above) actually descends toward the line's fixed bottom.
    tree.sequence_end(down_seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        tree.remove(drop_line);
        build_icons(&mut tree, router, forward, back, shadows, lines);
    });
}

/// Fires once settled at rest: `forward`'s icon fades in, and only once THAT finishes
/// does `forward` become clickable ("enabled when the icon is done appearing"), both
/// click observers get registered, and the one-time `NavigatorLanded` signal fires.
/// `back` itself already exists and already landed -- it rode down together with
/// `forward` in `build_down_move` -- so this only spawns its icon (spawning icons at this
/// exact late point, once things are truly settled, is what actually renders reliably;
/// spawning one any earlier, e.g. back at `build()` time before the event loop starts,
/// never rendered despite every ECS-level property checking out under a diagnostic dump).
/// `back` fades from its full intro visibility down to muted (nothing to go back to yet,
/// at index 0) while `back_icon` fades up to the same muted level, together.
fn build_icons(
    tree: &mut Tree,
    router: Entity,
    forward: Entity,
    back: Entity,
    shadows: [Entity; 2],
    lines: [Entity; 2],
) {
    let icon_seq = tree.sequence();

    let forward_icon = tree.leaf(icon_bundle(forward));
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(forward_icon)
            .during(icon_seq)
            .start(0)
            .finish(ICON_FADE)
            .eased(Ease::DECELERATE),
    );

    tree.sequence_end(
        icon_seq,
        move |_: Trigger<OnEnd>, handles: Query<&RouterHandle>, mut tree: Tree| {
            tree.enable(forward);

            let back_icon = tree.leaf(icon_bundle(back));
            // `back` never changes what it means, so its icon is set once, here, and
            // never touched again (unlike `forward`'s, which swaps meaning between
            // "home" and "forward").
            tree.write_to(back_icon, IconValue(IconHandles::SkipLeft.into()));
            let back_seq = tree.sequence();
            for target in [back, back_icon] {
                tree.animate(
                    Animation::new(Opacity::new(MUTED_OPACITY))
                        .targeting(target)
                        .during(back_seq)
                        .start(BACK_STAGGER)
                        .finish(BACK_STAGGER + BACK_FADE)
                        .eased(Ease::DECELERATE),
                );
            }

            let nav = Nav {
                router,
                forward,
                forward_icon,
                shadows,
                back,
                back_icon,
                lines,
            };
            tree.on_click(
                forward,
                move |_: Trigger<OnClick>,
                      page_state: Query<(&PageIndex, &PageCount)>,
                      polygons: Query<&Polygon>,
                      handles: Query<&RouterHandle>,
                      branches: Query<&Branch>,
                      mut tree: Tree| {
                    on_nav_click(
                        &mut tree,
                        nav,
                        forward,
                        1,
                        &page_state,
                        &polygons,
                        &handles,
                        &branches,
                    );
                },
            );
            tree.on_click(
                back,
                move |_: Trigger<OnClick>,
                      page_state: Query<(&PageIndex, &PageCount)>,
                      polygons: Query<&Polygon>,
                      handles: Query<&RouterHandle>,
                      branches: Query<&Branch>,
                      mut tree: Tree| {
                    on_nav_click(
                        &mut tree,
                        nav,
                        back,
                        -1,
                        &page_state,
                        &polygons,
                        &handles,
                        &branches,
                    );
                },
            );

            // persistent for the navigator's whole lifetime, registered here (not
            // `build()`) since it needs `nav` -- `reconcile` runs on *every* page change
            // regardless of source, not just the ones driven by clicking `forward`/`back`
            // themselves (`on_nav_click`'s own chain already calls it too, but a page
            // change can also arrive directly from `crate::chrome`'s Home/ToC buttons,
            // which write `PageIndex` straight to the router with none of that ceremony --
            // without this, jumping there left `forward`/`back`'s enabled/muted state
            // stuck at whatever it was before the jump). Resending `NavigatorLanded` on
            // every later return to home is the other half (the very first one is
            // handled directly right below, before this subscription even exists yet).
            tree.subscribe(
                router,
                move |trigger: Trigger<PageChanged>,
                      landed: Query<(), With<Landed>>,
                      counts: Query<&PageCount>,
                      handles: Query<&RouterHandle>,
                      mut tree: Tree| {
                    on_page_changed(&mut tree, router, trigger.index, &landed, &handles);
                    let count = counts.get(router).unwrap().0;
                    reconcile(&mut tree, nav, trigger.index, count);
                },
            );

            tree.write_to(router, Landed);

            // structural, not timed: fires exactly when the icon is actually visible,
            // whatever that took, at whatever slot home actually built into.
            let slot = Router::slot(router, &handles)
                .expect("router already landed, has a built slot by then");
            tree.trigger_targets(NavigatorLanded::new(), slot);
        },
    );
}

/// Fires on every route change. Home is revisitable -- its `home()` re-registers a
/// fresh `NavigatorLanded` subscription on its own fresh `slot` each time it builds, but
/// the *first* landing (before the intro has even started) is handled directly by
/// `build_icons` above, not here. `Landed`'s presence is exactly the distinction: the
/// first-ever `PageChanged` for index 0 fires before it's inserted, every later one
/// fires after.
fn on_page_changed(
    tree: &mut Tree,
    router: Entity,
    index: usize,
    landed: &Query<(), With<Landed>>,
    handles: &Query<&RouterHandle>,
) {
    if index == 0 && landed.get(router).is_ok() {
        let slot =
            Router::slot(router, handles).expect("router already landed, has a built slot by then");
        tree.trigger_targets(NavigatorLanded::new(), slot);
    }
}

/// The whole click transition, shared by both `forward` (+1) and `back` (-1): fade the
/// current scene's content out, pull both lines back in, spin the clicked polygon (with
/// a little up-pause-down hop, as if it briefly left the ground to change page), redraw
/// the lines, advance/retreat the route, then mute/unmute each polygon for the new
/// position.
fn on_nav_click(
    tree: &mut Tree,
    nav: Nav,
    clicked: Entity,
    direction: i32,
    page_state: &Query<(&PageIndex, &PageCount)>,
    polygons: &Query<&Polygon>,
    handles: &Query<&RouterHandle>,
    branches: &Query<&Branch>,
) {
    tree.disable(nav.forward);
    tree.disable(nav.back);

    let (index, count) = page_state.get(nav.router).unwrap();
    let count = count.0;
    let next_index = (index.0 as i32 + direction).clamp(0, count as i32 - 1) as usize;
    let current = *polygons.get(clicked).unwrap();

    // fade out whatever the router's current scene actually contains. `branches` here is
    // legitimately about the *slot's own content children* (a different concern from
    // finding the router's slot itself, which now reads `RouterHandle` directly).
    let slot =
        Router::slot(nav.router, handles).expect("router already landed, has a built slot by then");
    let content: Vec<Entity> = branches
        .get(slot)
        .map(|b| b.ids.iter().copied().collect())
        .unwrap_or_default();

    if content.is_empty() {
        // nothing to fade out (e.g. the still-blank ToC route) -- a `sequence_end` on a
        // sequence with zero animations ever added to it never fires (`OnEnd` only comes
        // from an animation actually finishing and decrementing the count), so waiting on
        // one here would hang the rest of the click transition forever, leaving `forward`
        // and `back` permanently disabled (they were just disabled above).
        build_pull_in(tree, nav, clicked, current, next_index, count);
        return;
    }

    let fade_seq = tree.sequence();
    for content in content {
        tree.animate(
            Animation::new(Opacity::new(0.0))
                .targeting(content)
                .during(fade_seq)
                .start(0)
                .finish(CONTENT_FADE_OUT)
                .eased(Ease::Linear),
        );
    }

    tree.sequence_end(fade_seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        build_pull_in(&mut tree, nav, clicked, current, next_index, count);
    });
}

fn build_pull_in(
    tree: &mut Tree,
    nav: Nav,
    clicked: Entity,
    current: Polygon,
    next_index: usize,
    count: usize,
) {
    let pull_seq = tree.sequence();
    // collapses each line down to `forward`'s own edge (plus `LINE_MARGIN` breathing room)
    // -- its own real anchor, not the percentage estimate `back`/the screen edge use.
    for (line, edge) in [(nav.lines[0], fwd_left_x()), (nav.lines[1], fwd_right_x())] {
        let point = edge.with(REST_CENTER_Y.pct().as_y());
        tree.animate(
            Animation::new(Location::new().xs(point, point))
                .targeting(line)
                .during(pull_seq)
                .start(0)
                .finish(LINES_PULL_IN)
                .eased(Ease::DECELERATE),
        );
    }
    tree.sequence_end(pull_seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        build_spin_hop(&mut tree, nav, clicked, current, next_index, count);
    });
}

fn build_spin_hop(
    tree: &mut Tree,
    nav: Nav,
    clicked: Entity,
    current: Polygon,
    next_index: usize,
    count: usize,
) {
    let spin_seq = tree.sequence();

    tree.animate(
        Animation::new(Polygon {
            rotation: current.rotation + REVOLUTION,
            ..current
        })
        .targeting(clicked)
        .during(spin_seq)
        .start(0)
        .finish(SPIN_TRANSITION)
        .eased(Ease::ACCELERATE), // start-slow, fast-finish
    );

    let clicked_x = if clicked == nav.forward {
        END_CENTER_X
    } else {
        BACK_X
    };
    // up a tiny bit, pause, back down -- as if it briefly left the ground.
    tree.animate(
        Animation::new(box_at(clicked_x, REST_CENTER_Y - HOP_HEIGHT_PCT))
            .targeting(clicked)
            .during(spin_seq)
            .start(0)
            .finish(HOP_UP)
            .eased(Ease::DECELERATE),
    );
    let down_start = HOP_UP + HOP_PAUSE;
    tree.animate(
        Animation::new(box_at(clicked_x, REST_CENTER_Y))
            .targeting(clicked)
            .during(spin_seq)
            .start(down_start)
            .finish(down_start + HOP_DOWN)
            .eased(Ease::ACCELERATE),
    );

    tree.sequence_end(spin_seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        // the same long, slow "keeps going" flourish the intro's `build_morph` finishes
        // on -- fired on every page switch now, not just once at boot. Its own sequence,
        // not added to `spin_seq`/gating `build_redraw_lines`: a multi-second spin
        // stalling the actual page navigation would be wrong.
        build_continuous_spin(&mut tree, clicked, current);
        build_redraw_lines(&mut tree, nav, next_index, count);
    });
}

/// Spins `target` a full turn past wherever `build_spin_hop`'s own short transition spin
/// left it (`current.rotation + REVOLUTION`), continuing in the same direction, over
/// `POST_CLICK_SPIN_DURATION` -- the same *kind* of flourish as the intro's `build_morph`
/// finishing spin, just re-triggered after every click instead of only the first landing,
/// and on its own separate duration rather than reusing `TURN_DURATION`.
fn build_continuous_spin(tree: &mut Tree, target: Entity, current: Polygon) {
    let spin_seq = tree.sequence();
    tree.animate(
        Animation::new(Polygon {
            rotation: current.rotation + REVOLUTION + REVOLUTION,
            ..current
        })
        .targeting(target)
        .during(spin_seq)
        .start(0)
        .finish(POST_CLICK_SPIN_DURATION)
        .eased(Ease::Linear),
    );
}

fn build_redraw_lines(tree: &mut Tree, nav: Nav, next_index: usize, count: usize) {
    let redraw_seq = tree.sequence();
    tree.animate(
        Animation::new(
            Location::new().xs(
                fwd_left_x().with(REST_CENTER_Y.pct().as_y()),
                back_r_estimate()
                    .pct()
                    .as_x()
                    .with(REST_CENTER_Y.pct().as_y()),
            ),
        )
        .targeting(nav.lines[0])
        .during(redraw_seq)
        .start(0)
        .finish(REDRAW_LINES)
        .eased(Ease::DECELERATE),
    );
    tree.animate(
        Animation::new(
            Location::new().xs(
                fwd_right_x().with(REST_CENTER_Y.pct().as_y()),
                (100.0 - SCREEN_MARGIN)
                    .pct()
                    .as_x()
                    .with(REST_CENTER_Y.pct().as_y()),
            ),
        )
        .targeting(nav.lines[1])
        .during(redraw_seq)
        .start(0)
        .finish(REDRAW_LINES)
        .eased(Ease::DECELERATE),
    );
    tree.sequence_end(redraw_seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        // the route switch itself: the router tears down the old scene and builds the
        // new one synchronously off this write.
        tree.write_to(nav.router, PageIndex(next_index));
        reconcile(&mut tree, nav, next_index, count);
    });
}

/// One rule, applied at every landing: `back` is active (full opacity, clickable)
/// exactly when `index > 0`, muted otherwise; `forward` active exactly when
/// `index < count - 1`, muted otherwise. Neither ever disappears -- both polygons and
/// their lines are always there, only which one is currently usable changes.
fn reconcile(tree: &mut Tree, nav: Nav, index: usize, count: usize) {
    let has_back = index > 0;
    let has_forward = index < count - 1;

    let boundary_seq = tree.sequence();

    let back_opacity = if has_back { 1.0 } else { MUTED_OPACITY };
    if has_back {
        tree.enable(nav.back);
    } else {
        tree.disable(nav.back);
    }
    for target in [nav.back, nav.back_icon] {
        tree.animate(
            Animation::new(Opacity::new(back_opacity))
                .targeting(target)
                .during(boundary_seq)
                .start(0)
                .finish(BOUNDARY_FADE)
                .eased(Ease::DECELERATE),
        );
    }

    let forward_opacity = if has_forward { 1.0 } else { MUTED_OPACITY };
    if has_forward {
        tree.enable(nav.forward);
    } else {
        tree.disable(nav.forward);
    }
    for target in [
        nav.forward,
        nav.forward_icon,
        nav.shadows[0],
        nav.shadows[1],
    ] {
        tree.animate(
            Animation::new(Opacity::new(forward_opacity))
                .targeting(target)
                .during(boundary_seq)
                .start(0)
                .finish(BOUNDARY_FADE)
                .eased(Ease::DECELERATE),
        );
    }
    // `forward`'s icon is the one that changes meaning: the original `Terminal` glyph
    // when it's alone (nothing to go back to yet), `SkipRight` once `back` exists too.
    let forward_icon = if has_back {
        IconHandles::SkipRight
    } else {
        IconHandles::Terminal
    };
    tree.write_to(nav.forward_icon, IconValue(forward_icon.into()));
}
