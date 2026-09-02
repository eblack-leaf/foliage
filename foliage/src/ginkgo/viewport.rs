//! The projection every renderer draws through.

use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{Buffer, BufferUsages, Device, Queue};

use crate::coordinate::Area;

/// The matrix taking logical pixels to clip space, held in the one uniform every pipeline binds.
///
/// Built from the *logical* area, so an instance is written in the same units an app stated it in
/// and the display's scale factor is applied by the rasteriser mapping clip space onto a surface
/// that was configured in physical pixels. Depth passes through untouched: a renderer hands over a
/// value already inside [`Depth`](crate::ginkgo::depth::Depth)'s range.
pub(crate) struct Viewport {
    buffer: Buffer,
}

impl Viewport {
    pub(crate) fn new(device: &Device, area: Area) -> Self {
        Self {
            buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some("viewport"),
                contents: bytemuck::cast_slice(&projection(area)),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            }),
        }
    }

    pub(crate) fn resize(&mut self, queue: &Queue, area: Area) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&projection(area)));
    }

    pub(crate) fn binding(&self) -> wgpu::BindingResource<'_> {
        self.buffer.as_entire_binding()
    }
}

/// Column-major, as WGSL reads a `mat4x4<f32>`.
///
/// `x` runs left to right across the area and `y` top to bottom, which is the sense
/// [`Position`](crate::Position) is written in, so the projection is what flips it rather than
/// every value that passes through it.
fn projection(area: Area) -> [[f32; 4]; 4] {
    [
        [2.0 / area.width, 0.0, 0.0, 0.0],
        [0.0, -2.0 / area.height, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [-1.0, 1.0, 0.0, 1.0],
    ]
}
