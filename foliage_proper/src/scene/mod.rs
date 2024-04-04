pub mod micro_grid;

use crate::conditional::{
    Branch, ConditionHandle, Conditional, ConditionalCommand, SceneBranch, SpawnTarget,
};
use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::{Coordinate, InterfaceContext, PositionAdjust};
use crate::differential::Despawn;
use crate::elm::config::{CoreSet, ExternalSet};
use crate::elm::leaf::{EmptySetDescriptor, Leaf, Tag};
use crate::elm::{Disabled, Elm};
use crate::scene::micro_grid::MicroGrid;
use crate::view::BranchPool;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::prelude::{Commands, Component, Entity, IntoSystemConfigs, Query};
use bevy_ecs::query::{Changed, Or, ReadOnlyWorldQuery, With, Without};
use bevy_ecs::system::{Command, ParamSet, StaticSystemParam, SystemParam, SystemParamItem};
use micro_grid::MicroGridAlignment;
use std::collections::{HashMap, HashSet};

#[derive(Component, Copy, Clone, Default)]
pub struct Anchor(Coordinate<InterfaceContext>);

impl Anchor {
    pub(crate) fn aligned(&self, grid: &MicroGrid, alignment: &MicroGridAlignment) -> Option<Self> {
        if let Some(a) = grid.determine(self.0, alignment) {
            return Some(Anchor(a));
        }
        None
    }
}
#[derive(Debug, Clone)]
pub struct SceneHandle {
    root: Entity,
    bindings: Bindings,
    branches: Option<BranchPool>,
}

impl SceneHandle {
    pub(crate) fn new(root: Entity, bindings: Bindings, branches: Option<BranchPool>) -> Self {
        Self {
            root,
            bindings,
            branches,
        }
    }
    pub fn root(&self) -> Entity {
        self.root
    }
    pub fn bindings(&self) -> &Bindings {
        &self.bindings
    }
    pub fn branches(&self) -> Option<&BranchPool> {
        self.branches.as_ref()
    }
}
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct SceneBinding(pub i32);
impl From<i32> for SceneBinding {
    fn from(value: i32) -> Self {
        Self(value)
    }
}
#[derive(Clone, Debug)]
pub struct SceneNode {
    entity: Entity,
    bindings: Option<Bindings>,
    branch: Option<ConditionHandle>,
}
impl SceneNode {
    fn new(entity: Entity, bindings: Option<Bindings>, branch: Option<ConditionHandle>) -> Self {
        Self {
            entity,
            bindings,
            branch,
        }
    }
    pub fn entity(&self) -> Entity {
        self.entity
    }
    pub fn bindings(&self) -> Option<&Bindings> {
        self.bindings.as_ref()
    }
    pub fn branch(&self) -> Option<ConditionHandle> {
        self.branch
    }
}
pub struct Binder<'a, 'w, 's> {
    nodes: HashMap<SceneBinding, SceneNode>,
    branches: BranchPool,
    root: Entity,
    cmd: &'a mut Commands<'w, 's>,
}

impl<'a, 'w, 's> Binder<'a, 'w, 's> {
    pub fn finish<S: Scene>(self, comps: SceneComponents<S>) -> SceneHandle {
        let entity = self.root();
        tracing::trace!("finishing binder:{:?}", entity);
        let bindings = Bindings(self.nodes);
        self.cmd
            .entity(entity)
            .insert(comps)
            .insert(bindings.clone());
        SceneHandle::new(
            entity,
            bindings,
            if self.branches.is_empty() {
                None
            } else {
                Some(self.branches)
            },
        )
    }
    pub fn branches(&self) -> &BranchPool {
        &self.branches
    }
    pub fn new(cmd: &'a mut Commands<'w, 's>, root: Option<Entity>) -> Self {
        Self {
            nodes: HashMap::new(),
            branches: Vec::new(),
            root: root.unwrap_or(cmd.spawn_empty().id()),
            cmd,
        }
    }
    pub fn binding<SB: Into<SceneBinding>>(&self, sb: SB) -> &SceneNode {
        self.nodes.get(&sb.into()).unwrap()
    }
    pub fn root(&self) -> Entity {
        self.root
    }
    pub fn bind<SB: Into<SceneBinding>, SA: Into<MicroGridAlignment>, B: Bundle>(
        &mut self,
        sb: SB,
        sa: SA,
        b: B,
    ) -> Entity {
        // add alignment stuff
        let entity = self
            .cmd
            .spawn(b)
            .insert(SceneBindingComponents::new(
                self.root,
                Anchor::default(),
                sa.into(),
            ))
            .id();
        tracing::trace!("binding:{:?}", entity);
        self.nodes
            .insert(sb.into(), SceneNode::new(entity, None, None));
        entity
    }
    pub fn bind_scene<S: Scene, SB: Into<SceneBinding>, SA: Into<MicroGridAlignment>>(
        &mut self,
        sb: SB,
        sa: SA,
        s: S,
    ) -> SceneHandle {
        // add alignment + scene stuff
        let scene_desc = s.create(Binder::new(self.cmd, None));
        self.cmd
            .entity(scene_desc.root())
            .insert(SceneBindingComponents::new(
                self.root,
                Anchor::default(),
                sa.into(),
            ));
        tracing::trace!("binding-scene:{:?}", scene_desc.root());
        self.nodes.insert(
            sb.into(),
            SceneNode::new(scene_desc.root(), Some(scene_desc.bindings.clone()), None),
        );
        scene_desc
    }
    pub fn bind_conditional<
        SB: Into<SceneBinding>,
        SA: Into<MicroGridAlignment>,
        C: Clone + Send + Sync + 'static,
    >(
        &mut self,
        sb: SB,
        sa: SA,
        b: C,
    ) -> ConditionHandle {
        let pre_spawned = self.cmd.spawn_empty().id();
        let main = self
            .cmd
            .spawn(Branch::new(b, SpawnTarget::This(pre_spawned), false))
            .insert(Conditional::new(
                SceneBindingComponents::new(self.root, Anchor::default(), sa.into()),
                SpawnTarget::This(pre_spawned),
                false,
            ))
            .id();
        let handle = ConditionHandle::new(main, pre_spawned);
        tracing::trace!("binding-conditional:{:?}", handle);
        self.nodes
            .insert(sb.into(), SceneNode::new(pre_spawned, None, Some(handle)));
        self.branches.push(handle);
        handle
    }
    pub fn bind_conditional_scene<
        S: Scene + Clone,
        SA: Into<MicroGridAlignment>,
        SB: Into<SceneBinding>,
    >(
        &mut self,
        sb: SB,
        sa: SA,
        s: S,
    ) -> ConditionHandle {
        let pre_spawned = self.cmd.spawn_empty().id();
        let main = self
            .cmd
            .spawn(SceneBranch::new(s, SpawnTarget::This(pre_spawned), false))
            .insert(Conditional::new(
                SceneBindingComponents::new(self.root, Anchor::default(), sa.into()),
                SpawnTarget::This(pre_spawned),
                false,
            ))
            .id();
        let handle = ConditionHandle::new(main, pre_spawned);
        tracing::trace!("binding-conditional-scene:{:?}", handle);
        self.nodes
            .insert(sb.into(), SceneNode::new(pre_spawned, None, Some(handle)));
        self.branches.push(handle);
        handle
    }
    pub fn extend<Ext: Bundle>(&mut self, entity: Entity, ext: Ext) {
        tracing::trace!("extending:{:?}", entity);
        self.cmd.entity(entity).insert(ext);
    }
    pub fn add_command_to<C: Command + Clone + Sync>(&mut self, entity: Entity, comm: C) {
        tracing::trace!("adding command to:{:?}", entity);
        self.cmd.entity(entity).insert(ConditionalCommand(comm));
    }

    pub fn extend_conditional<Ext: Bundle + Clone>(
        &mut self,
        condition_handle: ConditionHandle,
        ext: Ext,
    ) {
        tracing::trace!("extending :{:?} w/ conditional", condition_handle);
        self.cmd
            .entity(condition_handle.this())
            .insert(Conditional::new(
                ext,
                SpawnTarget::This(condition_handle.target()),
                false,
            ));
    }
}
#[derive(Component, Default, Clone, Debug)]
pub struct Bindings(HashMap<SceneBinding, SceneNode>);
impl Bindings {
    pub fn get<SB: Into<SceneBinding>>(&self, sb: SB) -> Entity {
        self.0.get(&sb.into()).expect("no-scene-binding").entity
    }
    pub fn node<SB: Into<SceneBinding>>(&self, sb: SB) -> Option<&SceneNode> {
        self.0.get(&sb.into())
    }
    pub fn nodes(&self) -> &HashMap<SceneBinding, SceneNode> {
        &self.0
    }
}
#[derive(Component, Copy, Clone)]
pub struct IsScene;
#[derive(Component, Copy, Clone)]
pub struct IsDep;
#[derive(Bundle)]
pub struct SceneComponents<S: Scene> {
    t: S::Components,
    bindings: Bindings,
    coordinate: Coordinate<InterfaceContext>,
    despawn: Despawn,
    disabled: Disabled,
    tag: Tag<S>,
    scene_tag: Tag<IsScene>,
    grid: MicroGrid,
}
impl<S: Scene> SceneComponents<S> {
    pub fn new(grid: MicroGrid, t: S::Components) -> Self {
        Self {
            t,
            bindings: Bindings::default(),
            coordinate: Coordinate::default(),
            despawn: Default::default(),
            disabled: Default::default(),
            tag: Tag::new(),
            scene_tag: Tag::new(),
            grid,
        }
    }
}
#[derive(Bundle, Clone)]
pub(crate) struct SceneBindingComponents {
    tag: Tag<IsDep>,
    anchor: Anchor,
    alignment: MicroGridAlignment,
    ptr: ScenePtr,
    despawn: Despawn,
}
impl SceneBindingComponents {
    fn new(ptr: Entity, anchor: Anchor, alignment: MicroGridAlignment) -> Self {
        Self {
            tag: Tag::new(),
            anchor,
            alignment,
            ptr: ScenePtr(ptr),
            despawn: Despawn::default(),
        }
    }
}
// will need to add this for every scene added
pub fn config<S: Scene + Send + Sync + 'static>(
    query: Query<(Entity, &Despawn, &Bindings), (With<Tag<S>>, Or<(S::Filter,)>)>,
    mut ext: StaticSystemParam<S::Params>,
) where
    <S as Scene>::Filter: ReadOnlyWorldQuery,
{
    for (entity, despawn, bindings) in query.iter() {
        if despawn.is_despawned() {
            continue;
        }
        tracing::trace!("scene::config for:{:?}", entity);
        // disabled?
        S::config(entity, &mut ext, bindings);
    }
}
pub trait Scene
where
    Self: Sized + Send + Sync + 'static,
{
    type Params: SystemParam + 'static;
    type Filter;
    type Components: Bundle;
    fn config(entity: Entity, ext: &mut SystemParamItem<Self::Params>, bindings: &Bindings);
    fn create(self, binder: Binder) -> SceneHandle;
}
#[derive(Component, Copy, Clone)]
pub struct ScenePtr(Entity);
impl ScenePtr {
    pub fn value(self) -> Entity {
        self.0
    }
}
#[derive(Bundle, Copy, Clone, Default)]
pub struct BlankNode {
    coordinate: Coordinate<InterfaceContext>,
    despawn: Despawn,
    disabled: Disabled,
}
impl Scene for BlankNode {
    type Params = ();
    type Filter = ();
    type Components = ();

    fn config(_entity: Entity, _ext: &mut SystemParamItem<Self::Params>, _bindings: &Bindings) {}

    fn create(self, binder: Binder) -> SceneHandle {
        binder.finish::<Self>(SceneComponents::new(MicroGrid::new(), ()))
    }
}
fn recursive_fetch(
    root_coordinate: Coordinate<InterfaceContext>,
    target_entity: Entity,
    query: &Query<
        (
            &Anchor,
            &MicroGridAlignment,
            Option<&Bindings>,
            &ScenePtr,
            Option<&PositionAdjust>,
        ),
        With<Tag<IsDep>>,
    >,
    grids: &Query<&MicroGrid>,
) -> Vec<(Entity, Anchor)> {
    let mut fetch = vec![];
    if let Ok(res) = query.get(target_entity) {
        if let Some(bindings) = res.2 {
            for (_, bind) in bindings.0.iter() {
                if let Ok(dep) = query.get(bind.entity) {
                    tracing::trace!(
                        "aligning dependent:{:?} to root-coordinate:{:?}",
                        bind.entity,
                        root_coordinate
                    );
                    let alignment = dep.1;
                    let ptr = *dep.3;
                    let grid = grids.get(ptr.0).expect("scene-grid");
                    if let Some(anchor) = Anchor(root_coordinate).aligned(grid, alignment) {
                        let anchor = Anchor(anchor.0.with_position(
                            anchor.0.section.position + dep.4.cloned().unwrap_or_default().0,
                        ));
                        tracing::trace!("adjusted-anchor:{:?}", anchor.0);
                        fetch.push((bind.entity, anchor));
                        if query.get(bind.entity).unwrap().2.is_some() {
                            let others = recursive_fetch(anchor.0, bind.entity, query, grids);
                            fetch.extend(others);
                        }
                    }
                }
            }
        }
    }
    fetch
}
pub(crate) fn resolve_anchor(
    roots: Query<
        (
            &Position<InterfaceContext>,
            &Area<InterfaceContext>,
            &Layer,
            &Bindings,
        ),
        (With<Tag<IsScene>>, Without<Tag<IsDep>>),
    >,
    mut deps: ParamSet<(
        Query<
            (
                &Anchor,
                &MicroGridAlignment,
                Option<&Bindings>,
                &ScenePtr,
                Option<&PositionAdjust>,
            ),
            With<Tag<IsDep>>,
        >,
        Query<&mut Anchor, With<Tag<IsDep>>>,
    )>,
    grids: Query<&MicroGrid>,
) {
    for (pos, area, layer, bindings) in roots.iter() {
        let root_coordinate = Coordinate::new((*pos, *area), *layer);
        for (_, bind) in bindings.0.iter() {
            if let Ok(dep) = deps.p0().get(bind.entity) {
                tracing::trace!(
                    "aligning dependent:{:?} to root-coordinate:{:?}",
                    bind.entity,
                    root_coordinate
                );
                let ptr = dep.3 .0;
                let adjust = dep.4.cloned().unwrap_or_default().0;
                let grid = grids.get(ptr).expect("scene-grid");
                if let Some(aligned_anchor) =
                    Anchor(root_coordinate).aligned(grid, deps.p0().get(bind.entity).unwrap().1)
                {
                    let aligned_anchor = Anchor(
                        aligned_anchor
                            .0
                            .with_position(aligned_anchor.0.section.position + adjust),
                    );
                    tracing::trace!("aligned-anchor:{:?}", aligned_anchor.0);
                    *deps.p1().get_mut(bind.entity).unwrap() = aligned_anchor;
                    if deps.p0().get(bind.entity).unwrap().2.is_some() {
                        let rf = recursive_fetch(aligned_anchor.0, bind.entity, &deps.p0(), &grids);
                        for (e, a) in rf {
                            *deps.p1().get_mut(e).unwrap() = a;
                        }
                    }
                }
            }
        }
    }
}
pub(crate) fn update_from_anchor(
    mut anchors: Query<
        (
            &Anchor,
            &mut Position<InterfaceContext>,
            &mut Area<InterfaceContext>,
            &mut Layer,
        ),
        Changed<Anchor>,
    >,
) {
    for (anchor, mut pos, mut area, mut layer) in anchors.iter_mut() {
        *pos = anchor.0.section.position;
        *area = anchor.0.section.area;
        *layer = anchor.0.layer;
    }
}
pub(crate) fn recursive_bindings(
    root: Entity,
    query: &Query<
        (Option<&Bindings>, &Despawn, &Disabled),
        Or<(With<Tag<IsScene>>, With<Tag<IsDep>>)>,
    >,
) -> HashSet<Entity> {
    let mut dependents = HashSet::new();
    if let Ok(res) = query.get(root) {
        if let Some(binds) = res.0 {
            for b in binds.0.iter() {
                dependents.insert(b.1.entity);
                dependents.extend(recursive_bindings(b.1.entity, query));
            }
        }
    }
    dependents
}
pub(crate) fn despawn_bindings(
    mut changed: ParamSet<(
        Query<
            (Option<&Bindings>, &Despawn, &Disabled),
            Or<(
                With<Tag<IsScene>>,
                With<Tag<IsDep>>,
                Or<(Changed<Despawn>, Changed<Disabled>)>,
            )>,
        >,
        Query<(&mut Despawn, &mut Disabled)>,
        Query<(Option<&Bindings>, &Despawn, &Disabled), Or<(With<Tag<IsScene>>, With<Tag<IsDep>>)>>,
    )>,
) {
    let mut to_despawn = HashSet::new();
    let mut to_disable = HashSet::new();
    let mut to_enable = HashSet::new();
    for (bindings, despawn, disable) in changed.p0().iter() {
        if despawn.is_despawned() {
            if let Some(binds) = bindings {
                for b in binds.0.iter() {
                    to_despawn.insert(b.1.entity);
                }
            }
        } else {
            if disable.is_disabled() {
                if let Some(binds) = bindings {
                    for b in binds.0.iter() {
                        to_disable.insert(b.1.entity);
                    }
                }
            } else if let Some(binds) = bindings {
                for b in binds.0.iter() {
                    to_enable.insert(b.1.entity);
                }
            }
        }
    }
    for e in to_despawn.clone() {
        let entities = recursive_bindings(e, &changed.p2());
        to_despawn.extend(entities);
    }
    for e in to_disable.clone() {
        let entities = recursive_bindings(e, &changed.p2());
        to_disable.extend(entities);
    }
    for e in to_enable.clone() {
        let entities = recursive_bindings(e, &changed.p2());
        to_enable.extend(entities);
    }
    for e in to_despawn {
        if let Ok(mut d) = changed.p1().get_mut(e) {
            d.0.despawn();
        }
    }
    for e in to_disable {
        if let Ok(mut d) = changed.p1().get_mut(e) {
            d.1.disable();
        }
    }
    for e in to_enable {
        if let Ok(mut d) = changed.p1().get_mut(e) {
            d.1.enable();
        }
    }
}

pub enum ExtendTarget {
    This,
    Binding(SceneBinding),
}
impl Leaf for SceneHandle {
    type SetDescriptor = EmptySetDescriptor;

    fn attach(elm: &mut Elm) {
        elm.main().add_systems((
            despawn_bindings.in_set(ExternalSet::ConditionalExt),
            (resolve_anchor, update_from_anchor)
                .chain()
                .in_set(CoreSet::SceneCoordinate),
        ));
    }
}