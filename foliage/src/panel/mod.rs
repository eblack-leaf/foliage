use crate::ash::render::{Render, RenderPhase};
use crate::ash::render_packet::RenderPacket;
use crate::ash::renderer::{RenderPackage, RenderRecordBehavior};
use crate::color::Color;
use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::{CPosition, Position};
use crate::coordinate::InterfaceContext;
use crate::differential::DifferentialBundle;
use crate::differential_enable;
use crate::elm::{Elm, Leaf};
use crate::ginkgo::Ginkgo;
use crate::texture::TextureCoordinates;
use bytemuck::{Pod, Zeroable};

pub struct Panel {
    position: DifferentialBundle<Position<InterfaceContext>>,
    area: DifferentialBundle<Area<InterfaceContext>>,
    layer: DifferentialBundle<Layer>,
    color: DifferentialBundle<Color>,
}
impl Leaf for Panel {
    fn attach(elm: &mut Elm) {
        differential_enable!(
            elm,
            Position<InterfaceContext>,
            Area<InterfaceContext>,
            Layer,
            Color
        );
    }
}
#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Default)]
struct Vertex {
    position: CPosition,
    texture_coordinates: TextureCoordinates,
}
impl Vertex {
    const fn new(position: CPosition, texture_coordinates: TextureCoordinates) -> Self {
        Self {
            position,
            texture_coordinates,
        }
    }
}
const VERTICES: [Vertex; 6] = [
    Vertex::new(
        CPosition::new(0f32, 0f32),
        TextureCoordinates::new(0f32, 0f32),
    ),
    Vertex::new(
        CPosition::new(1f32, 0f32),
        TextureCoordinates::new(1f32, 0f32),
    ),
    Vertex::new(
        CPosition::new(0f32, 1f32),
        TextureCoordinates::new(0f32, 1f32),
    ),
    Vertex::new(
        CPosition::new(1f32, 0f32),
        TextureCoordinates::new(1f32, 0f32),
    ),
    Vertex::new(
        CPosition::new(0f32, 1f32),
        TextureCoordinates::new(0f32, 1f32),
    ),
    Vertex::new(
        CPosition::new(1f32, 1f32),
        TextureCoordinates::new(1f32, 1f32),
    ),
];
pub struct PanelRenderResources {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}
impl PanelRenderResources {
    const TEXTURE_DIMENSION: u32 = 100;
}
impl Render for Panel {
    type Resources = PanelRenderResources;
    type RenderPackage = ();
    const RENDER_PHASE: RenderPhase = RenderPhase::Opaque;

    fn resources(ginkgo: &Ginkgo) -> Self::Resources {
        let shader = ginkgo
            .device()
            .create_shader_module(wgpu::include_wgsl!("panel.wgsl"));
        let (texture, view) = ginkgo.texture_r8unorm_d2(
            PanelRenderResources::TEXTURE_DIMENSION,
            PanelRenderResources::TEXTURE_DIMENSION,
            include_bytes!("panel-texture.cov"),
        );
        let sampler = ginkgo
            .device()
            .create_sampler(&wgpu::SamplerDescriptor::default());
        let bind_group_layout =
            ginkgo
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("panel-render-pipeline-bind-group-layout"),
                    entries: &[
                        Ginkgo::vertex_uniform_bind_group_layout_entry(0),
                        Ginkgo::texture_d2_bind_group_entry(1),
                        Ginkgo::sampler_bind_group_layout_entry(2),
                    ],
                });
        let bind_group = ginkgo
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("panel-render-pipeline-bind-group"),
                layout: &bind_group_layout,
                entries: &[
                    ginkgo.viewport_bind_group_entry(0),
                    Ginkgo::texture_bind_group_entry(&view, 1),
                    Ginkgo::sampler_bind_group_entry(&sampler, 2),
                ],
            });
        let pipeline_layout =
            ginkgo
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("panel-render-pipeline-layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });
        let pipeline = ginkgo
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("panel-render-pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vertex_entry",
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
                    }],
                },
                primitive: Ginkgo::triangle_list_primitive(),
                depth_stencil: ginkgo.depth_stencil_state(),
                multisample: ginkgo.msaa_multisample_state(),
                fragment: ginkgo.fragment_state(
                    &shader,
                    "fragment_entry",
                    &ginkgo.alpha_color_target_state(),
                ),
                multiview: None,
            });
        let vertex_buffer = ginkgo.vertex_buffer_with_data(&VERTICES, "panel-vertex-buffer");
        PanelRenderResources {
            pipeline,
            vertex_buffer,
            bind_group,
            texture,
            view,
            sampler,
        }
    }

    fn package(
        ginkgo: &Ginkgo,
        resources: &Self::Resources,
        render_packet: RenderPacket,
    ) -> Self::RenderPackage {
        todo!()
    }

    fn prepare_package(
        ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        package: &mut RenderPackage<Self>,
        render_packet: RenderPacket,
    ) {
        todo!()
    }

    fn prepare_resources(
        resources: &mut Self::Resources,
        ginkgo: &Ginkgo,
        per_renderer_record_hook: &mut bool,
    ) {
        todo!()
    }

    fn record_behavior() -> RenderRecordBehavior<Self> {
        todo!()
    }
}
