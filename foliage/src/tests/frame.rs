use core::time::Duration;

use super::{Observer, advance, grove, resize, tick, tick_with};
use crate::coordinate::Area;
use crate::grove::Grove;
use crate::leaf::{Leaf, Presence};
use crate::pollen::Pollen;
use crate::root::Rooted;
use crate::stem::Stem;
use crate::vein::{Sap, Vein};
use crate::verbs::Grow;

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
