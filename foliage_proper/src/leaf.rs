use crate::EcsExtension;
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
#[require(Stem, Branch)]
#[require(Opacity, Visibility, ClipSection)]
#[require(Section<Logical>, LayoutSection, Elevation, InteractionShape, InteractionPropagation)]
#[require(FocusBehavior)]
#[component(on_add = Self::on_add)]
#[component(on_remove = Self::on_remove)]
/// Marks an entity as part of the display tree, bringing in the components everything
/// positioned and drawn needs -- a `Section`, an `Elevation`, and the interaction
/// defaults.
///
/// Added automatically by every spawn through [`EcsExtension`](crate::EcsExtension), so
/// authors meet it as [`Leaf::sprout`] -- a node with no primitive of its own, used to
/// group children or to be a bare hit area.
pub struct Leaf {}

impl Default for Leaf {
    fn default() -> Self {
        Self::new()
    }
}

impl Leaf {
    /// The bare marker. To spawn one, use [`Leaf::sprout`].
    pub fn new() -> Leaf {
        Leaf {}
    }
    /// Entry point for a child with no rendering primitive of its own (a bare interaction
    /// hit-area, say) -- the same `.at()/.elevate()/.with()` chain into `tree.leaf(..)`/
    /// `tree.branch(..)` as `Panel::new()`/`Text::new()`, named after `Leaf` instead of
    /// requiring callers to know the `LeafSprout` type directly.
    pub fn sprout() -> crate::LeafSprout {
        crate::LeafSprout::new()
    }
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        world
            .commands()
            .entity(this)
            .observe(Self::anim_opacity)
            .observe(Self::anim_elevation)
            .observe(Self::anim_location);
    }
    fn anim_opacity(
        trigger: Trigger<Resolve<Animation<Opacity>>>,
        opacities: Query<&Opacity>,
        mut tree: Tree,
    ) {
        if let Ok(o) = opacities.get(trigger.event_target()) {
            tree.entity(trigger.event_target()).insert(*o);
        }
    }
    fn anim_elevation(
        trigger: Trigger<Resolve<Animation<Elevation>>>,
        mut tree: Tree,
        elevation: Query<&Elevation>,
    ) {
        if let Ok(e) = elevation.get(trigger.event_target()) {
            tree.entity(trigger.event_target()).insert(*e);
        }
    }
    fn anim_location(trigger: Trigger<Resolve<Animation<Location>>>, mut tree: Tree) {
        tree.trigger_targets(Resolve::<Location>::new(), trigger.event_target());
    }
    fn on_remove(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
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
/// [`branch`](crate::EcsExtension::branch); rewriting it reparents the entity, and the
/// old and new parents' [`Branch`] sets are updated to match.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[component(on_insert = Stem::on_insert)]
#[component(on_discard = Stem::on_replace)]
pub struct Stem {
    pub id: Option<Entity>,
}
impl Default for Stem {
    fn default() -> Self {
        Stem::none()
    }
}

impl Stem {
    /// A stem that may or may not have a parent.
    pub fn new(id: Option<Entity>) -> Self {
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
        let stem = world.get::<Stem>(this).copied().unwrap();
        if let Some(s) = stem.id {
            if let Some(mut deps) = world.get_mut::<Branch>(s) {
                deps.ids.insert(this);
            }
        }
    }
    fn on_replace(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        let stem = world.get::<Stem>(this).copied().unwrap();
        if let Some(s) = stem.id {
            if let Some(mut deps) = world.get_mut::<Branch>(s) {
                deps.ids.remove(&this);
            }
        }
    }
    /// Walks the `Stem` chain upward from `entity` until an ancestor carrying `C` is found
    /// (returned), or returns `entity` itself if nothing does. For a reactive system on a
    /// composite's descendant that needs to route back to the composite's own root: `C` is
    /// that composite's own marker component (`TextInput`, `Dropdown`, ...), which every
    /// composite already has on its root for other reasons, so there's nothing separate to
    /// keep in sync. Walking through a *different* composite type nested in between (e.g.
    /// Dropdown's option rows sit inside a nested `List`) is harmless: `C` only matches its
    /// own type, so the walk passes through unrelated ancestors and keeps going.
    pub fn ascend_to<C: Component>(
        entity: Entity,
        stems: &Query<&Stem>,
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

/// This entity's children -- the reverse of [`Stem`], maintained by the engine so a
/// change here can be cascaded down without searching the world.
#[derive(Component, Clone, Default)]
pub struct Branch {
    pub ids: HashSet<Entity>,
}
