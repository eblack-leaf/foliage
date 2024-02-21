use crate::animate::trigger::Trigger;
use crate::compositor::segment::{MacroGrid, ResponsiveSegment};
use crate::scene::{Bindings, ExtendTarget, Scene, SceneDesc};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Bundle, Component, DetectChanges, Res};
use bevy_ecs::system::{
    Commands, ResMut, Resource, StaticSystemParam, SystemParam, SystemParamItem,
};
use std::collections::{HashMap, HashSet};
#[derive(Resource, Copy, Clone, Default)]
pub struct CurrentTree(pub TreeHandle);
fn sprout<SEED: Seed>(
    mut forest: ResMut<Forest>,
    ct: ResMut<CurrentTree>,
    mut cmd: Commands,
    mut ext: StaticSystemParam<SEED::Resources>,
) {
    if ct.is_changed() {
        // despawn current tree + all in pool
        let entity = forest.roots.get(&ct.0).expect("no-tree");
        let tree = SEED::plant(&mut cmd, &mut ext);
        forest.trees.insert(ct.0, tree);
    }
}
pub trait Seed {
    const GRID: MacroGrid;
    type Resources: SystemParam + 'static;
    fn plant(cmd: &mut Commands, res: &mut SystemParamItem<Self::Resources>) -> Tree;
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Default)]
pub struct TreeHandle(pub i32);
#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct BranchHandle(pub i32);
#[derive(Component)]
pub struct Conditional<T> {
    wrapped: Option<T>,
}
impl<T> Conditional<T> {
    pub fn new(t: T) -> Self {
        Self { wrapped: Some(t) }
    }
}
#[derive(Component)]
pub struct ConditionalScene<S: Scene> {
    wrapped: Option<S>,
}
impl<S: Scene> ConditionalScene<S> {
    pub fn new(t: S) -> Self {
        Self { wrapped: Some(t) }
    }
}
pub(crate) fn conditional_spawn<C>() {
    // use target to spawn cond comp
}
pub(crate) fn conditional_scene_spawn<CS: Scene>() {
    // use target to spawn cond scene
}
#[derive(Component, Copy, Clone)]
pub struct SpawnTarget(pub Entity);
#[derive(Bundle)]
pub struct Branch<T: Send + Sync + 'static> {
    conditional: Conditional<T>,
    trigger: Trigger,
    spawn_target: SpawnTarget,
}
#[derive(Bundle)]
pub struct SceneBranch<T: Scene + Send + Sync + 'static> {
    conditional: ConditionalScene<T>,
    trigger: Trigger,
    spawn_target: SpawnTarget,
}
impl<S: Scene> SceneBranch<S> {
    pub fn new(t: S, e: Entity) -> Self {
        Self {
            conditional: ConditionalScene::<S>::new(t),
            trigger: Trigger::default(),
            spawn_target: SpawnTarget(e),
        }
    }
}
#[derive(Component, Clone, Default)]
pub struct Tree(pub HashSet<Entity>, HashMap<BranchHandle, Entity>);
impl Tree {
    // bind | bind-scene for this need
    // create branch handle + forward Responsive::responsive_scene(&mut cmd) ...
}
impl<T: Send + Sync + 'static> Branch<T> {
    pub fn new(t: T, e: Entity) -> Self {
        Self {
            conditional: Conditional::<T>::new(t),
            trigger: Trigger::default(),
            spawn_target: SpawnTarget(e),
        }
    }
}
#[derive(Default, Resource)]
pub struct Forest {
    roots: HashMap<TreeHandle, Entity>,
    trees: HashMap<TreeHandle, Tree>,
}
impl Forest {
    pub fn navigate_to(&self, th: TreeHandle) {
        // delete current tree (or all others)
        // and Seed::plant() with self.trees.get(&th).expect("navigation")
        // navigation
    }
}
#[derive(Component, Copy, Clone)]
pub struct TreeSet(pub TreeHandle);
#[derive(Component, Copy, Clone)]
pub struct BranchSet(pub TreeHandle, pub BranchHandle);
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
        let branch_id = self.spawn(Branch::new(br, pre_spawned)).id();
        BranchDesc::new(branch_id, pre_spawned)
    }
    fn branch_scene<S: Scene>(&mut self, s: S) -> BranchDesc {
        let pre_spawned = self.spawn_empty().id();
        let branch_id = self.spawn(SceneBranch::new(s, pre_spawned)).id();
        BranchDesc::new(branch_id, pre_spawned)
    }
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
        cmd.entity(self.branch_entity).insert(Conditional::new(e));
        self
    }
}
