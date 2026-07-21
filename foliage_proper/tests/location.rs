//! `Location`'s resolve pipeline (`grid/location.rs`) is the single highest-blast-radius
//! piece of the whole framework -- every visible pixel of every entity passes through it.
//! The actual `resolve()` function is private and takes several hand-constructed internal
//! types (`GridConfiguration`, `View`, `Designator`-paired `Section`s); rather than guess
//! at those semantics directly, these tests exercise the same distinct configs through the
//! public API (spawn via `Sprout`, one `world.flush()`, assert on the resolved
//! `Section<Logical>`) -- the same harness `harness_smoke.rs` already proved settles fully
//! headless with no schedule run required.

use foliage_proper::{EcsExtension, Elevation, Foliage, Grid, GridExt, Layout, Leaf, Location, Logical, Section, Sprout};

fn section_of(foliage: &mut Foliage, entity: foliage_proper::Entity) -> Section<Logical> {
    *foliage
        .world
        .get::<Section<Logical>>(entity)
        .expect("Section<Logical> is required on every Leaf")
}

#[test]
fn plain_px_left_and_right_resolve_to_the_span_between_them() {
    let mut foliage = Foliage::new();
    let leaf = foliage.world.leaf(
        Leaf::sprout()
            .at(Location::new().xs(
                10.px().as_left().with(110.px().as_right()),
                5.px().as_top().with(55.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );
    foliage.world.flush();
    let section = section_of(&mut foliage, leaf);
    assert_eq!(section.left(), 10.0);
    assert_eq!(section.width(), 100.0, "right(110) - left(10)");
    assert_eq!(section.top(), 5.0);
    assert_eq!(section.height(), 50.0);
}

#[test]
fn percentage_of_parent_resolves_relative_to_the_parents_own_resolved_section() {
    let mut foliage = Foliage::new();
    let parent = foliage.world.leaf(
        Leaf::sprout()
            .at(Location::new().xs(
                0.px().as_left().with(200.px().as_width()),
                0.px().as_top().with(100.px().as_height()),
            ))
            .elevate(Elevation::up(1))
            // required: any entity that's going to have children with a real (non-empty)
            // Location needs `Grid` on itself -- resolving a child's Location always reads
            // its parent's `Grid` unconditionally (`grids.get(id).unwrap()` in
            // `location.rs`), regardless of whether the *child* anchors with px or with
            // pct/col/row. (First read as "only pct/col/row children need it" -- wrong;
            // confirmed by a plain-px child under a Grid-less parent panicking the same
            // way in `foliage/examples/opacity_and_elevation.rs`.) This bit `Polyline`'s
            // own authoring earlier -- this test exists to catch exactly that class of
            // mistake, not just document it.
            .with(Grid::default()),
    );
    let child = foliage.world.branch(
        parent,
        Leaf::sprout()
            .at(Location::new().xs(
                0.pct().as_left().with(50.pct().as_right()),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );
    foliage.world.flush();
    let child_section = section_of(&mut foliage, child);
    assert_eq!(child_section.width(), 100.0, "50% of the parent's 200-wide context");
    assert_eq!(child_section.height(), 100.0, "100% of the parent's 100-tall context");
}

#[test]
#[should_panic(expected = "QueryDoesNotMatch")]
fn a_grid_less_parent_panics_resolving_a_child_even_with_a_plain_px_location() {
    // the gap that slipped through above: it's tempting to read "a child anchored with
    // pct/col/row needs its parent's Grid" and assume plain-px children are exempt.
    // They're not -- resolving *any* child with a real (non-empty) Location reads its
    // parent's Grid unconditionally, regardless of anchor style. This is a `#[should_panic]`
    // rather than a passing case on purpose: it documents a real, current sharp edge (an
    // easy mistake to make authoring a new composite, as `Polyline` did) rather than
    // silently working around it -- if this ever stops panicking, that's worth noticing,
    // not something to quietly let this test start failing on.
    let mut foliage = Foliage::new();
    let parent = foliage
        .world
        .leaf(Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)));
    foliage.world.branch(
        parent,
        Leaf::sprout()
            .at(Location::new().xs(
                10.px().as_left().with(50.px().as_width()),
                10.px().as_top().with(50.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    foliage.world.flush();
}

#[test]
fn column_anchors_split_the_parents_grid_evenly() {
    let mut foliage = Foliage::new();
    let parent = foliage.world.leaf(
        Leaf::sprout()
            .at(Location::new().xs(
                0.px().as_left().with(400.px().as_width()),
                0.px().as_top().with(100.px().as_height()),
            ))
            .elevate(Elevation::up(1))
            .with(Grid::new(4.col().gap(0), 1.row().gap(0))),
    );
    // second of 4 equal columns in a 400-wide grid: [100, 200)
    let child = foliage.world.branch(
        parent,
        Leaf::sprout()
            .at(Location::new().xs(
                2.col().as_left().with(2.col().as_right()),
                1.row().as_top().with(1.row().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );
    foliage.world.flush();
    let child_section = section_of(&mut foliage, child);
    assert_eq!(child_section.left(), 100.0);
    assert_eq!(child_section.width(), 100.0);
}

#[test]
fn grids_nested_two_levels_deep_resolve_relative_to_their_own_immediate_parent() {
    // only one level of parent-Grid nesting had ever been exercised -- a bug in how a
    // child's resolved Section feeds into ITS OWN Grid for further resolution (as opposed
    // to always resolving against some outer/root Grid) wouldn't have been caught.
    let mut foliage = Foliage::new();
    let outer = foliage.world.leaf(
        Leaf::sprout()
            .at(Location::new().xs(
                0.px().as_left().with(400.px().as_width()),
                0.px().as_top().with(200.px().as_height()),
            ))
            .elevate(Elevation::up(1))
            .with(Grid::new(2.col().gap(0), 1.row().gap(0))),
    );
    // left half of outer's 400-wide grid: [0, 200)
    let middle = foliage.world.branch(
        outer,
        Leaf::sprout()
            .at(Location::new().xs(
                1.col().as_left().with(1.col().as_right()),
                1.row().as_top().with(1.row().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .with(Grid::new(2.col().gap(0), 1.row().gap(0))),
    );
    // right half of middle's OWN 200-wide grid: [100, 200) relative to middle, i.e. [300, 400)
    // in absolute terms -- only correct if resolution used middle's resolved Section (not
    // outer's) as the basis for this Grid.
    let innermost = foliage.world.branch(
        middle,
        Leaf::sprout()
            .at(Location::new().xs(
                2.col().as_left().with(2.col().as_right()),
                1.row().as_top().with(1.row().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );
    foliage.world.flush();

    let middle_section = section_of(&mut foliage, middle);
    assert_eq!(middle_section.left(), 0.0);
    assert_eq!(middle_section.width(), 200.0);

    let innermost_section = section_of(&mut foliage, innermost);
    assert_eq!(innermost_section.left(), 100.0, "middle's own left(0) + its second-half column start(100)");
    assert_eq!(innermost_section.width(), 100.0);
}

#[test]
fn a_non_xs_layout_falls_back_to_the_nearest_smaller_configured_breakpoint() {
    let mut foliage = Foliage::new();
    foliage.world.insert_resource(Layout::Md);
    let leaf = foliage.world.leaf(
        Leaf::sprout()
            .at(Location::new().xs(
                10.px().as_left().with(60.px().as_width()),
                10.px().as_top().with(60.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    foliage.world.flush();

    let section = section_of(&mut foliage, leaf);
    assert_eq!(section.width(), 60.0, "only xs is configured -- an active Md layout should fall back to it");
}

#[test]
fn a_specific_breakpoint_config_wins_over_the_fallback_when_it_is_the_active_layout() {
    let mut foliage = Foliage::new();
    foliage.world.insert_resource(Layout::Md);
    let leaf = foliage.world.leaf(
        Leaf::sprout()
            .at(Location::new()
                .xs(
                    0.px().as_left().with(50.px().as_width()),
                    0.px().as_top().with(50.px().as_height()),
                )
                .md(
                    0.px().as_left().with(150.px().as_width()),
                    0.px().as_top().with(150.px().as_height()),
                ))
            .elevate(Elevation::up(1)),
    );
    foliage.world.flush();

    let section = section_of(&mut foliage, leaf);
    assert_eq!(section.width(), 150.0, "Md is explicitly configured and active -- it should win over the xs fallback");
}
