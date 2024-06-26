use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use bevy_ecs::component::Component;
use bevy_ecs::schedule::SystemSet;

use crate::elm::config::ElmConfiguration;
use crate::elm::Elm;
use crate::Foliage;

pub trait Leaf {
    type SetDescriptor: SystemSet + Hash + Eq + PartialEq + Copy + Clone + Debug;
    fn config(_elm_configuration: &mut ElmConfiguration) {}
    fn attach(elm: &mut Elm);
}
#[derive(SystemSet, Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum EmptySetDescriptor {}
pub(crate) struct Leaflet(
    pub(crate) Box<fn(&mut ElmConfiguration)>,
    pub(crate) Box<fn(&mut Elm)>,
);

impl Leaflet {
    pub(crate) fn leaf_fn<T: Leaf>() -> Self {
        Self(Box::new(T::config), Box::new(T::attach))
    }
}
#[macro_export]
macro_rules! set_descriptor {
    ($desc:item) => {
        #[derive($crate::bevy_ecs::prelude::SystemSet, Hash, Eq, PartialEq, Copy, Clone, Debug)]
        $desc
    };
}
#[derive(Component, Copy, Clone)]
pub struct Tag<T> {
    _phantom: PhantomData<T>,
}

impl<T> Tag<T> {
    #[allow(unused)]
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T> Default for Tag<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub trait Leaves {
    fn leaves(f: Foliage) -> Foliage;
}
