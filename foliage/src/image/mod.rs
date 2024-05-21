use crate::ash::{Render, RenderPhase, Renderer};
use crate::coordinate::layer::Layer;
use crate::coordinate::section::GpuSection;
use crate::coordinate::Coordinates;
use crate::elm::{Elm, RenderQueueHandle};
use crate::ginkgo::texture::TextureCoordinates;
use crate::ginkgo::Ginkgo;
use crate::Leaf;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::prelude::Component;
use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;
use wgpu::{
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor,
    PipelineLayoutDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderStages,
    TextureSampleType, TextureView, TextureViewDimension, VertexState, VertexStepMode,
};

#[derive(Bundle)]
pub struct Image {
    id: ImageId,
}
impl Leaf for Image {
    fn attach(elm: &mut Elm) {
        todo!()
    }
}
#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Default)]
pub struct Vertex {
    position: Coordinates,
    tx_index: Coordinates,
}

impl Vertex {
    pub(crate) const fn new(position: Coordinates, tx_index: Coordinates) -> Self {
        Self { position, tx_index }
    }
}

pub(crate) const VERTICES: [Vertex; 6] = [
    Vertex::new(Coordinates::new(1f32, 0f32), Coordinates::new(0f32, 2f32)),
    Vertex::new(Coordinates::new(0f32, 0f32), Coordinates::new(0f32, 1f32)),
    Vertex::new(Coordinates::new(0f32, 1f32), Coordinates::new(0f32, 3f32)),
    Vertex::new(Coordinates::new(1f32, 0f32), Coordinates::new(0f32, 2f32)),
    Vertex::new(Coordinates::new(0f32, 1f32), Coordinates::new(0f32, 3f32)),
    Vertex::new(Coordinates::new(1f32, 1f32), Coordinates::new(2f32, 3f32)),
];
#[derive(Copy, Clone, Component, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct ImageId(pub i32);
pub struct ImageResources {
    pipeline: RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    bind_group: BindGroup,
    group_layout: BindGroupLayout,
    images: HashMap<ImageId, ImageGroup>,
}
pub(crate) struct ImageGroup {
    view: TextureView,
    bind_group: BindGroup,
}
impl Render for Image {
    type DirectiveGroupKey = ImageId;
    const RENDER_PHASE: RenderPhase = RenderPhase::Opaque;
    type Resources = ImageResources;

    fn create_resources(ginkgo: &Ginkgo) -> Self::Resources {
        let shader = ginkgo.create_shader(include_wgsl!("image.wgsl"));
        let group_layout = ginkgo.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("image-group-bind-group-layout"),
            entries: &[Ginkgo::bind_group_layout_entry(0)
                .at_stages(ShaderStages::FRAGMENT)
                .texture_entry(
                    TextureViewDimension::D2,
                    TextureSampleType::Float { filterable: false },
                )],
        });
        let bind_group_layout = ginkgo.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("image-bind-group-layout"),
            entries: &[
                Ginkgo::bind_group_layout_entry(0)
                    .at_stages(ShaderStages::VERTEX)
                    .uniform_entry(),
                Ginkgo::bind_group_layout_entry(1)
                    .at_stages(ShaderStages::FRAGMENT)
                    .sampler_entry(),
            ],
        });
        let sampler = ginkgo.create_sampler();
        let bind_group = ginkgo.create_bind_group(&BindGroupDescriptor {
            label: Some("image-bind-group"),
            layout: &bind_group_layout,
            entries: &[
                ginkgo.viewport_bind_group_entry(0),
                Ginkgo::sampler_bind_group_entry(&sampler, 1),
            ],
        });
        let pipeline_layout = ginkgo.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("image-pipeline-layout"),
            bind_group_layouts: &[&bind_group_layout, &group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = ginkgo.create_pipeline(&RenderPipelineDescriptor {
            label: Some("image-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vertex_entry",
                compilation_options: Default::default(),
                buffers: &[
                    Ginkgo::vertex_buffer_layout::<Vertex>(
                        VertexStepMode::Vertex,
                        &wgpu::vertex_attr_array![0 => Float32x4],
                    ),
                    Ginkgo::vertex_buffer_layout::<GpuSection>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![1 => Float32x4],
                    ),
                    Ginkgo::vertex_buffer_layout::<Layer>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![2 => Float32],
                    ),
                    Ginkgo::vertex_buffer_layout::<TextureCoordinates>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![3 => Float32x4],
                    ),
                ],
            },
            primitive: Ginkgo::triangle_list_primitive(),
            depth_stencil: ginkgo.depth_stencil_state(),
            multisample: ginkgo.msaa_state(),
            fragment: Ginkgo::fragment_state(
                &shader,
                "fragment_entry",
                &ginkgo.alpha_color_target_state(),
            ),
            multiview: None,
        });
        ImageResources {
            pipeline,
            vertex_buffer: ginkgo.create_vertex_buffer(VERTICES),
            bind_group,
            group_layout,
            images: HashMap::new(),
        }
    }

    fn prepare(
        renderer: &mut Renderer<Self>,
        queue_handle: &mut RenderQueueHandle,
        ginkgo: &Ginkgo,
    ) -> bool {
        todo!()
    }

    fn record(renderer: &mut Renderer<Self>, ginkgo: &Ginkgo) {
        todo!()
    }
}
