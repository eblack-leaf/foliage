use crate::animate::trigger::Trigger;
use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::InterfaceContext;
use crate::elm::config::CoreSet;
use crate::elm::leaf::{EmptySetDescriptor, Leaf};
use crate::elm::{Disabled, Elm};
use crate::ginkgo::viewport::ViewportHandle;
use crate::layout::Layout;
use crate::scene::{Binder, Bindings, ExtendTarget, Scene, SceneBinding, SceneDesc};
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
fn sprout<SEED: Seed + Send + Sync + 'static>(
    mut forest: ResMut<Forest>,
    navigation: Query<(Entity, &Navigation<SEED>), Changed<Navigation<SEED>>>,
    mut cmd: Commands,
    mut ext: StaticSystemParam<SEED::Resources>,
    mut grid: ResMut<MacroGrid>,
) {
    if let Some((_, n)) = navigation.iter().last() {
        // despawn current tree + all in pool
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
    // use target to spawn cond comp
    for (trigger, cond) in query.iter() {
        // handle extensions later
        if cond.is_extension {
            continue;
        }
        if trigger.active() {
            match cond.target {
                BranchExtendTarget::This(entity) => {
                    cmd.entity(entity).insert(cond.wrapped.clone());
                }
                BranchExtendTarget::BindingOf(_, _) => {}
            }
        }
    }
}
pub(crate) fn conditional_scene_spawn<CS: Scene + Clone>(
    query: Query<(&Trigger, &ConditionalScene<CS>), Changed<Trigger>>,
    mut cmd: Commands,
) {
    // use target to spawn cond scene
    for (trigger, cond) in query.iter() {
        // handle extensions later
        if cond.is_extension {
            panic!("scenes-are-not allowed as extensions")
        }
        if trigger.active() {
            match cond.target {
                BranchExtendTarget::This(entity) => {
                    let _scene_desc = cond
                        .wrapped
                        .clone()
                        .create(Binder::new(&mut cmd, Some(entity)));
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
        if trigger.active() {
            if cond.is_extension {
                match cond.target {
                    BranchExtendTarget::This(entity) => {
                        cmd.entity(entity).insert(cond.wrapped.clone());
                    }
                    BranchExtendTarget::BindingOf(parent, bind) => {
                        cmd.entity(bindings.get(parent).unwrap().get(bind))
                            .insert(cond.wrapped.clone());
                    }
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
        let desc = self.cmd.responsive_scene(s, rs);
        self.tree.0.insert(desc.root());
        desc
    }
    pub fn responsive<B: Bundle>(&mut self, b: B, rs: ResponsiveSegment) -> Entity {
        let ent = self.cmd.responsive(b, rs);
        self.tree.0.insert(ent);
        ent
    }
    pub fn branch<BR: Clone + Send + Sync + 'static, BH: Into<BranchHandle>>(
        &mut self,
        bh: BH,
        br: BR,
    ) -> BranchDesc {
        let desc = self.cmd.branch(br);
        self.tree.1.insert(bh.into(), desc.branch_entity);
        desc
    }
    pub fn branch_scene<S: Scene + Clone, BH: Into<BranchHandle>>(
        &mut self,
        bh: BH,
        s: S,
    ) -> BranchDesc {
        let desc = self.cmd.branch_scene(s);
        self.tree.1.insert(bh.into(), desc.branch_entity);
        desc
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
pub struct BranchSet(pub BranchHandle);
fn set_branch(query: Query<(Entity, &BranchSet)>, mut cmd: Commands, forest: Res<Forest>) {
    // set condition of branch-set.0 in forest.current.branches.get(bh)
    if forest.current.is_some() {
        for (entity, branch_request) in query.iter() {
            cmd.entity(
                *forest
                    .current
                    .as_ref()
                    .unwrap()
                    .1
                    .get(&branch_request.0)
                    .expect("invalid-branch-request"),
            )
            .insert(Trigger::activated());
            cmd.entity(entity).despawn();
        }
    }
}
trait Responsive {
    fn responsive_scene<S: Scene>(&mut self, s: S, rs: ResponsiveSegment) -> SceneDesc;
    fn responsive<B: Bundle>(&mut self, b: B, rs: ResponsiveSegment) -> Entity;
    fn branch<BR: Clone + Send + Sync + 'static>(&mut self, br: BR) -> BranchDesc;
    fn branch_scene<S: Scene + Clone>(&mut self, s: S) -> BranchDesc;
}
impl<'w, 's> Responsive for Commands<'w, 's> {
    fn responsive_scene<S: Scene>(&mut self, s: S, rs: ResponsiveSegment) -> SceneDesc {
        let scene_desc = s.create(Binder::new(self, None));
        self.entity(scene_desc.root()).insert(rs);
        scene_desc
    }
    fn responsive<B: Bundle>(&mut self, b: B, rs: ResponsiveSegment) -> Entity {
        self.spawn(b).insert(rs).id()
    }
    fn branch<BR: Clone + Send + Sync + 'static>(&mut self, br: BR) -> BranchDesc {
        let pre_spawned = self.spawn_empty().id();
        let branch_id = self
            .spawn(Branch::new(
                br,
                BranchExtendTarget::This(pre_spawned),
                false,
            ))
            .id();
        BranchDesc::new(branch_id, pre_spawned)
    }
    fn branch_scene<S: Scene + Clone>(&mut self, s: S) -> BranchDesc {
        let pre_spawned = self.spawn_empty().id();
        let branch_id = self
            .spawn(SceneBranch::new(
                s,
                BranchExtendTarget::This(pre_spawned),
                false,
            ))
            .id();
        BranchDesc::new(branch_id, pre_spawned)
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
    pub fn extend<E: Clone + Send + Sync + 'static>(
        self,
        target: ExtendTarget,
        e: E,
        cmd: &mut Commands,
    ) -> Self {
        match target {
            ExtendTarget::This => {
                cmd.entity(self.branch_entity).insert(Conditional::new(
                    BranchExtendTarget::This(self.pre_spawned),
                    e,
                    true,
                ));
            }
            ExtendTarget::Binding(bind) => {
                cmd.entity(self.branch_entity).insert(Conditional::new(
                    BranchExtendTarget::BindingOf(self.pre_spawned, bind),
                    e,
                    true,
                ));
            }
        }
        self
    }
}
// Derived-Value handler + other
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
    grid: Res<MacroGrid>,
) {
    for (res_seg, mut pos, mut area, mut layer, mut disabled) in query.iter_mut() {
        // calc
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
        elm.main().add_systems((
            viewport_changed.in_set(CoreSet::Compositor),
            responsive_segment_changed.in_set(CoreSet::Compositor),
            set_branch.in_set(CoreSet::BranchPrepare),
        ));
    }
}
