use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::{Coordinate, InterfaceContext};
use crate::differential::Despawn;
use crate::generator::HandleGenerator;
use crate::scene::align::SceneAlignment;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::change_detection::ResMut;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Commands, Component, DetectChanges, ParamSet, Query, Resource};
use bevy_ecs::query::Changed;
use bevy_ecs::system::{SystemParam, SystemParamItem};
use indexmap::IndexSet;
use std::collections::{HashMap, HashSet};

pub mod align;

#[derive(Copy, Clone, Default)]
pub struct Anchor(pub Coordinate<InterfaceContext>);
#[derive(Copy, Clone, Default, Hash, Eq, PartialEq, Debug)]
pub struct SceneBinding(pub i32);
impl From<i32> for SceneBinding {
    fn from(value: i32) -> Self {
        SceneBinding(value)
    }
}
#[derive(Resource, Default)]
pub struct SceneCoordinator {
    pub(crate) anchors: HashMap<SceneHandle, Anchor>,
    pub(crate) dependents: HashMap<SceneHandle, HashMap<SceneBinding, SceneHandle>>,
    pub(crate) dependent_bindings: HashMap<SceneHandle, HashMap<SceneBinding, Entity>>,
    pub(crate) root_bindings: HashMap<SceneHandle, Entity>,
    pub(crate) generator: HandleGenerator,
    pub(crate) alignments: HashMap<SceneHandle, HashMap<SceneBinding, SceneAlignment>>,
    changed: bool,
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
pub enum SceneTarget {
    Root,
    Binding(SceneBinding),
}
impl<SB: Into<SceneBinding>> From<SB> for SceneTarget {
    fn from(value: SB) -> Self {
        SceneTarget::Binding(value.into())
    }
}
pub struct SceneAccessChain(pub SceneHandle, pub Vec<SceneBinding>, pub SceneTarget);
impl SceneAccessChain {
    pub fn binding<SB: Into<SceneBinding>>(mut self, b: SB) -> Self {
        self.1.push(b.into());
        self
    }
    pub fn target<ST: Into<SceneTarget>>(mut self, scene_target: ST) -> Self {
        self.2 = scene_target.into();
        self
    }
}
impl SceneCoordinator {
    pub fn update_anchor_area<A: Into<Area<InterfaceContext>>>(
        &mut self,
        handle: SceneHandle,
        area: A,
    ) {
        let coordinate = self.anchor(handle).0.with_area(area);
        self.update_anchor(handle, coordinate);
    }
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
            .insert(anchor.0);
        self.update_anchor(handle, anchor.0);
        self.root_bindings.insert(handle, this);
        (handle, this)
    }
    pub(crate) fn resolve_non_scene(
        &self,
        handle: SceneHandle,
        coordinated: &mut Query<(
            &mut Position<InterfaceContext>,
            &Area<InterfaceContext>,
            &mut Layer,
        )>,
    ) {
        let anchor = self.anchor(handle);
        for (binding, entity) in self.dependent_bindings.get(&handle).unwrap().iter() {
            if self.dependents.get(&handle).unwrap().get(binding).is_none() {
                let alignment = *self.alignments.get(&handle).unwrap().get(binding).unwrap();
                let area = *coordinated.get(*entity).unwrap().1;
                let coordinate = Coordinate::default()
                    .with_position(alignment.pos.calc_pos(anchor, area))
                    .with_area(area)
                    .with_layer(alignment.layer.calc_layer(anchor.0.layer));
                *coordinated.get_mut(*entity).unwrap().0 = coordinate.section.position;
                *coordinated.get_mut(*entity).unwrap().2 = coordinate.layer;
            }
        }
    }
    pub fn binding_entity(&self, scene_access_chain: &SceneAccessChain) -> Entity {
        let (m_root, handle) = self.resolve_handle(scene_access_chain);
        return match scene_access_chain.2 {
            SceneTarget::Root => {
                if let Some(root) = m_root {
                    *self
                        .dependent_bindings
                        .get(&root)
                        .unwrap()
                        .get(scene_access_chain.1.last().unwrap())
                        .unwrap()
                } else {
                    *self.root_bindings.get(&handle).unwrap()
                }
            }
            SceneTarget::Binding(binding) => *self
                .dependent_bindings
                .get(&handle)
                .unwrap()
                .get(&binding)
                .unwrap(),
        };
    }
    pub fn despawn(&mut self, handle: SceneHandle) -> HashSet<Entity> {
        let mut removes = HashSet::new();
        if let Some(e) = self.root_bindings.remove(&handle) {
            removes.insert(e);
        }
        for (root, binding, dep) in self.dependents(handle) {
            self.alignments.remove(&root);
            if let Some(deps) = self.dependent_bindings.remove(&root) {
                for (_k, v) in deps {
                    removes.insert(v);
                }
            }
            self.alignments.remove(&dep);
            if let Some(deps) = self.dependent_bindings.remove(&dep) {
                for (_k, v) in deps {
                    removes.insert(v);
                }
            }
            self.dependents.remove(&root);
            self.dependents.remove(&dep);
        }
        removes
    }
    pub fn get_alignment_mut(
        &mut self,
        scene_access_chain: &SceneAccessChain,
    ) -> &mut SceneAlignment {
        let (m_root, handle) = self.resolve_handle(scene_access_chain);
        self.changed = true;
        return match scene_access_chain.2 {
            SceneTarget::Root => self
                .alignments
                .get_mut(&m_root.unwrap())
                .unwrap()
                .get_mut(scene_access_chain.1.last().unwrap())
                .unwrap(),
            SceneTarget::Binding(binding) => self
                .alignments
                .get_mut(&handle)
                .unwrap()
                .get_mut(&binding)
                .unwrap(),
        };
    }
    pub fn resolve_handle(
        &self,
        scene_access_chain: &SceneAccessChain,
    ) -> (Option<SceneHandle>, SceneHandle) {
        let mut handle = scene_access_chain.0;
        let mut root = None;
        for binding in scene_access_chain.1.iter() {
            if let Some(dep) = self.dependents.get(&handle) {
                if let Some(d) = dep.get(binding) {
                    root.replace(handle);
                    handle = *d;
                }
            }
        }
        (root, handle)
    }
    pub fn dependents(
        &self,
        handle: SceneHandle,
    ) -> IndexSet<(SceneHandle, SceneBinding, SceneHandle)> {
        let mut set = IndexSet::new();
        if let Some(deps) = self.dependents.get(&handle) {
            for (binding, dep_handle) in deps.iter() {
                set.insert((handle, *binding, *dep_handle));
                set.extend(self.dependents(*dep_handle))
            }
        }
        set
    }
    pub fn anchor(&self, scene_handle: SceneHandle) -> Anchor {
        *self.anchors.get(&scene_handle).unwrap()
    }
    pub fn update_anchor(
        &mut self,
        scene_handle: SceneHandle,
        coordinate: Coordinate<InterfaceContext>,
    ) {
        self.anchors.insert(scene_handle, Anchor(coordinate));
        self.changed = true;
    }
}
pub struct SceneBinder<'a> {
    coordinator_ref: &'a mut SceneCoordinator,
    this: Entity,
    root: SceneHandle,
}
impl<'a> SceneBinder<'a> {
    pub(crate) fn new(
        coordinator_ref: &'a mut SceneCoordinator,
        this: Entity,
        root: SceneHandle,
    ) -> Self {
        coordinator_ref.dependents.insert(root, HashMap::new());
        coordinator_ref
            .dependent_bindings
            .insert(root, HashMap::new());
        coordinator_ref.alignments.insert(root, HashMap::new());
        Self {
            coordinator_ref,
            this,
            root,
        }
    }
    pub fn this(&self) -> Entity {
        self.this
    }
    pub fn root(&self) -> SceneHandle {
        self.root
    }
    pub fn bind<B: Bundle, SB: Into<SceneBinding>, SA: Into<SceneAlignment>>(
        &mut self,
        binding: SB,
        alignment: SA,
        b: B,
        cmd: &mut Commands,
    ) -> Entity {
        let entity = cmd.spawn(b).id();
        let bind = binding.into();
        self.coordinator_ref
            .dependent_bindings
            .get_mut(&self.root)
            .unwrap()
            .insert(bind, entity);
        self.coordinator_ref
            .alignments
            .get_mut(&self.root)
            .unwrap()
            .insert(bind, alignment.into());
        entity
    }
    pub fn bind_scene<S: Scene>(
        &mut self,
        binding: SceneBinding,
        alignment: SceneAlignment,
        area: Area<InterfaceContext>,
        args: &S::Args<'_>,
        external_args: &SystemParamItem<S::ExternalArgs>,
        cmd: &mut Commands,
    ) -> (SceneHandle, Entity) {
        let entity = cmd.spawn_empty().id();
        let handle = SceneHandle::from(self.coordinator_ref.generator.generate());
        self.coordinator_ref
            .dependent_bindings
            .get_mut(&self.root)
            .unwrap()
            .insert(binding, entity);
        let anchor = Anchor(Coordinate::default().with_area(area));
        self.coordinator_ref.update_anchor(handle, anchor.0);
        self.coordinator_ref
            .alignments
            .get_mut(&self.root)
            .unwrap()
            .insert(binding, alignment);
        self.coordinator_ref
            .dependents
            .get_mut(&self.root)
            .unwrap()
            .insert(binding, handle);
        let scene = S::bind_nodes(
            cmd,
            anchor,
            args,
            external_args,
            SceneBinder::new(self.coordinator_ref, entity, handle),
        );
        cmd.entity(entity)
            .insert(scene)
            .insert(anchor.0)
            .insert(handle);
        (handle, entity)
    }
}
pub trait Scene
where
    Self: Bundle,
{
    type Bindings: Into<SceneBinding>;
    type Args<'a>: Send + Sync;
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
impl From<i32> for SceneHandle {
    fn from(value: i32) -> Self {
        SceneHandle(value)
    }
}
impl SceneHandle {
    pub fn access_chain(&self) -> SceneAccessChain {
        SceneAccessChain(*self, vec![], SceneTarget::Root)
    }
}
pub(crate) fn despawn_scenes(
    mut scenes: ParamSet<(
        Query<(&SceneHandle, &Despawn), Changed<Despawn>>,
        Query<&mut Despawn>,
    )>,
    mut coordinator: ResMut<SceneCoordinator>,
) {
    let mut removes = HashSet::new();
    for (handle, despawn) in scenes.p0().iter() {
        if despawn.should_despawn() {
            removes.insert(*handle);
        }
    }
    for handle in removes {
        for e in coordinator.despawn(handle) {
            if let Ok(mut d) = scenes.p1().get_mut(e) {
                *d = Despawn::signal_despawn();
            }
        }
    }
}
pub(crate) fn place_scenes(
    mut coordinator: ResMut<SceneCoordinator>,
    mut coordinated: Query<(
        &mut Position<InterfaceContext>,
        &Area<InterfaceContext>,
        &mut Layer,
    )>,
) {
    if coordinator.is_changed() && coordinator.changed {
        for root in coordinator
            .root_bindings
            .keys()
            .cloned()
            .collect::<Vec<SceneHandle>>()
        {
            coordinator.resolve_non_scene(root, &mut coordinated);
            let dependents = coordinator.dependents(root);
            for (dep_root, dep_binding, dep_handle) in dependents {
                let root_anchor = *coordinator.anchors.get(&dep_root).unwrap();
                let alignment = *coordinator
                    .alignments
                    .get(&dep_root)
                    .unwrap()
                    .get(&dep_binding)
                    .unwrap();
                let entity = *coordinator
                    .dependent_bindings
                    .get(&dep_root)
                    .unwrap()
                    .get(&dep_binding)
                    .unwrap();
                let area = *coordinated.get(entity).unwrap().1;
                let anchor = Anchor(
                    Coordinate::default()
                        .with_position(alignment.pos.calc_pos(root_anchor, area))
                        .with_area(area)
                        .with_layer(alignment.layer.calc_layer(root_anchor.0.layer)),
                );
                coordinator.anchors.insert(dep_handle, anchor);
                *coordinated.get_mut(entity).unwrap().0 = anchor.0.section.position;
                *coordinated.get_mut(entity).unwrap().2 = anchor.0.layer;
                coordinator.resolve_non_scene(dep_handle, &mut coordinated);
            }
        }
        coordinator.changed = false;
    }
}
