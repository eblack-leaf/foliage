//! An overlay closing on `Escape`, grown in the greenhouse.

use foliage::{Area, Elevation, Greenhouse, Grove, Grow, Key, Leaf, Location, Pollen, Root, Stem};
use lichen::{Closed, Overlay};

/// A page, and an overlay over it that the first `Escape` opens -- as a drawer closed by the key
/// might open a sheet asking whether to keep what was in it.
struct Page {
    overlay: Overlay,
    opened: bool,
    closed: Option<Closed>,
}

impl Root for Page {
    fn take_root(grove: &mut Grove) -> Self {
        let root: Leaf = grove.plant(Stem::new());
        let page = grove.branch(root, Stem::new());
        let overlay = Overlay::grow(grove, root, page, Location::new(), Elevation::up(10));
        Page {
            overlay,
            opened: false,
            closed: None,
        }
    }

    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        if pollen.escaped() && !self.opened {
            self.opened = true;
            self.overlay.open(grove);
        }
        if let Some(closed) = self.overlay.frame(grove, &pollen) {
            self.closed = Some(closed);
        }
    }
}

#[test]
fn an_escape_from_before_an_overlay_opened_does_not_close_it() {
    let mut house = Greenhouse::<Page>::new(Area::new(200.0, 100.0));
    house.frame();
    house.key(Key::Escape);
    house.settle();
    assert!(house.root().opened);
    assert!(
        house.root().overlay.is_open(),
        "closed by the Escape that opened it"
    );
    assert_eq!(house.root().closed, None);

    // One pressed while it is open closes it.
    house.key(Key::Escape);
    house.settle();
    assert!(!house.root().overlay.is_open());
    assert_eq!(house.root().closed, Some(Closed::Escape));
}
