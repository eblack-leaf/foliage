use core::time::Duration;

use super::{Observer, advance, grove, resize, section, tick, tick_with};
use crate::coordinate::{Area, Section};
use crate::grove::Grove;
use crate::leaf::{Leaf, Presence};
use crate::placement::grid::Grid;
use crate::placement::location::Location;
use crate::pollen::Pollen;
use crate::root::Rooted;
use crate::stem::Stem;
use crate::vein::{Sap, Vein};
use crate::verbs::Grow;
use crate::{Divide, Place, Source, anchor, left, top};

/// How many elements are branched off `leaf`, or `None` if it is not there to ask.
fn branches(grove: &Grove, leaf: Leaf) -> Option<usize> {
    match grove.tap(leaf, Vein::Branches)? {
        Sap::Leaves(leaves) => Some(leaves.len()),
        _ => None,
    }
}

/// `first` and `second` are the same op against the same trunk. Only their position in the queue
/// differs, and that alone decides whether either lands.
#[test]
fn the_drain_is_fifo_within_one_frame() {
    let mut grove = grove();
    let trunk = grove.plant(Stem::new());
    let first = grove.branch(trunk, Stem::new());
    grove.prune(trunk);
    let second = grove.branch(trunk, Stem::new());
    tick(&mut grove);

    assert_eq!(grove.presence(trunk), Presence::Withered);
    assert_eq!(grove.presence(first), Presence::Withered);
    assert_eq!(grove.presence(second), Presence::Planted);
}

/// The consequence F2 exists to guarantee, and the reason the drain is in arrival order rather than
/// grouped by kind: a grow and a write to the same `Leaf` in one frame behave the way they read. A
/// `Leaf` is usable the instant it is handed out, so nothing has to be deferred to the frame after.
#[test]
fn a_write_lands_in_the_frame_that_planted_its_leaf() {
    let mut grove = grove();
    let trunk = grove.plant(Stem::new());
    let leaf = grove.branch(trunk, Stem::new());
    grove.at(
        leaf,
        Location::new().xs(left(2.col()).right(2.col()), top(0.px()).height(10.px())),
    );
    grove.grid(trunk, Grid::new().xs(2.columns(), 1.rows()));
    grove.anchor(leaf, trunk);
    tick(&mut grove);

    assert_eq!(grove.presence(leaf), Presence::Live);
    // The second of the trunk's two columns, which the trunk was only divided into after the
    // element addressing it had been branched.
    assert_eq!(
        section(&grove, leaf),
        Section::from_edges(200.0, 0.0, 400.0, 10.0)
    );
    assert_eq!(grove.tap(leaf, Vein::Anchor), Some(Sap::Leaf(Some(trunk))));
}

/// Two writes to one property in one frame are not a conflict and not a half-written state: they
/// are ordered, and the later one is what the element ends the frame with.
#[test]
fn the_last_write_of_a_frame_is_the_one_that_lands() {
    let mut grove = grove();
    let leaf = grove.plant(Stem::new());
    grove.at(
        leaf,
        Location::new().xs(left(0.px()).width(10.px()), top(0.px()).height(10.px())),
    );
    grove.at(
        leaf,
        Location::new().xs(left(50.px()).width(20.px()), top(60.px()).height(30.px())),
    );
    tick(&mut grove);

    assert_eq!(
        section(&grove, leaf),
        Section::from_edges(50.0, 60.0, 70.0, 90.0)
    );
}

/// Position in the queue is the whole of what decides, and the kind of op is no part of it: the same
/// `anchor` lands or is dropped depending only on which side of the `prune` it arrived on.
#[test]
fn a_write_that_arrives_after_a_prune_is_dropped() {
    let mut grove = grove();
    let first = grove.plant(
        Stem::new()
            .at(Location::new().xs(left(0.px()).width(10.px()), top(0.px()).height(10.px()))),
    );
    let second = grove.plant(
        Stem::new()
            .at(Location::new().xs(left(0.px()).width(10.px()), top(0.px()).height(10.px()))),
    );
    let follower = grove.plant(Stem::new().at(Location::new().xs(
        left(anchor().right()).width(10.px()),
        top(0.px()).height(10.px()),
    )));
    tick(&mut grove);

    grove.anchor(follower, first);
    grove.prune(second);
    grove.anchor(follower, second);
    tick(&mut grove);

    assert_eq!(
        grove.tap(follower, Vein::Anchor),
        Some(Sap::Leaf(Some(first)))
    );
}

/// The drain does not group by kind. A `grid` either side of an unrelated `at` applies in the order
/// the two were written, not in an order the engine chose for them.
#[test]
fn ops_of_different_kinds_are_not_reordered_against_each_other() {
    let mut grove = grove();
    let trunk = grove.plant(Stem::new());
    let leaf = grove.branch(
        trunk,
        Stem::new()
            .at(Location::new().xs(left(2.col()).right(2.col()), top(0.px()).height(10.px()))),
    );
    tick(&mut grove);

    grove.grid(trunk, Grid::new().xs(2.columns(), 1.rows()));
    grove.at(
        leaf,
        Location::new().xs(left(2.col()).right(2.col()), top(0.px()).height(20.px())),
    );
    grove.grid(trunk, Grid::new().xs(4.columns(), 1.rows()));
    tick(&mut grove);

    // Both writes landed, and the second of four columns is what the later `grid` left in force.
    assert_eq!(
        section(&grove, leaf),
        Section::from_edges(100.0, 0.0, 200.0, 20.0)
    );
}

/// Moves its own element every frame, reading its box either side of the write.
#[derive(Default)]
struct Mover {
    leaf: Option<Leaf>,
    reads: Vec<(Option<Sap>, Option<Sap>)>,
}

impl Rooted for Mover {
    fn frame(&mut self, grove: &mut Grove, _pollen: Pollen) {
        let leaf = *self.leaf.get_or_insert_with(|| grove.plant(Stem::new()));
        let step = self.reads.len() as f32 * 10.0;
        let before = grove.tap(leaf, Vein::Drawn);
        grove.at(
            leaf,
            Location::new().xs(left(step.px()).width(10.px()), top(0.px()).height(10.px())),
        );
        let after = grove.tap(leaf, Vein::Drawn);
        self.reads.push((before, after));
    }
}

/// F3 for a write rather than a grow: the state read at the top of a frame is the state read at the
/// bottom, so an app cannot observe its own write and never has to guess which side of it a read is
/// on.
#[test]
fn a_move_is_not_visible_inside_the_frame_that_made_it() {
    let mut grove = grove();
    let mut app = Mover::default();
    for _ in 0..3 {
        tick_with(&mut grove, &mut app);
    }

    for (before, after) in &app.reads {
        assert_eq!(before, after);
    }
    // And the reads differ from one frame to the next, so the equality above is not vacuous: each
    // write did land, at the drain that followed it.
    let box_at = |left: f32| {
        Some(Sap::Section(Section::from_edges(
            left,
            0.0,
            left + 10.0,
            10.0,
        )))
    };
    assert_eq!(app.reads[0].0, None);
    assert_eq!(app.reads[1].0, box_at(0.0));
    assert_eq!(app.reads[2].0, box_at(10.0));
}

/// Prunes its own element every frame, reading its presence either side.
#[derive(Default)]
struct Pruner {
    leaf: Option<Leaf>,
    reads: Vec<(Presence, Presence)>,
}

impl Rooted for Pruner {
    fn frame(&mut self, grove: &mut Grove, _pollen: Pollen) {
        let leaf = *self.leaf.get_or_insert_with(|| grove.plant(Stem::new()));
        let before = grove.presence(leaf);
        grove.prune(leaf);
        let after = grove.presence(leaf);
        self.reads.push((before, after));
    }
}

/// F3 for a teardown. An element pruned inside a frame is still there for the rest of it, so code
/// after the call reads a tree that has not moved under it.
#[test]
fn a_prune_is_not_visible_inside_the_frame_that_made_it() {
    let mut grove = grove();
    let mut app = Pruner::default();

    tick_with(&mut grove, &mut app);
    tick_with(&mut grove, &mut app);

    // Planted and pruned in one frame: the drain grows it and then takes it down, and neither is
    // visible to the frame that asked for them.
    assert_eq!(app.reads[0], (Presence::Planted, Presence::Planted));
    assert_eq!(app.reads[1], (Presence::Withered, Presence::Withered));
}

/// Reads the branches of its own element either side of writing one, every frame it runs.
#[derive(Default)]
struct Reader {
    leaf: Option<Leaf>,
    reads: Vec<(Option<usize>, Option<usize>)>,
}

impl Rooted for Reader {
    fn frame(&mut self, grove: &mut Grove, _pollen: Pollen) {
        let leaf = *self.leaf.get_or_insert_with(|| grove.plant(Stem::new()));
        let before = branches(grove, leaf);
        grove.branch(leaf, Stem::new());
        let after = branches(grove, leaf);
        self.reads.push((before, after));
    }
}

/// What a frame reads cannot change while it runs, and what it wrote is what the next frame reads.
#[test]
fn reads_do_not_change_inside_a_frame() {
    let mut grove = grove();
    let mut app = Reader::default();
    for _ in 0..3 {
        tick_with(&mut grove, &mut app);
    }

    assert_eq!(
        app.reads.as_slice(),
        [(None, None), (Some(1), Some(1)), (Some(2), Some(2))]
    );
}

#[test]
fn a_resize_is_reported_once_and_takes_effect_that_frame() {
    let mut grove = grove();
    let mut app = Observer::default();
    let resized = Area::new(800.0, 600.0);

    assert_eq!(grove.viewport(), Area::new(400.0, 300.0));
    resize(&mut grove, resized);
    assert_eq!(grove.viewport(), Area::new(400.0, 300.0));

    tick_with(&mut grove, &mut app);
    assert_eq!(grove.viewport(), resized);
    assert_eq!(app.last().resized(), Some(resized));

    tick_with(&mut grove, &mut app);
    assert_eq!(grove.viewport(), resized);
    assert_eq!(app.last().resized(), None);
}

/// Asks for another frame on its first run and never again.
#[derive(Default)]
struct Asks {
    frames: usize,
}

impl Rooted for Asks {
    fn frame(&mut self, grove: &mut Grove, _pollen: Pollen) {
        if self.frames == 0 {
            grove.again();
        }
        self.frames += 1;
    }
}

#[test]
fn asking_for_another_frame_lasts_exactly_one_frame() {
    let mut grove = grove();
    let mut app = Asks::default();

    tick_with(&mut grove, &mut app);
    assert!(grove.again);

    tick_with(&mut grove, &mut app);
    assert!(!grove.again);
}

#[test]
fn advance_is_exact() {
    let mut grove = grove();

    advance(&mut grove, 250);
    tick(&mut grove);
    assert_eq!(grove.frame_time(), Duration::from_millis(250));
    assert_eq!(grove.elapsed(), Duration::from_millis(250));

    tick(&mut grove);
    assert_eq!(grove.frame_time(), Duration::ZERO);
    assert_eq!(grove.elapsed(), Duration::from_millis(250));

    advance(&mut grove, 100);
    advance(&mut grove, 150);
    tick(&mut grove);
    assert_eq!(grove.frame_time(), Duration::from_millis(250));
    assert_eq!(grove.elapsed(), Duration::from_millis(500));
}

#[test]
fn the_clock_does_not_move_without_being_advanced() {
    let mut grove = grove();
    for _ in 0..8 {
        tick(&mut grove);
        assert_eq!(grove.elapsed(), Duration::ZERO);
        assert_eq!(grove.frame_time(), Duration::ZERO);
    }
}

/// Everything about the tree a test can see, for comparing one run against another.
fn state(grove: &Grove, leaves: &[Leaf]) -> Vec<(u64, Presence, Option<Sap>, Option<Sap>)> {
    leaves
        .iter()
        .map(|leaf| {
            (
                leaf.id(),
                grove.presence(*leaf),
                grove.tap(*leaf, Vein::Branches),
                grove.tap(*leaf, Vein::Trunk),
            )
        })
        .collect()
}

fn script(grove: &mut Grove, idle: bool) -> Vec<Leaf> {
    let a = grove.plant(Stem::new());
    let b = grove.branch(a, Stem::new());
    tick(grove);
    if idle {
        tick(grove);
        tick(grove);
    }

    let c = grove.branch(b, Stem::new());
    let d = grove.branch(a, Stem::new());
    grove.prune(b);
    tick(grove);
    if idle {
        tick(grove);
    }

    let e = grove.branch(d, Stem::new());
    tick(grove);

    vec![a, b, c, d, e]
}

#[test]
fn the_same_script_produces_the_same_state() {
    let mut one = grove();
    let mut two = grove();

    let first = script(&mut one, false);
    let second = script(&mut two, false);

    assert_eq!(state(&one, &first), state(&two, &second));
}

#[test]
fn idle_frames_change_nothing() {
    let mut busy = grove();
    let mut idle = grove();

    let without = script(&mut busy, false);
    let with = script(&mut idle, true);

    assert_eq!(state(&busy, &without), state(&idle, &with));
}
