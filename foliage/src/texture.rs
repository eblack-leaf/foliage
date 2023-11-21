use crate::coordinate::CoordinateUnit;
use bytemuck::{Pod, Zeroable};
#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Default)]
pub struct TextureCoordinates(pub [CoordinateUnit; 2]);
impl TextureCoordinates {
    pub const fn new(x: CoordinateUnit, y: CoordinateUnit) -> Self {
        Self([x, y])
    }
}
