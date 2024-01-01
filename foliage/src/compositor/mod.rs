use crate::compositor::layout::Layout;
use crate::compositor::segment::ResponsiveSegment;
use crate::compositor::workflow::{resize_segments, ThresholdChange, WorkflowStage};
use crate::coordinate::area::Area;
use crate::coordinate::section::Section;
use crate::coordinate::{Coordinate, InterfaceContext};
use crate::differential::Despawn;
use crate::elm::config::{CoreSet, ElmConfiguration};
use crate::elm::leaf::{EmptySetDescriptor, Leaf};
use crate::elm::{Elm, EventStage};
use crate::generator::HandleGenerator;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EventReader;
use bevy_ecs::prelude::{Component, IntoSystemConfigs, Query, Resource};
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Commands, ResMut};
use std::collections::HashMap;
use workflow::{
    TransitionEngaged, TransitionRemovals, Workflow, WorkflowHandle, WorkflowTransition,
};

pub mod layout;
pub mod segment;
pub mod workflow;
#[derive(Copy, Clone, Hash, Eq, PartialEq, Component, Debug)]
pub struct SegmentHandle(pub i32);
#[derive(Resource)]
pub struct Compositor {
    pub(crate) segments: HashMap<SegmentHandle, ResponsiveSegment>,
    pub(crate) bindings: HashMap<SegmentHandle, Entity>,
    pub(crate) workflow_groups: HashMap<WorkflowHandle, Workflow>,
    pub(crate) generator: HandleGenerator,
    pub(crate) layout: Layout,
}
impl Compositor {
    pub(crate) fn engage_transition(
        &mut self,
        cmd: &mut Commands,
        workflow_handle: WorkflowHandle,
        workflow_stage: WorkflowStage,
    ) {
        if let Some(transition) = self
            .workflow_groups
            .get(&workflow_handle)
            .unwrap()
            .transitions
            .get(&workflow_stage)
        {
            cmd.entity(*transition).insert(TransitionEngaged(true));
        }
        self.workflow_groups
            .get_mut(&workflow_handle)
            .unwrap()
            .stage
            .replace(workflow_stage);
    }
    pub(crate) fn new(area: Area<InterfaceContext>) -> Self {
        let layout = Layout::from_area(area);
        Self {
            segments: Default::default(),
            bindings: Default::default(),
            workflow_groups: Default::default(),
            generator: Default::default(),
            layout,
        }
    }
    pub fn add_segment(&mut self, segment: ResponsiveSegment) -> SegmentHandle {
        let handle = SegmentHandle(self.generator.generate());
        self.segments.insert(handle, segment);
        handle
    }
    pub fn add_workflow(&mut self, workflow_desc: (WorkflowHandle, Workflow)) {
        self.workflow_groups
            .insert(workflow_desc.0, workflow_desc.1);
    }
    pub fn layout(&self) -> Layout {
        self.layout
    }
    pub fn coordinate(
        &self,
        viewport_section: Section<InterfaceContext>,
        handle: &SegmentHandle,
    ) -> Option<Coordinate<InterfaceContext>> {
        let segment = self.segments.get(handle).unwrap();
        segment.coordinate(&self.layout(), viewport_section)
    }
}

fn workflow_update(
    mut compositor: ResMut<Compositor>,
    mut events: EventReader<WorkflowTransition>,
    mut cmd: Commands,
) {
    tracing::trace!("updating-workflow");
    for event in events.read() {
        compositor.engage_transition(&mut cmd, event.0, event.1);
    }
}
fn clear_engaged(
    mut engaged: Query<
        (Entity, &mut TransitionEngaged, &TransitionRemovals),
        Changed<TransitionEngaged>,
    >,
    mut compositor: ResMut<Compositor>,
    mut cmd: Commands,
) {
    tracing::trace!("clear-engaged");
    for (entity, mut transition_engaged, removals) in engaged.iter_mut() {
        if transition_engaged.0 {
            transition_engaged.0 = false;
            cmd.entity(entity).remove::<ThresholdChange>();
            if let Some(rem) = removals.0.get(&compositor.layout()) {
                for r in rem.iter() {
                    let old = compositor.bindings.remove(r);
                    if let Some(o) = old {
                        cmd.entity(o).insert(Despawn::signal_despawn());
                    }
                }
            }
        }
    }
}
impl Leaf for Compositor {
    type SetDescriptor = EmptySetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        elm.add_event::<WorkflowTransition>(EventStage::Process);
        elm.main().add_systems((
            resize_segments.in_set(CoreSet::CompositorSetup),
            workflow_update.in_set(CoreSet::CompositorSetup),
            clear_engaged.in_set(CoreSet::CompositorTeardown),
        ));
    }
}
