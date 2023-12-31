use crate::coordinate::area::Area;
use crate::coordinate::DeviceContext;
use bevy_ecs::component::Component;
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

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
                let fractional_mips = (mip as f32 - ratio).min((mips - 1) as f32);
                return Self(fractional_mips);
            }
            mip_level_area *= Area::new(2f32, 2f32);
        }
        Self(0.0)
    }
}

#[repr(C)]
#[derive(Component, Copy, Clone, PartialEq, Default, Pod, Zeroable, Serialize, Deserialize)]
pub struct Progress(pub(crate) f32, pub(crate) f32);

impl Progress {
    pub fn full() -> Self {
        Self(0.0, 1.0)
    }
    pub fn empty() -> Self {
        Self(0.0, 0.0)
    }
    pub fn start(&self) -> f32 {
        self.0
    }
    pub fn end(&self) -> f32 {
        self.1
    }
    pub fn set_start(&mut self, start: f32) {
        self.0 = start.max(0f32);
    }
    pub fn set_end(&mut self, end: f32) {
        self.1 = end.min(1f32);
    }
    pub fn end_mut(&mut self) -> &mut f32 {
        &mut self.1
    }
    pub fn start_mut(&mut self) -> &mut f32 {
        &mut self.0
    }
    pub fn new(start: f32, end: f32) -> Self {
        Self(start.max(0f32), end.min(1f32))
    }
}
