use crate::animate::trigger::Trigger;
use crate::compositor::segment::{MacroGrid, ResponsiveSegment};
use crate::scene::{ExtendTarget, Scene, SceneBinding, SceneDesc};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Bundle, Component};
use bevy_ecs::query::{Changed, With};
use bevy_ecs::system::{Commands, Query, Res, ResMut, Resource, StaticSystemParam, SystemParam, SystemParamItem};
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
    navigation: Query<&Navigation<SEED>, Changed<Navigation<SEED>>>,
    mut cmd: Commands,
    mut ext: StaticSystemParam<SEED::Resources>,
    mut grid: ResMut<MacroGrid>,
) {
    if let Some(n) = navigation.iter().last() {
        // despawn current tree + all in pool
        // or anim-out && @-end trigger despawn
        *grid = SEED::GRID;
        let tree = SEED::plant(&mut cmd, &mut ext);
        forest.current.replace(tree);
    }
}
fn clean_navigation<SEED: Seed + Send + Sync + 'static>(
    navigation: Query<Entity, With<Navigation<SEED>>>,
    mut cmd: Commands,
) {
    for n in navigation.iter() {
        cmd.entity(n).despawn();
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
pub struct Conditional<T> {
    wrapped: Option<T>,
    target: BranchExtendTarget,
}
impl<T> Conditional<T> {
    pub fn new(e: BranchExtendTarget, t: T) -> Self {
        Self {
            wrapped: Some(t),
            target: e,
        }
    }
}
#[derive(Component)]
pub struct ConditionalScene<S: Scene> {
    wrapped: Option<S>,
    target: BranchExtendTarget,
}
impl<S: Scene> ConditionalScene<S> {
    pub fn new(e: BranchExtendTarget, t: S) -> Self {
        Self {
            wrapped: Some(t),
            target: e,
        }
    }
}
pub(crate) fn conditional_spawn<C>() {
    // use target to spawn cond comp
}
pub(crate) fn conditional_scene_spawn<CS: Scene>() {
    // use target to spawn cond scene
}
#[derive(Bundle)]
pub struct Branch<T: Send + Sync + 'static> {
    conditional: Conditional<T>,
    trigger: Trigger,
}
impl<T: Send + Sync + 'static> Branch<T> {
    pub fn new(t: T, e: BranchExtendTarget) -> Self {
        Self {
            conditional: Conditional::<T>::new(e, t),
            trigger: Trigger::default(),
        }
    }
}
#[derive(Bundle)]
pub struct SceneBranch<T: Scene + Send + Sync + 'static> {
    conditional: ConditionalScene<T>,
    trigger: Trigger,
}
impl<S: Scene> SceneBranch<S> {
    pub fn new(t: S, e: BranchExtendTarget) -> Self {
        Self {
            conditional: ConditionalScene::<S>::new(e, t),
            trigger: Trigger::default(),
        }
    }
}
#[derive(Component, Clone, Default)]
pub struct Tree(pub HashSet<Entity>, HashMap<BranchHandle, Entity>);
impl Tree {
    // bind | bind-scene for this need
    // create branch handle + forward Responsive::responsive_scene(&mut cmd) ...
}
#[derive(Default, Resource)]
pub struct Forest {
    current: Option<Tree>,
}
impl Forest {
    pub fn navigate<N: Send + Sync + 'static>(cmd: &mut Commands) {
        cmd.spawn(Navigation::<N>::new());
    }
}
// Uses current-tree and sets condition for that branch using tree.branches
#[derive(Component, Copy, Clone)]
pub struct BranchSet(pub BranchHandle);
fn set_branch(query: Query<&BranchSet>, mut cmd: Commands, forest: Res<Forest>) {
    // set condition of branch-set.0 in forest.current.branches.get(bh)
}
pub trait Responsive {
    fn responsive_scene<S: Scene>(&mut self, s: S, rs: ResponsiveSegment) -> SceneDesc;
    fn responsive<B: Bundle>(&mut self, b: B, rs: ResponsiveSegment) -> Entity;
    fn branch<BR: Send + Sync + 'static>(&mut self, br: BR) -> BranchDesc;
    fn branch_scene<S: Scene>(&mut self, s: S) -> BranchDesc;
}
impl<'w, 's> Responsive for Commands<'w, 's> {
    fn responsive_scene<S: Scene>(&mut self, s: S, rs: ResponsiveSegment) -> SceneDesc {
        let scene_desc = s.create(self);
        self.entity(scene_desc.root()).insert(rs);
        scene_desc
    }
    fn responsive<B: Bundle>(&mut self, b: B, rs: ResponsiveSegment) -> Entity {
        self.spawn(b).insert(rs).id()
    }
    fn branch<BR: Send + Sync + 'static>(&mut self, br: BR) -> BranchDesc {
        let pre_spawned = self.spawn_empty().id();
        let branch_id = self
            .spawn(Branch::new(br, BranchExtendTarget::This(pre_spawned)))
            .id();
        BranchDesc::new(branch_id, pre_spawned)
    }
    fn branch_scene<S: Scene>(&mut self, s: S) -> BranchDesc {
        let pre_spawned = self.spawn_empty().id();
        let branch_id = self
            .spawn(SceneBranch::new(s, BranchExtendTarget::This(pre_spawned)))
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
    pub fn extend<E: Send + Sync + 'static>(
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
                ));
            }
            ExtendTarget::Binding(bind) => {
                cmd.entity(self.branch_entity).insert(Conditional::new(
                    BranchExtendTarget::BindingOf(self.pre_spawned, bind),
                    e,
                ));
            }
        }
        self
    }
}
// Derived-Value handler + other
pub struct OnEnter<T> {

}
fn viewport_changed() {}
fn responsive_segment_changed() {}
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