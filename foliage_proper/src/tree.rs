use crate::animate::trigger::Trigger;
use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::InterfaceContext;
use crate::differential::Despawn;
use crate::elm::config::CoreSet;
use crate::elm::leaf::{EmptySetDescriptor, Leaf};
use crate::elm::{Disabled, Elm};
use crate::ginkgo::viewport::ViewportHandle;
use crate::layout::Layout;
use crate::scene::{
    Binder, Bindings, ExtendTarget, Scene, SceneBinding, SceneComponents, SceneDesc,
};
use crate::segment::{MacroGrid, ResponsiveSegment};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Bundle, Component, IntoSystemConfigs};
use bevy_ecs::query::Changed;
use bevy_ecs::system::{
    Commands, Query, Res, ResMut, Resource, StaticSystemParam, SystemParam, SystemParamItem,
};
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

#[derive(Component, Copy, Clone)]
pub struct Navigation<SEED>(PhantomData<SEED>);
impl<SEED> Navigation<SEED> {
    pub fn new() -> Self {
        Self { 0: PhantomData }
    }
}
pub(crate) fn sprout<SEED: Seed + Send + Sync + 'static>(
    mut forest: ResMut<Forest>,
    navigation: Query<(Entity, &Navigation<SEED>), Changed<Navigation<SEED>>>,
    mut cmd: Commands,
    mut ext: StaticSystemParam<SEED::Resources>,
    mut grid: ResMut<MacroGrid>,
) {
    if let Some((_, n)) = navigation.iter().last() {
        // TODO despawn current tree + all in pool
        // or anim-out && @-end trigger despawn
        *grid = SEED::GRID;
        let tree = SEED::plant(&mut cmd, &mut ext);
        forest.current.replace(tree);
    }
    for (e, _) in navigation.iter() {
        cmd.entity(e).despawn();
    }
}
pub trait Seed {
    const GRID: MacroGrid;
    type Resources: SystemParam + 'static;
    fn plant(cmd: &mut Commands, res: &mut SystemParamItem<Self::Resources>) -> Tree;
}
#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct BranchHandle(pub i32);
impl From<i32> for BranchHandle {
    fn from(value: i32) -> Self {
        Self(value)
    }
}
#[derive(Component)]
pub struct Conditional<T: Clone> {
    wrapped: T,
    target: BranchExtendTarget,
    is_extension: bool,
}
impl<T: Clone> Conditional<T> {
    pub fn new(e: BranchExtendTarget, t: T, is_extension: bool) -> Self {
        Self {
            wrapped: t,
            target: e,
            is_extension,
        }
    }
}
#[derive(Component)]
pub struct ConditionalScene<S: Scene + Clone> {
    wrapped: S,
    target: BranchExtendTarget,
    is_extension: bool,
}
impl<S: Scene + Clone> ConditionalScene<S> {
    pub fn new(e: BranchExtendTarget, t: S, is_extension: bool) -> Self {
        Self {
            wrapped: t,
            target: e,
            is_extension,
        }
    }
}
pub(crate) fn conditional_spawn<C: Bundle + Clone + Send + Sync + 'static>(
    query: Query<(&Trigger, &Conditional<C>), Changed<Trigger>>,
    mut cmd: Commands,
) {
    for (trigger, cond) in query.iter() {
        if cond.is_extension {
            continue;
        }
        if trigger.is_active() {
            match cond.target {
                BranchExtendTarget::This(entity) => {
                    cmd.entity(entity).insert(cond.wrapped.clone());
                }
                BranchExtendTarget::BindingOf(_, _) => {}
            }
        } else if trigger.is_inverse() {
            match cond.target {
                BranchExtendTarget::This(entity) => {
                    cmd.entity(entity).remove::<C>();
                }
                BranchExtendTarget::BindingOf(_, _) => {}
            }
        }
    }
}
pub(crate) fn conditional_scene_spawn<CS: Scene + Clone>(
    query: Query<(&Trigger, &ConditionalScene<CS>), Changed<Trigger>>,
    bindings: Query<&Bindings>,
    mut cmd: Commands,
) {
    for (trigger, cond) in query.iter() {
        if cond.is_extension {
            panic!("scenes-are-not allowed as extensions")
        }
        if trigger.is_active() {
            match cond.target {
                BranchExtendTarget::This(entity) => {
                    let _scene_desc = cond
                        .wrapped
                        .clone()
                        .create(Binder::new(&mut cmd, Some(entity)));
                }
                BranchExtendTarget::BindingOf(_, _) => {}
            }
        } else if trigger.is_inverse() {
            match cond.target {
                BranchExtendTarget::This(entity) => {
                    // also despawn bindings completely which will trigger subscenes
                    if let Ok(binds) = bindings.get(entity) {
                        for (_, b) in binds.nodes().iter() {
                            cmd.entity(b.entity()).insert(Despawn::signal_despawn());
                        }
                    }
                    cmd.entity(entity).remove::<SceneComponents<CS>>();
                }
                BranchExtendTarget::BindingOf(_, _) => {}
            }
        }
    }
}
pub(crate) fn conditional_extension<C: Bundle + Clone + Send + Sync + 'static>(
    query: Query<(&Trigger, &Conditional<C>), Changed<Trigger>>,
    mut cmd: Commands,
    bindings: Query<&Bindings>,
) {
    for (trigger, cond) in query.iter() {
        if !cond.is_extension {
            continue;
        }
        if trigger.is_active() {
            match cond.target {
                BranchExtendTarget::This(entity) => {
                    cmd.entity(entity).insert(cond.wrapped.clone());
                }
                BranchExtendTarget::BindingOf(parent, bind) => {
                    cmd.entity(bindings.get(parent).unwrap().get(bind))
                        .insert(cond.wrapped.clone());
                }
            }
        } else if trigger.is_inverse() {
            match cond.target {
                BranchExtendTarget::This(entity) => {
                    cmd.entity(entity).remove::<C>();
                }
                BranchExtendTarget::BindingOf(parent, bind) => {
                    cmd.entity(bindings.get(parent).unwrap().get(bind))
                        .remove::<C>();
                }
            }
        }
    }
}
#[derive(Bundle)]
pub struct Branch<T: Clone + Send + Sync + 'static> {
    conditional: Conditional<T>,
    trigger: Trigger,
}
impl<T: Clone + Send + Sync + 'static> Branch<T> {
    pub fn new(t: T, e: BranchExtendTarget, is_extension: bool) -> Self {
        Self {
            conditional: Conditional::<T>::new(e, t, is_extension),
            trigger: Trigger::default(),
        }
    }
}
#[derive(Bundle)]
pub struct SceneBranch<T: Clone + Scene + Send + Sync + 'static> {
    conditional: ConditionalScene<T>,
    trigger: Trigger,
}
impl<S: Scene + Clone> SceneBranch<S> {
    pub fn new(t: S, e: BranchExtendTarget, is_extension: bool) -> Self {
        Self {
            conditional: ConditionalScene::<S>::new(e, t, is_extension),
            trigger: Trigger::default(),
        }
    }
}
#[derive(Component, Clone, Default)]
pub struct Tree(pub HashSet<Entity>, HashMap<BranchHandle, Entity>);
pub struct TreeBinder<'a, 'w, 's> {
    cmd: &'a mut Commands<'w, 's>,
    tree: Tree,
}
impl<'a, 'w, 's> TreeBinder<'a, 'w, 's> {
    pub fn new(cmd: &'a mut Commands<'w, 's>) -> Self {
        Self {
            cmd,
            tree: Tree::default(),
        }
    }
    pub fn tree(self) -> Tree {
        self.tree
    }
    pub fn responsive_scene<S: Scene>(&mut self, s: S, rs: ResponsiveSegment) -> SceneDesc {
        let desc = {
            let scene_desc = s.create(Binder::new(self.cmd, None));
            self.cmd.entity(scene_desc.root()).insert(rs);
            scene_desc
        };
        self.tree.0.insert(desc.root());
        desc
    }
    pub fn responsive<B: Bundle>(&mut self, b: B, rs: ResponsiveSegment) -> Entity {
        let ent = { self.cmd.spawn(b).insert(rs).id() };
        self.tree.0.insert(ent);
        ent
    }
    pub fn branch<BR: Clone + Send + Sync + 'static, BH: Into<BranchHandle>>(
        &mut self,
        bh: BH,
        br: BR,
        rs: ResponsiveSegment,
    ) -> BranchDesc {
        let desc = {
            let pre_spawned = self.cmd.spawn_empty().id();
            let branch_id = self
                .cmd
                .spawn(Branch::new(
                    br,
                    BranchExtendTarget::This(pre_spawned),
                    false,
                ))
                .insert(Conditional::new(
                    BranchExtendTarget::This(pre_spawned),
                    rs,
                    false,
                ))
                .id();
            BranchDesc::new(branch_id, pre_spawned)
        };
        self.tree.1.insert(bh.into(), desc.branch_entity);
        desc
    }
    pub fn branch_scene<S: Scene + Clone, BH: Into<BranchHandle>>(
        &mut self,
        bh: BH,
        s: S,
        rs: ResponsiveSegment,
    ) -> BranchDesc {
        let desc = {
            let pre_spawned = self.cmd.spawn_empty().id();
            let branch_id = self
                .cmd
                .spawn(SceneBranch::new(
                    s,
                    BranchExtendTarget::This(pre_spawned),
                    false,
                ))
                .insert(Conditional::new(
                    BranchExtendTarget::This(pre_spawned),
                    rs,
                    false,
                ))
                .id();
            BranchDesc::new(branch_id, pre_spawned)
        };
        self.tree.1.insert(bh.into(), desc.branch_entity);
        desc
    }
    pub fn extend<Ext: Bundle>(&mut self, entity: Entity, ext: Ext) {
        self.cmd.entity(entity).insert(ext);
    }
    pub fn conditional_extend<Ext: Bundle + Clone>(
        &mut self,
        branch_desc: BranchDesc,
        extend_target: ExtendTarget,
        ext: Ext,
    ) {
        match extend_target {
            ExtendTarget::This => {
                self.cmd
                    .entity(branch_desc.branch_entity)
                    .insert(Conditional::<Ext>::new(
                        BranchExtendTarget::This(branch_desc.pre_spawned),
                        ext,
                        true,
                    ));
            }
            ExtendTarget::Binding(bind) => {
                self.cmd
                    .entity(branch_desc.branch_entity)
                    .insert(Conditional::<Ext>::new(
                        BranchExtendTarget::BindingOf(branch_desc.pre_spawned, bind),
                        ext,
                        true,
                    ));
            }
        }
    }
}
#[derive(Default, Resource)]
pub struct Forest {
    current: Option<Tree>,
}
impl Forest {
    pub fn navigate<N: Seed + Send + Sync + 'static>(cmd: &mut Commands) {
        cmd.spawn(Navigation::<N>::new());
    }
}
// Uses current-tree and sets condition for that branch using tree.branches
#[derive(Component, Copy, Clone)]
pub struct BranchSet(pub BranchHandle, pub bool);
fn set_branch(query: Query<(Entity, &BranchSet)>, mut cmd: Commands, forest: Res<Forest>) {
    if forest.current.is_some() {
        for (entity, branch_request) in query.iter() {
            let trigger = if branch_request.1 {
                Trigger::inverse()
            } else {
                Trigger::active()
            };
            cmd.entity(
                *forest
                    .current
                    .as_ref()
                    .unwrap()
                    .1
                    .get(&branch_request.0)
                    .expect("invalid-branch-request"),
            )
            .insert(trigger);
            cmd.entity(entity).despawn();
        }
    }
}
pub enum BranchExtendTarget {
    This(Entity),
    BindingOf(Entity, SceneBinding),
}
pub struct BranchDesc {
    branch_entity: Entity,
    pre_spawned: Entity,
}
impl BranchDesc {
    fn new(branch_entity: Entity, pre_spawned: Entity) -> Self {
        Self {
            branch_entity,
            pre_spawned,
        }
    }
}
// TODO Derived-Value handler + other
// pub struct OnEnter<T> {}
fn viewport_changed(
    mut query: Query<(
        &ResponsiveSegment,
        &mut Position<InterfaceContext>,
        &mut Area<InterfaceContext>,
        &mut Layer,
        &mut Disabled,
    )>,
    viewport_handle: Res<ViewportHandle>,
    grid: Res<MacroGrid>,
    mut layout: ResMut<Layout>,
) {
    if viewport_handle.area_updated() {
        *layout = Layout::from_area(viewport_handle.section().area);
        for (res_seg, mut pos, mut area, mut layer, mut disabled) in query.iter_mut() {
            // calc
            if let Some(coord) = res_seg.coordinate(*layout, viewport_handle.section(), &grid) {
                *pos = coord.section.position;
                *area = coord.section.area;
                *layer = coord.layer;
                if disabled.is_disabled() {
                    *disabled = Disabled::not_disabled();
                }
            } else {
                *disabled = Disabled::disabled();
            }
        }
    }
}
fn responsive_segment_changed(
    mut query: Query<
        (
            &ResponsiveSegment,
            &mut Position<InterfaceContext>,
            &mut Area<InterfaceContext>,
            &mut Layer,
            &mut Disabled,
        ),
        Changed<ResponsiveSegment>,
    >,
    viewport_handle: Res<ViewportHandle>,
    layout: Res<Layout>,
    grid: Res<MacroGrid>,
) {
    for (res_seg, mut pos, mut area, mut layer, mut disabled) in query.iter_mut() {
        // calc
        if let Some(coord) = res_seg.coordinate(*layout, viewport_handle.section(), &grid) {
            *pos = coord.section.position;
            *area = coord.section.area;
            *layer = coord.layer;
            if disabled.is_disabled() {
                *disabled = Disabled::not_disabled();
            }
        } else {
            *disabled = Disabled::disabled();
        }
    }
}
macro_rules! enable_conditional {
    () => {};
}
macro_rules! enable_conditional_scene {
    () => {};
}
macro_rules! enable_tree {
    () => {};
}
macro_rules! enable_on_enter {
    () => {};
}
impl Leaf for Tree {
    type SetDescriptor = EmptySetDescriptor;

    fn attach(elm: &mut Elm) {
        elm.container().insert_resource(Forest::default());
        elm.container().insert_resource(Layout::PORTRAIT_MOBILE);
        elm.container().insert_resource(MacroGrid::default());
        elm.main().add_systems((
            viewport_changed.in_set(CoreSet::Compositor),
            responsive_segment_changed.in_set(CoreSet::Compositor),
            set_branch.in_set(CoreSet::BranchPrepare),
        ));
    }
}