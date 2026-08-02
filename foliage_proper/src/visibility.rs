use crate::AsTree;
use crate::Trigger;
use crate::ash::differential::RenderRemoveQueue;
use crate::{
    AnchorDeps, Attachment, Children, Component, Foliage, Parent, Resolve, Resolved, Tree,
};
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::prelude::Query;
use bevy_ecs::system::ResMut;
use bevy_ecs::world::DeferredWorld;

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq, Hash, Component)]
#[component(on_insert = Visibility::on_insert)]
#[require(
    InheritedVisibility,
    ResolvedVisibility,
    CachedVisibility,
    AutoVisibility
)]
/// Whether an entity is drawn -- the author's own switch, and the only one of the four
/// visibility components meant to be written.
///
/// Hiding an entity hides everything beneath it: the flag is combined down the `Parent`
/// chain into [`InheritedVisibility`], and the answer the renderer reads is
/// [`ResolvedVisibility`]. A child therefore cannot show itself out of a hidden parent.
///
/// Hidden entities stay spawned and keep their state; they are skipped by drawing and by
/// hit-testing. To take one out of the tree entirely, despawn it.
pub struct Visibility {
    visible: bool,
}
impl Attachment for Visibility {
    fn attach(foliage: &mut Foliage) {
        foliage.define(Visibility::stem_insert);
        foliage.define(Visibility::update);
    }
}
impl Visibility {
    /// `true` to draw, `false` to hide this entity and its subtree.
    pub fn new(v: bool) -> Self {
        Self { visible: v }
    }
    /// This entity's own flag, ignoring its ancestors. For the effective answer, read
    /// [`ResolvedVisibility`].
    pub fn visible(&self) -> bool {
        self.visible
    }
    fn stem_insert(
        trigger: Trigger<Insert, Parent>,
        mut tree: Tree,
        stems: Query<&Parent>,
        res: Query<&ResolvedVisibility>,
    ) {
        let this = trigger.event_target();
        let stem = stems.get(this).unwrap();
        if let Some(s) = stem.id {
            let resolved = *res.get(s).unwrap();
            tracing::trace!(
                entity = ?this,
                parent = ?s,
                parent_resolved_visible = resolved.visible,
                "visibility: stem_insert captured parent"
            );
            tree.write_to(
                this,
                InheritedVisibility {
                    visible: resolved.visible,
                },
            );
        }
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        world.tree().send_to(Resolve::<Visibility>::new(), this);
    }
    pub(crate) fn update(
        trigger: Trigger<Resolve<Visibility>>,
        inheriteds: Query<&InheritedVisibility>,
        vis: Query<&Visibility>,
        auto: Query<&AutoVisibility>,
        cached: Query<&CachedVisibility>,
        mut tree: Tree,
        branches: Query<&Children>,
        sd: Query<&AnchorDeps>,
    ) {
        let this = trigger.event_target();
        let inherited = inheriteds.get(this).unwrap();
        let current = vis.get(this).unwrap();
        let auto = auto.get(this).unwrap();
        let resolved = ResolvedVisibility {
            visible: inherited.visible && current.visible && auto.visible,
        };
        let cached = cached.get(this).unwrap();
        tracing::trace!(
            entity = ?this,
            inherited = inherited.visible,
            current = current.visible,
            auto = auto.visible,
            resolved = resolved.visible,
            was_cached = cached.visible,
            "visibility: resolved"
        );
        if cached.visible != resolved.visible {
            tree.write_to(
                this,
                (
                    resolved,
                    CachedVisibility {
                        visible: resolved.visible,
                    },
                ),
            );
            tree.send_to(Resolved::<Visibility>::new(), this);
            let mut deps = branches.get(this).unwrap().ids.clone();
            if let Some(stack_deps) = sd.get(this).ok() {
                deps.extend(stack_deps.ids.clone());
            }
            tracing::trace!(
                entity = ?this,
                new_resolved = resolved.visible,
                deps = ?deps,
                "visibility: cascading to deps (real flip)"
            );
            for d in deps {
                tree.write_to(
                    d,
                    InheritedVisibility {
                        visible: resolved.visible,
                    },
                );
            }
        }
    }
    pub(crate) fn push_remove_packet<R: Clone + Send + Sync + 'static>(
        trigger: Trigger<Resolved<Visibility>>,
        visibilities: Query<&ResolvedVisibility>,
        mut queue: ResMut<RenderRemoveQueue<R>>,
    ) {
        let value = visibilities.get(trigger.event_target()).unwrap();
        if !value.visible {
            queue.queue.insert(trigger.event_target());
        }
    }
}
/// The engine's own veto, held separately from the author's [`Visibility`] so the two
/// never overwrite each other.
///
/// Cleared when an entity's `Location` cannot resolve -- an anchor pointing at something
/// hidden or gone, leaving no box to draw in. Restored by the resolve that succeeds.
#[derive(Component, Copy, Clone)]
#[component(on_insert = Visibility::on_insert)]
pub(crate) struct AutoVisibility {
    pub(crate) visible: bool,
}
impl AutoVisibility {
    pub(crate) fn new(v: bool) -> Self {
        Self { visible: v }
    }
}
impl Default for AutoVisibility {
    fn default() -> Self {
        Self::new(true)
    }
}
/// Last frame's [`ResolvedVisibility`], so a change of state can be detected and acted on
/// once -- sending render removals on hide, and re-sending cached attributes on show --
/// rather than every frame it stays hidden.
#[derive(Component, Copy, Clone)]
pub(crate) struct CachedVisibility {
    pub(crate) visible: bool,
}
impl Default for CachedVisibility {
    fn default() -> Self {
        Self { visible: true }
    }
}
impl Default for Visibility {
    fn default() -> Self {
        Self::new(true)
    }
}
/// What this entity's ancestors permit: `false` as soon as any one of them is hidden.
/// Maintained by the engine; combined with the entity's own [`Visibility`] to give
/// [`ResolvedVisibility`].
#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq, Hash, Component)]
#[component(on_insert = Visibility::on_insert)]
pub struct InheritedVisibility {
    visible: bool,
}
impl Default for InheritedVisibility {
    fn default() -> Self {
        Self { visible: true }
    }
}
/// Whether this entity is actually drawn: its own [`Visibility`], every ancestor's, and
/// the engine's [`AutoVisibility`] veto, combined.
///
/// Read-only -- this is the answer, not a control. Write [`Visibility`] to change it.
#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq, Hash, Component)]
pub struct ResolvedVisibility {
    visible: bool,
}
impl ResolvedVisibility {
    /// `true` when this entity is drawn and hit-tested.
    pub fn visible(&self) -> bool {
        self.visible
    }
}
impl Default for ResolvedVisibility {
    fn default() -> Self {
        Self { visible: true }
    }
}
