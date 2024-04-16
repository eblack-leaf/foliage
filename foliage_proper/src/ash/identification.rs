use bevy_ecs::component::Component;
use compact_str::CompactString;
use serde::{Deserialize, Serialize};

use crate::ash::render::Render;
use crate::elm::compact_string_type_id;

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Hash, Component, Debug)]
pub struct RenderId(pub CompactString);

pub trait RenderIdentification {
    fn render_id() -> RenderId;
}

impl<T: Render + 'static> RenderIdentification for T {
    fn render_id() -> RenderId {
        RenderId(compact_string_type_id::<T>())
    }
}