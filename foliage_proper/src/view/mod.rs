mod button;

use crate::element::{Element, TargetHandle};
use crate::grid::{Grid, GridPlacement};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, World};
use bevy_ecs::system::Command;
use std::collections::HashMap;

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
    grid: Grid,
    grid_placement: GridPlacement,
    target_handle: TargetHandle,
}
impl<V: Viewable> View<V> {
    pub(crate) fn new(
        v: V,
        grid: Grid,
        grid_placement: GridPlacement,
        target_handle: TargetHandle,
    ) -> Self {
        Self {
            view: v,
            grid,
            grid_placement,
            target_handle,
        }
    }
}
impl<V: Viewable> Command for View<V> {
    fn apply(self, world: &mut World) {
        let entity = world.spawn(Element::default()).id();
        let handle = ViewHandle {
            world_handle: Some(world),
            view_grid: self.grid,
            grid_placement: self.grid_placement,
            target_handle: self.target_handle,
            entity,
        };
        self.view.build(handle);
    }
}
pub struct ViewHandle<'a> {
    // world connection
    world_handle: Option<&'a mut World>,
    view_grid: Grid,
    grid_placement: GridPlacement,
    target_handle: TargetHandle,
    entity: Entity,
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
