use bevy_ecs::prelude::Entity;
use bytemuck::{Pod, Zeroable};
use wgpu::{include_wgsl, BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor, PipelineLayoutDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderModule, VertexState, ShaderStages};

use crate::ash::{RenderPhase, Renderer};
use crate::ginkgo::Ginkgo;
use crate::{Coordinates, Elm, Render};

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
    shader: ShaderModule,
}

impl Render for PanelResources {
    type Vertex = Vertex;
    type DirectiveGroupKey = i32;
    const RENDER_PHASE: RenderPhase = RenderPhase::Alpha(0);

    fn create_resources(ginkgo: &Ginkgo) -> Self {
        let shader = ginkgo.create_shader(include_wgsl!("panel.wgsl"));
        let vertex_buffer = ginkgo.create_vertex_buffer(VERTICES);
        let bind_group_layout = ginkgo.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("panel-bind-group-layout"),
            entries: &[
                Ginkgo::bind_group_layout_entry(0).at_stages(ShaderStages::VERTEX).uniform_entry(),
                // uniforms
                // texture (pre-solved)
            ],
        });
        let bind_group = ginkgo.create_bind_group(&BindGroupDescriptor {
            label: Some("panel-bind-group"),
            layout: &bind_group_layout,
            entries: &[
                ginkgo.viewport_bind_group_entry(0),
                // Ginkgo::texture_bind_group_entry(1),
            ],
        });
        let pl_layout = ginkgo.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("panel-pipeline-layout-descriptor"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = ginkgo.create_pipeline(&RenderPipelineDescriptor {
            label: Some("panel-render-pipeline"),
            layout: Option::from(&pl_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vertex_entry",
                compilation_options: Default::default(),
                buffers: &[
                    // vertex-buffer-layout
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
        Self {
            pipeline,
            vertex_buffer,
            bind_group_layout,
            bind_group,
            shader,
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
