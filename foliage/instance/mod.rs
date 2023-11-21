use std::collections::HashMap;
use std::marker::PhantomData;

use bevy_ecs::prelude::Component;
use bytemuck::{Pod, Zeroable};

use crate::gfx::GfxSurface;
pub use index::Index;
pub use index::Indexer;
pub use key::{Key, KeyFactory};

pub mod index;
pub mod key;
/// Cpu side buffer for an attribute of instances
#[derive(Component)]
pub struct CpuAttributeBuffer<T: Default + Clone> {
    pub buffer: Vec<T>,
}

impl<T: Default + Clone> CpuAttributeBuffer<T> {
    /// creates a new buffer initially filled with ::default()
    pub fn new(max: u32) -> Self {
        Self {
            buffer: {
                let mut vec = Vec::new();
                vec.resize(max as usize, T::default());
                vec
            },
        }
    }
}
/// Temporary storage for staging writes to the Gpu
#[derive(Component)]
pub struct AttributeWrite<Attribute> {
    pub write: HashMap<Index, Attribute>,
}

impl<Attribute> AttributeWrite<Attribute> {
    pub fn new() -> Self {
        Self {
            write: HashMap::new(),
        }
    }
}
impl<Attribute> Default for AttributeWrite<Attribute> {
    fn default() -> Self {
        AttributeWrite::new()
    }
}
/// Gpu side buffer of an attribute of an instance
#[derive(Component)]
pub struct GpuAttributeBuffer<T> {
    pub buffer: wgpu::Buffer,
    _phantom_data: PhantomData<T>,
}

impl<T> GpuAttributeBuffer<T> {
    /// creates a new buffer on the Gpu with size of max * `size_of<T>`
    pub fn new(gfx_surface: &GfxSurface, max: u32, label: &'static str) -> Self {
        Self {
            buffer: gfx_surface.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: (std::mem::size_of::<T>() * max as usize) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            _phantom_data: PhantomData,
        }
    }
}
/// Combines a cpu/gpu/write for an attribute for full management of the data for an attribute
pub struct InstanceAttributeManager<
    Attribute: Send + Sync + Default + Clone + Pod + Zeroable + 'static,
> {
    pub cpu: CpuAttributeBuffer<Attribute>,
    pub gpu: GpuAttributeBuffer<Attribute>,
    pub write: AttributeWrite<Attribute>,
}

impl<Attribute: Send + Sync + Default + Clone + Pod + Zeroable + 'static>
    InstanceAttributeManager<Attribute>
{
    pub fn new(gfx_surface: &GfxSurface, max: u32) -> Self {
        Self {
            cpu: CpuAttributeBuffer::new(max),
            gpu: GpuAttributeBuffer::new(gfx_surface, max, "instance_tool gpu_buffer"),
            write: AttributeWrite::new(),
        }
    }
    /// queues a write to the staging buffer for writing
    pub fn queue_write(&mut self, index: Index, attr: Attribute) {
        self.write.write.insert(index, attr);
    }
    /// when needed, the attribute buffers must grow if more instances are requested
    pub fn grow(&mut self, gfx_surface: &GfxSurface, max: u32) {
        self.cpu.buffer.resize(max as usize, Attribute::default());
        self.gpu = GpuAttributeBuffer::<Attribute>::new(gfx_surface, max, "attribute buffer");
        gfx_surface
            .queue
            .write_buffer(&self.gpu.buffer, 0, bytemuck::cast_slice(&self.cpu.buffer));
    }
    /// this writes all the staged data in write, to the gpu
    pub fn write(&mut self, gfx_surface: &GfxSurface) {
        let attributes = self
            .write
            .write
            .drain()
            .collect::<Vec<(Index, Attribute)>>();
        let mut write_range: (Option<Index>, Option<Index>) = (None, None);
        for (index, attr) in attributes {
            *self.cpu.buffer.get_mut(index.value as usize).unwrap() = attr;
            if let Some(start) = write_range.0.as_mut() {
                if index.value < start.value {
                    *start = index;
                }
            } else {
                write_range.0.replace(index);
            }
            if let Some(end) = write_range.1.as_mut() {
                if index.value > end.value {
                    *end = index;
                }
            } else {
                write_range.1.replace(index);
            }
        }
        if let Some(start) = write_range.0 {
            let end = write_range.1.take().unwrap();
            let cpu_range = &self.cpu.buffer[start.value as usize..end.value as usize + 1];
            let offset = offset::<Attribute>(&start);
            gfx_surface.queue.write_buffer(
                &self.gpu.buffer,
                offset,
                bytemuck::cast_slice(cpu_range),
            );
        }
    }
}
/// helper for obtaining an offset sized for a type
pub fn offset<T>(index: &Index) -> wgpu::BufferAddress {
    (std::mem::size_of::<T>() * index.value as usize) as wgpu::BufferAddress
}
/// NullBit is useful for dropping instances without having to resize the buffer or swap elements.
/// NullBit sets all other attributes to 0 in the shader when multiplied by to cancel one instance
/// while still having it included in the buffer.
#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct NullBit {
    bit: u32,
}

impl Default for NullBit {
    fn default() -> Self {
        NullBit::null()
    }
}

impl NullBit {
    pub(crate) const NOT_NULL: u32 = 0u32;
    pub(crate) const NULL: u32 = 1u32;
    fn new(bit: u32) -> Self {
        Self { bit }
    }
    /// creates a NullBit that does not cancel the instance
    pub fn not_null() -> NullBit {
        Self::new(Self::NOT_NULL)
    }
    /// creates a NullBit that cancels the instance
    pub fn null() -> Self {
        Self::new(Self::NULL)
    }
}
