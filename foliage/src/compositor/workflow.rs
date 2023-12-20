use crate::compositor::layout::Layout;
use crate::compositor::{Compositor, SegmentHandle};
use crate::differential::Despawn;
use crate::ginkgo::viewport::ViewportHandle;
use crate::scene::align::SceneAnchor;
use crate::scene::bind::SceneRoot;
use crate::scene::{Scene, SceneSpawn};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::change_detection::{Res, ResMut};
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::prelude::{Changed, Commands, Query};
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
pub struct TransitionRemovals(pub HashSet<SegmentHandle>);

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
pub struct TransitionBindValidity(pub HashSet<Layout>);
#[derive(Component)]
pub struct TransitionBindRequest<B: Bundle>(pub Vec<(SegmentHandle, Option<B>)>);

pub(crate) fn fill_bind_requests<B: Bundle>(
    mut cmd: Commands,
    mut query: Query<(&mut TransitionBindRequest<B>, &TransitionEngaged), Changed<TransitionEngaged>>,
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
pub(crate) fn fill_scene_bind_requests<S: Scene>(
    mut compositor: ResMut<Compositor>,
    query: Query<(&TransitionSceneBindRequest<S>, &TransitionEngaged)>,
    viewport_handle: Res<ViewportHandle>,
    external_res: StaticSystemParam<<S as Scene>::ExternalResources>,
    mut cmd: Commands,
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

#[derive(Component)]
pub struct TransitionSceneBindRequest<S: Scene>(pub Vec<(SegmentHandle, S::Args<'static>)>);
