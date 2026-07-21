//! `Pagination` (`composite/pagination.rs`) has no tests at all.

use bevy_ecs::observer::On;
use bevy_ecs::system::ResMut;
use foliage_proper::{
    Color, EcsExtension, Elevation, Entity, Foliage, GridExt, Location, PageChanged, PageCount,
    PageIndex, Pagination, PaginationMode, PaginationStyle, Resource, Sprout, Stem,
};

fn children_of(foliage: &mut Foliage, parent: Entity) -> Vec<Entity> {
    let mut q = foliage.world.query::<(Entity, &Stem)>();
    q.iter(&foliage.world)
        .filter(|(_, stem)| stem.id == Some(parent))
        .map(|(e, _)| e)
        .collect()
}

/// Every descendant, not just direct children -- the indicator strip is one extra level
/// down (`Pagination` branches a `strip` leaf, indicators branch under *that*), and dot
/// mode nests one level further still (an invisible hit region, then the visual pip).
fn descendants_of(foliage: &mut Foliage, root: Entity) -> Vec<Entity> {
    let mut out = Vec::new();
    let mut frontier = vec![root];
    while let Some(e) = frontier.pop() {
        let kids = children_of(foliage, e);
        frontier.extend(kids.iter().copied());
        out.extend(kids);
    }
    out
}

fn spawn(foliage: &mut Foliage, count: usize) -> Entity {
    foliage.world.leaf(
        Pagination::new(count)
            .colors(Color::green(500), Color::gray(600))
            .at(Location::new().xs(
                0.px().as_left().with(200.px().as_width()),
                0.px().as_top().with(24.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    )
}

#[test]
fn dots_mode_produces_one_dot_per_page() {
    let mut foliage = Foliage::new();
    let pagination = spawn(&mut foliage, 5);
    foliage.world.flush();

    // each dot is (hit region -> Panel with InteractionListener) -> (visual pip -> Panel,
    // no listener) -- 5 pages means 5 hit regions, 10 total Panel-shaped descendants.
    let hit_regions = descendants_of(&mut foliage, pagination)
        .iter()
        .filter(|e| foliage.world.get::<foliage_proper::InteractionListener>(**e).is_some())
        .count();
    assert_eq!(hit_regions, 5);
}

#[test]
fn writing_page_index_fires_page_changed_with_the_clamped_value() {
    #[derive(Resource, Default)]
    struct Last(Option<usize>);
    fn mark(trigger: On<PageChanged>, mut r: ResMut<Last>) {
        r.0 = Some(trigger.event().index);
    }

    let mut foliage = Foliage::new();
    foliage.world.insert_resource(Last::default());
    foliage.world.add_observer(mark);
    let pagination = spawn(&mut foliage, 5);
    foliage.world.flush();
    assert_eq!(foliage.world.resource::<Last>().0, Some(0));

    foliage.write_to(pagination, PageIndex(3));
    foliage.world.flush();
    assert_eq!(foliage.world.resource::<Last>().0, Some(3));

    foliage.write_to(pagination, PageIndex(999));
    foliage.world.flush();
    assert_eq!(foliage.world.resource::<Last>().0, Some(4), "out-of-range should clamp to the last page, not panic");
}

#[test]
fn a_page_change_does_not_respawn_the_indicator_entities() {
    let mut foliage = Foliage::new();
    let pagination = spawn(&mut foliage, 5);
    foliage.world.flush();
    let mut before = descendants_of(&mut foliage, pagination);
    before.sort();

    foliage.write_to(pagination, PageIndex(2));
    foliage.world.flush();

    let mut after = descendants_of(&mut foliage, pagination);
    after.sort();
    assert_eq!(before, after, "PATCH recolors stable indicators -- no spawns, no removes");
}

#[test]
fn rewriting_page_count_rebuilds_the_indicator_set() {
    let mut foliage = Foliage::new();
    let pagination = spawn(&mut foliage, 5);
    foliage.world.flush();

    foliage.write_to(pagination, PageCount(2));
    foliage.world.flush();

    let hit_regions = descendants_of(&mut foliage, pagination)
        .iter()
        .filter(|e| foliage.world.get::<foliage_proper::InteractionListener>(**e).is_some())
        .count();
    assert_eq!(hit_regions, 2);
}

#[test]
fn numbered_mode_shows_at_most_five_slots_regardless_of_page_count() {
    let mut foliage = Foliage::new();
    let pagination = foliage.world.leaf(
        Pagination::new(20)
            .mode(PaginationMode::Numbered)
            .colors(Color::green(500), Color::gray(600))
            .at(Location::new().xs(
                0.px().as_left().with(300.px().as_width()),
                0.px().as_top().with(24.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    foliage.world.flush();

    let listeners = descendants_of(&mut foliage, pagination)
        .iter()
        .filter(|e| foliage.world.get::<foliage_proper::InteractionListener>(**e).is_some())
        .count();
    assert_eq!(listeners, 5, "Numbered mode caps at a 5-slot sliding window even with 20 pages");
}

#[test]
fn spawning_with_a_zero_style_writes_a_default_pagination_style() {
    let mut foliage = Foliage::new();
    let pagination = spawn(&mut foliage, 5);
    foliage.world.flush();

    let style = foliage.world.get::<PaginationStyle>(pagination).unwrap();
    assert!(matches!(style.mode, PaginationMode::Dots), "Dots is the documented default mode");
}
