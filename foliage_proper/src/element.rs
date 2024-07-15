use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Resource;
use std::collections::{HashMap, HashSet};

pub struct Root(pub TargetHandle);
impl Root {
    pub fn new<TH: Into<TargetHandle>>(th: TH) -> Self {
        Self(th.into())
    }
}
#[derive(Clone, PartialEq)]
pub struct Dependents(pub HashSet<TargetHandle>);
impl Dependents {
    pub fn new<THS: AsRef<[TargetHandle]>>(ths: THS) -> Self {
        let mut set = HashSet::new();
        for d in ths.as_ref() {
            let th = d.clone().into();
            set.insert(th);
        }
        Self(set)
    }
}
#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct TargetHandle(pub String);
impl<S: AsRef<str>> From<S> for TargetHandle {
    fn from(value: S) -> Self {
        Self(value.as_ref().to_string())
    }
}
#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct ActionHandle(pub String);
impl<S: AsRef<str>> From<S> for ActionHandle {
    fn from(value: S) -> Self {
        Self(value.as_ref().to_string())
    }
}
#[derive(Resource, Default)]
pub struct IdTable {
    pub(crate) targets: HashMap<TargetHandle, Entity>,
    pub(crate) actions: HashMap<ActionHandle, Entity>,
}
impl IdTable {
    pub fn lookup_target<TH: Into<TargetHandle>>(&self, th: TH) -> Entity {
        todo!()
    }
    pub fn lookup_action<AH: Into<ActionHandle>>(&self, ah: AH) -> Entity {
        todo!()
    }
}
