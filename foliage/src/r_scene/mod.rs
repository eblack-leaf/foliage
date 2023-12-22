use crate::coordinate::area::Area;
use crate::coordinate::{Coordinate, InterfaceContext};
use crate::elm::leaf::Tag;
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
pub struct IsScene();
#[derive(Resource)]
pub struct SceneCoordinator {
    pub(crate) anchors: HashMap<SceneHandle, Anchor>,
    pub(crate) dependents: HashMap<SceneHandle, HashMap<SceneBinding, SceneHandle>>,
    pub(crate) dependent_bindings: HashMap<SceneHandle, HashMap<SceneBinding, Entity>>,
    pub(crate) root_bindings: HashMap<SceneHandle, Entity>,
    pub(crate) generator: HandleGenerator,
    pub(crate) alignments: HashMap<SceneHandle, HashMap<SceneBinding, SceneAlignment>>,
}
pub struct BindingCoordinate {
    pub handle: SceneHandle,
    pub binding: SceneBinding,
    pub entity: Entity,
    pub coordinate: Coordinate<InterfaceContext>,
}
impl BindingCoordinate {
    pub fn new(
        handle: SceneHandle,
        binding: SceneBinding,
        entity: Entity,
        coordinate: Coordinate<InterfaceContext>,
    ) -> Self {
        Self {
            handle,
            binding,
            entity,
            coordinate,
        }
    }
}
pub struct SceneAccessChain(pub SceneHandle, pub Vec<SceneBinding>);
impl SceneAccessChain {
    pub fn binding<SB: Into<SceneBinding>>(mut self, b: SB) -> Self {
        self.1.push(b.into());
        self
    }
}
impl SceneCoordinator {
    pub fn spawn_scene<S: Scene>(
        &mut self,
        anchor: Anchor,
        args: &S::Args<'_>,
        external_args: &SystemParamItem<S::ExternalArgs>,
        cmd: &mut Commands,
    ) -> (SceneHandle, Entity) {
        let this = cmd.spawn_empty().id();
        let handle = SceneHandle(self.generator.generate());
        let scene = S::bind_nodes(
            cmd,
            anchor,
            args,
            external_args,
            SceneBinder::new(self, this, handle),
        );
        cmd.entity(this)
            .insert(scene)
            .insert(handle)
            .insert(Tag::<IsScene>::new());
        self.anchors.insert(handle, anchor);
        self.root_bindings.insert(handle, this);
        (handle, this)
    }
    pub fn binding_entity(&self, scene_access_chain: &SceneAccessChain) -> Entity {
        todo!()
    }
    pub fn update_alignment(
        &mut self,
        scene_access_chain: &SceneAccessChain,
    ) -> &mut SceneAlignment {
        todo!()
    }
    pub fn resolve_handle(&self, scene_access_chain: &SceneAccessChain) -> SceneHandle {
        let mut handle = scene_access_chain.0;
        for binding in scene_access_chain.1.iter() {
            if let Some(dep) = self.dependents.get(&handle) {
                if let Some(d) = dep.get(binding) {
                    handle = *d;
                }
            }
        }
        handle
    }
    pub fn dependents(&self, handle: SceneHandle) -> Option<HashSet<SceneHandle>> {
        todo!()
    }
    pub fn entities(&self, handle: SceneHandle) -> Option<HashSet<Entity>> {
        todo!()
    }
    pub fn binding_coordinate(&self, scene_access_chain: &SceneAccessChain) -> BindingCoordinate {
        todo!()
    }
    pub fn anchor(&self, scene_handle: SceneHandle) -> Anchor {
        *self.anchors.get(&scene_handle).unwrap()
    }
    pub fn update_anchor(
        &mut self,
        scene_handle: SceneHandle,
        coordinate: Coordinate<InterfaceContext>,
    ) -> Vec<BindingCoordinate> {
        todo!()
    }
}
pub struct SceneBinder<'a> {
    c: &'a mut SceneCoordinator,
    this: Entity,
    handle: SceneHandle,
}
impl<'a> SceneBinder<'a> {
    pub(crate) fn new(c: &'a mut SceneCoordinator, this: Entity, handle: SceneHandle) -> Self {
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
impl SceneHandle {
    pub fn access_chain(&self) -> SceneAccessChain {
        SceneAccessChain(*self, vec![])
    }
}
