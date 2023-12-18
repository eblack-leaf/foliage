use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{Coordinate, CoordinateUnit, InterfaceContext};
use crate::differential::Despawn;
use crate::ginkgo::viewport::ViewportHandle;
use crate::scene::align::{AlignmentPoint, SceneAnchor};
use crate::scene::bind::SceneRoot;
use crate::scene::{Scene, SceneSpawn};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Event, Query, Resource};
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Commands, Res, ResMut};
use std::collections::{HashMap, HashSet};
use crate::elm::config::ElmConfiguration;
use crate::elm::Elm;
use crate::elm::leaf::{EmptySetDescriptor, Leaf};

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct SegmentHandle(pub i32);
#[derive(Resource)]
pub struct Compositor {
    pub segments: HashMap<SegmentHandle, Segment>,
    pub bindings: HashMap<SegmentHandle, Entity>,
    pub workflow: HashMap<WorkflowHandle, Workflow>,
    pub segment_handle_factory: HandleGenerator,
}
impl Compositor {
    pub fn coordinate(
        &self,
        viewport_section: Section<InterfaceContext>,
        handle: &SegmentHandle,
    ) -> Coordinate<InterfaceContext> {
        let segment = self.segments.get(handle).unwrap();
        let mut coordinate = Coordinate::<InterfaceContext>::default();
        coordinate.section.position = segment.pos.calc(viewport_section);
        coordinate.section.area = segment.area.calc(viewport_section);
        coordinate.layer = segment.layer;
        coordinate
    }
}
#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct WorkflowHandle(pub i32);
pub struct Workflow {
    stage: WorkflowStage,
    bindings: HashSet<SegmentHandle>,
    transitions: HashMap<WorkflowStage, Entity>,
}
#[derive(Default)]
pub struct HandleGenerator {
    segment: i32,
    holes: Vec<i32>,
}
impl HandleGenerator {
    pub fn generate_segment(&mut self) -> SegmentHandle {
        let handle = if !self.holes.is_empty() {
            self.holes.pop().unwrap()
        } else {
            let h = self.segment;
            self.segment += 1;
            h
        };
        SegmentHandle(handle)
    }
    pub fn release(&mut self, handle: SegmentHandle) {
        self.holes.push(handle.0);
    }
}
#[derive(Default, Copy, Clone, Hash, Eq, PartialEq)]
pub struct WorkflowStage(pub i32);
#[derive(Event)]
pub struct WorkflowTransition(pub WorkflowHandle, pub WorkflowStage);
#[derive(Component)]
pub struct TransitionEngaged(pub bool);
#[derive(Component)]
pub struct TransitionRemovals(pub HashSet<SegmentHandle>);
#[derive(Hash, Eq, PartialEq)]
pub struct TransitionKey {
    segment_handle: SegmentHandle,
    stage: WorkflowStage,
}
#[derive(Component)]
pub struct TransitionBindRequest<B: Bundle>(pub Vec<(SegmentHandle, Option<B>)>);
fn workflow_update(
    mut compositor: ResMut<Compositor>,
) {}
fn fill_bind_requests<B: Bundle>(
    mut cmd: Commands,
    mut query: Query<
        (&mut TransitionBindRequest<B>, &TransitionEngaged),
        Changed<TransitionEngaged>,
    >,
    mut compositor: ResMut<Compositor>,
    viewport_handle: Res<ViewportHandle>,
) {
    for (mut request, engaged) in query.iter_mut() {
        if engaged.0 {
            for (handle, bundle) in request.0.iter_mut() {
                let coordinate = compositor.coordinate(viewport_handle.section(), handle);
                let entity = cmd.spawn(bundle.take().unwrap()).insert(coordinate).id();
                let old = compositor.bindings.insert(*handle, entity);
                if let Some(o) = old {
                    cmd.entity(o).insert(Despawn::signal_despawn());
                }
            }
        }
    }
}
#[derive(Component)]
pub struct TransitionSceneRequest<S: Scene>(pub Vec<(SegmentHandle, S::Args<'static>)>);
fn fill_scene_bind_requests<S: Scene>(
    mut compositor: ResMut<Compositor>,
    query: Query<(&TransitionSceneRequest<S>, &TransitionEngaged)>,
    mut cmd: Commands,
    viewport_handle: Res<ViewportHandle>,
    external_res: S::ExternalResources<'_>,
) {
    for (request, engaged) in query.iter() {
        if engaged.0 {
            for (handle, args) in request.0.iter() {
                let coordinate = compositor.coordinate(viewport_handle.section(), handle);
                let entity = cmd.spawn_scene::<S>(
                    SceneAnchor(coordinate),
                    args,
                    &external_res,
                    SceneRoot::default(),
                );
                let old = compositor.bindings.insert(*handle, entity);
                if let Some(o) = old {
                    cmd.entity(o).insert(Despawn::signal_despawn());
                }
            }
        }
    }
}
fn clear_engaged(
    mut engaged: Query<(&mut TransitionEngaged, &TransitionRemovals), Changed<TransitionEngaged>>,
    mut compositor: ResMut<Compositor>,
    mut cmd: Commands,
) {
    for (mut e, removals) in engaged.iter_mut() {
        e.0 = false;
        for r in removals.0.iter() {
            let old = compositor.bindings.remove(r);
            if let Some(o) = old {
                cmd.entity(o).insert(Despawn::signal_despawn());
            }
        }
    }
}
pub enum SegmentUnit {
    Fixed(CoordinateUnit),
    Relative(f32),
}
pub struct SegmentPosition {
    pub x: SegmentUnit,
    pub y: SegmentUnit,
}

impl SegmentPosition {
    pub(crate) fn calc(
        &self,
        viewport_section: Section<InterfaceContext>,
    ) -> Position<InterfaceContext> {
        let x = match self.x {
            SegmentUnit::Fixed(fix) => fix,
            SegmentUnit::Relative(rel) => viewport_section.width() * rel,
        };
        let y = match self.y {
            SegmentUnit::Fixed(fix) => fix,
            SegmentUnit::Relative(rel) => viewport_section.height() * rel,
        };
        (x, y).into()
    }
}
pub struct SegmentArea {
    pub width: SegmentUnit,
    pub height: SegmentUnit,
}

impl SegmentArea {
    pub(crate) fn calc(
        &self,
        viewport_section: Section<InterfaceContext>,
    ) -> Area<InterfaceContext> {
        let w = match self.width {
            SegmentUnit::Fixed(fix) => fix,
            SegmentUnit::Relative(rel) => viewport_section.width() * rel,
        };
        let h = match self.height {
            SegmentUnit::Fixed(fix) => fix,
            SegmentUnit::Relative(rel) => viewport_section.height() * rel,
        };
        (w, h).into()
    }
}

pub struct Segment {
    pub pos: SegmentPosition,
    pub area: SegmentArea,
    pub layer: Layer,
}

impl Leaf for Compositor {
    type SetDescriptor = EmptySetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {
    }

    fn attach(elm: &mut Elm) {
        elm.main().add_systems(());
    }
}