use std::collections::{HashMap, HashSet};

use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::prelude::{DetectChanges, Entity};
use bevy_ecs::query::{Changed, Or, With};
use bevy_ecs::system::{Commands, Query, Res};

use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::placement::Placement;
use crate::coordinate::position::Position;
use crate::coordinate::LogicalContext;
use crate::grid::{Grid, GridPlacement, Layout, LayoutGrid};
use crate::signal::ActionHandle;
use crate::signal::{Clean, Signal, TriggerTarget};

#[derive(Bundle)]
pub(crate) struct ViewComponents {
    view: View,
    active: ViewActive,
    current: CurrentViewStage,
    grid: ViewGrid, // targets use this grid instead of main
    placement: Placement<LogicalContext>,
    grid_placement: GridPlacement,
}
impl ViewComponents {
    pub(crate) fn new(grid_placement: GridPlacement, grid: Grid) -> Self {
        Self {
            view: View::new(),
            active: ViewActive(false),
            current: Default::default(),
            grid: ViewGrid(grid),
            placement: Default::default(),
            grid_placement,
        }
    }
}
#[derive(Component, Clone, Copy)]
pub struct ViewHandle(pub(crate) Entity);
impl ViewHandle {
    pub fn repr(&self) -> Entity {
        self.0
    }
}
#[derive(Copy, Clone, Default)]
pub struct Stage(pub(crate) i32);
#[derive(Component)]
pub struct View {
    pub(crate) stages: Vec<ViewStage>,
    pub(crate) targets: HashSet<TriggerTarget>,
}
#[derive(Component, Clone)]
pub struct ViewGrid(pub Grid);
pub(crate) fn adjust_view_grid_on_change(
    mut views: Query<
        (
            &Position<LogicalContext>,
            &Area<LogicalContext>,
            &Layer,
            &mut ViewGrid,
            &ViewActive,
        ),
        Or<(
            Changed<Position<LogicalContext>>,
            Changed<Area<LogicalContext>>,
            Changed<Layer>,
        )>,
    >,
) {
    for (pos, area, layer, mut view_grid, active) in views.iter_mut() {
        if active.0 {
            view_grid.0 = view_grid
                .0
                .clone()
                .placed_at(Placement::new((pos.coordinates, area.coordinates), *layer));
        }
    }
}
pub(crate) fn on_view_grid_change(
    views: Query<(&View, &ViewGrid, &ViewActive), Changed<ViewGrid>>,
    mut targets: Query<
        (
            &mut Position<LogicalContext>,
            &mut Area<LogicalContext>,
            &mut Layer,
            &GridPlacement,
        ),
        With<ViewHandle>,
    >,
    config: Res<Layout>,
) {
    for (view, grid, active) in views.iter() {
        if active.0 {
            for target in view.targets.iter() {
                if let Ok((mut pos, mut area, mut layer, grid_placement)) =
                    targets.get_mut(target.0)
                {
                    // calculate with view-grid + give to pos / area / layer
                    let placement = grid.0.place(grid_placement.clone(), *config);
                    *pos = placement.section.position;
                    *area = placement.section.area;
                    *layer = placement.layer;
                }
            }
        }
    }
}
pub(crate) fn on_target_grid_placement_change(
    views: Query<(&ViewGrid, &ViewActive)>,
    mut targets: Query<
        (
            &mut Position<LogicalContext>,
            &mut Area<LogicalContext>,
            &mut Layer,
            &GridPlacement,
            &ViewHandle,
        ),
        Changed<GridPlacement>,
    >,
    config: Res<Layout>,
) {
    for (mut pos, mut area, mut layer, grid_placement, handle) in targets.iter_mut() {
        if let Ok((grid, active)) = views.get(handle.0) {
            if active.0 {
                // calculate with view-grid + give to pos / area / layer
                let placement = grid.0.place(grid_placement.clone(), *config);
                *pos = placement.section.position;
                *area = placement.section.area;
                *layer = placement.layer;
            }
        }
    }
}
#[derive(Component)]
pub struct ViewActive(pub(crate) bool);
pub(crate) fn cleanup_view(
    mut views: Query<(&View, &ViewActive), Changed<ViewActive>>,
    mut cmd: Commands,
) {
    for (view, active) in views.iter() {
        if !active.0 {
            for target in view.targets.iter() {
                cmd.entity(target.0).insert(Clean::should_clean());
            }
        }
    }
}
#[derive(Component, Copy, Clone, Default)]
pub struct CurrentViewStage {
    pub(crate) stage: Stage,
}
impl CurrentViewStage {
    pub fn set(&mut self, stage: Stage) {
        self.stage = stage;
    }
}
impl View {
    pub(crate) fn new() -> Self {
        Self {
            stages: vec![],
            targets: Default::default(),
        }
    }
    pub(crate) fn awaiting_confirmation(&self, stage: Stage) -> bool {
        self.stages
            .get(stage.0 as usize)
            .expect("no-stage")
            .confirmed
            != self.targets
    }
}
pub(crate) struct StagedSignal {
    pub(crate) handle: SignalHandle,
    pub(crate) state_on_stage_start: Signal,
}
pub struct ViewStage {
    pub(crate) signals: HashMap<Entity, StagedSignal>,
    confirmed: HashSet<TriggerTarget>,
    pub(crate) on_end: Option<ActionHandle>,
}
impl Default for ViewStage {
    fn default() -> Self {
        ViewStage {
            signals: HashMap::new(),
            confirmed: HashSet::new(),
            on_end: None,
        }
    }
}
pub struct SignalHandle(pub(crate) Entity);
pub(crate) fn signal_stage(
    mut views: Query<
        (&mut View, &CurrentViewStage, &ViewActive),
        Or<(Changed<CurrentViewStage>, Changed<ViewActive>)>,
    >,
    mut cmd: Commands,
) {
    for (mut view, current, active) in views.iter_mut() {
        if active.0 {
            for target in view.targets.iter() {
                cmd.entity(target.0).insert(SignalConfirmation::Engaged);
            }
            view.stages
                .get_mut(current.stage.0 as usize)
                .expect("no-stage")
                .confirmed
                .clear();
            for signal in view
                .stages
                .get(current.stage.0 as usize)
                .expect("no-stage")
                .signals
                .iter()
            {
                cmd.entity(*signal.0).insert(signal.1.state_on_stage_start);
            }
        }
    }
}
pub(crate) fn resignal_on_layout_change(
    mut views: Query<(&mut View, &CurrentViewStage, &ViewActive)>,
    layout: Res<Layout>,
    mut cmd: Commands,
) {
    if layout.is_changed() {
        for (mut view, current, active) in views.iter_mut() {
            if active.0 {
                for target in view.targets.iter() {
                    cmd.entity(target.0).insert(SignalConfirmation::Engaged);
                }
                view.stages
                    .get_mut(current.stage.0 as usize)
                    .expect("no-stage")
                    .confirmed
                    .clear();
                for signal in view
                    .stages
                    .get(current.stage.0 as usize)
                    .expect("no-stage")
                    .signals
                    .iter()
                {
                    cmd.entity(*signal.0).insert(signal.1.state_on_stage_start);
                }
            }
        }
    }
}
// TODO transitions will need to set to ::Engaged if ::Confirmed && has transition after this
pub(crate) fn attempt_to_confirm(mut confirmees: Query<&mut SignalConfirmation>) {
    for mut confirm in confirmees.iter_mut() {
        if *confirm == SignalConfirmation::Engaged {
            *confirm = SignalConfirmation::Confirmed;
        }
    }
}
#[derive(Component, Eq, PartialEq, Copy, Clone, Default)]
pub enum SignalConfirmation {
    Engaged, // switch to engaged on stage-change for each target
    #[default]
    Neutral, // will need to attempt to set confirmed
    Confirmed, // and if transition still running => set back to engaged
}
pub(crate) fn signal_confirmation(
    mut views: Query<(&mut View, &CurrentViewStage, &ViewActive)>,
    mut targets: Query<&mut SignalConfirmation, Changed<SignalConfirmation>>,
    mut cmd: Commands,
) {
    for (mut view, current, active) in views.iter_mut() {
        if active.0 {
            if view.awaiting_confirmation(current.stage) {
                let mut confirmed_targets = HashSet::new();
                for target in view.targets.iter() {
                    if let Ok(mut c) = targets.get_mut(target.0) {
                        if *c == SignalConfirmation::Confirmed {
                            confirmed_targets.insert(*target);
                            *c = SignalConfirmation::Neutral;
                        }
                    }
                }
                for target in confirmed_targets {
                    let index = current.stage.0 as usize;
                    view.stages
                        .get_mut(index)
                        .expect("no-stage")
                        .confirmed
                        .insert(target);
                }
                if !view.awaiting_confirmation(current.stage) {
                    if let Some(end) = view
                        .stages
                        .get(current.stage.0 as usize)
                        .expect("no-stage")
                        .on_end
                    {
                        cmd.entity(end.0).insert(Signal::spawn());
                    }
                }
            }
        }
    }
}
