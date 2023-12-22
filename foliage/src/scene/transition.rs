use crate::compositor::layout::Layout;
use crate::compositor::workflow::{
    TransitionEngaged, Workflow, WorkflowHandle, WorkflowTransition,
};
use crate::compositor::Compositor;
use crate::coordinate::area::Area;
use crate::coordinate::InterfaceContext;
use crate::differential::Despawn;
use crate::elm::config::ElmConfiguration;
use crate::elm::leaf::{EmptySetDescriptor, Leaf};
use crate::elm::Elm;
use crate::scene::align::{SceneAlignment, SceneAnchor};
use crate::scene::bind::{SceneBind, SceneBinding, SceneNodeEntry, SceneNodes, SceneRoot};
use crate::scene::{Scene, SceneSpawn};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Commands;
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Query, Res, StaticSystemParam};
use std::collections::{HashMap, HashSet};
#[macro_export]
macro_rules! scene_transition_bind_enable {
    ($elm:ident $(,$typename:ty)+) => {
        $($elm.enable_scene_transition_bind::<$typename>();)+
    };
}
#[macro_export]
macro_rules! scene_transition_scene_bind_enable {
    ($elm:ident $(,$typename:ty)+) => {
        $($elm.enable_scene_transition_scene_bind::<$typename>();)+
    };
}
#[derive(Component, Default)]
pub struct SceneTransitionRemovals(pub HashMap<Layout, HashSet<SceneBinding>>);
#[derive(Default, Bundle)]
pub struct SceneTransition {
    removals: SceneTransitionRemovals,
    engaged: TransitionEngaged,
}
#[derive(Component, Copy, Clone)]
pub struct SceneTransitionRoot(pub(crate) Entity);
#[derive(Component)]
pub(crate) struct SceneTransitionBindRequest<B: Bundle + Clone>(
    pub Vec<(SceneBinding, SceneAlignment, B)>,
);
pub(crate) fn fill_scene_transition_scene_bind_requests<S: Scene>(
    query: Query<
        (
            &SceneTransitionSceneBindRequest<S>,
            &TransitionEngaged,
            &SceneTransitionRoot,
        ),
        Changed<TransitionEngaged>,
    >,
    mut cmd: Commands,
    external_res: StaticSystemParam<<S as Scene>::ExternalResources>,
    mut scene_roots: Query<(&SceneRoot, &SceneAnchor, &mut SceneNodes)>,
) {
    for (request, engaged, t_root) in query.iter() {
        if engaged.0 {
            for (binding, alignment, area, args) in request.0.iter() {
                if let Ok((root, anchor, mut nodes)) = scene_roots.get_mut(t_root.0) {
                    let mut a = *anchor;
                    a.0.section.area = area(a);
                    let e = cmd.spawn_scene::<S>(a, args, &external_res, *root);
                    cmd.entity(e).insert(*alignment).insert(a.0);
                    if let Some(old) = nodes.0.insert(*binding, SceneNodeEntry::new(e, true)) {
                        cmd.entity(old.entity()).insert(Despawn::signal_despawn());
                    }
                }
            }
        }
    }
}
pub(crate) fn fill_scene_transition_bind_requests<B: Bundle + Clone>(
    query: Query<(
        &SceneTransitionBindRequest<B>,
        &TransitionEngaged,
        &SceneTransitionRoot,
    )>,
    mut cmd: Commands,
    mut scene_roots: Query<(&SceneAnchor, &mut SceneNodes)>,
) {
    for (request, engaged, t_root) in query.iter() {
        if engaged.0 {
            for (binding, alignment, bundle) in request.0.iter() {
                if let Ok((anchor, mut nodes)) = scene_roots.get_mut(t_root.0) {
                    let entity = cmd
                        .spawn(bundle.clone())
                        .insert(SceneBind::new(*alignment, *binding, *anchor))
                        .id();
                    if let Some(old) = nodes.0.insert(*binding, SceneNodeEntry::new(entity, false))
                    {
                        cmd.entity(old.entity()).insert(Despawn::signal_despawn());
                    }
                }
            }
        }
    }
}
pub(crate) fn clear_engaged(
    mut engaged_transitions: Query<
        (
            &mut TransitionEngaged,
            &SceneTransitionRemovals,
            &SceneTransitionRoot,
        ),
        Changed<TransitionEngaged>,
    >,
    mut cmd: Commands,
    mut nodes_query: Query<&mut SceneNodes>,
    compositor: Res<Compositor>,
) {
    for (mut engaged, removals, t_root) in engaged_transitions.iter_mut() {
        engaged.0 = false;
        if let Some(rem) = removals.0.get(&compositor.layout()) {
            for binding in rem.iter() {
                if let Ok(mut nodes) = nodes_query.get_mut(t_root.0) {
                    if let Some(old) = nodes.0.remove(binding) {
                        cmd.entity(old.entity()).insert(Despawn::signal_despawn());
                    }
                }
            }
        }
    }
}
#[derive(Component)]
pub(crate) struct SceneTransitionSceneBindRequest<S: Scene>(
    pub  Vec<(
        SceneBinding,
        SceneAlignment,
        fn(SceneAnchor) -> Area<InterfaceContext>,
        S::Args<'static>,
    )>,
);
pub struct SceneTransitionDescriptor<'a, 'w, 's> {
    cmd: &'a mut Commands<'w, 's>,
    transition: SceneTransition,
    root: SceneTransitionRoot,
    entity: Entity,
}
impl<'a, 'w, 's> SceneTransitionDescriptor<'a, 'w, 's> {
    pub fn new(cmd: &'a mut Commands<'w, 's>, scene_transition_root: SceneTransitionRoot) -> Self {
        let entity = cmd.spawn_empty().id();
        Self {
            cmd,
            transition: SceneTransition::default(),
            root: scene_transition_root,
            entity,
        }
    }
    pub fn add_removal(mut self, layout: Layout, r: HashSet<SceneBinding>) -> Self {
        self.transition.removals.0.insert(layout, r);
        self
    }
    pub fn bind<B: Bundle + std::clone::Clone>(
        self,
        b: Vec<(SceneBinding, SceneAlignment, B)>,
    ) -> Self {
        self.cmd
            .entity(self.entity)
            .insert(SceneTransitionBindRequest::<B>(b))
            .insert(self.root);
        self
    }
    pub fn bind_scene<S: Scene>(
        self,
        s: Vec<(
            SceneBinding,
            SceneAlignment,
            fn(SceneAnchor) -> Area<InterfaceContext>,
            S::Args<'static>,
        )>,
    ) -> Self {
        self.cmd
            .entity(self.entity)
            .insert(SceneTransitionSceneBindRequest::<S>(s))
            .insert(self.root);
        self
    }
    pub fn build(self) -> Entity {
        self.cmd.entity(self.entity).insert(self.transition);
        self.entity
    }
}
#[derive(Component)]
pub struct SceneWorkflow(pub HashMap<WorkflowHandle, Workflow>);
impl SceneWorkflow {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn with_workflow(mut self, wh: (WorkflowHandle, Workflow)) -> Self {
        self.0.insert(wh.0, wh.1);
        self
    }
}
#[derive(Component)]
pub struct WorkflowTransitionQueue(pub Vec<WorkflowTransition>);
pub(crate) fn trigger_workflow(
    mut query: Query<
        (&mut SceneWorkflow, &mut WorkflowTransitionQueue),
        Changed<WorkflowTransitionQueue>,
    >,
    mut cmd: Commands,
) {
    for (mut workflow, mut queue) in query.iter_mut() {
        for queued_transition_event in queue.0.drain(..) {
            if let Some(trans) = workflow
                .0
                .get(&queued_transition_event.0)
                .unwrap()
                .transitions
                .get(&queued_transition_event.1)
            {
                cmd.entity(*trans).insert(TransitionEngaged(true));
            }
            workflow
                .0
                .get_mut(&queued_transition_event.0)
                .unwrap()
                .stage
                .replace(queued_transition_event.1);
        }
    }
}

impl Leaf for SceneTransition {
    type SetDescriptor = EmptySetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        elm.main().add_systems((trigger_workflow, clear_engaged));
    }
}
