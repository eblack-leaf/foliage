use crate::coordinate::area::Area;
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, NumericalContext};
use crate::ginkgo::Ginkgo;
use crate::texture::coord::TexturePartition;
use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use wgpu::util::DeviceExt;
use wgpu::{Extent3d, TextureDimension, TextureUsages};

pub struct FixedAtlas<
    ReferenceKey: Hash + Eq + Clone,
    TexelData: Default + Sized + Clone + Pod + Zeroable,
> {
    key_to_partition: HashMap<ReferenceKey, TexturePartition>,
    dimensions: Area<NumericalContext>,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    format: wgpu::TextureFormat,
    _phantom: PhantomData<TexelData>,
}
impl<ReferenceKey: Hash + Eq + Clone, TexelData: Default + Sized + Clone + Pod + Zeroable>
    FixedAtlas<ReferenceKey, TexelData>
{
    pub fn new(ginkgo: &Ginkgo, dims: Area<NumericalContext>, format: wgpu::TextureFormat) -> Self {
        let texture = ginkgo.device.as_ref().unwrap().create_texture_with_data(
            ginkgo.queue.as_ref().unwrap(),
            &wgpu::TextureDescriptor {
                label: Some("fixed-texture-atlas"),
                size: Extent3d {
                    width: dims.width as u32,
                    height: dims.height as u32,
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
                (dims.width * dims.height) as usize
            ]),
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            key_to_partition: HashMap::new(),
            dimensions: dims,
            texture,
            view,
            format,
            _phantom: Default::default(),
        }
    }
    pub fn get(&self, key: ReferenceKey) -> TexturePartition {
        *self.key_to_partition.get(&key).unwrap()
    }
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
    pub fn write_key(
        &mut self,
        ginkgo: &Ginkgo,
        key: ReferenceKey,
        location: Section<NumericalContext>,
        data: &[TexelData],
    ) -> TexturePartition {
        ginkgo.queue().write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: location.position.x as u32,
                    y: location.position.y as u32,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(data),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(
                    (location.area.width * std::mem::size_of::<TexelData>() as CoordinateUnit)
                        as u32,
                ),
                rows_per_image: Some(
                    (location.area.height * std::mem::size_of::<TexelData>() as CoordinateUnit)
                        as u32,
                ),
            },
            wgpu::Extent3d {
                width: location.area.width as u32,
                height: location.area.height as u32,
                depth_or_array_layers: 1,
            },
        );
        let partition = TexturePartition::new(location, self.dimensions);
        self.key_to_partition.insert(key, partition);
        partition
    }
}
