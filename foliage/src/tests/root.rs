use super::{grove, tick_with};
use crate::grove::Grove;
use crate::leaf::{Leaf, Presence};
use crate::pollen::Pollen;
use crate::root::{Registered, Root};
use crate::stem::Stem;
use crate::vein::{Sap, Vein};
use crate::verbs::Grow;

/// Grows a trunk and a branch beneath it as it takes root, and counts the frames it is called in.
struct App {
    trunk: Leaf,
    branch: Leaf,
    frames: usize,
}

impl Root for App {
    fn take_root(grove: &mut Grove) -> Self {
        let trunk = grove.plant(Stem::new());
        let branch = grove.branch(trunk, Stem::new());
        Self {
            trunk,
            branch,
            frames: 0,
        }
    }

    fn frame(&mut self, _grove: &mut Grove, _pollen: Pollen) {
        self.frames += 1;
    }
}

#[test]
fn taking_root_happens_inside_the_first_frame() {
    let mut grove = grove();
    let mut registered = Registered::<App>::new();

    tick_with(&mut grove, &mut registered);

    let app = registered.0.as_ref().expect("took root");
    assert_eq!(app.frames, 1);
    assert_eq!(grove.presence(app.trunk), Presence::Live);
    assert_eq!(grove.presence(app.branch), Presence::Live);
}

#[test]
fn a_leaf_taken_root_with_is_usable_as_a_trunk_in_that_frame() {
    let mut grove = grove();
    let mut registered = Registered::<App>::new();

    tick_with(&mut grove, &mut registered);

    let app = registered.0.as_ref().expect("took root");
    assert_eq!(
        grove.tap(app.trunk, Vein::Branches),
        Some(Sap::Leaves(vec![app.branch]))
    );
    assert_eq!(
        grove.tap(app.branch, Vein::Trunk),
        Some(Sap::Leaf(Some(app.trunk)))
    );
}

#[test]
fn taking_root_happens_once_however_many_frames_run() {
    let mut grove = grove();
    let mut registered = Registered::<App>::new();

    for _ in 0..4 {
        tick_with(&mut grove, &mut registered);
    }

    let app = registered.0.as_ref().expect("took root");
    assert_eq!(app.frames, 4);
    assert_eq!(
        grove.tap(app.trunk, Vein::Branches),
        Some(Sap::Leaves(vec![app.branch]))
    );
}
