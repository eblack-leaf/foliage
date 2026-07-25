use crate::EcsExtension;
use crate::Trigger;
use crate::ginkgo::viewport::ViewportHandle;
use crate::{Branch, Component, Logical, Section, Tree, Update, Write};
use crate::{Differential, Stem};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::system::Query;
use bevy_ecs::world::DeferredWorld;

/// Marker: this entity's own `ResolvedClip` is the real window viewport, directly -- not
/// the usual intersection with every real `Stem`-ancestor's own clip. For content that
/// deliberately needs to render outside whatever small trigger (or larger containing
/// composite) it's positioned relative to, without needing to escape to a disconnected
/// world-root entity just to dodge that clip (Dropdown's option list, Popover's content
/// surface): it can stay a genuine `Stem`-child -- ordinary elevation comparison against its
/// real siblings, ordinary removal-cascade, no `Anchor`-relinking tricks -- while still
/// rendering fully, bounded only by the one clip that's always meaningful regardless of
/// context: the actual window. Cascades normally to this entity's own children afterward
/// (the viewport becomes *their* inherited base, not something they need their own marker
/// for).
#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
pub struct ClipToViewport;
#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
#[require(InheritedClip, ResolvedClip, Differential<(), ResolvedClip>)]
// #[component(on_add = Self::on_add)]
#[component(on_insert = Self::on_insert)]
pub(crate) struct ClipSection(pub(crate) Section<Logical>);
impl ClipSection {
    pub(crate) fn write_section(
        trigger: Trigger<Write<Section<Logical>>>,
        sections: Query<&Section<Logical>>,
        mut tree: Tree,
    ) {
        // trigger on-insert w/ current section
        let value = *sections.get(trigger.event_target()).unwrap();
        tree.entity(trigger.event_target())
            .insert(ClipSection(value));
    }
    pub(crate) fn stem_insert(trigger: Trigger<Insert, Stem>, mut tree: Tree) {
        tree.trigger_targets(Update::<InheritedClip>::new(), trigger.event_target());
    }
    pub(crate) fn update_inherited(
        trigger: Trigger<Update<InheritedClip>>,
        mut tree: Tree,
        stems: Query<&Stem>,
        sections: Query<&Section<Logical>>,
        clip_to_viewport: Query<&ClipToViewport>,
    ) {
        // calculate all upward in tree
        let this = trigger.event_target();
        let mut stem = *stems.get(this).unwrap();
        let mut section = *sections.get(this).unwrap();
        let mut inherited = InheritedClip(None);
        while stem.id.is_some() {
            let id = stem.id.unwrap();
            // ordinary ancestor-section intersection -- unchanged even when `id` itself is
            // `ClipToViewport`: `id`'s own real bounds still normally clip whatever's nested
            // inside it (only `id`'s *own* resolved clip skips being restricted by what's
            // further up -- see `on_insert`).
            let next = *sections.get(id).unwrap();
            let blended = next.intersection(section).unwrap_or_default();
            if inherited.0.is_none() {
                inherited.0.replace(next);
            } else {
                inherited.0.replace(
                    blended
                        .intersection(inherited.0.unwrap())
                        .unwrap_or_default(),
                );
            }
            if clip_to_viewport.get(id).is_ok() {
                // stop at the *closest* `ClipToViewport` ancestor -- it isn't restricted by
                // whatever's further up (a tiny real trigger, say), so nothing beyond it
                // should re-enter this intersection. A descendant that's itself explicitly
                // `ClipToViewport` gets its own independent walk instead of relying on this
                // one at all (see `on_insert`'s own check on `this`).
                break;
            }
            stem = *stems.get(id).unwrap();
            section = next;
        }
        tree.entity(this).insert(inherited);
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        let current = *world.get::<Section<Logical>>(this).unwrap();
        let is_marked = world.get::<ClipToViewport>(this).is_some();
        let resolved = if is_marked {
            // unbounded by real ancestors -- but NOT by its own section; that's still
            // exactly what gets handed down to its children below.
            ResolvedClip(
                world
                    .get_resource::<ViewportHandle>()
                    .expect("ViewportHandle")
                    .section(),
            )
        } else {
            let inherited = *world.get::<InheritedClip>(this).unwrap();
            if let Some(i) = inherited.0 {
                ResolvedClip(i.intersection(current).unwrap_or_default())
            } else {
                ResolvedClip(current)
            }
        };
        world.commands().entity(this).insert(resolved);
        let clip_context = if is_marked {
            ClipContext(Stem::some(this))
        } else {
            ClipContext(*world.get::<Stem>(this).unwrap())
        };
        world.commands().entity(this).insert(clip_context);
        let deps = world.get::<Branch>(this).unwrap().ids.clone();
        // children inherit this entity's own real bounds, not its (possibly
        // viewport-wide) resolved clip -- a `ClipToViewport` entity isn't restricted by
        // its own ancestors, but ordinary clipping resumes from its own section for
        // everything nested inside it (unless a child is itself independently marked).
        let cascade_base = if is_marked { current } else { resolved.0 };
        for d in deps {
            world
                .commands()
                .entity(d)
                .insert(InheritedClip(Some(cascade_base)));
        }
    }
}
#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
#[component(on_insert = ClipSection::on_insert)]
pub(crate) struct InheritedClip(pub(crate) Option<Section<Logical>>);
#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct ResolvedClip(pub(crate) Section<Logical>);
/// Which entity's own `ResolvedClip` the render pass should scissor a given entity's rendered
/// geometry against -- ordinarily the real `Stem` (structural parent), same as every sibling
/// under that parent (batches them into one scissor-rect span, since an ordinary entity's own
/// clip never diverges from what its parent already bounds). A `ClipToViewport`-marked entity
/// is the one case where that's false by design, so it points at itself instead of its parent.
/// A separate component, not a rewrite of `Stem` itself, because `Stem` is load-bearing
/// everywhere else (`Branch` traversal, `Elevation::update`, interaction's LCA walk, removal
/// cascade) -- corrupting it for marked entities would break all of those.
#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct ClipContext(pub(crate) Stem);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::area::Area;
    use crate::{Elevation, Foliage, Grid, GridExt, Leaf, Location, Sprout};

    fn set_realistic_viewport(foliage: &mut Foliage) {
        foliage
            .world
            .insert_resource(ViewportHandle::new(Area::logical((1000.0, 1000.0))));
    }

    #[test]
    fn a_grandchild_of_a_clip_to_viewport_ancestor_clips_to_its_own_bounds_not_the_tiny_real_ancestor_or_the_whole_viewport()
     {
        // reproduces the real bug (the grandchild used to get silently re-clipped to the
        // tiny real trigger further up), while also pinning down the correct, non-coarse
        // fix: an unmarked descendant of a `ClipToViewport` entity should clip to *that
        // entity's own real bounds* -- not the tiny trigger further up (the bug), and not
        // the entire viewport either (too coarse -- would let overflow content leak out
        // over the rest of the page instead of staying inside the marked entity's own box).
        let mut foliage = Foliage::new();
        set_realistic_viewport(&mut foliage);

        let tiny_trigger = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    10.px().as_left().with(20.px().as_width()),
                    10.px().as_top().with(20.px().as_height()),
                ))
                .elevate(Elevation::abs(0))
                .with(Grid::default()),
        );
        let surface = foliage.world.branch(
            tiny_trigger,
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(300.px().as_width()),
                    30.px().as_top().with(300.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with((ClipToViewport, Grid::default())),
        );
        // deliberately overflows surface's own 300x300 box (extends to 500 wide) -- if this
        // clips to `surface`'s own bounds (the correct behavior), the resolved clip's width
        // must still be capped at surface's own 300, not stretch out to the overflow size.
        let grandchild = foliage.world.branch(
            surface,
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(500.px().as_width()),
                    0.px().as_top().with(300.px().as_height()),
                ))
                .elevate(Elevation::up(1)),
        );
        foliage.world.flush();

        let grandchild_clip = foliage.world.get::<ResolvedClip>(grandchild).unwrap().0;
        let surface_section = *foliage.world.get::<Section<Logical>>(surface).unwrap();
        let viewport = foliage.world.resource::<ViewportHandle>().section();
        assert_eq!(
            grandchild_clip.width(),
            surface_section.width(),
            "the overflowing grandchild must be clipped to surface's own 300-wide box, not \
             its own wider 500 request"
        );
        assert_ne!(
            grandchild_clip, viewport,
            "must not be the whole viewport either -- that would let overflow leak past surface's own box"
        );
        assert_eq!(
            grandchild_clip.top(),
            surface_section.top(),
            "and not the tiny trigger further up, which sits at a different position entirely"
        );
    }

    #[test]
    fn the_clip_to_viewport_entity_itself_still_resolves_to_the_viewport() {
        let mut foliage = Foliage::new();
        set_realistic_viewport(&mut foliage);

        let tiny_trigger = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    10.px().as_left().with(20.px().as_width()),
                    10.px().as_top().with(20.px().as_height()),
                ))
                .elevate(Elevation::abs(0))
                .with(Grid::default()),
        );
        let surface = foliage.world.branch(
            tiny_trigger,
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(300.px().as_width()),
                    30.px().as_top().with(300.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with(ClipToViewport),
        );
        foliage.world.flush();

        let surface_clip = foliage.world.get::<ResolvedClip>(surface).unwrap().0;
        let viewport = foliage.world.resource::<ViewportHandle>().section();
        assert_eq!(surface_clip, viewport);
    }

    #[test]
    fn without_clip_to_viewport_a_grandchild_is_still_clipped_to_the_tiny_ancestor() {
        // control case: confirms the fix is actually doing something, not just always
        // returning the viewport regardless of the marker.
        let mut foliage = Foliage::new();
        set_realistic_viewport(&mut foliage);

        let tiny_trigger = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    10.px().as_left().with(20.px().as_width()),
                    10.px().as_top().with(20.px().as_height()),
                ))
                .elevate(Elevation::abs(0))
                .with(Grid::default()),
        );
        let child = foliage.world.branch(
            tiny_trigger,
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(300.px().as_width()),
                    30.px().as_top().with(300.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with(Grid::default()),
        );
        let grandchild = foliage.world.branch(
            child,
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(300.px().as_width()),
                    0.px().as_top().with(300.px().as_height()),
                ))
                .elevate(Elevation::up(1)),
        );
        foliage.world.flush();

        let grandchild_clip = foliage.world.get::<ResolvedClip>(grandchild).unwrap().0;
        let viewport = foliage.world.resource::<ViewportHandle>().section();
        assert_ne!(
            grandchild_clip, viewport,
            "sanity: without the marker, it really is clipped to the tiny ancestor, proving the marked case above is the fix actually doing something"
        );
    }

    #[test]
    fn a_nested_clip_to_viewport_descendant_is_independently_unbounded_by_its_own_marked_ancestor()
    {
        // "unless they are explicit clip-to-viewport": a doubly-nested case (a Popover
        // opened from inside a Dropdown's option list, say) -- the inner marked entity must
        // resolve to the viewport too, not get clipped to the outer marked entity's own
        // (smaller) bounds the way an ordinary descendant would.
        let mut foliage = Foliage::new();
        set_realistic_viewport(&mut foliage);

        let tiny_trigger = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    10.px().as_left().with(20.px().as_width()),
                    10.px().as_top().with(20.px().as_height()),
                ))
                .elevate(Elevation::abs(0))
                .with(Grid::default()),
        );
        let outer_surface = foliage.world.branch(
            tiny_trigger,
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(300.px().as_width()),
                    30.px().as_top().with(300.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with((ClipToViewport, Grid::default())),
        );
        let inner_surface = foliage.world.branch(
            outer_surface,
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(900.px().as_width()),
                    0.px().as_top().with(900.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with(ClipToViewport),
        );
        foliage.world.flush();

        let inner_clip = foliage.world.get::<ResolvedClip>(inner_surface).unwrap().0;
        let outer_section = *foliage
            .world
            .get::<Section<Logical>>(outer_surface)
            .unwrap();
        let viewport = foliage.world.resource::<ViewportHandle>().section();
        assert_eq!(
            inner_clip, viewport,
            "the independently-marked inner surface should resolve to the viewport, not to outer_surface's own smaller bounds"
        );
        assert_ne!(
            inner_clip, outer_section,
            "sanity: outer_surface's own bounds are genuinely different from the viewport, so this is a real assertion, not a tautology"
        );
    }
}
