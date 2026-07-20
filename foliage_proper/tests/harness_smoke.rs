//! Confirms the one assumption every other test in this suite depends on: that
//! `Foliage::new()` + spawning/writing via the public API + a single `world.flush()`
//! is enough to fully settle both reactive composite structure and resolved layout
//! geometry, with no GPU/window and no schedule run required. If this fails, the fix
//! is one additional `foliage.main.run(&mut foliage.world);` before the flush -- see
//! this file's own assertions for exactly what "settled" means in practice.

use foliage_proper::{
    EcsExtension, Elevation, Foliage, GridExt, Leaf, Location, Logical, Section, Sprout,
};

#[test]
fn a_bare_leaf_resolves_its_section_after_one_flush() {
    let mut foliage = Foliage::new();
    let leaf = foliage.world.leaf(
        Leaf::sprout()
            .at(Location::new().xs(
                10.px().as_left().with(110.px().as_width()),
                20.px().as_top().with(220.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    foliage.world.flush();

    let section = foliage
        .world
        .get::<Section<Logical>>(leaf)
        .expect("Section<Logical> is required on every Leaf");
    assert_eq!(section.left(), 10.0, "left should resolve from the authored Location, not stay at its zeroed default");
    assert_eq!(section.top(), 20.0);
    assert_eq!(section.width(), 110.0, "as_left().with(110.px().as_width()) sets width directly to 110");
    assert_eq!(section.height(), 220.0);
}
