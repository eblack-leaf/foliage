use std::path::Path;

use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Component;
use bytemuck::{Pod, Zeroable};
use wgpu::{
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor,
    PipelineLayoutDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderStages,
    TextureFormat, TextureSampleType, TextureViewDimension, VertexState, VertexStepMode,
};

use crate::ash::{RenderDirectiveRecorder, RenderPhase, Renderer};
use crate::color::Color;
use crate::coordinate::area::{Area, GpuArea};
use crate::coordinate::layer::Layer;
use crate::coordinate::placement::Placement;
use crate::coordinate::position::{GpuPosition, Position};
use crate::coordinate::{Coordinates, LogicalContext};
use crate::differential::{Differentiable, RenderLink};
use crate::elm::RenderQueueHandle;
use crate::ginkgo::Ginkgo;
use crate::instances::Instances;
use crate::{Elm, Leaf, Render};

impl Leaf for Panel {
    fn attach(elm: &mut Elm) {
        elm.enable_differential::<Panel, GpuPosition>();
        elm.enable_differential::<Panel, GpuArea>();
        elm.enable_differential::<Panel, Layer>();
        elm.enable_differential::<Panel, Color>();
        elm.enable_differential::<Panel, CornerDepth>();
    }
}

#[derive(Bundle)]
pub struct Panel {
    render_link: RenderLink,
    pos: Position<LogicalContext>,
    area: Area<LogicalContext>,
    layer: Differentiable<Layer>,
    gpu_section: Differentiable<GpuPosition>,
    gpu_area: Differentiable<GpuArea>,
    color: Differentiable<Color>,
    corner_percent_rounded: CornerPercentRounded,
    corner_depths: Differentiable<CornerDepth>,
}
impl Panel {
    pub fn new(
        placement: Placement<LogicalContext>,
        corner_percent_rounded: CornerPercentRounded,
        color: Color,
    ) -> Self {
        Self {
            render_link: RenderLink::new::<Self>(),
            pos: placement.section.position,
            area: placement.section.area,
            layer: Differentiable::new(placement.layer),
            gpu_section: Differentiable::new(GpuPosition::default()),
            gpu_area: Differentiable::new(GpuArea::default()),
            color: Differentiable::new(color),
            corner_percent_rounded,
            corner_depths: Differentiable::new(CornerDepth::default()),
        }
    }
}
#[derive(Component, Copy, Clone, Default)]
pub struct CornerPercentRounded(pub(crate) [f32; 4]);

impl CornerPercentRounded {
    pub fn set_top_left(&mut self, v: f32) {
        self.0[0] = v.min(1.0).max(0.0);
    }
    // ...
}

#[repr(C)]
#[derive(Component, Copy, Clone, Pod, Zeroable, PartialEq, Default, Debug)]
pub(crate) struct CornerDepth(pub(crate) [f32; 4]);

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
    instances: Instances<Entity>,
}

pub(crate) const PANEL_CIRCLE_TEXTURE_DIMS: Coordinates = Coordinates::new(100.0, 100.0);

impl Render for Panel {
    type Vertex = Vertex;
    type DirectiveGroupKey = i32;
    const RENDER_PHASE: RenderPhase = RenderPhase::Alpha(0);
    type Resources = PanelResources;
    fn create_resources(ginkgo: &Ginkgo) -> Self::Resources {
        let shader = ginkgo.create_shader(include_wgsl!("panel.wgsl"));
        let vertex_buffer = ginkgo.create_vertex_buffer(VERTICES);
        let bind_group_layout = ginkgo.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("panel-bind-group-layout"),
            entries: &[
                Ginkgo::bind_group_layout_entry(0)
                    .at_stages(ShaderStages::VERTEX)
                    .uniform_entry(),
                Ginkgo::bind_group_layout_entry(1)
                    .at_stages(ShaderStages::FRAGMENT)
                    .texture_entry(
                        TextureViewDimension::D2,
                        TextureSampleType::Float { filterable: false },
                    ),
                Ginkgo::bind_group_layout_entry(2)
                    .at_stages(ShaderStages::FRAGMENT)
                    .sampler_entry(),
            ],
        });
        let tex_data =
            rmp_serde::from_slice::<Vec<u8>>(include_bytes!("circ.cov")).expect("corrupt-bytes");
        let (_texture, texture_view) = ginkgo.create_texture(
            TextureFormat::R8Unorm,
            PANEL_CIRCLE_TEXTURE_DIMS,
            1,
            tex_data.as_slice(),
        );
        let sampler = ginkgo.create_sampler();
        let bind_group = ginkgo.create_bind_group(&BindGroupDescriptor {
            label: Some("panel-bind-group"),
            layout: &bind_group_layout,
            entries: &[
                ginkgo.viewport_bind_group_entry(0),
                Ginkgo::texture_bind_group_entry(&texture_view, 1),
                Ginkgo::sampler_bind_group_entry(&sampler, 2),
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
                    Ginkgo::vertex_buffer_layout::<GpuPosition>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![1 => Float32x2],
                    ),
                    Ginkgo::vertex_buffer_layout::<GpuArea>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![2 => Float32x2],
                    ),
                    Ginkgo::vertex_buffer_layout::<Layer>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![3 => Float32],
                    ),
                    Ginkgo::vertex_buffer_layout::<Color>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![4 => Float32x4],
                    ),
                    Ginkgo::vertex_buffer_layout::<CornerDepth>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![5 => Float32x4],
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
        let instances = Instances::<Entity>::new(4)
            .with_attribute::<GpuPosition>(ginkgo)
            .with_attribute::<GpuArea>(ginkgo)
            .with_attribute::<Layer>(ginkgo)
            .with_attribute::<Color>(ginkgo)
            .with_attribute::<CornerDepth>(ginkgo);
        Self::Resources {
            pipeline,
            vertex_buffer,
            bind_group_layout,
            bind_group,
            instances,
        }
    }

    fn prepare(
        renderer: &mut Renderer<Self>,
        queue_handle: &mut RenderQueueHandle,
        ginkgo: &Ginkgo,
    ) -> bool {
        for entity in queue_handle.read_removes::<Self>() {
            renderer.resource_handle.instances.remove(entity);
        }
        for packet in queue_handle.read_adds::<Self, GpuPosition>() {
            renderer
                .resource_handle
                .instances
                .checked_write(packet.entity, packet.value);
        }
        for packet in queue_handle.read_adds::<Self, GpuArea>() {
            renderer
                .resource_handle
                .instances
                .checked_write(packet.entity, packet.value);
        }
        for packet in queue_handle.read_adds::<Self, Layer>() {
            renderer
                .resource_handle
                .instances
                .checked_write(packet.entity, packet.value);
        }
        for packet in queue_handle.read_adds::<Self, Color>() {
            renderer
                .resource_handle
                .instances
                .checked_write(packet.entity, packet.value);
        }
        for packet in queue_handle.read_adds::<Self, CornerDepth>() {
            renderer
                .resource_handle
                .instances
                .checked_write(packet.entity, packet.value);
        }
        let should_record = renderer.resource_handle.instances.resolve_changes(ginkgo);
        renderer.resource_handle.instances.write_cpu_to_gpu(ginkgo); // can combine w/ above?
        true
    }

    fn record(renderer: &mut Renderer<Self>, ginkgo: &Ginkgo) {
        let mut recorder = RenderDirectiveRecorder::new(ginkgo);
        if renderer.resource_handle.instances.num_instances() > 0 {
            recorder.0.set_pipeline(&renderer.resource_handle.pipeline);
            recorder
                .0
                .set_bind_group(0, &renderer.resource_handle.bind_group, &[]);
            recorder
                .0
                .set_vertex_buffer(0, renderer.resource_handle.vertex_buffer.slice(..));
            recorder.0.set_vertex_buffer(
                1,
                renderer
                    .resource_handle
                    .instances
                    .buffer::<GpuPosition>()
                    .slice(..),
            );
            recorder.0.set_vertex_buffer(
                2,
                renderer
                    .resource_handle
                    .instances
                    .buffer::<GpuArea>()
                    .slice(..),
            );
            recorder.0.set_vertex_buffer(
                3,
                renderer
                    .resource_handle
                    .instances
                    .buffer::<Layer>()
                    .slice(..),
            );
            recorder.0.set_vertex_buffer(
                4,
                renderer
                    .resource_handle
                    .instances
                    .buffer::<Color>()
                    .slice(..),
            );
            recorder.0.set_vertex_buffer(
                5,
                renderer
                    .resource_handle
                    .instances
                    .buffer::<CornerDepth>()
                    .slice(..),
            );
            recorder.0.draw(
                0..VERTICES.len() as u32,
                0..renderer.resource_handle.instances.num_instances(),
            );
        }
        let directive = recorder.finish();
        renderer.directive_manager.fill(0, directive);
    }
}

#[test]
fn make_cov() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("panel");
    let png = root.join("circ.png");
    let cov = root.join("circ.cov");
    Ginkgo::png_to_cov(png, cov);
}
