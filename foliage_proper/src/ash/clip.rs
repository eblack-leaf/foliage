use crate::AsTree;
use crate::Trigger;
use crate::ginkgo::viewport::ViewportHandle;
use crate::{Children, Component, Logical, Resolve, Resolved, Section, Tree};
use crate::{Differential, Parent};
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::system::Query;
use bevy_ecs::world::DeferredWorld;

/// Marker: this entity's own `ResolvedClip` is the real window viewport, directly -- not
/// the usual intersection with every real `Parent`-ancestor's own clip. For content that
/// deliberately needs to render outside whatever small trigger (or larger containing
/// composite) it's positioned relative to, without needing to escape to a disconnected
/// world-root entity just to dodge that clip (Dropdown's option list, Popover's content
/// surface): it can stay a genuine `Parent`-child -- ordinary elevation comparison against its
/// real siblings, ordinary removal-cascade, no `Anchor`-relinking tricks -- while still
/// rendering fully, bounded only by the one clip that's always meaningful regardless of
/// context: the actual window. Cascades normally to this entity's own children afterward
/// (the viewport becomes *their* inherited base, not something they need their own marker
/// for).
///
/// TODO: this marker means two independent things, and callers can only get both.
///
/// 1. *clip escape* -- do not intersect with ancestor clips.
/// 2. *overlay tier* -- `coordinate/elevation.rs` resets this entity's `StackKey` prefix,
///    making it a fresh subtree root in the FRONT tier, in front of all ordinary content.
///
/// A dropdown or popover wants both. Something that merely overhangs its neighbour -- a
/// badge on a card's corner, a shape breaking out of a column -- wants only (1), and taking
/// (2) with it floats the thing over the entire page. There is currently no way to ask for
/// one without the other, so such callers have to be re-parented to a wider container
/// instead, which contorts the tree around a rendering concern.
///
/// Refactor to `ClipTo { Viewport, Entity(Entity) }` -- clip against the window, or against
/// a *named ancestor* (a scroll container, say), rather than only the two extremes of
/// "immediate chain" and "whole window". `Entity(..)` is what the overhang cases actually
/// want: unclipped by the narrow parent, still clipped by the region that owns the content.
///
/// The work is two systems, not a rename: `elevation.rs`'s tier reset must key off a
/// separate opt-in (an `Overlay` marker) rather than off this component, or every `ClipTo`
/// user silently keeps the front-tier behaviour that motivated the split.
#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
pub struct ClipToViewport;
/// The one statement of what an entity's clip is, and what its children inherit -- returned
/// together because the two differ for a marked entity and getting that pairing wrong is the
/// whole bug class this exists to prevent.
///
/// Called from two places, which is why it is a free function over plain values rather than
/// logic inside the hook: `ClipSection::on_insert` for an ordinary box change, and
/// `grid::view::propagate_offsets` for a scroll, where the whole subtree is walked top-down
/// and the clips come out of that walk directly.
///
/// A marked entity resolves to the viewport -- unbounded by real ancestors -- but hands its
/// children its own real bounds, since ordinary clipping resumes inside it.
pub(crate) fn clip_of(
    section: Section<Logical>,
    inherited: Option<Section<Logical>>,
    marked: bool,
    viewport: Section<Logical>,
) -> (ResolvedClip, Section<Logical>) {
    let resolved = if marked {
        ResolvedClip(viewport)
    } else if let Some(i) = inherited {
        ResolvedClip(i.intersection(section).unwrap_or_default())
    } else {
        ResolvedClip(section)
    };
    let cascade_base = if marked { section } else { resolved.0 };
    (resolved, cascade_base)
}
#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
#[require(InheritedClip, ResolvedClip, Differential<(), ResolvedClip>)]
// #[component(on_add = Self::on_add)]
#[component(on_insert = Self::on_insert)]
pub(crate) struct ClipSection(pub(crate) Section<Logical>);
impl ClipSection {
    pub(crate) fn write_section(
        trigger: Trigger<Resolved<Section<Logical>>>,
        sections: Query<&Section<Logical>>,
        mut tree: Tree,
    ) {
        // trigger on-insert w/ current section
        let value = *sections.get(trigger.event_target()).unwrap();
        tree.write_to(trigger.event_target(), ClipSection(value));
    }
    pub(crate) fn stem_insert(trigger: Trigger<Insert, Parent>, mut tree: Tree) {
        tree.send_to(Resolve::<InheritedClip>::new(), trigger.event_target());
    }
    pub(crate) fn update_inherited(
        trigger: Trigger<Resolve<InheritedClip>>,
        mut tree: Tree,
        stems: Query<&Parent>,
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
        tree.write_to(this, inherited);
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        let current = *world.get::<Section<Logical>>(this).unwrap();
        let is_marked = world.get::<ClipToViewport>(this).is_some();
        let viewport = world
            .get_resource::<ViewportHandle>()
            .expect("ViewportHandle")
            .section();
        let inherited = world.get::<InheritedClip>(this).unwrap().0;
        let (resolved, _) = clip_of(current, inherited, is_marked, viewport);
        world.tree().write_to(this, resolved);
        let clip_context = if is_marked {
            ClipContext(Parent::some(this))
        } else {
            ClipContext(*world.get::<Parent>(this).unwrap())
        };
        world.tree().write_to(this, clip_context);
        let deps = world.get::<Children>(this).unwrap().ids.clone();
        let (_, cascade_base) = clip_of(current, inherited, is_marked, viewport);
        for d in deps {
            world.tree().write_to(d, InheritedClip(Some(cascade_base)));
        }
    }
}
#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
#[component(on_insert = ClipSection::on_insert)]
pub(crate) struct InheritedClip(pub(crate) Option<Section<Logical>>);
#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct ResolvedClip(pub(crate) Section<Logical>);
/// Which entity's own `ResolvedClip` the render pass should scissor a given entity's rendered
/// geometry against -- ordinarily the real `Parent` (structural parent), same as every sibling
/// under that parent (batches them into one scissor-rect span, since an ordinary entity's own
/// clip never diverges from what its parent already bounds). A `ClipToViewport`-marked entity
/// is the one case where that's false by design, so it points at itself instead of its parent.
/// A separate component, not a rewrite of `Parent` itself, because `Parent` is load-bearing
/// everywhere else (`Children` traversal, `Elevation::update`, interaction's LCA walk, removal
/// cascade) -- corrupting it for marked entities would break all of those.
#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct ClipContext(pub(crate) Parent);
