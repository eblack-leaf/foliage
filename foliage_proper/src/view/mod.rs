mod button;

use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, World};
use std::collections::HashMap;
use bevy_ecs::system::Command;
use crate::element::TargetHandle;
use crate::grid::{Grid, GridPlacement};

#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Copy, Clone)]
pub struct ViewBinding(pub i32);

#[derive(Clone, Component)]
pub struct Bindings {
    mapping: HashMap<ViewBinding, Entity>,
}
impl Bindings {
    pub fn get<VB: Into<ViewBinding>>(&self, vb: VB) -> Entity {
        *self.mapping.get(&vb.into()).unwrap()
    }
}
pub struct View<V: Viewable> {
    view: V,
    view_grid: Grid,
    grid_placement: GridPlacement,
    target_handle: TargetHandle,
}
impl<V: Viewable> Command for View<V> {
    fn apply(self, world: &mut World) {
        let handle = ViewHandle {
            world_handle: Some(world)
        };
        // create root w/ element + grid

    }
}
pub struct ViewHandle<'a> {
    // world connection
    world_handle: Option<&'a mut World>
}
impl ViewHandle {
    // view-api
}
pub trait Viewable {
    fn build(self, view_handle: ViewHandle);
}

// Then elm_handle.build_view(Some("root-name"), "name", grid_placement, v) + attach grid to "name" + configure roots + deps
// elm_handle.update_binding_of("button", ButtonBindings::Text, |b| b.get_attr_mut(|tv: &mut TextValue| tv.stuff()))
// + b.get_attr + b.insert_attr
// filtered-views? enable like filtered-attr?
//
