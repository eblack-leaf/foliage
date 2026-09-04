//! The panel renderer: a filled, rounded rectangle per instance.

use core::ops::Range;

use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    Buffer, BufferUsages, PipelineLayoutDescriptor, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, VertexBufferLayout,
    VertexState, VertexStepMode,
};

use crate::ash::CORNERS;
use crate::ash::instances::Instances;
use crate::ginkgo::Ginkgo;
use crate::panel::PanelInstance;

/// The pipeline, the quad it is drawn over, and what the GPU is holding.
///
/// The instance is [`PanelInstance`] itself. What extraction compares and what the vertex buffer
/// holds are the same bytes, so there is nothing here that restates the panel's shape.
pub(crate) struct Panels {
    pipeline: RenderPipeline,
    corners: Buffer,
    pub(crate) instances: Instances<PanelInstance>,
}

impl Panels {
    pub(crate) fn new(ginkgo: &Ginkgo) -> Self {
        let device = ginkgo.device();
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("panel"),
            source: ShaderSource::Wgsl(
                format!("{}{}", include_str!("sdf.wgsl"), include_str!("panel.wgsl")).into(),
            ),
        });
        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("panel"),
            bind_group_layouts: &[Some(ginkgo.viewport_layout())],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("panel"),
            layout: Some(&layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vertex_entry"),
                compilation_options: Default::default(),
                buffers: &[
                    Some(VertexBufferLayout {
                        array_stride: size_of::<[f32; 2]>() as u64,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                    }),
                    Some(VertexBufferLayout {
                        array_stride: size_of::<PanelInstance>() as u64,
                        step_mode: VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![
                            1 => Float32x4,
                            2 => Float32x4,
                            3 => Float32x4,
                        ],
                    }),
                    Some(VertexBufferLayout {
                        array_stride: size_of::<f32>() as u64,
                        step_mode: VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![4 => Float32],
                    }),
                ],
            },
            primitive: Ginkgo::triangles(),
            depth_stencil: Ginkgo::depth_state(),
            multisample: Ginkgo::samples(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment_entry"),
                compilation_options: Default::default(),
                targets: &ginkgo.blending(),
            }),
            multiview_mask: None,
            cache: None,
        });
        Self {
            pipeline,
            corners: device.create_buffer_init(&BufferInitDescriptor {
                label: Some("panel-corners"),
                contents: bytemuck::cast_slice(&CORNERS),
                usage: BufferUsages::VERTEX,
            }),
            instances: Instances::new(device, "panel", 16),
        }
    }

    /// Draws one run of what is held.
    ///
    /// The range is a span of the stack, and which spans there are and what order they go in is
    /// [`Ash`](crate::ash::Ash)'s: a renderer draws the slots it is asked for and decides nothing
    /// about when it is asked.
    pub(crate) fn draw(&self, pass: &mut RenderPass<'_>, span: Range<u32>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.corners.slice(..));
        pass.set_vertex_buffer(1, self.instances.data());
        pass.set_vertex_buffer(2, self.instances.depths());
        pass.draw(0..CORNERS.len() as u32, span);
    }
}
