use std::collections::HashMap;

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
use crate::elm::{Elm, RenderQueueHandle};
use crate::ginkgo::Ginkgo;
use crate::instances::Instances;
use crate::Root;

#[derive(Bundle)]
pub struct Line {}
#[repr(C)]
#[derive(Component, Pod, Zeroable, Copy, Clone, Debug, Default)]
pub(crate) struct LinePoints {
    pub(crate) start: Coordinates,
    pub(crate) end: Coordinates,
}
// Offsets along edge line to where vertex is
#[repr(C)]
#[derive(Component, Pod, Zeroable, Copy, Clone, Debug, Default)]
pub(crate) struct LineVertexOffsets {
    pub(crate) top_left_offset: f32,
    pub(crate) top_right_offset: f32,
    pub(crate) bot_left_offset: f32,
    pub(crate) bot_right_offset: f32,
}
#[derive(Component, Copy, Clone)]
pub struct Weight(pub(crate) f32);
impl Weight {
    pub fn new(w: u32) -> Self {
        Self(w as f32)
    }
}
#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Default, Debug)]
pub struct LinePercent(pub(crate) f32);
// Edges and start/end without adjustments
#[repr(C)]
#[derive(Component, Pod, Zeroable, Copy, Clone, Debug, Default)]
pub(crate) struct LineDescriptor {
    pub(crate) right: LinePoints,
    pub(crate) left: LinePoints,
    pub(crate) start: LinePoints,
    pub(crate) end: LinePoints,
}
#[derive(Component)]
pub(crate) struct JoinedLines {
    pub(crate) joined: Vec<LineJoin>,
}
#[derive(Copy, Clone)]
pub enum LineJoinMethod {
    Start,
    Percent(LinePercent),
    End,
}
#[derive(Copy, Clone)]
pub(crate) struct LineJoinAngle {
    angle: f32,
}
#[derive(Copy, Clone)]
pub(crate) struct LineJoin {
    pub(crate) joined: Entity,
    pub(crate) method: LineJoinMethod,
    pub(crate) angle_to_joined: LineJoinAngle,
}
pub struct LineRenderResources {
    pipeline: RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
    instances: Instances<Entity>,
    render_layer_and_percent_map: HashMap<Entity, RenderLayerAndPercentDrawn>,
}
impl Root for Line {
    fn define(elm: &mut Elm) {
        todo!()
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
#[derive(Component, Copy, Clone)]
pub(crate) struct PercentDrawn {
    pub(crate) start: LinePercent,
    pub(crate) end: LinePercent,
}
#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Default, Debug)]
pub(crate) struct RenderLayerAndPercentDrawn {
    pub(crate) layer: RenderLayer,
    pub(crate) start: LinePercent,
    pub(crate) end: LinePercent,
}
impl Render for Line {
    type DirectiveGroupKey = i32;
    type Resources = LineRenderResources;

    fn create_resources(ginkgo: &Ginkgo) -> Self::Resources {
        let shader = ginkgo.create_shader(include_wgsl!("line.wgsl"));
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
                    Ginkgo::vertex_buffer_layout::<LineDescriptor>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![
                            1 => Float32x4,
                            2 => Float32x4,
                            3 => Float32x4,
                            4 => Float32x4,
                        ],
                    ),
                    Ginkgo::vertex_buffer_layout::<RenderLayerAndPercentDrawn>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![5 => Float32x3],
                    ),
                    Ginkgo::vertex_buffer_layout::<Color>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![6 => Float32x4],
                    ),
                    Ginkgo::vertex_buffer_layout::<LineVertexOffsets>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![7 => Float32x4],
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
        LineRenderResources {
            pipeline,
            vertex_buffer,
            bind_group_layout,
            bind_group,
            instances: Instances::new(1)
                .with_attribute::<LineDescriptor>(ginkgo)
                .with_attribute::<RenderLayerAndPercentDrawn>(ginkgo)
                .with_attribute::<Color>(ginkgo)
                .with_attribute::<LineVertexOffsets>(ginkgo),
            render_layer_and_percent_map: Default::default(),
        }
    }

    fn prepare(
        renderer: &mut Renderer<Self>,
        queue_handle: &mut RenderQueueHandle,
        ginkgo: &Ginkgo,
    ) {
        todo!()
    }

    fn draw<'a>(
        renderer: &'a Renderer<Self>,
        group_key: Self::DirectiveGroupKey,
        draw_range: DrawRange,
        render_pass: &mut RenderPass<'a>,
    ) {
        todo!()
    }
}
