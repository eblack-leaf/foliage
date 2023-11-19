use crate::ash::render::{Render, RenderPhase};
use crate::ash::render_packet::RenderPacket;
use crate::ash::renderer::{RenderPackage, RenderRecordBehavior};
use crate::color::Color;
use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::{Position, RawPosition};
use crate::coordinate::InterfaceContext;
use crate::differential::DifferentialBundle;
use crate::differential_enable;
use crate::elm::{Elm, Leaf};
use crate::ginkgo::Ginkgo;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

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
    position: RawPosition,
    texture_coordinates: RawPosition,
}
impl Vertex {
    const fn new(position: RawPosition, texture_coordinates: RawPosition) -> Self {
        Self {
            position,
            texture_coordinates,
        }
    }
}
const VERTICES: Vec<Vertex> = vec![];
pub struct PanelRenderResources {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
}
impl Render for Panel {
    type Resources = PanelRenderResources;
    type RenderPackage = ();
    const RENDER_PHASE: RenderPhase = RenderPhase::Opaque;

    fn resources(ginkgo: &Ginkgo) -> Self::Resources {
        let shader = ginkgo
            .device()
            .create_shader_module(wgpu::include_wgsl!("panel.wgsl"));
        let pipeline_layout =
            ginkgo
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("panel-render-pipeline-layout"),
                    bind_group_layouts: &[],
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
                    buffers: &[],
                },
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                fragment: None,
                multiview: None,
            });
        let vertex_buffer = ginkgo
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("panel-vertex-buffer"),
                contents: bytemuck::cast_slice(&VERTICES),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
        PanelRenderResources {
            pipeline,
            vertex_buffer,
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
