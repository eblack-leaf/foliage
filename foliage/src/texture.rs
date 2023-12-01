use bevy_ecs::prelude::Component;
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

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
#[derive(Pod, Zeroable, Copy, Clone, Default, Serialize, Deserialize, Component, PartialEq)]
pub struct MipsLevel(pub f32);

impl MipsLevel {
    pub fn new(size: Area<DeviceContext>, mips: u32, dims: Area<DeviceContext>) -> Self {
        if mips == 1 {
            return Self(0.0);
        }
        let mut mip_level_area = size;
        for _mip in 0..(mips - 1) {
            mip_level_area /= Area::<DeviceContext>::new(2f32, 2f32);
        }
        for mip in (0..mips).rev() {
            if mip_level_area.width >= dims.width && mip_level_area.height >= dims.height {
                let actual = dims / mip_level_area;
                let ratio = 1f32 - (actual.width + actual.height) / 2f32;
                let _fractional_mips = (mip as f32 - ratio).min((mips - 1) as f32);
                return Self(mip as f32);
            }
            mip_level_area *= Area::new(2f32, 2f32);
        }
        Self(0.0)
    }
}

#[repr(C)]
#[derive(Component, Copy, Clone, PartialEq, Default, Pod, Zeroable, Serialize, Deserialize)]
pub struct Progress(pub f32, pub f32);

impl Progress {
    pub fn full() -> Self {
        Self(0.0, 1.0)
    }
    pub fn empty() -> Self {
        Self(0.0, 0.0)
    }
    pub fn new(start: f32, end: f32) -> Self {
        Self(start, end)
    }
}
