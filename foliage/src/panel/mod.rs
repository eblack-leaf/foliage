use crate::ash::{RenderPhase, Renderer};
use crate::aspen::Aspen;
use crate::ginkgo::Ginkgo;
use crate::{Coordinates, Elm, Render};
use bevy_ecs::prelude::Entity;
use bytemuck::{Pod, Zeroable};
use wgpu::{
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor,
    PipelineLayoutDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderModule, VertexState,
};

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
    type DirectiveGroupKey = Entity;
    const RENDER_PHASE: RenderPhase = RenderPhase::Alpha(0);

    fn create_resources(ginkgo: &Ginkgo) -> Self {
        let shader = Aspen::create_shader(ginkgo, include_wgsl!("panel.wgsl"));
        let vertex_buffer = Aspen::create_vertex_buffer::<Self, _>(ginkgo, VERTICES);
        let bind_group_layout = Aspen::bind_group_layout(
            ginkgo,
            &BindGroupLayoutDescriptor {
                label: Some("panel-bind-group-layout"),
                entries: &[
                    Aspen::vertex_uniform_bind_group_layout_entry(0),
                    // uniforms
                    // texture (pre-solved)
                ],
            },
        );
        let bind_group = Aspen::bind_group(
            ginkgo,
            &BindGroupDescriptor {
                label: Some("panel-bind-group"),
                layout: &bind_group_layout,
                entries: &[
                    ginkgo.viewport_bind_group_entry(0),
                    // Aspen::texture_bind_group_entry(1),
                ],
            },
        );
        let pl_layout = Aspen::create_pipeline_layout(
            ginkgo,
            &PipelineLayoutDescriptor {
                label: Some("panel-pipeline-layout-descriptor"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            },
        );
        let pipeline = Aspen::create_pipeline(
            ginkgo,
            &RenderPipelineDescriptor {
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
                primitive: Aspen::triangle_list_primitive(),
                depth_stencil: ginkgo.depth_stencil_state(),
                multisample: ginkgo.msaa_state(),
                fragment: Aspen::fragment_state(
                    &shader,
                    "fragment_entry",
                    &ginkgo.alpha_color_target_state(),
                ),
                multiview: None,
            },
        );
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
