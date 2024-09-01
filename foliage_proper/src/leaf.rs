use std::collections::{HashMap, HashSet};

use crate::branch::LeafPtr;
use crate::coordinate::elevation::Elevation;
use crate::coordinate::placement::Placement;
use crate::coordinate::LogicalContext;
use crate::differential::{Remove, Visibility};
use crate::opacity::Opacity;
use crate::r_grid::GridLocation;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, Resource};

pub struct Leaf<LFN: for<'a> FnOnce(&mut LeafPtr<'a>)> {
    pub handle: LeafHandle,
    pub location: GridLocation,
    pub elevation: Elevation,
    pub l_fn: LFN,
}
impl<LFN: for<'a> FnOnce(&mut LeafPtr<'a>)> Leaf<LFN> {
    pub fn new(l_fn: LFN) -> Self {
        Self {
            handle: LeafHandle::Repr(String::default()),
            location: GridLocation::new(),
            elevation: Default::default(),
            l_fn,
        }
    }
    pub fn named<LH: Into<LeafHandle>>(mut self, lh: LH) -> Self {
        self
    }
    pub fn located<GL: Into<GridLocation>>(mut self, gl: GL) -> Self {
        self
    }
    pub fn elevation<E: Into<Elevation>>(mut self, e: E) -> Self {
        self
    }
}
#[derive(Bundle, Default)]
pub(crate) struct LeafBundle {
    stem: Stem,
    dependents: Dependents,
    placement: Placement<LogicalContext>,
    remove: Remove,
    visibility: Visibility,
    opacity: Opacity,
}
#[derive(Default, Component)]
pub(crate) struct Stem(pub(crate) Option<LeafHandle>);
impl Stem {
    pub(crate) fn new<TH: Into<LeafHandle>>(th: TH) -> Self {
        Self(Some(th.into()))
    }
}
#[derive(Clone, PartialEq, Component, Default)]
pub(crate) struct Dependents(pub(crate) HashSet<LeafHandle>);
impl Dependents {
    pub(crate) fn new<THS: AsRef<[LeafHandle]>>(ths: THS) -> Self {
        let mut set = HashSet::new();
        for d in ths.as_ref() {
            let th = d.clone();
            set.insert(th);
        }
        Self(set)
    }
}
#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum LeafHandle {
    Repr(String),
    Raw(Entity),
}
impl<S: AsRef<str>> From<S> for LeafHandle {
    fn from(value: S) -> Self {
        Self::Repr(value.as_ref().to_string())
    }
}
impl LeafHandle {
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        Self::Repr(s.as_ref().to_string())
    }
    pub const DELIMITER: &'static str = ":";
    pub fn extend<S: AsRef<str>>(&self, e: S) -> Self {
        match &self {
            LeafHandle::Repr(r) => Self::Repr(r.clone() + Self::DELIMITER + e.as_ref()),
            LeafHandle::Raw(_) => self.clone(),
        }
    }
}
#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct BranchHandle(String);
impl<S: AsRef<str>> From<S> for BranchHandle {
    fn from(value: S) -> Self {
        Self(value.as_ref().to_string())
    }
}
#[derive(Resource, Default)]
pub(crate) struct IdTable {
    pub(crate) leafs: HashMap<LeafHandle, Entity>,
    pub(crate) branches: HashMap<BranchHandle, Entity>,
}
impl IdTable {
    pub fn add_target<TH: Into<LeafHandle>>(&mut self, th: TH, entity: Entity) -> Option<Entity> {
        self.leafs.insert(th.into(), entity)
    }
    pub fn add_branch<AH: Into<BranchHandle>>(&mut self, ah: AH, entity: Entity) -> Option<Entity> {
        self.branches.insert(ah.into(), entity)
    }
    pub fn lookup_leaf<TH: Into<LeafHandle>>(&self, th: TH) -> Option<Entity> {
        let handle = th.into();
        match handle.clone() {
            LeafHandle::Repr(_r) => self.leafs.get(&handle).copied(),
            LeafHandle::Raw(e) => Some(e),
        }
    }
    pub fn lookup_branch<AH: Into<BranchHandle>>(&self, ah: AH) -> Option<Entity> {
        self.branches.get(&ah.into()).copied()
    }
}

#[derive(Default)]
pub struct OnEnd {
    pub actions: HashSet<BranchHandle>,
}
impl OnEnd {
    pub fn new<AH: Into<BranchHandle>>(ah: AH) -> Self {
        Self {
            actions: {
                let mut a = HashSet::new();
                a.insert(ah.into());
                a
            },
        }
    }
    pub fn with<AH: Into<BranchHandle>>(mut self, ah: AH) -> Self {
        self.actions.insert(ah.into());
        self
    }
}
