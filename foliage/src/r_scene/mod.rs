use crate::coordinate::area::Area;
use crate::coordinate::{Coordinate, InterfaceContext};
use crate::generator::HandleGenerator;
use crate::r_scene::align::SceneAlignment;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Commands, Component, Resource};
use bevy_ecs::system::{SystemParam, SystemParamItem};
use std::collections::{HashMap, HashSet};

mod align;

#[derive(Copy, Clone, Default)]
pub struct Anchor(pub Coordinate<InterfaceContext>);
#[derive(Copy, Clone, Default, Hash, Eq, PartialEq, Debug)]
pub struct SceneBinding(pub i32);
#[derive(Resource)]
pub struct Coordinator {
    pub(crate) anchors: HashMap<SceneHandle, Anchor>,
    pub(crate) dependents: HashMap<SceneHandle, HashMap<SceneBinding, SceneHandle>>,
    pub(crate) dependent_bindings: HashMap<SceneHandle, HashMap<SceneBinding, Entity>>,
    pub(crate) root_bindings: HashMap<SceneHandle, Entity>,
    pub(crate) generator: HandleGenerator,
    pub(crate) alignments: HashMap<SceneHandle, HashMap<SceneBinding, SceneAlignment>>,
}
pub struct BindingCoordinate(
    pub SceneHandle,
    pub SceneBinding,
    pub Entity,
    pub Coordinate<InterfaceContext>,
);
pub struct SceneAccessChain(pub SceneHandle, pub Vec<SceneBinding>);
impl Coordinator {
    pub fn spawn_scene<S: Scene>(
        &mut self,
        anchor: Anchor,
        args: &S::Args<'_>,
        external_args: &SystemParamItem<S::ExternalArgs>,
        cmd: &mut Commands,
    ) -> SceneHandle {
        let this = cmd.spawn_empty().id();
        let handle = SceneHandle::default();
        let scene = S::bind_nodes(
            cmd,
            anchor,
            args,
            external_args,
            SceneBinder::new(self, this, handle),
        );
        // handle -> self.anchors
        // entity -> self.root_bindings
    }
    pub fn resolve_handle(&self, scene_access_chain: &SceneAccessChain) -> Option<SceneHandle> {
        let mut handle = scene_access_chain.0;
        for binding in scene_access_chain.1.iter() {
            if let Some(dep) = self.dependents.get(&handle) {
                if let Some(d) = dep.get(binding) {
                    handle = *d;
                }
            }
        }
        return if handle != scene_access_chain.0 {
            Some(handle)
        } else {
            None
        };
    }
    pub fn dependents(&self, handle: SceneHandle) -> Option<HashSet<SceneHandle>> {
        todo!()
    }
    pub fn entities(&self, handle: SceneHandle) -> Option<HashSet<Entity>> {
        todo!()
    }
}
pub struct SceneBinder<'a> {
    c: &'a mut Coordinator,
    this: Entity,
    handle: SceneHandle,
}
impl<'a> SceneBinder<'a> {
    pub(crate) fn new(c: &'a mut Coordinator, this: Entity, handle: SceneHandle) -> Self {
        Self { c, this, handle }
    }
    pub(crate) fn spawn_subscene<S: Scene>(
        &self,
        anchor: Anchor,
        args: &S::Args<'_>,
        external_args: &SystemParamItem<S::ExternalArgs>,
        cmd: &mut Commands,
    ) -> SceneHandle {
        todo!()
    }
    pub fn this(&self) -> Entity {
        todo!()
    }
    pub fn root(&self) -> SceneHandle {
        todo!()
    }
    pub fn bind<B: Bundle, SB: Into<SceneBinding>, SA: Into<SceneAlignment>>(
        &self,
        binding: SB,
        alignment: SA,
        b: B,
        cmd: &mut Commands,
    ) -> Entity {
        todo!()
    }
    pub fn bind_scene<S: Scene>(
        &self,
        binding: SceneBinding,
        alignment: SceneAlignment,
        area: Area<InterfaceContext>,
        args: &S::Args<'_>,
        external_args: &SystemParamItem<S::ExternalArgs>,
        cmd: &mut Commands,
    ) -> Entity {
        todo!()
    }
}
pub trait Scene
where
    Self: Bundle,
{
    type Args<'a>;
    type ExternalArgs: SystemParam;
    fn bind_nodes(
        cmd: &mut Commands,
        anchor: Anchor,
        args: &Self::Args<'_>,
        external_args: &SystemParamItem<Self::ExternalArgs>,
        binder: SceneBinder<'_>,
    ) -> Self;
}
#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone, Default, Component)]
pub struct SceneHandle(pub i32);