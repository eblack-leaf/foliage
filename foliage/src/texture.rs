use bytemuck::{Pod, Zeroable};

use crate::coordinate::area::Area;
use crate::coordinate::{CoordinateUnit, DeviceContext};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Default)]
pub struct TextureCoordinates(pub [CoordinateUnit; 2]);

impl TextureCoordinates {
    pub const fn new(x: CoordinateUnit, y: CoordinateUnit) -> Self {
        Self([x, y])
    }
}

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Default)]
pub struct MipsLevel(pub u32);

impl MipsLevel {
    pub fn new(size: Area<DeviceContext>, mips: u32, dims: Area<DeviceContext>) -> Self {
        if mips == 1 {
            return Self(0);
        }
        let mut mip_level_area = size;
        for _mip in 0..(mips - 1) {
            mip_level_area /= Area::<DeviceContext>::new(2f32, 2f32);
        }
        for mip in (0..mips).rev() {
            if mip_level_area.width >= dims.width && mip_level_area.height >= dims.height {
                return Self(mip);
            }
            mip_level_area *= Area::new(2f32, 2f32);
        }
        Self(0)
    }
}
