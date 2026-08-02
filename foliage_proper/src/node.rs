use crate::AsTree;
use crate::Elevation;
use crate::LayoutSection;
use crate::Logical;
use crate::Opacity;
use crate::Section;
use crate::Trigger;
use crate::Visibility;
use crate::ash::clip::ClipSection;
use crate::interaction::CurrentInteraction;
use crate::{
    Animation, Component, FocusBehavior, InteractionPropagation, InteractionShape, Location,
    Resolve, Tree,
};
use bevy_ecs::entity::Entity;
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::system::Query;
use bevy_ecs::world::DeferredWorld;
use std::collections::HashSet;

#[derive(Component)]
#[require(Parent, Children)]
#[require(Opacity, Visibility, ClipSection)]
#[require(Section<Logical>, LayoutSection, Elevation, InteractionShape, InteractionPropagation)]
#[require(FocusBehavior)]
#[component(on_add = Self::on_add)]
#[component(on_remove = Self::on_remove)]
/// Marks an entity as part of the display tree, bringing in the components everything
/// positioned and drawn needs -- a `Section`, an `Elevation`, and the interaction
/// defaults.
///
/// Added automatically by every spawn through [`Author`](crate::Author), so
/// authors meet it as [`Node::sprout`] -- a node with no primitive of its own, used to
/// group children or to be a bare hit area.
pub struct Node {}

impl Default for Node {
    fn default() -> Self {
        Self::new()
    }
}

impl Node {
    /// The bare marker. To spawn one, use [`Node::sprout`].
    pub fn new() -> Node {
        Node {}
    }
    /// Internal spelling of [`Parent::new`].
    pub(crate) fn sprout() -> crate::LeafSprout {
        crate::LeafSprout::new()
    }
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        let mut tree = world.tree();
        tree.subscribe(this, Self::anim_opacity);
        tree.subscribe(this, Self::anim_elevation);
        tree.subscribe(this, Self::anim_location);
    }
    fn anim_opacity(
        trigger: Trigger<Resolve<Animation<Opacity>>>,
        opacities: Query<&Opacity>,
        mut tree: Tree,
    ) {
        if let Ok(o) = opacities.get(trigger.event_target()) {
            tree.write_to(trigger.event_target(), *o);
        }
    }
    fn anim_elevation(
        trigger: Trigger<Resolve<Animation<Elevation>>>,
        mut tree: Tree,
        elevation: Query<&Elevation>,
    ) {
        if let Ok(e) = elevation.get(trigger.event_target()) {
            tree.write_to(trigger.event_target(), *e);
        }
    }
    fn anim_location(trigger: Trigger<Resolve<Animation<Location>>>, mut tree: Tree) {
        tree.send_to(Resolve::<Location>::new(), trigger.event_target());
    }
    fn on_remove(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        // Before the early return below: every despawn in the engine reaches this hook -- a
        // prune's cascade, a timer expiring, an animation finishing, a widget rebuilding its
        // own children -- so reporting from here is what makes the notice complete. The name
        // itself needs no cleanup; the entity generation is what makes it stale.
        if world.get::<crate::boundary::leaf::Grown>(this).is_some()
            && let Some(mut emissions) =
                world.get_resource_mut::<crate::boundary::bloom::Emissions>()
        {
            emissions.push(crate::Bloom::Withered(crate::Leaf(this)));
        }
        if let Some(mut current) = world.get_resource_mut::<CurrentInteraction>() {
            if let Some(p) = current.primary {
                if p == this {
                    current.primary.take();
                    return;
                }
            }
            let mut found = false;
            for ps in current.pass_through.iter() {
                if *ps == this {
                    found = true;
                    break;
                }
            }
            if found {
                current.pass_through.retain(|p| *p != this);
            }
        }
    }
}

/// An element with no primitive of its own: a layout container, a bare hit area, a group to
/// hang children off.
///
/// The counterpart to `Panel::new()`/`Text::new()` for the cases where nothing is drawn.
/// Takes the same `.at()`/`.elevate()`/`.grid()` chain as any other spec.
///
/// Named plainly rather than botanically: it has no content of its own to describe, so
/// unlike `Panel`/`Text`/`Icon` there's nothing for a more evocative name to point at.
pub struct Bare;
impl Bare {
    /// Starts a bare element.
    pub fn new() -> crate::LeafSprout {
        crate::LeafSprout::new()
    }
}
impl Default for Bare {
    fn default() -> Self {
        Self
    }
}

/// The source location of the `leaf`/`branch` call that spawned this entity.
///
/// Layout failures surface long after the spawn, in a system that only knows entity ids --
/// and `188v0 resolves relative to 187v0` tells an author nothing about their own code.
/// Recording the call site at spawn is what lets those panics point back at the line that
/// actually needs changing.
///
/// A `&'static Location` is a pointer the compiler already materialised for the panic
/// machinery, so carrying it costs one word per entity and nothing at runtime.
#[derive(Component, Copy, Clone, Debug)]
pub struct SpawnedAt(pub &'static core::panic::Location<'static>);

/// This entity's parent, and so what its `Location` resolves against, what clips it,
/// what its opacity and visibility inherit from, and where it sits in draw order.
///
/// `None` is a root, resolving against the viewport. Set by
/// [`Tree::branch`](crate::Tree::branch); rewriting it reparents the entity, and the
/// old and new parents' [`Children`] sets are updated to match.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[component(on_insert = Parent::on_insert)]
#[component(on_discard = Parent::on_replace)]
pub struct Parent {
    pub id: Option<Entity>,
}
impl Default for Parent {
    fn default() -> Self {
        Parent::none()
    }
}

impl Parent {
    /// A link that may or may not point at a parent.
    #[allow(dead_code)]
    pub(crate) fn new(id: Option<Entity>) -> Self {
        Self { id }
    }
    /// Parents this entity to `entity`.
    pub fn some(entity: Entity) -> Self {
        Self { id: Some(entity) }
    }
    /// Makes this entity a root, resolving against the viewport.
    pub fn none() -> Self {
        Self { id: None }
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        let stem = world.get::<Parent>(this).copied().unwrap();
        if let Some(s) = stem.id {
            if let Some(mut deps) = world.get_mut::<Children>(s) {
                deps.ids.insert(this);
            }
        }
    }
    fn on_replace(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        let stem = world.get::<Parent>(this).copied().unwrap();
        if let Some(s) = stem.id {
            if let Some(mut deps) = world.get_mut::<Children>(s) {
                deps.ids.remove(&this);
            }
        }
    }
    /// Walks the `Parent` chain upward from `entity` until an ancestor carrying `C` is found
    /// (returned), or returns `entity` itself if nothing does. For a reactive system on a
    /// composite's descendant that needs to route back to the composite's own root: `C` is
    /// that composite's own marker component (`TextInput`, `Dropdown`, ...), which every
    /// composite already has on its root for other reasons, so there's nothing separate to
    /// keep in sync. Walking through a *different* composite type nested in between (e.g.
    /// Dropdown's option rows sit inside a nested `List`) is harmless: `C` only matches its
    /// own type, so the walk passes through unrelated ancestors and keeps going.
    pub fn ascend_to<C: Component>(
        entity: Entity,
        stems: &Query<&Parent>,
        markers: &Query<&C>,
    ) -> Entity {
        let mut current = entity;
        loop {
            if markers.get(current).is_ok() {
                return current;
            }
            match stems.get(current).ok().and_then(|s| s.id) {
                Some(parent) => current = parent,
                None => return current,
            }
        }
    }
}

/// This entity's children -- the reverse of [`Parent`], maintained by the engine so a
/// change here can be cascaded down without searching the world.
#[derive(Component, Clone, Default)]
pub struct Children {
    pub ids: HashSet<Entity>,
}
