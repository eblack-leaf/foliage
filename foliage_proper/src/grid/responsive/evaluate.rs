use crate::coordinate::points::Points;
use crate::coordinate::section::Section;
use crate::coordinate::LogicalContext;
use crate::ginkgo::viewport::ViewportHandle;
use crate::grid::responsive::anim::{ResponsiveAnimationHook, ResponsivePointsAnimationHook};
use crate::grid::responsive::resolve::ResolvedConfiguration;
use crate::grid::responsive::resolve::ResolvedPoints;
use crate::grid::Grid;
use crate::layout::LayoutGrid;
use crate::leaf::{Dependents, Stem};
use crate::twig::Configure;
use bevy_ecs::component::StorageType::SparseSet;
use bevy_ecs::component::{ComponentHooks, StorageType};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Component;
use bevy_ecs::world::DeferredWorld;

#[derive(Copy, Clone, Default, Debug)]
pub(crate) struct ReferentialData {
    pub(crate) section: Section<LogicalContext>,
    pub(crate) grid: Grid,
    pub(crate) points: Points<LogicalContext>,
}

#[derive(Copy, Clone)]
pub struct EvaluateLocation {
    skip_deps: bool,
}
impl Component for EvaluateLocation {
    const STORAGE_TYPE: StorageType = SparseSet;
    fn register_component_hooks(_hooks: &mut ComponentHooks) {
        _hooks.on_insert(|mut world: DeferredWorld, entity: Entity, _| {
            let screen = ReferentialData {
                section: world.get_resource::<ViewportHandle>().unwrap().section(),
                grid: world.get_resource::<LayoutGrid>().unwrap().grid,
                points: Default::default(),
            };
            if let Some(stem) = world.get::<Stem>(entity).copied() {
                let stem = if let Some(s) = stem.0 {
                    ReferentialData {
                        section: world
                            .get::<Section<LogicalContext>>(s)
                            .copied()
                            .unwrap_or_default(),
                        grid: world.get::<Grid>(s).copied().unwrap_or_default(),
                        points: world
                            .get::<Points<LogicalContext>>(s)
                            .copied()
                            .unwrap_or_default(),
                    }
                } else {
                    screen
                };
                let mut resolved = None;
                if let Some(res) = world.get::<ResolvedConfiguration>(entity) {
                    if let Some(r) = res.evaluate(stem, screen) {
                        resolved = Some(
                            r + world
                                .get::<ResponsiveAnimationHook>(entity)
                                .copied()
                                .unwrap_or_default()
                                .value(),
                        );
                    }
                }
                if let Some(r) = resolved {
                    world
                        .commands()
                        .entity(entity)
                        .insert(r)
                        .insert(Configure {});
                }
                let mut resolved = None;
                if let Some(res) = world.get::<ResolvedPoints>(entity) {
                    if let Some(r) = res.evaluate(stem, screen) {
                        resolved.replace(
                            r + world
                                .get::<ResponsivePointsAnimationHook>(entity)
                                .copied()
                                .unwrap_or_default()
                                .value(),
                        );
                    }
                }
                if let Some(r) = resolved {
                    world
                        .commands()
                        .entity(entity)
                        .insert(r)
                        .insert(r.bbox())
                        .insert(Configure {});
                }
            }
            if world.get::<EvaluateLocation>(entity).unwrap().skip_deps {
                return;
            }
            if let Some(deps) = world.get::<Dependents>(entity).cloned() {
                for dep in deps.0 {
                    world
                        .commands()
                        .entity(dep)
                        .insert(EvaluateLocation::full());
                }
            }
        });
    }
}
impl EvaluateLocation {
    pub fn no_deps() -> Self {
        Self { skip_deps: true }
    }
    pub fn full() -> Self {
        Self { skip_deps: false }
    }
}
