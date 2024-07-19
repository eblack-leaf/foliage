use bevy_ecs::bundle::Bundle;
use foliage_macros::inner_view_bindings;
use crate::style::Coloring;
use crate::view::{Bindings, Viewable, ViewHandle};

pub struct Button {

}
impl Viewable for Button {
    fn build(self, view_handle: ViewHandle) {
        todo!()
    }
}
#[derive(Bundle, Clone)]
pub struct ButtonComponents {
    bindings: Bindings,
    coloring: Coloring,
}
#[inner_view_bindings]
pub enum ButtonBindings {
    Text,
    Panel,
    Icon,
}
