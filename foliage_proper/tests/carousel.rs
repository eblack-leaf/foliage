//! `Carousel` (`composite/carousel.rs`) has no tests at all. Swipe reads `CurrentInteraction`
//! live click data (`pub(crate)`, no public constructor) so that path isn't externally
//! testable -- same boundary as `Slider`'s drag. What's testable: page-set structure,
//! `PageIndex` patching (no respawn), and the embedded `Pagination`'s existence/absence.

use foliage_proper::{
    Carousel, CarouselPages, Color, EcsExtension, Elevation, Entity, Foliage, GridExt, Leaf,
    Location, PageChanged, PageCount, PageIndex, Pagination, PaginationMode, Sprout, Stem,
};
use bevy_ecs::observer::On;
use bevy_ecs::system::ResMut;
use foliage_proper::Resource;

fn children_of(foliage: &mut Foliage, parent: Entity) -> Vec<Entity> {
    let mut q = foliage.world.query::<(Entity, &Stem)>();
    q.iter(&foliage.world)
        .filter(|(_, stem)| stem.id == Some(parent))
        .map(|(e, _)| e)
        .collect()
}

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
        Carousel::new()
            .pages(CarouselPages::new(count, |tree, slot, _i| {
                tree.branch(
                    slot,
                    Leaf::sprout()
                        .at(Location::new().xs(
                            0.px().as_left().with(10.px().as_width()),
                            0.px().as_top().with(10.px().as_height()),
                        ))
                        .elevate(Elevation::up(1)),
                );
            }))
            .at(Location::new().xs(
                0.px().as_left().with(300.px().as_width()),
                0.px().as_top().with(200.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    )
}

#[test]
fn spawning_builds_one_slot_per_page() {
    let mut foliage = Foliage::new();
    let carousel = spawn(&mut foliage, 3);
    foliage.world.flush();

    assert_eq!(foliage.world.get::<PageCount>(carousel).unwrap().0, 3);
    // viewport -> N slots -> each slot's own author content (one Leaf each): 3 slots means
    // at least 3 descendants beyond the viewport itself.
    assert!(descendants_of(&mut foliage, carousel).len() >= 3);
}

#[test]
fn writing_page_index_fires_page_changed_and_does_not_respawn_slots() {
    #[derive(Resource, Default)]
    struct Last(Option<usize>);
    fn mark(trigger: On<PageChanged>, mut r: ResMut<Last>) {
        r.0 = Some(trigger.event().index);
    }

    let mut foliage = Foliage::new();
    foliage.world.insert_resource(Last::default());
    foliage.world.add_observer(mark);
    let carousel = spawn(&mut foliage, 3);
    foliage.world.flush();
    assert_eq!(foliage.world.resource::<Last>().0, Some(0));
    let mut before = descendants_of(&mut foliage, carousel);
    before.sort();

    foliage.write_to(carousel, PageIndex(2));
    foliage.world.flush();

    assert_eq!(foliage.world.resource::<Last>().0, Some(2));
    let mut after = descendants_of(&mut foliage, carousel);
    after.sort();
    assert_eq!(before, after, "a page change slides the strip -- it must not touch slot identity");
}

#[test]
fn rewriting_carousel_pages_rebuilds_the_slot_set() {
    let mut foliage = Foliage::new();
    let carousel = spawn(&mut foliage, 3);
    foliage.world.flush();

    foliage.write_to(
        carousel,
        CarouselPages::new(5, |tree, slot, _i| {
            tree.branch(
                slot,
                Leaf::sprout()
                    .at(Location::new())
                    .elevate(Elevation::up(1)),
            );
        }),
    );
    foliage.world.flush();

    assert_eq!(foliage.world.get::<PageCount>(carousel).unwrap().0, 5);
}

#[test]
fn no_pagination_is_embedded_unless_configured() {
    let mut foliage = Foliage::new();
    spawn(&mut foliage, 3);
    foliage.world.flush();

    let mut q = foliage.world.query::<&Pagination>();
    assert!(q.iter(&foliage.world).next().is_none());
}

#[test]
fn configuring_pagination_embeds_one_and_keeps_its_page_count_in_sync() {
    let mut foliage = Foliage::new();
    let carousel = foliage.world.leaf(
        Carousel::new()
            .pages(CarouselPages::new(3, |tree, slot, _i| {
                tree.branch(
                    slot,
                    Leaf::sprout()
                        .at(Location::new())
                        .elevate(Elevation::up(1)),
                );
            }))
            .pagination(PaginationMode::Dots)
            .colors(Color::green(500), Color::gray(600))
            .at(Location::new().xs(
                0.px().as_left().with(300.px().as_width()),
                0.px().as_top().with(200.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    foliage.world.flush();

    let mut q = foliage.world.query::<(Entity, &PageCount)>();
    let pagination_counts: Vec<usize> = q
        .iter(&foliage.world)
        .filter(|(e, _)| *e != carousel)
        .map(|(_, c)| c.0)
        .collect();
    assert_eq!(pagination_counts, vec![3], "the embedded Pagination's own PageCount should mirror the carousel's");
}
