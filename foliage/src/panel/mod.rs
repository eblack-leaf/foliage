use bevy_ecs::bundle::Bundle;
use bevy_ecs::prelude::Component;
use bytemuck::{Pod, Zeroable};
use wgpu::{
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor,
    PipelineLayoutDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderStages,
    TextureFormat, TextureSampleType, TextureViewDimension, VertexState, VertexStepMode,
};

use crate::ash::{RenderPhase, Renderer};
use crate::coordinate::area::CArea;
use crate::coordinate::layer::Layer;
use crate::coordinate::placement::Placement;
use crate::coordinate::position::{CPosition, Position};
use crate::coordinate::{Coordinates, LogicalContext};
use crate::ginkgo::{Ginkgo, Uniform};
use crate::instances::Instances;
use crate::{Elm, Render};

#[derive(Bundle)]
pub struct Panel {
    placement: Placement<LogicalContext>,
    intersection: PanelIntersection,
    corner_depths: PanelCornerDepths,
    corner_positions: PanelCornerPositions,
}
#[derive(Component, Copy, Clone)]
pub(crate) struct PanelIntersection(pub(crate) [Position<LogicalContext>; 2]);
#[derive(Component, Copy, Clone)]
pub struct PanelCornerDepths([f32; 4]);
impl PanelCornerDepths {
    pub fn set_top_left(&mut self, v: f32) {
        self.0[0] = v;
    }
    // ...
}
#[repr(C)]
#[derive(Component, Copy, Clone, Pod, Zeroable, Default, PartialEq)]
pub(crate) struct PanelCornerPositions(pub(crate) [Coordinates; 4]);
#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Default)]
pub struct Vertex {
    position: Coordinates,
}

impl Vertex {
    pub(crate) const fn new(position: Coordinates) -> Self {
        Self { position }
    }
}

pub(crate) const VERTICES: [Vertex; 6] = [
    Vertex::new(Coordinates::new(1f32, 0f32)),
    Vertex::new(Coordinates::new(0f32, 0f32)),
    Vertex::new(Coordinates::new(0f32, 1f32)),
    Vertex::new(Coordinates::new(1f32, 0f32)),
    Vertex::new(Coordinates::new(0f32, 1f32)),
    Vertex::new(Coordinates::new(1f32, 1f32)),
];

pub struct PanelResources {
    pipeline: RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
    instances: Instances<i32>,
}
pub(crate) const PANEL_CIRCLE_TEXTURE_DIMS: Coordinates = Coordinates::new(100.0, 100.0);
impl Render for PanelResources {
    type Vertex = Vertex;
    type DirectiveGroupKey = i32;
    const RENDER_PHASE: RenderPhase = RenderPhase::Alpha(0);

    fn create_resources(ginkgo: &Ginkgo) -> Self {
        let uniform = Uniform::new(ginkgo.context(), PanelCornerPositions::default());
        let shader = ginkgo.create_shader(include_wgsl!("panel.wgsl"));
        let vertex_buffer = ginkgo.create_vertex_buffer(VERTICES);
        let bind_group_layout = ginkgo.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("panel-bind-group-layout"),
            entries: &[
                Ginkgo::bind_group_layout_entry(0)
                    .at_stages(ShaderStages::VERTEX)
                    .uniform_entry(),
                Ginkgo::bind_group_layout_entry(1)
                    .at_stages(ShaderStages::VERTEX)
                    .uniform_entry(),
                Ginkgo::bind_group_layout_entry(2)
                    .at_stages(ShaderStages::FRAGMENT)
                    .texture_entry(
                        TextureViewDimension::D2,
                        TextureSampleType::Float { filterable: false },
                    ),
                Ginkgo::bind_group_layout_entry(3)
                    .at_stages(ShaderStages::FRAGMENT)
                    .sampler_entry(),
            ],
        });
        let (_texture, texture_view) = ginkgo.create_texture(
            TextureFormat::Rgba8Unorm,
            PANEL_CIRCLE_TEXTURE_DIMS,
            1,
            vec![].as_slice(),
        );
        let sampler = ginkgo.create_sampler();
        let bind_group = ginkgo.create_bind_group(&BindGroupDescriptor {
            label: Some("panel-bind-group"),
            layout: &bind_group_layout,
            entries: &[
                ginkgo.viewport_bind_group_entry(0),
                Ginkgo::uniform_bind_group_entry(&uniform, 1),
                Ginkgo::texture_bind_group_entry(&texture_view, 2),
                Ginkgo::sampler_bind_group_entry(&sampler, 3),
            ],
        });
        let pipeline_layout = ginkgo.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("panel-pipeline-layout-descriptor"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = ginkgo.create_pipeline(&RenderPipelineDescriptor {
            label: Some("panel-render-pipeline"),
            layout: Option::from(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vertex_entry",
                compilation_options: Default::default(),
                buffers: &[
                    Ginkgo::vertex_buffer_layout::<Vertex>(
                        VertexStepMode::Vertex,
                        &wgpu::vertex_attr_array![0 => Float32x2],
                    ),
                    Ginkgo::vertex_buffer_layout::<CPosition>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![1 => Float32x2],
                    ),
                    Ginkgo::vertex_buffer_layout::<CArea>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![2 => Float32x2],
                    ),
                    Ginkgo::vertex_buffer_layout::<Layer>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![3 => Float32],
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
        let instances = Instances::<i32>::new(4)
            .with_attribute::<CPosition>(ginkgo)
            .with_attribute::<CArea>(ginkgo)
            .with_attribute::<Layer>(ginkgo);
        Self {
            pipeline,
            vertex_buffer,
            bind_group_layout,
            bind_group,
            instances,
        }
    }

    type Extraction = ();

    fn extract(elm: &Elm) -> Self::Extraction {
        todo!()
    }

    fn prepare(renderer: &mut Renderer<Self>, extract: Self::Extraction) -> bool {
        todo!()
    }

    fn record(renderer: &mut Renderer<Self>, ginkgo: &Ginkgo) {
        todo!()
    }
}
