use crate::icons::IconHandles;
use foliage::bevy_ecs::query::With;
use foliage::{
    Anchor, Animation, Branch, Color, Ease, EcsExtension, Elevation, Entity, GridExt, Icon,
    IconValue, InteractionListener, InteractionPropagation, InteractionShape, Line, Location,
    OnClick, OnEnd, Opacity, PageChanged, PageCount, PageIndex, Polygon, Query, Sprout, Tree,
    Trigger, anchor, component, targeted_event,
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
const START_CENTER_X: f32 = -10.0; // fully offscreen left
const END_CENTER_X: f32 = 200.0 / 3.0; // `forward`'s fixed x, always
/// `back`'s fixed x, always -- enough clearance from the screen edge that the whole
/// polygon (accounting for its own half-width) actually sits on-screen, not partially
/// clipped off it.
const BACK_X: f32 = 15.0;
const CENTER_Y: f32 = 30.0; // upper third of the page, during the intro
/// Resting spot once settled -- low enough to read as a nav control, leaving the rest of
/// the screen for whatever scene is showing above it.
const REST_CENTER_Y: f32 = 91.0;

const MOVE_END: u64 = 3400; // slower glide to its resting horizontal position
const SPIN_DURATION: u64 = 180; // fast spin into the next shape
const BOUNCE_DURATION: u64 = 140; // quick overcorrect back to rest angle
const SHAPE_PAUSE: u64 = 380; // longer hold once it settles
const STAGE_DURATION: u64 = SPIN_DURATION + BOUNCE_DURATION + SHAPE_PAUSE;
/// Timed so the LAST stage's "spin into" lands exactly when the location move finishes.
const MORPH_DELAY: u64 = MOVE_END - SPIN_DURATION - (STAGES.len() as u64 - 1) * STAGE_DURATION;
const TURN_DURATION: u64 = 7000;
/// Per stage: spin past the resting angle by this much, then bounce back.
const ROTATION_PER_STAGE: f32 = PI / 2.0;
const OVERSHOOT: f32 = PI / 10.0;

const LINE_WEIGHT: i32 = 2;
const LINE_GAP: f32 = 4.0; // clearance from each polygon's own edge
const SCREEN_MARGIN: f32 = 6.0; // clearance from the screen edge
const LINE_DRAW: u64 = 1200;

const DOWN_DURATION: u64 = 900; // `forward` + both lines, together, to the resting spot
const ICON_PX: i32 = 20;
const ICON_FADE: u64 = 500;

/// `back` fades in this long after `forward`'s own icon lands -- a staggered second
/// entrance so the intro doesn't read as bare with just the one polygon.
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
const CONTENT_FADE_OUT: u64 = 200;
const LINES_PULL_IN: u64 = 400;
const SPIN_TRANSITION: u64 = 700;
const HOP_UP: u64 = 250;
const HOP_PAUSE: u64 = 200;
const HOP_DOWN: u64 = 250; // HOP_UP + HOP_PAUSE + HOP_DOWN == SPIN_TRANSITION: hop and
// spin read as one beat, not two staggered motions.
const HOP_HEIGHT_PCT: f32 = 4.0;
const REDRAW_LINES: u64 = 700;
const REVOLUTION: f32 = 2.0 * PI;
const BOUNDARY_FADE: u64 = 250; // muting/unmuting at the ends

fn box_at(center_x: f32, center_y: f32) -> Location {
    box_of_size(center_x, center_y, WIDTH, HEIGHT)
}

fn box_of_size(center_x: f32, center_y: f32, w: f32, h: f32) -> Location {
    Location::new().xs(
        (center_x - w / 2.0).pct().as_left().with(w.pct().as_width()),
        (center_y - h / 2.0).pct().as_top().with(h.pct().as_height()),
    )
}

/// Fixed, always: both polygons exist the whole time (see `MUTED_OPACITY`), so the
/// lines connecting them never move. Deliberately asymmetric: `forward` is the sole
/// anchor for *both* lines -- the first element of each pair, the fixed point every
/// draw/pull-in/redraw treats as immobile -- so everything always emanates from (and
/// pulls back toward) `forward`, never `back`, matching the original single-polygon
/// design this grew out of.
fn line_spans() -> [(f32, f32); 2] {
    let half_w = WIDTH / 2.0 + LINE_GAP;
    let fwd_l = END_CENTER_X - half_w;
    let fwd_r = END_CENTER_X + half_w;
    let back_r = BACK_X + half_w;
    [(fwd_l, back_r), (fwd_r, 100.0 - SCREEN_MARGIN)]
}

/// Router keeps exactly one child (its current slot) at a time, by its own design
/// (destroy-then-rebuild, each fully cleaning up its own `Stem`/`Branch` entries) --
/// stated plainly, not looped over as if there could be several.
fn current_slot(router: Entity, branches: &Query<&Branch>) -> Entity {
    let router_branch = branches.get(router).unwrap();
    *router_branch
        .ids
        .iter()
        .next()
        .expect("router always has exactly one slot (its own build_route discipline)")
}

/// The whole navigator's live entities, threaded through the click-transition chain as
/// one value instead of a growing parameter list.
#[derive(Copy, Clone)]
struct Nav {
    router: Entity,
    forward: Entity,
    forward_icon: Entity,
    back: Entity,
    back_icon: Entity,
    lines: [Entity; 2], // [left, right]
}

fn icon_bundle(target: Entity) -> impl foliage::Sprout {
    Icon::new(IconHandles::Terminal)
        .color(Color::gray(900))
        .at(Location::new().xs(
            anchor().center_x().as_center_x().with(ICON_PX.px().as_width()),
            anchor().center_y().as_center_y().with(ICON_PX.px().as_height()),
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
            .rotation(PI) // upside-down triangle
            .color(Color::orange(400))
            .at(box_at(START_CENTER_X, CENTER_Y))
            .elevate(Elevation::up(10))
            .with((InteractionListener::new(), InteractionShape::Circle)),
    );
    tree.disable(forward); // not clickable until the intro finishes

    // persistent for the navigator's whole lifetime: resends `NavigatorLanded` on every
    // later return to home (the very first one is handled directly once the intro
    // actually lands, further down).
    tree.subscribe(
        router,
        move |trigger: Trigger<PageChanged>,
              landed: Query<(), With<Landed>>,
              branches: Query<&Branch>,
              mut tree: Tree| {
            on_page_changed(&mut tree, router, trigger.index, &landed, &branches);
        },
    );

    let arrive_seq = tree.sequence();
    tree.animate(
        Animation::new(box_at(END_CENTER_X, CENTER_Y))
            .targeting(forward)
            .during(arrive_seq)
            .start(0)
            .finish(MOVE_END)
            .eased(Ease::ACCELERATE), // slow start, fast end
    );
    tree.sequence_end(arrive_seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        build_lines(&mut tree, router, forward);
    });

    // morph: self-contained, nothing downstream depends on when this finishes.
    let morph_seq = tree.sequence();
    let mut t = MORPH_DELAY;
    let mut rotation = PI;
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
            .targeting(forward)
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
            .targeting(forward)
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
        .targeting(forward)
        .during(morph_seq)
        .start(morph_end)
        .finish(morph_end + TURN_DURATION)
        .eased(Ease::Linear),
    );
}

/// Fires once `arrive` truly ends: a blueprint line draws out from `forward`'s center on
/// each side -- one to `back`, one out to the right edge.
fn build_lines(tree: &mut Tree, router: Entity, forward: Entity) {
    let cy = CENTER_Y;
    let lines_seq = tree.sequence();
    let mut lines = [Entity::PLACEHOLDER; 2];
    for (i, (anchor_x, tip_x)) in line_spans().into_iter().enumerate() {
        let line = tree.leaf(
            Line::new(LINE_WEIGHT)
                .color(Color::stone(400))
                .at(Location::new().xs(
                    anchor_x.pct().as_x().with(cy.pct().as_y()),
                    anchor_x.pct().as_x().with(cy.pct().as_y()),
                ))
                .elevate(Elevation::up(10)),
        );
        tree.animate(
            Animation::new(Location::new().xs(
                anchor_x.pct().as_x().with(cy.pct().as_y()),
                tip_x.pct().as_x().with(cy.pct().as_y()),
            ))
            .targeting(line)
            .during(lines_seq)
            .start(0)
            .finish(LINE_DRAW)
            .eased(Ease::DECELERATE),
        );
        lines[i] = line;
    }
    tree.sequence_end(lines_seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        build_down_move(&mut tree, router, forward, lines);
    });
}

/// Fires once the lines are truly done drawing: `forward` + both lines ride down
/// together to the resting spot.
fn build_down_move(tree: &mut Tree, router: Entity, forward: Entity, lines: [Entity; 2]) {
    let down_seq = tree.sequence();
    tree.animate(
        Animation::new(box_at(END_CENTER_X, REST_CENTER_Y))
            .targeting(forward)
            .during(down_seq)
            .start(0)
            .finish(DOWN_DURATION)
            .eased(Ease::DECELERATE),
    );
    for (line, (anchor_x, tip_x)) in lines.into_iter().zip(line_spans()) {
        tree.animate(
            Animation::new(Location::new().xs(
                anchor_x.pct().as_x().with(REST_CENTER_Y.pct().as_y()),
                tip_x.pct().as_x().with(REST_CENTER_Y.pct().as_y()),
            ))
            .targeting(line)
            .during(down_seq)
            .start(0)
            .finish(DOWN_DURATION)
            .eased(Ease::DECELERATE),
        );
    }
    tree.sequence_end(down_seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        build_icons(&mut tree, router, forward, lines);
    });
}

/// Fires once settled at rest: `forward`'s icon fades in, and only once THAT finishes
/// does `forward` become clickable ("enabled when the icon is done appearing"), both
/// click observers get registered, and the one-time `NavigatorLanded` signal fires.
/// `back` and its icon spawn *here* too -- not pre-created back at `build()` time (t=0,
/// before the event loop has even started) the way an earlier version had it. That
/// early-spawned `back_icon` never rendered despite every ECS-level property (opacity,
/// position, icon id, elevation ordering) checking out fine under a diagnostic dump --
/// spawning both here instead, at the exact same late point `forward_icon` already
/// spawns from successfully, sidesteps whatever that was.
fn build_icons(tree: &mut Tree, router: Entity, forward: Entity, lines: [Entity; 2]) {
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
        move |_: Trigger<OnEnd>, branches: Query<&Branch>, mut tree: Tree| {
            tree.enable(forward);

            // spawned now, already at its permanent rest spot -- muted, since there's
            // nothing to go back to yet -- then a staggered fade-in a beat later, for a
            // more playful two-part opener than a single polygon settling alone.
            let back = tree.leaf(
                Polygon::new()
                    .sides(7.0)
                    .rounding(0.55)
                    .rotation(0.0)
                    .color(Color::orange(400))
                    .at(box_at(BACK_X, REST_CENTER_Y))
                    .elevate(Elevation::up(10))
                    .with((
                        InteractionListener::new(),
                        InteractionShape::Circle,
                        Opacity::new(0.0),
                    )),
            );
            tree.disable(back); // nothing to go back to at index 0
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
                back,
                back_icon,
                lines,
            };
            tree.on_click(
                forward,
                move |_: Trigger<OnClick>,
                      page_state: Query<(&PageIndex, &PageCount)>,
                      polygons: Query<&Polygon>,
                      branches: Query<&Branch>,
                      mut tree: Tree| {
                    on_nav_click(&mut tree, nav, forward, 1, &page_state, &polygons, &branches);
                },
            );
            tree.on_click(
                back,
                move |_: Trigger<OnClick>,
                      page_state: Query<(&PageIndex, &PageCount)>,
                      polygons: Query<&Polygon>,
                      branches: Query<&Branch>,
                      mut tree: Tree| {
                    on_nav_click(&mut tree, nav, back, -1, &page_state, &polygons, &branches);
                },
            );

            tree.write_to(router, Landed);

            // structural, not timed: fires exactly when the icon is actually visible,
            // whatever that took, at whatever slot home actually built into.
            let slot = current_slot(router, &branches);
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
    branches: &Query<&Branch>,
) {
    if index == 0 && landed.get(router).is_ok() {
        let slot = current_slot(router, branches);
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
    branches: &Query<&Branch>,
) {
    tree.disable(nav.forward);
    tree.disable(nav.back);

    let (index, count) = page_state.get(nav.router).unwrap();
    let count = count.0;
    let next_index = (index.0 as i32 + direction).clamp(0, count as i32 - 1) as usize;
    let current = *polygons.get(clicked).unwrap();

    // fade out whatever the router's current scene actually contains.
    let slot = current_slot(nav.router, branches);

    let fade_seq = tree.sequence();
    if let Ok(slot_branch) = branches.get(slot) {
        for &content in slot_branch.ids.iter() {
            tree.animate(
                Animation::new(Opacity::new(0.0))
                    .targeting(content)
                    .during(fade_seq)
                    .start(0)
                    .finish(CONTENT_FADE_OUT)
                    .eased(Ease::Linear),
            );
        }
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
    for (line, (anchor_x, _tip_x)) in nav.lines.into_iter().zip(line_spans()) {
        tree.animate(
            Animation::new(Location::new().xs(
                anchor_x.pct().as_x().with(REST_CENTER_Y.pct().as_y()),
                anchor_x.pct().as_x().with(REST_CENTER_Y.pct().as_y()),
            ))
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
        build_redraw_lines(&mut tree, nav, next_index, count);
    });
}

fn build_redraw_lines(tree: &mut Tree, nav: Nav, next_index: usize, count: usize) {
    let redraw_seq = tree.sequence();
    for (line, (anchor_x, tip_x)) in nav.lines.into_iter().zip(line_spans()) {
        tree.animate(
            Animation::new(Location::new().xs(
                anchor_x.pct().as_x().with(REST_CENTER_Y.pct().as_y()),
                tip_x.pct().as_x().with(REST_CENTER_Y.pct().as_y()),
            ))
            .targeting(line)
            .during(redraw_seq)
            .start(0)
            .finish(REDRAW_LINES)
            .eased(Ease::DECELERATE),
        );
    }
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
    for target in [nav.forward, nav.forward_icon] {
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
