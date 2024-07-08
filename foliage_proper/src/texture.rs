use crate::coordinate::section::Section;
use crate::coordinate::{Coordinates, NumericalContext};
use crate::ginkgo::Ginkgo;
use bevy_ecs::prelude::Component;
use bytemuck::{Pod, Zeroable};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use wgpu::{Texture, TextureFormat, TextureView};

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
        let pos_coords = s.position.min((1.0, 1.0)).max((0.0, 0.0)).coordinates;
        Self::new(
            pos_coords,
            pos_coords + s.area.min((1.0, 1.0)).max((0.0, 0.0)).coordinates,
        )
    }
}
#[repr(C)]
#[derive(Pod, Zeroable, Component, Copy, Clone, PartialEq, Default, Debug)]
pub struct Mips(pub f32);
#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct AtlasLocation(pub u32, pub u32);
pub struct PartitionInfo {
    tex_coords: TextureCoordinates,
    location: AtlasLocation,
}
pub struct TextureAtlas<
    Key: Hash + Clone,
    Referrer: Hash + Clone,
    TexelData: Default + Sized + Clone + Pod + Zeroable,
> {
    texture: Texture,
    view: TextureView,
    partitions: HashMap<Key, PartitionInfo>,
    possible_locations: HashSet<AtlasLocation>,
    block_size: Coordinates,
    texture_extent: Coordinates,
    capacity: u32,
    format: TextureFormat,
    references: HashMap<Key, HashSet<Referrer>>,
    pub(crate) entries: HashMap<Key, AtlasEntry<TexelData>>,
}
#[derive(Clone)]
pub struct AtlasChangeInfo<Referrer: Clone> {
    pub key: Referrer,
    pub tex_coords: TextureCoordinates,
}
impl<
        Key: Hash + Clone + Eq,
        Referrer: Hash + Eq + Clone,
        TexelData: Default + Sized + Clone + Pod + Zeroable,
    > TextureAtlas<Key, Referrer, TexelData>
{
    pub const PADDING: f32 = 1.0;
    pub fn new<C: Into<Coordinates>>(
        ginkgo: &Ginkgo,
        block: C,
        capacity: u32,
        format: TextureFormat,
    ) -> Self {
        let block = block.into();
        let (possible_locations, texture_extent) = Self::config(capacity, block);
        let data = vec![
            TexelData::default();
            (texture_extent.horizontal() * texture_extent.vertical()) as usize
        ];
        let (texture, view) =
            ginkgo.create_texture(format, texture_extent, 1, bytemuck::cast_slice(&data));
        let actual_capacity = possible_locations.len() as u32;
        Self {
            texture,
            view,
            partitions: HashMap::new(),
            possible_locations,
            block_size: block,
            texture_extent,
            capacity: actual_capacity,
            format,
            references: Default::default(),
            entries: Default::default(),
        }
    }
    pub fn view(&self) -> &TextureView {
        &self.view
    }
    fn config(capacity: u32, block: Coordinates) -> (HashSet<AtlasLocation>, Coordinates) {
        let mut logical_dim = ((capacity as f32).sqrt().floor() as u32).max(1);
        while logical_dim.pow(2) < capacity {
            logical_dim += 1;
        }
        let mut possible_locations = HashSet::new();
        for x in 0..logical_dim {
            for y in 0..logical_dim {
                possible_locations.insert(AtlasLocation(x, y));
            }
        }
        if capacity == 0 {
            possible_locations.clear();
        }
        let texture_extent = Coordinates::new(
            logical_dim as f32 * (block.horizontal() + Self::PADDING),
            logical_dim as f32 * (block.vertical() + Self::PADDING),
        );
        (possible_locations, texture_extent)
    }

    pub fn has_key(&self, key: Key) -> bool {
        self.partitions.get(&key).is_some()
    }
    pub fn add_entry(&mut self, key: Key, entry: AtlasEntry<TexelData>) {
        self.entries.insert(key.clone(), entry);
        self.references.insert(key, HashSet::new());
    }
    pub fn remove_entry(&mut self, key: Key) {
        self.entries.remove(&key);
        let partition = self.partitions.remove(&key);
        if let Some(part) = partition {
            self.possible_locations.insert(part.location);
        }
        self.references.remove(&key);
    }
    pub fn add_reference(&mut self, key: Key, referrer: Referrer) {
        self.references.get_mut(&key).unwrap().insert(referrer);
    }
    pub fn remove_reference(&mut self, key: Key, referrer: Referrer) {
        self.references.get_mut(&key).unwrap().remove(&referrer);
        if self.references.get(&key).unwrap().is_empty() {
            self.remove_entry(key);
        }
    }
    pub fn resolve(&mut self, ginkgo: &Ginkgo) -> (Vec<AtlasChangeInfo<Referrer>>, bool) {
        let mut added = Vec::new();
        for entry in self.entries.iter() {
            if self.partitions.get(entry.0).is_none() {
                added.push((
                    entry.0.clone(),
                    AtlasEntry::new(entry.1.data.clone(), entry.1.extent),
                ));
            }
        }
        let mut grown = false;
        if added.len() > self.possible_locations.len() {
            grown = true;
            let difference = added.len() - self.possible_locations.len();
            let new_capacity = self.capacity + difference as u32;
            let (possible_locations, texture_extent) = Self::config(new_capacity, self.block_size);
            self.texture_extent = texture_extent;
            self.possible_locations = possible_locations;
            self.capacity = self.possible_locations.len() as u32;
            let (texture, view) = ginkgo.create_texture(
                self.format,
                texture_extent,
                1,
                bytemuck::cast_slice(&vec![
                    TexelData::default();
                    (texture_extent.horizontal() * texture_extent.vertical())
                        as usize
                ]),
            );
            self.texture = texture;
            self.view = view;
            for key in self.partitions.keys() {
                added.push((key.clone(), self.entries.get(&key).unwrap().clone()));
            }
        }
        let mut changed = Vec::new();
        for add in added {
            let location = *self.possible_locations.iter().last().unwrap();
            self.possible_locations.remove(&location);
            let position = Coordinates::from((location.0, location.1))
                * (self.block_size + (Self::PADDING, Self::PADDING).into());
            ginkgo.write_texture(&self.texture, position, add.1.extent, add.1.data);
            let tex_coords = TextureCoordinates::from_section(
                Section::new(position, add.1.extent),
                self.texture_extent,
            );
            let partition_info = PartitionInfo {
                tex_coords,
                location,
            };
            self.partitions.insert(add.0.clone(), partition_info);
            for referrer in self.references.get(&add.0).unwrap().iter() {
                changed.push(AtlasChangeInfo {
                    key: referrer.clone(),
                    tex_coords,
                });
            }
        }
        (changed, grown)
    }
    pub fn tex_coordinates(&self, key: Key) -> TextureCoordinates {
        self.partitions.get(&key).unwrap().tex_coords
    }
}

#[derive(Clone, Debug)]
pub struct AtlasEntry<TexelData: Default + Sized + Clone + Pod + Zeroable> {
    data: Vec<TexelData>,
    extent: Coordinates,
}

impl<TexelData: Default + Sized + Clone + Pod + Zeroable> AtlasEntry<TexelData> {
    pub fn new<C: Into<Coordinates>>(data: Vec<TexelData>, extent: C) -> Self {
        Self {
            data,
            extent: extent.into(),
        }
    }
}
