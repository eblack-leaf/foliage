use crate::ash::render::Render;
use bevy_ecs::component::Component;
use compact_str::{CompactString, ToCompactString};
use serde::{Deserialize, Serialize};
use std::any::TypeId;

#[derive(Component, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct RenderTag(pub CompactString);

pub trait RenderTagged {
    fn tag() -> RenderTag;
}

impl<T: Render + 'static> RenderTagged for T {
    fn tag() -> RenderTag {
        RenderTag(format!("{:?}", TypeId::of::<T>()).to_compact_string())
    }
}
