use bytemuck::{Pod, Zeroable};
use coord::TexturePartition;
use std::collections::hash_map::IterMut;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use wgpu::util::DeviceExt;
use wgpu::{Extent3d, TextureDimension, TextureUsages};

use crate::coordinate::area::Area;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, NumericalContext};
use crate::ginkgo::Ginkgo;

pub mod coord;
pub mod factors;
#[allow(unused)]
pub mod fixed;

#[derive(Hash, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Debug)]
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
pub struct TextureAtlas<
    ReferenceKey: Hash + Eq + Clone,
    ResourceKey: Hash + Eq + Clone,
    TexelData: Default + Sized + Clone + Pod + Zeroable,
> {
    logical: LogicalAtlas<ReferenceKey, ResourceKey, TexelData>,
    hardware: HardwareAtlas<ResourceKey, TexelData>,
}
pub(crate) struct HardwareAtlas<
    ResourceKey: Hash + Eq + Clone,
    TexelData: Default + Sized + Clone + Pod + Zeroable,
> {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    locations: HashMap<TextureAtlasLocation, Option<ResourceKey>>,
    block: AtlasBlock,
    actual: Area<NumericalContext>,
    key_to_partition: HashMap<ResourceKey, TexturePartition>,
    capacity: u32,
    format: wgpu::TextureFormat,
    phantom: PhantomData<TexelData>,
}
impl<ResourceKey: Hash + Eq + Clone, TexelData: Default + Sized + Clone + Pod + Zeroable>
    HardwareAtlas<ResourceKey, TexelData>
{
    fn write_location(
        &mut self,
        key: ResourceKey,
        ginkgo: &Ginkgo,
        location: TextureAtlasLocation,
        extent: Area<NumericalContext>,
        data: &[TexelData],
    ) -> TexturePartition {
        let position = location.position(self.block);
        tracing::trace!(
            "writing to location: {:?} w/ extent: {:?}",
            position,
            extent
        );
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
    pub(crate) fn new(
        ginkgo: &Ginkgo,
        block: AtlasBlock,
        capacity: u32,
        format: wgpu::TextureFormat,
    ) -> Self {
        let mut logical_dim = ((capacity as f32).sqrt().floor() as u32).max(1);
        while logical_dim.pow(2) < capacity {
            logical_dim += 1;
        }
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
            locations,
            block,
            actual,
            key_to_partition: HashMap::new(),
            capacity,
            format,
            phantom: Default::default(),
        }
    }
    fn next_location(&mut self) -> Option<TextureAtlasLocation> {
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
pub(crate) struct LogicalAtlas<
    ReferenceKey: Hash + Eq + Clone,
    ResourceKey: Hash + Eq + Clone,
    TexelData: Default + Sized + Clone + Pod + Zeroable,
> {
    pub references: HashMap<ResourceKey, HashSet<ReferenceKey>>,
    pub entries: HashMap<ResourceKey, AtlasEntry<TexelData>>,
}
pub struct AtlasEntry<TexelData: Default + Sized + Clone + Pod + Zeroable> {
    data: Vec<TexelData>,
    extent: Area<NumericalContext>,
}
impl<TexelData: Default + Sized + Clone + Pod + Zeroable> AtlasEntry<TexelData> {
    pub fn set(&mut self, extent: Area<NumericalContext>, data: Vec<TexelData>) {
        self.data = data;
        self.extent = extent;
    }
}
impl<
        ReferenceKey: Hash + Eq + Clone + Debug,
        ResourceKey: Hash + Eq + Clone + Debug,
        TexelData: Default + Sized + Clone + Pod + Zeroable,
    > TextureAtlas<ReferenceKey, ResourceKey, TexelData>
{
    pub fn write_next(
        &mut self,
        key: ResourceKey,
        ginkgo: &Ginkgo,
        extent: Area<NumericalContext>,
        data: Vec<TexelData>,
    ) {
        let location = self.hardware.next_location().unwrap();
        tracing::trace!("writing atlas key: {:?}:{:?}", key, location);
        self.hardware
            .write_location(key.clone(), ginkgo, location, extent, &data);
        self.logical
            .entries
            .insert(key, AtlasEntry { data, extent });
    }
    pub fn remove_reference(&mut self, ref_key: ReferenceKey, res_key: ResourceKey) {
        if self.logical.references.get(&res_key).is_none() {
            return;
        }
        tracing::trace!(
            "removing reference ref-key: {:?} res-key:{:?}",
            ref_key,
            res_key
        );
        self.logical
            .references
            .get_mut(&res_key)
            .unwrap()
            .remove(&ref_key);
        // if size == 0 remove?
        if self.logical.references.get(&res_key).unwrap().is_empty() {
            self.hardware.key_to_partition.remove(&res_key);
            self.logical.entries.remove(&res_key);
            for (loc, key) in self.hardware.locations.iter_mut() {
                if let Some(k) = key {
                    if *k == res_key {
                        key.take();
                    }
                }
            }
        }
    }
    pub fn add_reference(&mut self, ref_key: ReferenceKey, res_key: ResourceKey) {
        if self.logical.references.get(&res_key).is_none() {
            self.logical
                .references
                .insert(res_key.clone(), HashSet::new());
        }
        tracing::trace!(
            "adding reference ref-key: {:?} res-key:{:?}",
            ref_key,
            res_key
        );
        self.logical
            .references
            .get_mut(&res_key)
            .unwrap()
            .insert(ref_key);
    }
    pub fn new(
        ginkgo: &Ginkgo,
        block: AtlasBlock,
        capacity: u32,
        format: wgpu::TextureFormat,
    ) -> Self {
        Self {
            logical: LogicalAtlas {
                references: HashMap::new(),
                entries: HashMap::new(),
            },
            hardware: HardwareAtlas::new(ginkgo, block, capacity, format),
        }
    }
    pub fn capacity(&self) -> u32 {
        self.hardware.capacity
    }
    pub fn entries_mut(&mut self) -> IterMut<'_, ResourceKey, AtlasEntry<TexelData>> {
        self.logical.entries.iter_mut()
    }
    pub fn block(&self) -> AtlasBlock {
        self.hardware.block
    }
    pub fn would_grow(&self, requested: u32) -> bool {
        self.capacity() < self.num_filled_locations() + requested
    }
    fn inner_rewrite_entry(
        logical: &LogicalAtlas<ReferenceKey, ResourceKey, TexelData>,
        hardware: &mut HardwareAtlas<ResourceKey, TexelData>,
        ginkgo: &Ginkgo,
    ) {
        for (key, entry) in logical.entries.iter() {
            let location = hardware.next_location().unwrap();
            hardware.write_location(key.clone(), ginkgo, location, entry.extent, &entry.data);
        }
    }
    pub fn grow_by(
        &mut self,
        requested: u32,
        ginkgo: &Ginkgo,
        block: AtlasBlock,
    ) -> Vec<(ReferenceKey, TexturePartition)> {
        self.hardware = HardwareAtlas::new(
            ginkgo,
            block,
            self.num_filled_locations() + requested,
            self.hardware.format,
        );
        // refill from logical entries
        tracing::trace!("growing atlas by {:?}", requested);
        Self::inner_rewrite_entry(&self.logical, &mut self.hardware, ginkgo);
        let mut changed = vec![];
        for (key, references) in self.logical.references.iter() {
            for refer in references.iter() {
                changed.push((refer.clone(), self.get(key).unwrap()));
            }
        }
        changed
    }
    pub fn view(&self) -> &wgpu::TextureView {
        &self.hardware.view
    }
    pub fn num_filled_locations(&self) -> u32 {
        let mut num = 0;
        for a in self.hardware.locations.iter() {
            if a.1.is_some() {
                num += 1;
            }
        }
        num
    }
    pub fn has_key(&self, key: &ResourceKey) -> bool {
        for (_, val) in self.hardware.locations.iter() {
            if let Some(v) = val {
                if v == key {
                    return true;
                }
            }
        }
        false
    }
    pub fn get(&self, key: &ResourceKey) -> Option<TexturePartition> {
        self.hardware.key_to_partition.get(key).cloned()
    }
}