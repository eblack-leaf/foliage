mod button;

use bevy_ecs::entity::Entity;
use std::collections::HashMap;
use bevy_ecs::prelude::Component;

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
pub struct ViewHandle {
    // world connection
}
impl ViewHandle {
    // view-api
}
pub trait Viewable {
    fn build(self, view_handle: ViewHandle);
}

// Then elm_handle.build_view("name", grid_placement, v)
// elm_handle.update_binding_of("button", ButtonBindings::Text, |b| b.get_attr_mut(|tv: &mut TextValue| tv.stuff()))
// + b.get_attr + b.insert_attr
// filtered-views? enable like filtered-attr?
//

