use crate::coordinate::area::Area;
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, NumericalContext};
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Default)]
pub struct TextureCoordinates(pub [CoordinateUnit; 2]);

impl TextureCoordinates {
    pub const fn new(x: CoordinateUnit, y: CoordinateUnit) -> Self {
        Self([x, y])
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Serialize, Deserialize, Default, Debug)]
pub struct TexturePartition(pub [CoordinateUnit; 4]);

impl TexturePartition {
    pub fn new(part: Section<NumericalContext>, whole: Area<NumericalContext>) -> Self {
        let section = part.normalized(whole);
        Self([
            section.left().min(1.0).max(0.0),
            section.top().min(1.0).max(0.0),
            section.right().min(1.0).max(0.0),
            section.bottom().min(1.0).max(0.0),
        ])
    }
    pub fn full() -> Self {
        Self([0.0, 0.0, 1.0, 1.0])
    }
}