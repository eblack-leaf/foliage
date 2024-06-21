use bevy_ecs::prelude::Component;
use bytemuck::{Pod, Zeroable};

use crate::coordinate::section::Section;
use crate::coordinate::{Coordinates, NumericalContext};

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
    pub fn from_section<S: Into<Section<NumericalContext>>, C: Into<Coordinates>>(
        part: S,
        whole: C,
    ) -> Self {
        let s = part.into().normalized(whole);
        Self::new(
            s.position.min((1.0, 1.0)).max((0.0, 0.0)).coordinates,
            s.area.min((1.0, 1.0)).max((0.0, 0.0)).coordinates,
        )
    }
}

#[repr(C)]
#[derive(Pod, Zeroable, Component, Copy, Clone, PartialEq, Default, Debug)]
pub struct Mips(pub f32);
