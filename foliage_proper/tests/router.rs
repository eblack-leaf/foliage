//! `Router` (`composite/router.rs`): destructive scene switching. The load-bearing claims
//! are (1) exactly one route's content exists at a time, (2) navigating DESTROYS the old
//! scene rather than hiding it, (3) re-entering a route rebuilds it from nothing, and
//! (4) it speaks the shared paging vocabulary (`PageIndex` in, `PageChanged` out).

use bevy_ecs::observer::On;
use bevy_ecs::system::ResMut;
use foliage_proper::{
    Component, EcsExtension, Elevation, Entity, Foliage, GridExt, Leaf, Location, PageChanged,
    PageCount, PageIndex, Resource, Router, RouterRoutes, Sprout, Tree,
};

#[derive(Component)]
struct SceneA;
#[derive(Component)]
struct SceneB;

// Bare `fn`s, as `RouteFn` requires -- the type itself forbids capturing, so these can only
// build from what they're handed.
fn scene_a(tree: &mut Tree, slot: Entity) {
    tree.branch(
        slot,
        Leaf::sprout()
            .at(Location::new())
            .elevate(Elevation::up(1))
            .with(SceneA),
    );
}
fn scene_b(tree: &mut Tree, slot: Entity) {
    tree.branch(
        slot,
        Leaf::sprout()
            .at(Location::new())
            .elevate(Elevation::up(1))
            .with(SceneB),
    );
}

fn spawn(foliage: &mut Foliage) -> Entity {
    foliage.world.leaf(
        Router::new()
            .routes(RouterRoutes::new([
                scene_a as foliage_proper::RouteFn,
                scene_b,
            ]))
            .at(Location::new().xs(
                0.px().as_left().with(300.px().as_width()),
                0.px().as_top().with(200.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    )
}

fn entities_with<C: Component>(foliage: &mut Foliage) -> Vec<Entity> {
    let mut q = foliage
        .world
        .query_filtered::<Entity, bevy_ecs::query::With<C>>();
    q.iter(&foliage.world).collect()
}

#[test]
fn spawning_builds_only_the_initial_route() {
    let mut foliage = Foliage::new();
    spawn(&mut foliage);
    foliage.world.flush();

    assert_eq!(entities_with::<SceneA>(&mut foliage).len(), 1);
    assert!(
        entities_with::<SceneB>(&mut foliage).is_empty(),
        "only the active route's scene may exist -- Router must not pre-build others"
    );
}

#[test]
fn navigating_destroys_the_old_scene_and_builds_the_new_one() {
    let mut foliage = Foliage::new();
    let router = spawn(&mut foliage);
    foliage.world.flush();

    foliage.write_to(router, PageIndex(1));
    foliage.world.flush();

    assert!(
        entities_with::<SceneA>(&mut foliage).is_empty(),
        "the departed scene must be despawned, not hidden"
    );
    assert_eq!(entities_with::<SceneB>(&mut foliage).len(), 1);
}

#[test]
fn re_entering_a_route_rebuilds_it_from_nothing() {
    let mut foliage = Foliage::new();
    let router = spawn(&mut foliage);
    foliage.world.flush();
    let first_visit = entities_with::<SceneA>(&mut foliage);

    foliage.write_to(router, PageIndex(1));
    foliage.world.flush();
    foliage.write_to(router, PageIndex(0));
    foliage.world.flush();

    let second_visit = entities_with::<SceneA>(&mut foliage);
    assert_eq!(second_visit.len(), 1);
    assert_ne!(
        first_visit, second_visit,
        "re-entry must be a fresh build, not a resurrection of the old entities"
    );
}

#[test]
fn rewriting_the_same_index_does_not_rebuild() {
    let mut foliage = Foliage::new();
    let router = spawn(&mut foliage);
    foliage.world.flush();
    let before = entities_with::<SceneA>(&mut foliage);

    foliage.write_to(router, PageIndex(0));
    foliage.world.flush();

    assert_eq!(
        before,
        entities_with::<SceneA>(&mut foliage),
        "same-index writes are no-ops -- the live scene must not be torn down"
    );
}

#[test]
fn navigation_fires_page_changed_and_maintains_page_count() {
    #[derive(Resource, Default)]
    struct Last(Option<usize>);
    fn mark(trigger: On<PageChanged>, mut r: ResMut<Last>) {
        r.0 = Some(trigger.event().index);
    }

    let mut foliage = Foliage::new();
    foliage.world.insert_resource(Last::default());
    foliage.world.add_observer(mark);
    let router = spawn(&mut foliage);
    foliage.world.flush();
    assert_eq!(foliage.world.resource::<Last>().0, Some(0));
    assert_eq!(foliage.world.get::<PageCount>(router).unwrap().0, 2);

    foliage.write_to(router, PageIndex(1));
    foliage.world.flush();

    assert_eq!(foliage.world.resource::<Last>().0, Some(1));
}

#[test]
fn rewriting_routes_rebuilds_the_current_scene() {
    let mut foliage = Foliage::new();
    let router = spawn(&mut foliage);
    foliage.world.flush();

    // route 0 becomes scene_b: the rewrite must tear down the live scene_a build and run
    // the new builder in its place.
    foliage.write_to(
        router,
        RouterRoutes::new([scene_b as foliage_proper::RouteFn, scene_b]),
    );
    foliage.world.flush();

    assert!(entities_with::<SceneA>(&mut foliage).is_empty());
    assert_eq!(entities_with::<SceneB>(&mut foliage).len(), 1);
    assert_eq!(foliage.world.get::<PageCount>(router).unwrap().0, 2);
}
