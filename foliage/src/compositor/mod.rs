use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{Coordinate, CoordinateUnit, InterfaceContext};
use crate::differential::Despawn;
use crate::elm::config::{CoreSet, ElmConfiguration};
use crate::elm::leaf::{EmptySetDescriptor, Leaf};
use crate::elm::{Elm, EventStage};
use crate::scene::Scene;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EventReader;
use bevy_ecs::prelude::{IntoSystemConfigs, Query, Resource};
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Commands, ResMut};
use std::collections::HashMap;
use workflow::{
    TransitionEngaged, TransitionRemovals, Workflow, WorkflowHandle, WorkflowTransition,
};

pub mod workflow;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct SegmentHandle(pub i32);
#[derive(Resource, Default)]
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

fn workflow_update(
    mut compositor: ResMut<Compositor>,
    mut events: EventReader<WorkflowTransition>,
    mut cmd: Commands,
) {
    for event in events.read() {
        if let Some(transition) = compositor
            .workflow
            .get(&event.0)
            .unwrap()
            .transitions
            .get(&event.1)
        {
            cmd.entity(*transition).insert(TransitionEngaged(true));
        }
        compositor.workflow.get_mut(&event.0).unwrap().stage = event.1;
    }
}

#[derive(Component)]
pub struct TransitionSceneRequest<S: Scene>(pub Vec<(SegmentHandle, S::Args<'static>)>);

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

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        elm.job.container.insert_resource(Compositor::default());
        elm.add_event::<WorkflowTransition>(EventStage::Process);
        elm.main().add_systems((
            workflow_update.in_set(CoreSet::CompositorSetup),
            clear_engaged.in_set(CoreSet::CompositorTeardown),
        ));
    }
}