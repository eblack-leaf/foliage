use crate::coordinate::Coordinates;
use bevy_ecs::prelude::Component;
use bytemuck::{Pod, Zeroable};
#[repr(C)]
#[derive(Copy, Clone, Default, Debug, Component, PartialEq, Pod, Zeroable)]
pub struct TextureCoordinates {
    pub top_left: Coordinates,
    pub bottom_right: Coordinates,
}
impl TextureCoordinates {
    pub fn new<TL: Into<Coordinates>, BR: Into<Coordinates>>(tl: TL, br: BR) -> Self {
        Self {
            top_left: tl.into(),
            bottom_right: br.into(),
        }
    }
}
