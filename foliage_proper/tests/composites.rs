//! Structural reactivity across a spread of composite *shapes*, not concentrated in one
//! composite: `Toggle` (simplest possible boolean-state-driven case), `List` (count-driven
//! children), and `Polyline` (the newest, and the one whose `reconcile` pool-reuse guarantee
//! is a real correctness property worth pinning down, not just a math check).

use foliage_proper::{
    Color, EcsExtension, Elevation, Entity, Foliage, GridExt, Line, Location, Logical, Polygon,
    Polyline, PolylineDroppedPoints, PolylinePoints, Position, Section, Sprout, Stem, Toggle,
};

fn children_of(foliage: &mut Foliage, parent: Entity) -> Vec<Entity> {
    let mut q = foliage.world.query::<(Entity, &Stem)>();
    q.iter(&foliage.world)
        .filter(|(_, stem)| stem.id == Some(parent))
        .map(|(e, _)| e)
        .collect()
}

fn left_of(foliage: &mut Foliage, entity: Entity) -> f32 {
    foliage.world.get::<Section<Logical>>(entity).unwrap().left()
}

fn p(x: f32, y: f32) -> Position<Logical> {
    Position::logical((x, y))
}

// ---------- Toggle: simplest possible boolean-state case ----------

#[test]
fn a_freshly_spawned_toggles_knob_sits_further_right_when_on_than_off() {
    // avoids the composite's own "first fire places directly, later changes animate"
    // split (see `Toggle::build`) by comparing two independently-spawned toggles rather
    // than flipping one after spawn -- an animated re-placement wouldn't have resolved
    // its new position yet from a single `flush()` with no `main` schedule ticks, so
    // this sticks to the direct-placement path, which is what's actually being tested.
    let mut on_foliage = Foliage::new();
    let on_toggle = on_foliage.world.leaf(
        Toggle::new()
            .on(true)
            .at(Location::new().xs(
                0.px().as_left().with(60.px().as_width()),
                0.px().as_top().with(30.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    on_foliage.world.flush();

    let mut off_foliage = Foliage::new();
    let off_toggle = off_foliage.world.leaf(
        Toggle::new()
            .on(false)
            .at(Location::new().xs(
                0.px().as_left().with(60.px().as_width()),
                0.px().as_top().with(30.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    off_foliage.world.flush();

    // Toggle spawns exactly two children: track, then knob -- the knob is the second one.
    let on_knob = children_of(&mut on_foliage, on_toggle)[1];
    let off_knob = children_of(&mut off_foliage, off_toggle)[1];

    assert!(
        left_of(&mut on_foliage, on_knob) > left_of(&mut off_foliage, off_knob),
        "the on knob should sit further right than the off knob on an identically-sized toggle"
    );
}

// ---------- List: count-driven children ----------

#[test]
fn list_item_count_drives_child_count_one_to_one() {
    let mut foliage = Foliage::new();
    let list = foliage.world.leaf(
        foliage_proper::List::new()
            .items(foliage_proper::ListItems::new(5, |_tree, _slot, _i| {}))
            .row_height(20)
            .at(Location::new().xs(
                0.px().as_left().with(100.px().as_width()),
                0.px().as_top().with(200.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    foliage.world.flush();
    assert_eq!(children_of(&mut foliage, list).len(), 5);
}

#[test]
fn rewriting_list_items_to_a_new_count_matches_the_new_count_exactly() {
    let mut foliage = Foliage::new();
    let list = foliage.world.leaf(
        foliage_proper::List::new()
            .items(foliage_proper::ListItems::new(5, |_tree, _slot, _i| {}))
            .row_height(20)
            .at(Location::new().xs(
                0.px().as_left().with(100.px().as_width()),
                0.px().as_top().with(200.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    foliage.world.flush();
    foliage.write_to(list, foliage_proper::ListItems::new(2, |_tree, _slot, _i| {}));
    foliage.world.flush();
    assert_eq!(children_of(&mut foliage, list).len(), 2);
}

// ---------- Polyline: segment/joint count, and the reconcile pool-reuse guarantee ----------

#[test]
fn a_straight_polyline_produces_n_minus_one_segments_and_n_minus_two_joints() {
    let mut foliage = Foliage::new();
    let polyline = foliage.world.leaf(
        Polyline::new()
            .points(vec![p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0), p(20.0, 10.0)])
            .weight(3)
            .color(Color::gray(300))
            .at(Location::new().xs(
                0.px().as_left().with(100.px().as_width()),
                0.px().as_top().with(100.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    foliage.world.flush();
    let children = children_of(&mut foliage, polyline);
    let lines = children
        .iter()
        .filter(|e| foliage.world.get::<Line>(**e).is_some())
        .count();
    let joints = children
        .iter()
        .filter(|e| foliage.world.get::<Polygon>(**e).is_some())
        .count();
    assert_eq!(lines, 3, "4 points -> 3 segments");
    assert_eq!(joints, 2, "4 points -> 2 interior joints");
}

#[test]
fn rewriting_polyline_points_with_an_unchanged_segment_count_reuses_the_same_entities() {
    // the load-bearing guarantee `reconcile`'s own doc comment promises: a `PolylinePoints`
    // write that doesn't change the segment/joint count never spawns or despawns anything.
    // This regresses silently if it breaks -- respawned entities instead of reused ones
    // means restarted animations, lost focus/interaction state -- without ever panicking,
    // which is exactly why it needs a test, not just a comment.
    let mut foliage = Foliage::new();
    let polyline = foliage.world.leaf(
        Polyline::new()
            .points(vec![p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0)])
            .weight(3)
            .color(Color::gray(300))
            .at(Location::new().xs(
                0.px().as_left().with(100.px().as_width()),
                0.px().as_top().with(100.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    foliage.world.flush();
    let mut before = children_of(&mut foliage, polyline);
    before.sort();

    foliage.write_to(
        polyline,
        PolylinePoints(vec![p(0.0, 0.0), p(20.0, 0.0), p(20.0, 20.0)]),
    );
    foliage.world.flush();
    let mut after = children_of(&mut foliage, polyline);
    after.sort();

    assert_eq!(before, after, "same point/segment count -- must be the exact same entity set");
}

#[test]
fn dropping_points_from_the_front_still_produces_the_correct_final_child_count() {
    // PolylineDroppedPoints' own doc comment hedges "getting this wrong only costs perf,
    // never correctness" -- that kind of self-aware hedge is exactly what should have a
    // test next to it. This doesn't probe the optimization itself, just the thing its own
    // doc claims can never break: the end state is correct regardless.
    let mut foliage = Foliage::new();
    let polyline = foliage.world.leaf(
        Polyline::new()
            .points(vec![p(0.0, 0.0), p(10.0, 0.0), p(20.0, 0.0), p(30.0, 0.0)])
            .weight(3)
            .color(Color::gray(300))
            .at(Location::new().xs(
                0.px().as_left().with(100.px().as_width()),
                0.px().as_top().with(100.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    foliage.world.flush();

    // drop the first point: 4 points -> 3, reporting 1 total point ever dropped.
    foliage.write_to(
        polyline,
        (
            PolylinePoints(vec![p(10.0, 0.0), p(20.0, 0.0), p(30.0, 0.0)]),
            PolylineDroppedPoints(1),
        ),
    );
    foliage.world.flush();

    let children = children_of(&mut foliage, polyline);
    let lines = children
        .iter()
        .filter(|e| foliage.world.get::<Line>(**e).is_some())
        .count();
    let joints = children
        .iter()
        .filter(|e| foliage.world.get::<Polygon>(**e).is_some())
        .count();
    assert_eq!(lines, 2, "3 points -> 2 segments");
    assert_eq!(joints, 1, "3 points -> 1 interior joint");
}
