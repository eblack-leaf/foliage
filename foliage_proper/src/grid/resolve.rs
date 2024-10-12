use crate::coordinate::points::Points;
use crate::coordinate::section::Section;
use crate::coordinate::LogicalContext;
use crate::ginkgo::viewport::ViewportHandle;
use crate::grid::animation::GridLocationAnimationHook;
use crate::grid::location::GridLocation;
use crate::grid::Grid;
use crate::layout::{Layout, LayoutGrid};
use crate::leaf::{Dependents, Stem};
use crate::tree::Tree;
use bevy_ecs::change_detection::Res;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::prelude::{Query, Trigger};

#[derive(Default, Event, Copy, Clone)]
pub struct ResolveGridLocation {}
pub(crate) fn triggered_resolve_grid_locations(
    trigger: Trigger<ResolveGridLocation>,
    mut locations: Query<&mut GridLocation>,
    dependents: Query<&Dependents>,
    grids: Query<&Grid>,
    stems: Query<&Stem>,
    layout: Res<Layout>,
    layout_grid: Res<LayoutGrid>,
    viewport: Res<ViewportHandle>,
    mut sections: Query<&mut Section<LogicalContext>>,
    mut points: Query<&mut Points<LogicalContext>>,
    mut tree: Tree,
) {
    let screen = ReferentialData {
        section: viewport.section(),
        points: None,
        grid: layout_grid.grid,
    };
    let stem = if let Ok(stem) = stems.get(trigger.entity()) {
        if let Some(s) = stem.0 {
            let grid = if let Ok(grid) = grids.get(s) {
                *grid
            } else {
                Grid::default()
            };
            ReferentialData {
                section: sections.get(s).copied().unwrap_or(screen.section),
                points: None,
                grid,
            }
        } else {
            screen
        }
    } else {
        screen
    };
    let mut resolved = None;
    if let Ok(location) = locations.get(trigger.entity()) {
        if let Some(r) = location.resolve(stem, screen, *layout) {
            resolved.replace(r);
        }
    }
    if let Some(resolved) = resolved {
        if let Ok(mut section) = sections.get_mut(trigger.entity()) {
            *section = resolved.section;
        }
        if let Some(pts) = resolved.points {
            if let Ok(mut points) = points.get_mut(trigger.entity()) {
                *points = pts;
            }
        }
        if let Some(hook) = resolved.hook_update {
            if let Ok(mut location) = locations.get_mut(trigger.entity()) {
                match &mut location.animation_hook {
                    GridLocationAnimationHook::SectionDriven(s) => {
                        s.diff = hook[0].unwrap();
                    }
                    GridLocationAnimationHook::PointDriven(p) => {
                        if let Some(h) = hook[0] {
                            p.point_a.diff = h;
                        }
                        if let Some(h) = hook[1] {
                            p.point_b.diff = h;
                        }
                        if let Some(h) = hook[2] {
                            p.point_c.diff = h;
                        }
                        if let Some(h) = hook[3] {
                            p.point_d.diff = h;
                        }
                    }
                }
            }
        }
    }
    if let Ok(deps) = dependents.get(trigger.entity()) {
        if deps.0.is_empty() {
            return;
        }
        tree.trigger_targets(
            ResolveGridLocation {},
            deps.0.iter().copied().collect::<Vec<Entity>>(),
        );
    }
}
#[derive(Copy, Clone, Default, Debug)]
pub(crate) struct ReferentialData {
    pub(crate) section: Section<LogicalContext>,
    pub(crate) points: Option<Points<LogicalContext>>,
    pub(crate) grid: Grid,
}
#[derive(Debug, Clone, Copy)]
pub(crate) struct ResolvedLocation {
    pub(crate) section: Section<LogicalContext>,
    pub(crate) points: Option<Points<LogicalContext>>,
    pub(crate) hook_update: Option<[Option<Section<LogicalContext>>; 4]>,
}
impl ResolvedLocation {
    pub(crate) fn new() -> Self {
        Self {
            section: Section::default(),
            points: None,
            hook_update: None,
        }
    }
    pub(crate) fn section(mut self, section: Section<LogicalContext>) -> Self {
        self.section = section;
        self
    }
}
