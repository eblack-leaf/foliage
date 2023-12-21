use crate::compositor::layout::{Layout, Orientation, Threshold};
use crate::compositor::{Compositor, SegmentHandle};
use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::InterfaceContext;
use crate::differential::Despawn;
use crate::elm::leaf::Tag;
use crate::ginkgo::viewport::ViewportHandle;
use crate::scene::align::SceneAnchor;
use crate::scene::bind::SceneRoot;
use crate::scene::{IsScene, Scene, SceneSpawn};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::change_detection::{Res, ResMut};
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::prelude::{Changed, Commands, DetectChanges, Query};
use bevy_ecs::system::StaticSystemParam;
use std::collections::{HashMap, HashSet};
#[macro_export]
macro_rules! bind_enable {
    ($elm:ident $(,$typename:ty)+) => {
        $($elm.enable_bind::<$typename>();)+
    };
}
#[macro_export]
macro_rules! scene_bind_enable {
    ($elm:ident $(,$typename:ty)+) => {
        $($elm.enable_scene_bind::<$typename>();)+
    };
}
#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct WorkflowHandle(pub i32);

pub struct Workflow {
    pub(crate) stage: WorkflowStage,
    pub(crate) transitions: HashMap<WorkflowStage, Entity>,
}
impl Workflow {
    pub fn new(stage: WorkflowStage, transitions: HashMap<WorkflowStage, Entity>) -> Self {
        Self { stage, transitions }
    }
}
#[derive(Default, Copy, Clone, Hash, Eq, PartialEq)]
pub struct WorkflowStage(pub i32);

#[derive(Event)]
pub struct WorkflowTransition(pub WorkflowHandle, pub WorkflowStage);

#[derive(Component, Default)]
pub struct TransitionEngaged(pub(crate) bool);
#[derive(Component, Default)]
pub struct TransitionRemovals(pub HashMap<Layout, HashSet<SegmentHandle>>);

#[derive(Hash, Eq, PartialEq)]
pub struct TransitionKey {
    segment_handle: SegmentHandle,
    stage: WorkflowStage,
}
#[derive(Bundle, Default)]
pub struct Transition {
    engaged: TransitionEngaged,
    removals: TransitionRemovals,
}
pub struct RemovalDescriptor(pub HashSet<SegmentHandle>);
impl RemovalDescriptor {
    pub fn new() -> Self {
        Self(HashSet::new())
    }
    pub fn with_removal(mut self, handle: SegmentHandle) -> Self {
        self.0.insert(handle);
        self
    }
    pub fn finish(self) -> HashSet<SegmentHandle> {
        self.0
    }
}
pub struct TransitionDescriptor<'a, 'w, 's> {
    cmd: &'a mut Commands<'w, 's>,
    transition: Transition,
    entity: Entity,
}
impl<'a, 'w, 's> TransitionDescriptor<'a, 'w, 's> {
    pub fn new(cmd: &'a mut Commands<'w, 's>) -> Self {
        let entity = cmd.spawn_empty().id();
        Self {
            cmd,
            transition: Transition::default(),
            entity,
        }
    }
    pub fn add_removal(mut self, layout: Layout, r: HashSet<SegmentHandle>) -> Self {
        self.transition.removals.0.insert(layout, r);
        self
    }
    pub fn bind<B: Bundle>(
        self,
        b: Vec<(SegmentHandle, TransitionBindValidity, Option<B>)>,
    ) -> Self {
        self.cmd
            .entity(self.entity)
            .insert(TransitionBindRequest::<B>(b));
        self
    }
    pub fn bind_scene<S: Scene>(
        self,
        s: Vec<(SegmentHandle, TransitionBindValidity, S::Args<'static>)>,
    ) -> Self {
        self.cmd
            .entity(self.entity)
            .insert(TransitionSceneBindRequest::<S>(s));
        self
    }
    pub fn build(self) -> Entity {
        self.cmd.entity(self.entity).insert(self.transition);
        self.entity
    }
}
#[derive(Clone, Default)]
pub struct TransitionBindValidity(pub HashSet<Layout>);
impl TransitionBindValidity {
    pub fn all() -> Self {
        let mut set = HashSet::new();
        for l in Layout::PORTRAIT {
            set.insert(l);
        }
        for l in Layout::LANDSCAPE {
            set.insert(l);
        }
        Self(set)
    }
    pub fn mobile_portrait() -> Self {
        Self {
            0: {
                let mut map = HashSet::new();
                map.insert(Layout::new(Orientation::Portrait, Threshold::Mobile));
                map
            },
        }
    }
    pub fn landscape_mobile() -> Self {
        Self {
            0: {
                let mut map = HashSet::new();
                map.insert(Layout::new(Orientation::Landscape, Threshold::Mobile));
                map
            },
        }
    }
    pub fn portrait_tablet() -> Self {
        Self {
            0: {
                let mut map = HashSet::new();
                map.insert(Layout::new(Orientation::Portrait, Threshold::Tablet));
                map
            },
        }
    }
    pub fn landscape_tablet() -> Self {
        Self {
            0: {
                let mut map = HashSet::new();
                map.insert(Layout::new(Orientation::Landscape, Threshold::Tablet));
                map
            },
        }
    }
    pub fn portrait_desktop() -> Self {
        Self {
            0: {
                let mut map = HashSet::new();
                map.insert(Layout::new(Orientation::Portrait, Threshold::Desktop));
                map
            },
        }
    }
    pub fn landscape_desktop() -> Self {
        Self {
            0: {
                let mut map = HashSet::new();
                map.insert(Layout::new(Orientation::Landscape, Threshold::Desktop));
                map
            },
        }
    }
    pub fn portrait_workstation() -> Self {
        Self {
            0: {
                let mut map = HashSet::new();
                map.insert(Layout::new(Orientation::Portrait, Threshold::Workstation));
                map
            },
        }
    }
    pub fn landscape_workstation() -> Self {
        Self {
            0: {
                let mut map = HashSet::new();
                map.insert(Layout::new(Orientation::Landscape, Threshold::Workstation));
                map
            },
        }
    }
    pub fn with_landscape_mobile(mut self) -> Self {
        self.0
            .insert(Layout::new(Orientation::Landscape, Threshold::Mobile));
        self
    }
    pub fn with_portrait_tablet(mut self) -> Self {
        self.0
            .insert(Layout::new(Orientation::Portrait, Threshold::Tablet));
        self
    }
    pub fn with_landscape_tablet(mut self) -> Self {
        self.0
            .insert(Layout::new(Orientation::Landscape, Threshold::Tablet));
        self
    }
    pub fn with_portrait_desktop(mut self) -> Self {
        self.0
            .insert(Layout::new(Orientation::Portrait, Threshold::Desktop));
        self
    }
    pub fn with_landscape_desktop(mut self) -> Self {
        self.0
            .insert(Layout::new(Orientation::Landscape, Threshold::Desktop));
        self
    }
    pub fn with_portrait_workstation(mut self) -> Self {
        self.0
            .insert(Layout::new(Orientation::Portrait, Threshold::Workstation));
        self
    }
    pub fn with_landscape_workstation(mut self) -> Self {
        self.0
            .insert(Layout::new(Orientation::Landscape, Threshold::Workstation));
        self
    }
}
#[derive(Component)]
pub struct TransitionBindRequest<B: Bundle>(
    pub Vec<(SegmentHandle, TransitionBindValidity, Option<B>)>,
);
pub(crate) fn resize_segments(
    mut query: Query<(
        Entity,
        &mut Position<InterfaceContext>,
        &mut Area<InterfaceContext>,
        &mut Layer,
        &SegmentHandle,
        Option<&Tag<IsScene>>,
        Option<&mut SceneAnchor>,
    )>,
    mut compositor: ResMut<Compositor>,
    viewport_handle: Res<ViewportHandle>,
    mut cmd: Commands,
) {
    if viewport_handle.is_changed() {
        compositor.layout = Layout::from_area(viewport_handle.section.area);
        for (entity, mut pos, mut area, mut layer, handle, tag, anchor) in query.iter_mut() {
            if let Some(coordinate) = compositor.coordinate(viewport_handle.section(), handle) {
                if tag.is_some() {
                    if let Some(mut a) = anchor {
                        a.0 = coordinate;
                    }
                }
                *pos = coordinate.section.position;
                *area = coordinate.section.area;
                *layer = coordinate.layer;
            } else {
                cmd.entity(entity).insert(Despawn::signal_despawn());
            }
        }
    }
}
pub(crate) fn fill_bind_requests<B: Bundle>(
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
            for (handle, validity, bundle) in request.0.iter_mut() {
                if validity.0.contains(compositor.layout()) {
                    if let Some(coordinate) =
                        compositor.coordinate(viewport_handle.section(), handle)
                    {
                        let entity = cmd
                            .spawn(bundle.take().unwrap())
                            .insert(coordinate)
                            .insert(*handle)
                            .id();
                        let old = compositor.bindings.insert(*handle, entity);
                        if let Some(o) = old {
                            cmd.entity(o).insert(Despawn::signal_despawn());
                        }
                    }
                }
            }
        }
    }
}
pub(crate) fn fill_scene_bind_requests<S: Scene>(
    mut compositor: ResMut<Compositor>,
    query: Query<(&TransitionSceneBindRequest<S>, &TransitionEngaged)>,
    viewport_handle: Res<ViewportHandle>,
    external_res: StaticSystemParam<<S as Scene>::ExternalResources>,
    mut cmd: Commands,
) {
    for (request, engaged) in query.iter() {
        if engaged.0 {
            for (handle, validity, args) in request.0.iter() {
                if validity.0.contains(compositor.layout()) {
                    if let Some(coordinate) =
                        compositor.coordinate(viewport_handle.section(), handle)
                    {
                        let entity = cmd.spawn_scene::<S>(
                            SceneAnchor(coordinate),
                            args,
                            &external_res,
                            SceneRoot::default(),
                        );
                        cmd.entity(entity).insert(*handle);
                        let old = compositor.bindings.insert(*handle, entity);
                        if let Some(o) = old {
                            cmd.entity(o).insert(Despawn::signal_despawn());
                        }
                    }
                }
            }
        }
    }
}

#[derive(Component)]
pub struct TransitionSceneBindRequest<S: Scene>(
    pub Vec<(SegmentHandle, TransitionBindValidity, S::Args<'static>)>,
);

pub struct WorkflowDescriptor {
    pub handle: WorkflowHandle,
    pub transitions: HashMap<WorkflowStage, Entity>,
}
impl WorkflowDescriptor {
    pub fn new<WH: Into<WorkflowHandle>>(handle: WH) -> Self {
        Self {
            handle: handle.into(),
            transitions: Default::default(),
        }
    }
    pub fn with_transition(mut self, stage: WorkflowStage, transition: Entity) -> Self {
        self.transitions.insert(stage, transition);
        self
    }
    pub fn workflow(self) -> (WorkflowHandle, Workflow) {
        (
            self.handle,
            Workflow::new(WorkflowStage(0), self.transitions),
        )
    }
}