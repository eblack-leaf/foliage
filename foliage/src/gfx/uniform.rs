use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use wgpu::BindGroupEntry;

pub struct Uniform<Data: Pod + Zeroable> {
    pub data: Data,
    pub buffer: wgpu::Buffer,
}
impl<Data: Pod + Zeroable> Uniform<Data> {
    pub fn new(device: &wgpu::Device, data: Data) -> Self {
        Self {
            data,
            buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("uniform"),
                contents: bytemuck::cast_slice(&[data]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
        }
    }
    pub fn update(&mut self, queue: &wgpu::Queue, data: Data) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[data]));
    }
    pub fn bind_group_entry(&self, binding: u32) -> BindGroupEntry {
        wgpu::BindGroupEntry {
            binding,
            resource: self.buffer.as_entire_binding(),
        }
    }
}
#[allow(unused)]
pub type AlignedUniformData<Repr> = [Repr; 4];
pub struct AlignedUniform<Repr: Default + Copy + Clone + Pod + Zeroable> {
    pub uniform: Uniform<[Repr; 4]>,
    pub data: [Repr; 4],
}
impl<Repr: Default + Copy + Clone + Pod + Zeroable> AlignedUniform<Repr> {
    pub fn new(device: &wgpu::Device, data: Option<[Repr; 4]>) -> Self {
        let data = data.unwrap_or_default();
        Self {
            uniform: Uniform::new(device, data),
            data,
        }
    }
    pub fn update(&mut self, queue: &wgpu::Queue) {
        self.uniform.update(queue, self.data);
    }
    pub fn set_aspect(&mut self, index: usize, aspect: Repr) {
        self.data[index] = aspect;
    }
}
