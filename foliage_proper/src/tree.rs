use crate::animate::trigger::Trigger;
use crate::compositor::segment::{MacroGrid, ResponsiveSegment};
use crate::scene::{Bindings, Scene};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Bundle, Component};
use bevy_ecs::system::{Commands, SystemParam, SystemParamItem};
use std::collections::{HashMap, HashSet};

pub trait Tree {
    const GRID: MacroGrid;
    type Resources: SystemParam + 'static;
    fn plant(cmd: &mut Commands, res: &mut SystemParamItem<Self::Resources>) -> TreeDescriptor;
}
#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct TreeHandle(pub i32);
#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct BranchHandle(pub i32);
#[derive(Component)]
struct BranchWrapper<T> {
    wrapped: Option<T>,
}
impl<T> BranchWrapper<T> {
    fn new(t: T) -> Self {
        Self { wrapped: Some(t) }
    }
}
#[derive(Bundle)]
pub struct Branch {
    on_enter: OnEnter,
    trigger: Trigger,
    pool: TreeDescriptor,
}
#[derive(Component, Clone, Default)]
pub struct TreeDescriptor(pub HashSet<Entity>);
impl TreeDescriptor {}
impl<const N: usize> From<[Entity; N]> for TreeDescriptor {
    fn from(value: [Entity; N]) -> Self {
        let mut set = HashSet::new();
        for v in value {
            set.insert(v);
        }
        Self(set)
    }
}
impl Branch {
    pub fn new(t: OnEnterFn) -> Self {
        Self {
            on_enter: OnEnter::new(t),
            trigger: Trigger::default(),
            pool: TreeDescriptor::default(),
        }
    }
}
pub struct Forest {
    trees: HashMap<TreeHandle, Entity>,
}
#[derive(Component)]
pub struct OnEnter {
    logic: OnEnterFn,
}
pub type OnEnterFn = fn(cmd: &mut Commands) -> TreeDescriptor;
impl OnEnter {
    pub fn new(f: OnEnterFn) -> Self {
        Self { logic: f }
    }
}
pub struct BranchSet(pub BranchHandle);
pub trait Responsive {
    fn responsive_scene<S: Scene>(&mut self, s: S, rs: ResponsiveSegment) -> (Entity, Bindings);
    fn responsive<B: Bundle>(&mut self, b: B, rs: ResponsiveSegment) -> Entity;
    fn responsive_scene_w_ext<S: Scene, Ext: Bundle>(
        &mut self,
        s: S,
        rs: ResponsiveSegment,
        ext: Ext,
    ) -> (Entity, Bindings);
    fn responsive_w_ext<B: Bundle, Ext: Bundle>(
        &mut self,
        b: B,
        rs: ResponsiveSegment,
        ext: Ext,
    ) -> Entity;
    fn branch(&mut self, f: OnEnterFn) -> Entity;
}
impl<'w, 's> Responsive for Commands<'w, 's> {
    fn responsive_scene<S: Scene>(&mut self, s: S, rs: ResponsiveSegment) -> (Entity, Bindings) {
        let (entity, bindings) = s.create(self);
        self.entity(entity).insert(rs);
        (entity, bindings)
    }

    fn responsive<B: Bundle>(&mut self, b: B, rs: ResponsiveSegment) -> Entity {
        self.spawn(b).insert(rs).id()
    }

    fn responsive_scene_w_ext<S: Scene, Ext: Bundle>(
        &mut self,
        s: S,
        rs: ResponsiveSegment,
        ext: Ext,
    ) -> (Entity, Bindings) {
        let (entity, bindings) = s.create(self);
        self.entity(entity).insert(rs).insert(ext);
        (entity, bindings)
    }

    fn responsive_w_ext<B: Bundle, Ext: Bundle>(
        &mut self,
        b: B,
        rs: ResponsiveSegment,
        ext: Ext,
    ) -> Entity {
        self.spawn(b).insert(rs).insert(ext).id()
    }

    fn branch(&mut self, f: OnEnterFn) -> Entity {
        self.spawn(Branch::new(f)).id()
    }
}