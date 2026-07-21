//! `Tabs` (`composite/tabs.rs`) has no tests at all. Its documented design point is worth
//! pinning down directly: switching tabs toggles `Visibility` on pre-built content slots,
//! it never re-runs the author's builder closure or respawns anything.

use bevy_ecs::observer::On;
use bevy_ecs::system::ResMut;
use foliage_proper::{
    Color, EcsExtension, Elevation, Entity, Foliage, GridExt, Leaf, Location, PageChanged,
    PageCount, PageIndex, Resource, SegmentedControl, Sprout, Stem, Tabs, TabsPages, Visibility,
};

fn children_of(foliage: &mut Foliage, parent: Entity) -> Vec<Entity> {
    let mut q = foliage.world.query::<(Entity, &Stem)>();
    q.iter(&foliage.world)
        .filter(|(_, stem)| stem.id == Some(parent))
        .map(|(e, _)| e)
        .collect()
}

fn spawn(foliage: &mut Foliage, labels: &[&str]) -> Entity {
    let owned: Vec<String> = labels.iter().map(|s| s.to_string()).collect();
    foliage.world.leaf(
        Tabs::new()
            .pages(TabsPages::new(owned, |tree, slot, _i| {
                tree.branch(slot, Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)));
            }))
            .colors(Color::green(500), Color::gray(600), Color::default())
            .at(Location::new().xs(
                0.px().as_left().with(300.px().as_width()),
                0.px().as_top().with(200.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    )
}

/// Content slots are the root's own `Stem`-children that carry `Visibility` and are NOT
/// the embedded `SegmentedControl` header (the header has none of these markers directly
/// on itself the same way).
fn content_slots(foliage: &mut Foliage, root: Entity) -> Vec<Entity> {
    children_of(foliage, root)
        .into_iter()
        .filter(|e| foliage.world.get::<Visibility>(*e).is_some() && foliage.world.get::<SegmentedControl>(*e).is_none())
        .collect()
}

#[test]
fn spawning_builds_one_content_slot_per_label_with_only_the_current_one_visible() {
    let mut foliage = Foliage::new();
    let tabs = spawn(&mut foliage, &["One", "Two", "Three"]);
    foliage.world.flush();

    let slots = content_slots(&mut foliage, tabs);
    assert_eq!(slots.len(), 3);
    let visible: Vec<bool> = slots
        .iter()
        .map(|e| foliage.world.get::<Visibility>(*e).unwrap().visible())
        .collect();
    assert_eq!(visible.iter().filter(|v| **v).count(), 1, "exactly one tab's content should start visible");
}

#[test]
fn switching_pages_toggles_visibility_without_respawning_slots() {
    let mut foliage = Foliage::new();
    let tabs = spawn(&mut foliage, &["One", "Two", "Three"]);
    foliage.world.flush();
    let mut before = content_slots(&mut foliage, tabs);
    before.sort();

    foliage.write_to(tabs, PageIndex(2));
    foliage.world.flush();

    let mut after = content_slots(&mut foliage, tabs);
    after.sort();
    assert_eq!(before, after, "switching tabs must not respawn content -- only Visibility should change");

    let now_visible: Vec<Entity> = after
        .iter()
        .filter(|e| foliage.world.get::<Visibility>(**e).unwrap().visible())
        .copied()
        .collect();
    assert_eq!(now_visible.len(), 1, "still exactly one visible slot after switching");
}

#[test]
fn switching_pages_fires_page_changed() {
    #[derive(Resource, Default)]
    struct Last(Option<usize>);
    fn mark(trigger: On<PageChanged>, mut r: ResMut<Last>) {
        r.0 = Some(trigger.event().index);
    }

    let mut foliage = Foliage::new();
    foliage.world.insert_resource(Last::default());
    foliage.world.add_observer(mark);
    let tabs = spawn(&mut foliage, &["One", "Two", "Three"]);
    foliage.world.flush();
    assert_eq!(foliage.world.resource::<Last>().0, Some(0));

    foliage.write_to(tabs, PageIndex(1));
    foliage.world.flush();

    assert_eq!(foliage.world.resource::<Last>().0, Some(1));
}

#[test]
fn the_header_is_a_real_segmented_control_kept_in_sync_with_page_index() {
    let mut foliage = Foliage::new();
    let tabs = spawn(&mut foliage, &["One", "Two", "Three"]);
    foliage.world.flush();

    let header = children_of(&mut foliage, tabs)
        .into_iter()
        .find(|e| foliage.world.get::<SegmentedControl>(*e).is_some())
        .expect("Tabs should branch a real SegmentedControl as its header");

    foliage.write_to(tabs, PageIndex(2));
    foliage.world.flush();

    assert_eq!(
        foliage.world.get::<foliage_proper::SegmentedSelected>(header).unwrap().0,
        2,
        "the header's own selection should track PageIndex"
    );
}

#[test]
fn rewriting_tabs_pages_matches_the_new_label_count() {
    let mut foliage = Foliage::new();
    let tabs = spawn(&mut foliage, &["One", "Two", "Three"]);
    foliage.world.flush();

    foliage.write_to(
        tabs,
        TabsPages::new(vec!["A".into(), "B".into()], |tree, slot, _i| {
            tree.branch(slot, Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)));
        }),
    );
    foliage.world.flush();

    assert_eq!(foliage.world.get::<PageCount>(tabs).unwrap().0, 2);
    assert_eq!(content_slots(&mut foliage, tabs).len(), 2);
}

#[test]
fn switching_to_the_last_tab_after_shrinking_the_page_set_shows_a_real_slot() {
    // regression case for the PageCount-goes-stale bug above: shrink 3 tabs to 2, then
    // select the new last index (1) -- if PageCount had stayed stale at 3, this clamp
    // would have nothing to gate on and a later index-2 write could show no slot at all.
    let mut foliage = Foliage::new();
    let tabs = spawn(&mut foliage, &["One", "Two", "Three"]);
    foliage.world.flush();

    foliage.write_to(
        tabs,
        TabsPages::new(vec!["A".into(), "B".into()], |tree, slot, _i| {
            tree.branch(slot, Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)));
        }),
    );
    foliage.world.flush();
    foliage.write_to(tabs, PageIndex(1));
    foliage.world.flush();

    let slots = content_slots(&mut foliage, tabs);
    assert_eq!(slots.len(), 2);
    let visible: Vec<Entity> = slots
        .iter()
        .filter(|e| foliage.world.get::<Visibility>(**e).unwrap().visible())
        .copied()
        .collect();
    assert_eq!(visible.len(), 1, "exactly one of the two real slots should be visible");
}
