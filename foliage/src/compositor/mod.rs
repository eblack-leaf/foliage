use crate::coordinate::{Coordinate, InterfaceContext};
use crate::scene::align::SceneAnchor;
use crate::scene::bind::{SceneNodes, SceneRoot};
use crate::scene::{Scene, SceneSpawn, ToExternalArgs};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Commands, Query, Res, ResMut};
use std::collections::HashMap;
use bevy_ecs::prelude::{Component, Resource};

#[derive(Resource)]
pub struct Compositor {
    pub(crate) segments: HashMap<SegmentHandle, Segment>,
    pub(crate) nodes: HashMap<SegmentHandle, SceneNodes>,
    pub(crate) transitions: HashMap<SegmentHandle, TransitionHandles>,
    pub(crate) current_binding: HashMap<SegmentHandle, TransitionBinding>,
}
pub struct Padding {}
impl Padding {
    pub fn padded_anchor(&self, scene_anchor: SceneAnchor) -> SceneAnchor {
        todo!()
    }
}
pub struct Segment(pub Coordinate<InterfaceContext>);
pub struct SegmentNodes(pub HashMap<SegmentBinding, Entity>);
pub struct SegmentBinding(pub i32);
#[derive(Component, Copy, Clone, Hash, Eq, PartialEq)]
pub struct SegmentHandle(pub i32);
pub struct TransitionBinding(pub i32);
pub struct TransitionHandles(pub HashMap<TransitionBinding, Entity>);
#[derive(Component)]
pub struct TransitionBindRequests<B: Bundle>(pub HashMap<SegmentBinding, (B, Padding)>);
#[derive(Component)]
pub struct TransitionSceneBindRequests<S: Scene + Send + Sync + 'static>(
    pub HashMap<SegmentBinding, (S::Args<'static>, Padding)>,
);
#[derive(Component, Copy, Clone)]
pub struct TransitionSelected(pub bool);
fn spawn_scene_requests<'a, 'b, S: Scene>(
    transitions: Query<
        (
            Entity,
            &TransitionSceneBindRequests<S>,
            &TransitionSelected,
            &SegmentHandle,
        ),
        Changed<TransitionSelected>,
    >,
    mut cmd: Commands,
    external_res: S::ExternalResources<'b>,
    mut compositor: ResMut<Compositor>,
) {
    let external_args = external_res.to_external_args::<'a, 'b>();
    for (entity, requests, selected, handle) in transitions.iter() {
        if selected.0 {
            for request in requests.0.iter() {
                let anchor = SceneAnchor(compositor.segments.get(handle).unwrap().0);
                cmd.spawn_scene::<S>(
                    request.1 .1.padded_anchor(anchor),
                    &request.1 .0,
                    &external_args,
                    SceneRoot::default(),
                );
            }
            cmd.entity(entity)
                .remove::<TransitionSceneBindRequests<S>>();
        }
    }
}
