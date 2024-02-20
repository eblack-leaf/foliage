use crate::animate::trigger::Trigger;
use crate::compositor::segment::MacroGrid;
use crate::elm::leaf::Tag;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, Query};
use bevy_ecs::query::{Changed, With};
use bevy_ecs::system::Commands;
use std::collections::{HashMap, HashSet};
pub type BranchHandle = i32;
pub type TreeHandle = i32;
pub type Branches = HashMap<BranchHandle, Entity>;
pub struct EntityPool {
    per_branch: HashMap<BranchHandle, HashSet<Entity>>,
}
pub struct Forest {
    pub trees: HashMap<TreeHandle, Entity>,
    pub current_tree: TreeHandle,
}
pub struct Tree {
    pub grid: MacroGrid,
    pub entity_pool: EntityPool,
    pub branches: Branches,
    pub current_branch: BranchHandle,
}
pub struct TreeDescriptor<'a, 'w, 's> {
    root: Entity,
    cmd: &'a mut Commands<'w, 's>,
    tree: Tree,
}
impl<'a, 'w, 's> TreeDescriptor<'a, 'w, 's> {
    pub fn branch<BR: Branch>(mut self, i: BranchHandle, b: BR) -> Self {
        let entity = self.cmd.spawn(BranchComponents::new(self.root, i, b)).id();
        self.tree.branches.insert(i, entity);
        self
    }
    pub fn plant(self) -> Entity {
        // spawn tree component with all branches done and give back entity
        self.root
    }
}
#[macro_export]
macro_rules! enable_branch {
    (&elm:ident, $($t:ty),*) => {
        $($elm.enable_branch::<$t>();)*
    };
}
impl Tree {
    pub fn root<BR: Branch>(
        grid: MacroGrid,
        cmd: &mut Commands,
        root_branch: BR,
    ) -> TreeDescriptor {
        todo!()
    }
}
#[derive(Component, Copy, Clone)]
pub struct TreePtr(Entity);
impl TreePtr {
    pub fn value(&self) -> Entity {
        self.0
    }
}
pub trait Branch {
    fn grow(&self, grid: MacroGrid, cmd: &mut Commands);
}
struct BranchComponents<BR: Branch> {
    twig: Twig<BR>,
    ptr: TreePtr,
    cond: Trigger,
    tag: Tag<IsBranch>,
}
impl<BR: Branch> BranchComponents<BR> {
    fn new(ptr: Entity, bh: BranchHandle, br: BR) -> Self {
        Self {
            twig: Twig(bh, br),
            ptr: TreePtr(ptr),
            cond: Trigger::default(),
            tag: Tag::new(),
        }
    }
}
#[derive(Component)]
pub struct Twig<BR: Branch>(BranchHandle, BR);
pub struct IsBranch;
fn grow_branch<BR: Branch>(
    mut trees: Query<&mut Tree>,
    branches: Query<(&Trigger, &Twig<BR>, &TreePtr), (With<Tag<IsBranch>>, Changed<Trigger>)>,
    mut cmd: Commands,
) {
    for (cond, twig, ptr) in branches.iter() {
        if cond.triggered() {
            let tree = trees.get_mut(ptr.value()).unwrap();
            if let Some(old) = tree.branches.remove(&twig.0) {
                // despawn all in branch from pool
            } else {
                let entities = twig.1.grow(tree.grid, &mut cmd);
                // add to tree.entity_pool all entities
            }
        }
    }
}