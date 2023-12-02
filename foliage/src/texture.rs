use bevy_ecs::prelude::Component;
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use wgpu::util::DeviceExt;
use wgpu::{Extent3d, TextureDimension, TextureUsages};

use crate::coordinate::area::Area;
use crate::coordinate::position::Position;
use crate::coordinate::section::{CReprSection, Section};
use crate::coordinate::{CoordinateUnit, DeviceContext, NumericalContext};
use crate::ginkgo::Ginkgo;

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Default)]
pub struct TextureCoordinates(pub [CoordinateUnit; 2]);

impl TextureCoordinates {
    pub const fn new(x: CoordinateUnit, y: CoordinateUnit) -> Self {
        Self([x, y])
    }
}
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Serialize, Deserialize, Default)]
pub struct TexturePartition(pub CReprSection);
impl TexturePartition {
    pub fn new(part: Section<NumericalContext>, whole: Area<NumericalContext>) -> Self {
        Self(
            Section::new(
                part.position / (whole.width, whole.height).into(),
                part.area / whole,
            )
            .to_c(),
        )
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
#[derive(Hash, Eq, PartialEq, Copy, Clone, Ord, PartialOrd)]
pub struct TextureAtlasLocation(pub u32, pub u32);
impl TextureAtlasLocation {
    pub const PADDING: f32 = 1.0;
    pub fn position(&self, block: AtlasBlock) -> Position<NumericalContext> {
        (
            self.0 as f32 * (block.0.width + Self::PADDING),
            self.1 as f32 * (block.0.height + Self::PADDING),
        )
            .into()
    }
}
#[derive(Copy, Clone, Default)]
pub struct AtlasBlock(pub Area<NumericalContext>);
pub struct TextureAtlas<Key: Hash + Eq + Clone, TexelData: Default + Sized + Clone + Pod + Zeroable>
{
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub logical: Area<NumericalContext>,
    pub locations: HashMap<TextureAtlasLocation, Option<Key>>,
    pub block: AtlasBlock,
    pub actual: Area<NumericalContext>,
    pub key_to_partition: HashMap<Key, TexturePartition>,
    pub capacity: u32,
    pub format: wgpu::TextureFormat,
    texel_data: PhantomData<TexelData>,
}
impl<Key: Hash + Eq + Clone, TexelData: Default + Sized + Clone + Pod + Zeroable>
    TextureAtlas<Key, TexelData>
{
    pub fn new(
        ginkgo: &Ginkgo,
        block: AtlasBlock,
        capacity: u32,
        format: wgpu::TextureFormat,
    ) -> Self {
        let mut logical_dim = (capacity as f32).sqrt().floor() as u32;
        while logical_dim.pow(2) < capacity {
            logical_dim += 1;
        }
        let logical = Area::new(
            logical_dim.max(1) as CoordinateUnit,
            logical_dim.max(1) as CoordinateUnit,
        );
        let mut locations = HashMap::new();
        for x in 0..logical_dim {
            for y in 0..logical_dim {
                locations.insert(TextureAtlasLocation(x, y), None);
            }
        }
        let actual = TextureAtlasLocation(logical_dim, logical_dim).position(block);
        let actual = Area::new(actual.x, actual.y);
        let texture = ginkgo.device.as_ref().unwrap().create_texture_with_data(
            ginkgo.queue.as_ref().unwrap(),
            &wgpu::TextureDescriptor {
                label: Some("texture-atlas"),
                size: Extent3d {
                    width: actual.width as u32,
                    height: actual.height as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[format],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            bytemuck::cast_slice(&vec![
                TexelData::default();
                (actual.width * actual.height) as usize
            ]),
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            texture,
            view,
            logical,
            locations,
            block,
            actual,
            key_to_partition: HashMap::new(),
            capacity,
            format,
            texel_data: PhantomData,
        }
    }
    pub fn grow(&mut self, ginkgo: &Ginkgo, block: AtlasBlock, needed_capacity: u32) {
        *self = Self::new(ginkgo, block, needed_capacity, self.format);
    }
    pub fn num_filled_locations(&self) -> u32 {
        let mut num = 0;
        for a in self.locations.iter() {
            if a.1.is_some() {
                num += 1;
            }
        }
        num
    }
    pub fn has_key(&self, key: &Key) -> bool {
        for (_, val) in self.locations.iter() {
            if let Some(v) = val {
                if v == key {
                    return true;
                }
            }
        }
        false
    }
    pub fn get(&self, key: &Key) -> Option<TexturePartition> {
        self.key_to_partition.get(key).cloned()
    }
    pub fn write_location(
        &mut self,
        key: Key,
        ginkgo: &Ginkgo,
        location: TextureAtlasLocation,
        extent: Area<NumericalContext>,
        data: &Vec<TexelData>,
    ) -> TexturePartition {
        let position = location.position(self.block);
        ginkgo.queue().write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: position.x as u32,
                    y: position.y as u32,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(data),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(
                    (extent.width * std::mem::size_of::<TexelData>() as CoordinateUnit) as u32,
                ),
                rows_per_image: Some(
                    (extent.height * std::mem::size_of::<TexelData>() as CoordinateUnit) as u32,
                ),
            },
            wgpu::Extent3d {
                width: extent.width as u32,
                height: extent.height as u32,
                depth_or_array_layers: 1,
            },
        );
        self.locations
            .get_mut(&location)
            .unwrap()
            .replace(key.clone());
        let partition = TexturePartition::new(Section::new(position, extent), self.actual);
        self.key_to_partition.insert(key, partition);
        partition
    }
    pub fn next_location(&mut self) -> Option<TextureAtlasLocation> {
        if self.locations.is_empty() {
            return None;
        }
        let mut location = None;
        for (loc, val) in self.locations.iter() {
            if val.is_none() {
                location.replace(*loc);
                break;
            }
        }
        location
    }
}
