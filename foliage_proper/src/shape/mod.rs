use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Component;
use bytemuck::{Pod, Zeroable};
use wgpu::{
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor,
    PipelineLayoutDescriptor, RenderPass, RenderPipeline, RenderPipelineDescriptor, ShaderStages,
    VertexState, VertexStepMode,
};

use crate::ash::{DrawRange, Render, Renderer};
use crate::color::Color;
use crate::coordinate::elevation::RenderLayer;
use crate::coordinate::Coordinates;
use crate::differential::{Differential, RenderLink};
use crate::elm::{Elm, RenderQueueHandle};
use crate::ginkgo::Ginkgo;
use crate::instances::Instances;
use crate::Root;

#[derive(Bundle)]
pub struct Shape {
    render_link: RenderLink,
    descriptor: Differential<ShapeDescriptor>,
    color: Differential<Color>,
    layer: Differential<RenderLayer>,
}
pub struct LineDescriptor {}
pub struct TriangleDescriptor {}
pub struct RectDescriptor {}
pub enum ShapeForm {
    Line(LineDescriptor),
    Triangle(TriangleDescriptor),
    Rect(RectDescriptor),
}
impl Shape {
    pub fn new(desc: ShapeDescriptor, color: Color) -> Self {
        Self {
            render_link: RenderLink::new::<Self>(),
            descriptor: Differential::new(desc),
            color: Differential::new(color),
            layer: Differential::new(RenderLayer::default()),
        }
    }
}
#[repr(C)]
#[derive(Component, Pod, Zeroable, Copy, Clone, Debug, Default, PartialEq)]
pub struct EdgePoints {
    pub start: Coordinates,
    pub end: Coordinates,
}
impl EdgePoints {
    pub fn new(start: Coordinates, end: Coordinates) -> Self {
        Self { start, end }
    }
}
#[repr(C)]
#[derive(Component, Pod, Zeroable, Copy, Clone, Debug, Default, PartialEq)]
pub struct ShapeDescriptor {
    pub left: EdgePoints,
    pub top: EdgePoints,
    pub right: EdgePoints,
    pub bot: EdgePoints,
}
impl ShapeDescriptor {
    pub fn new(left: EdgePoints, top: EdgePoints, right: EdgePoints, bot: EdgePoints) -> Self {
        Self {
            left,
            top,
            right,
            bot,
        }
    }
}
pub struct ShapeRenderResources {
    pipeline: RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
    instances: Instances<Entity>,
}
impl Root for Shape {
    fn define(elm: &mut Elm) {
        elm.enable_differential::<Self, ShapeDescriptor>();
        elm.enable_differential::<Self, Color>();
        elm.enable_differential::<Self, RenderLayer>();
    }
}
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
impl Render for Shape {
    type DirectiveGroupKey = i32;
    type Resources = ShapeRenderResources;

    fn create_resources(ginkgo: &Ginkgo) -> Self::Resources {
        let shader = ginkgo.create_shader(include_wgsl!("shape.wgsl"));
        let vertex_buffer = ginkgo.create_vertex_buffer(VERTICES);
        let bind_group_layout = ginkgo.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("line-bind-group-layout"),
            entries: &[Ginkgo::bind_group_layout_entry(0)
                .at_stages(ShaderStages::VERTEX)
                .uniform_entry()],
        });
        let bind_group = ginkgo.create_bind_group(&BindGroupDescriptor {
            label: Some("line-bind-group"),
            layout: &bind_group_layout,
            entries: &[ginkgo.viewport_bind_group_entry(0)],
        });
        let pipeline_layout = ginkgo.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("line-render-pipeline"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = ginkgo.create_pipeline(&RenderPipelineDescriptor {
            label: Some("line-renderer-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vertex_entry",
                compilation_options: Default::default(),
                buffers: &[
                    Ginkgo::vertex_buffer_layout::<Vertex>(
                        VertexStepMode::Vertex,
                        &wgpu::vertex_attr_array![0 => Float32x2],
                    ),
                    Ginkgo::vertex_buffer_layout::<ShapeDescriptor>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![
                            1 => Float32x4,
                            2 => Float32x4,
                            3 => Float32x4,
                            4 => Float32x4,
                        ],
                    ),
                    Ginkgo::vertex_buffer_layout::<RenderLayer>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![5 => Float32],
                    ),
                    Ginkgo::vertex_buffer_layout::<Color>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![6 => Float32x4],
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
        ShapeRenderResources {
            pipeline,
            vertex_buffer,
            bind_group_layout,
            bind_group,
            instances: Instances::new(1)
                .with_attribute::<ShapeDescriptor>(ginkgo)
                .with_attribute::<RenderLayer>(ginkgo)
                .with_attribute::<Color>(ginkgo),
        }
    }

    fn prepare(
        renderer: &mut Renderer<Self>,
        queue_handle: &mut RenderQueueHandle,
        ginkgo: &Ginkgo,
    ) {
        for entity in queue_handle.read_removes::<Self>() {
            renderer.resource_handle.instances.queue_remove(entity);
        }
        for packet in queue_handle.read_adds::<Self, ShapeDescriptor>() {
            renderer.associate_alpha_pointer(0, 0);
            renderer
                .resource_handle
                .instances
                .checked_write(packet.entity, packet.value);
        }
        for packet in queue_handle.read_adds::<Self, RenderLayer>() {
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
        if renderer.resource_handle.instances.resolve_changes(ginkgo) {
            renderer
                .node_manager
                .set_nodes(0, renderer.resource_handle.instances.render_nodes());
        }
    }

    fn draw<'a>(
        renderer: &'a Renderer<Self>,
        group_key: Self::DirectiveGroupKey,
        draw_range: DrawRange,
        render_pass: &mut RenderPass<'a>,
    ) {
        render_pass.set_pipeline(&renderer.resource_handle.pipeline);
        render_pass.set_bind_group(0, &renderer.resource_handle.bind_group, &[]);
        render_pass.set_vertex_buffer(0, renderer.resource_handle.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(
            1,
            renderer
                .resource_handle
                .instances
                .buffer::<ShapeDescriptor>()
                .slice(..),
        );
        render_pass.set_vertex_buffer(
            2,
            renderer
                .resource_handle
                .instances
                .buffer::<RenderLayer>()
                .slice(..),
        );
        render_pass.set_vertex_buffer(
            3,
            renderer
                .resource_handle
                .instances
                .buffer::<Color>()
                .slice(..),
        );
        render_pass.draw(0..VERTICES.len() as u32, draw_range.start..draw_range.end);
    }
}
