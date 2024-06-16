use std::collections::HashMap;
use bevy_ecs::prelude::Resource;
use winit::event::{ElementState, MouseButton};
#[derive(Resource)]
pub(crate) struct MouseAdapter {
    cache: HashMap<MouseButton, ElementState>,
}
impl MouseAdapter {
    pub(crate) fn cache_different(&mut self, button: MouseButton, state: ElementState) -> Option<ClickInteraction>
}