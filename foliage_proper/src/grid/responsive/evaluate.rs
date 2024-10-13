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
use crate::tree::Tree;
use crate::twig::Configure;
use bevy_ecs::change_detection::Res;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::observer::Trigger;
use bevy_ecs::prelude::Query;

#[derive(Copy, Clone, Default, Debug)]
pub(crate) struct ReferentialData {
    pub(crate) section: Section<LogicalContext>,
    pub(crate) grid: Grid,
    pub(crate) points: Points<LogicalContext>,
}

pub(crate) fn evaluate_location(
    trigger: Trigger<EvaluateLocation>,
    stems: Query<&Stem>,
    dependents: Query<&Dependents>,
    viewport_handle: Res<ViewportHandle>,
    layout_grid: Res<LayoutGrid>,
    grids: Query<&Grid>,
    responsive: Query<&ResolvedConfiguration>,
    diff: Query<&ResponsiveAnimationHook>,
    mut eval: Query<&mut Section<LogicalContext>>,
    responsive_points: Query<&ResolvedPoints>,
    point_diff: Query<&ResponsivePointsAnimationHook>,
    mut point_eval: Query<&mut Points<LogicalContext>>,
    mut tree: Tree,
) {
    let screen = ReferentialData {
        section: viewport_handle.section(),
        grid: layout_grid.grid,
        points: Points::default(),
    };
    if let Ok(stem) = stems.get(trigger.entity()) {
        let stem = if let Some(s) = stem.0 {
            ReferentialData {
                section: eval.get(s).copied().unwrap_or_default(),
                grid: grids.get(s).copied().unwrap_or_default(),
                points: point_eval.get(s).copied().unwrap_or_default(),
            }
        } else {
            screen
        };
        if let Ok(res) = responsive.get(trigger.entity()) {
            let resolved = res.evaluate(stem, screen);
            if let Some(s) = resolved {
                *eval.get_mut(trigger.entity()).unwrap() = s + diff
                    .get(trigger.entity())
                    .copied()
                    .unwrap_or_default()
                    .value();
                tree.trigger_targets(Configure {}, trigger.entity());
            }
        }
        if let Ok(res) = responsive_points.get(trigger.entity()) {
            if let Some(resolved) = res.evaluate(stem, screen) {
                let solved = resolved
                    + point_diff
                        .get(trigger.entity())
                        .copied()
                        .unwrap_or_default()
                        .value();
                *point_eval.get_mut(trigger.entity()).unwrap() = solved;
                *eval.get_mut(trigger.entity()).unwrap() = solved.bbox();
                tree.trigger_targets(Configure {}, trigger.entity());
            }
        }
        if trigger.event().skip_deps {
            return;
        }
        if let Ok(mut deps) = dependents.get(trigger.entity()) {
            if deps.0.is_empty() {
                return;
            }
            tree.trigger_targets(
                EvaluateLocation::full(),
                deps.0
                    .iter()
                    .copied()
                    .filter(|d| responsive.contains(*d) || responsive_points.contains(*d))
                    .collect::<Vec<Entity>>(),
            )
        }
    }
}

#[derive(Copy, Clone, Event)]
pub struct EvaluateLocation {
    skip_deps: bool,
}

impl EvaluateLocation {
    pub fn no_deps() -> Self {
        Self { skip_deps: true }
    }
    pub fn full() -> Self {
        Self { skip_deps: false }
    }
}
