use crate::coordinate::layer::Layer;
use crate::coordinate::CoordinateUnit;
use crate::scene::align::AlignmentPoint;
use crate::scene::Scene;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Event, Query, Resource};
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Commands, ResMut};
use std::collections::{HashMap, HashSet};

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct SegmentHandle(pub i32);
#[derive(Resource)]
pub struct Compositor {
    pub segments: HashMap<SegmentHandle, Segment>,
    pub bindings: HashMap<SegmentHandle, HashMap<SegmentBinding, Entity>>,
    pub workflow_group: HashMap<WorkflowHandle, Workflow>,
    pub transitions: HashMap<WorkflowHandle, HashMap<TransitionKey, Entity>>,
}
#[derive(Event)]
pub struct WorkflowTransition(pub WorkflowHandle, pub WorkflowStage);
pub(crate) struct TransitionEngaged(pub(crate) bool);
pub struct TransitionKey {
    segment_handle: SegmentHandle,
    stage: WorkflowStage,
}
pub struct Workflow {
    stage: WorkflowStage,
    group: WorkflowGroup,
}
pub struct TransitionRemovals(pub HashSet<SegmentBinding>);
pub struct TransitionBindRequest<B: Bundle>(pub SegmentBinding, pub B);
fn fill_bind_requests<B: Bundle>(
    mut cmd: Commands,
    query: Query<
        (
            &TransitionBindRequest<B>,
            &TransitionRemovals,
            &TransitionEngaged,
        ),
        Changed<TransitionEngaged>,
    >,
    mut compositor: ResMut<Compositor>,
) {
}
pub struct TransitionSceneRequest<S: Scene>(pub SegmentBinding, pub S::Args<'static>);
fn fill_scene_bind_requests() {}
#[derive(Hash, PartialEq, Eq, Copy, Clone)]
pub struct WorkflowHandle(pub i32);
pub enum SegmentPositionUnit {
    Fixed(CoordinateUnit),
    Relative(f32),
    FixedAligned(AlignmentPoint, SegmentHandle),
    RelativeAligned(f32, SegmentHandle),
}
pub struct SegmentPosition {
    pub x: SegmentPositionUnit,
    pub y: SegmentPositionUnit,
}
pub enum SegmentAreaUnit {
    Fixed(CoordinateUnit),
    Relative(f32),
    RelativeAligned(f32, SegmentHandle),
}
pub struct SegmentArea {
    pub width: SegmentAreaUnit,
    pub height: SegmentAreaUnit,
}
pub struct Segment {
    pub pos: SegmentPosition,
    pub area: SegmentArea,
    pub layer: Layer,
}
#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct SegmentBinding(pub i32);
#[derive(Default, Copy, Clone, Hash, Eq, PartialEq)]
pub struct WorkflowStage(pub i32);
#[derive(Hash, Eq, PartialEq, Copy, Clone, Default)]
pub struct WorkflowBinding(pub i32);
#[derive(Default)]
pub struct WorkflowGroup(pub HashMap<WorkflowBinding, SegmentHandle>);