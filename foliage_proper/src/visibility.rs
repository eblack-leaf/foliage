use crate::ash::differential::RenderRemoveQueue;
use crate::{Attachment, Branch, Component, Foliage, StackDeps, Stem, Tree, Update, Write};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{OnInsert, Query, Trigger};
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
    pub fn new(v: bool) -> Self {
        Self { visible: v }
    }
    pub fn visible(&self) -> bool {
        self.visible
    }
    fn stem_insert(
        trigger: Trigger<OnInsert, Stem>,
        mut tree: Tree,
        stems: Query<&Stem>,
        res: Query<&ResolvedVisibility>,
    ) {
        let this = trigger.entity();
        let stem = stems.get(this).unwrap();
        if let Some(s) = stem.id {
            let resolved = *res.get(s).unwrap();
            tree.entity(this).insert(InheritedVisibility {
                visible: resolved.visible,
            });
        }
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world
            .commands()
            .trigger_targets(Update::<Visibility>::new(), this);
    }
    pub(crate) fn update(
        trigger: Trigger<Update<Visibility>>,
        inheriteds: Query<&InheritedVisibility>,
        vis: Query<&Visibility>,
        auto: Query<&AutoVisibility>,
        cached: Query<&CachedVisibility>,
        mut tree: Tree,
        branches: Query<&Branch>,
        sd: Query<&StackDeps>,
    ) {
        let this = trigger.entity();
        let inherited = inheriteds.get(this).unwrap();
        let current = vis.get(this).unwrap();
        let auto = auto.get(this).unwrap();
        let resolved = ResolvedVisibility {
            visible: inherited.visible && current.visible && auto.visible,
        };
        let cached = cached.get(this).unwrap();
        if cached.visible != resolved.visible {
            tree.entity(this).insert(resolved).insert(CachedVisibility {
                visible: resolved.visible,
            });
            tree.trigger_targets(Write::<Visibility>::new(), this);
            let mut deps = branches.get(this).unwrap().ids.clone();
            if let Some(stack_deps) = sd.get(this).ok() {
                deps.extend(stack_deps.ids.clone());
            }
            for d in deps {
                tree.entity(d).insert(InheritedVisibility {
                    visible: resolved.visible,
                });
            }
        }
    }
    pub(crate) fn push_remove_packet<R: Clone + Send + Sync + 'static>(
        trigger: Trigger<Write<Visibility>>,
        visibilities: Query<&ResolvedVisibility>,
        mut queue: ResMut<RenderRemoveQueue<R>>,
    ) {
        let value = visibilities.get(trigger.entity()).unwrap();
        if !value.visible {
            queue.queue.insert(trigger.entity());
        }
    }
}
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
#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq, Hash, Component)]
pub struct ResolvedVisibility {
    visible: bool,
}
impl ResolvedVisibility {
    pub fn visible(&self) -> bool {
        self.visible
    }
}
impl Default for ResolvedVisibility {
    fn default() -> Self {
        Self { visible: true }
    }
}
