use crate::style::Coloring;
use crate::view::{Bindings, ViewHandle, Viewable};
use bevy_ecs::bundle::Bundle;
use foliage_macros::inner_view_bindings;
#[derive(Clone)]
pub struct Button {}
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
