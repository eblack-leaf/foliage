use crate::coordinate::points::Points;
use crate::coordinate::section::Section;
use crate::coordinate::LogicalContext;
use crate::ginkgo::viewport::ViewportHandle;
use crate::grid::responsive::anim::{
    CalcDiff, ResponsiveAnimationHook, ResponsivePointsAnimationHook,
};
use crate::grid::responsive::resolve::ResolvedConfiguration;
use crate::grid::responsive::resolve::ResolvedPoints;
use crate::grid::Grid;
use crate::layout::LayoutGrid;
use crate::leaf::{Dependents, Stem};
use crate::twig::Configure;
use bevy_ecs::component::StorageType::SparseSet;
use bevy_ecs::component::{ComponentHooks, ComponentId, StorageType};
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
    pub(crate) skip_deps: bool,
}
impl EvaluateLocation {
    pub(crate) fn on_insert(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
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
                    let old_diff = world
                        .get::<ResponsiveAnimationHook>(entity)
                        .copied()
                        .unwrap_or_default();
                    let d = if world.get::<CalcDiff>(entity).is_some() {
                        let last = world
                            .get::<Section<LogicalContext>>(entity)
                            .copied()
                            .unwrap();
                        let diff_value = last - r;
                        let value = diff_value * old_diff.percent;
                        world
                            .commands()
                            .entity(entity)
                            .insert(ResponsiveAnimationHook {
                                section: diff_value,
                                percent: old_diff.percent,
                            });
                        value
                    } else {
                        old_diff.value()
                    };
                    if entity.index() == 58 {
                        tracing::trace!("r: {} + d: {} = res: {}", r, d, r + d);
                    }
                    resolved = Some(r + d);
                }
            }
            if let Some(r) = resolved {
                world.commands().entity(entity).insert(r);
                world.trigger_targets(Configure {}, entity);
            }
            let mut resolved = None;
            if let Some(res) = world.get::<ResolvedPoints>(entity) {
                if let Some(r) = res.evaluate(stem, screen) {
                    let old_diff = world
                        .get::<ResponsivePointsAnimationHook>(entity)
                        .copied()
                        .unwrap_or_default();
                    let d = if world.get::<CalcDiff>(entity).is_some() {
                        let last = world
                            .get::<Points<LogicalContext>>(entity)
                            .copied()
                            .unwrap_or_default();
                        let diff_value = last - r;
                        let value = diff_value * old_diff.percent;
                        world
                            .commands()
                            .entity(entity)
                            .insert(ResponsivePointsAnimationHook {
                                points: diff_value,
                                percent: old_diff.percent,
                            });
                        value
                    } else {
                        old_diff.value()
                    };
                    resolved.replace(r + d);
                }
            }
            if let Some(r) = resolved {
                world.commands().entity(entity).insert(r).insert(r.bbox());
                world.trigger_targets(Configure {}, entity);
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
                    .insert(EvaluateLocation::recursive());
            }
        }
    }
}
impl Component for EvaluateLocation {
    const STORAGE_TYPE: StorageType = SparseSet;
    fn register_component_hooks(_hooks: &mut ComponentHooks) {
        _hooks.on_insert(EvaluateLocation::on_insert);
    }
}
impl EvaluateLocation {
    pub fn no_deps() -> Self {
        Self { skip_deps: true }
    }
    pub fn recursive() -> Self {
        Self { skip_deps: false }
    }
}
