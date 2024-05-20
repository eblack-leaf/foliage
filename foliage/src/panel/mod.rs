use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, IntoSystemConfigs, Query};
use bevy_ecs::query::{Changed, Or};
use bevy_ecs::system::Res;
use bytemuck::{Pod, Zeroable};
use wgpu::{
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor,
    PipelineLayoutDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderStages, VertexState,
    VertexStepMode,
};

use crate::ash::{RenderDirectiveRecorder, RenderPhase, Renderer};
use crate::color::Color;
use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::placement::Placement;
use crate::coordinate::position::Position;
use crate::coordinate::section::{GpuSection, Section};
use crate::coordinate::{Coordinates, LogicalContext};
use crate::differential::{Differential, RenderLink};
use crate::elm::{RenderQueueHandle, ScheduleMarkers};
use crate::ginkgo::{Ginkgo, ScaleFactor};
use crate::instances::Instances;
use crate::{Elm, Leaf, Render};

impl Leaf for Panel {
    fn attach(elm: &mut Elm) {
        elm.enable_differential::<Panel, GpuSection>();
        elm.enable_differential::<Panel, Layer>();
        elm.enable_differential::<Panel, Color>();
        elm.enable_differential::<Panel, CornerI>();
        elm.enable_differential::<Panel, CornerII>();
        elm.enable_differential::<Panel, CornerIII>();
        elm.enable_differential::<Panel, CornerIV>();
        elm.scheduler
            .main
            .add_systems(percent_rounded_to_corner.in_set(ScheduleMarkers::Config));
    }
}

#[derive(Bundle)]
pub struct Panel {
    render_link: RenderLink,
    pos: Position<LogicalContext>,
    area: Area<LogicalContext>,
    layer: Differential<Layer>,
    gpu_section: Differential<GpuSection>,
    color: Differential<Color>,
    panel_corner_rounding: PanelCornerRounding,
    corner_i: Differential<CornerI>,
    corner_ii: Differential<CornerII>,
    corner_iii: Differential<CornerIII>,
    corner_iv: Differential<CornerIV>,
}
impl Panel {
    pub fn new(
        placement: Placement<LogicalContext>,
        panel_corner_rounding: PanelCornerRounding,
        color: Color,
    ) -> Self {
        Self {
            render_link: RenderLink::new::<Self>(),
            pos: placement.section.position,
            area: placement.section.area,
            layer: Differential::new(placement.layer),
            gpu_section: Differential::new(GpuSection::default()),
            color: Differential::new(color),
            panel_corner_rounding,
            corner_i: Differential::new(CornerI::default()),
            corner_ii: Differential::new(CornerII::default()),
            corner_iii: Differential::new(CornerIII::default()),
            corner_iv: Differential::new(CornerIV::default()),
        }
    }
}
#[derive(Component, Copy, Clone, Default)]
pub struct PanelCornerRounding(pub(crate) [f32; 4]);

impl PanelCornerRounding {
    pub fn all(v: f32) -> Self {
        let v = v.max(0.0).min(1.0);
        Self([v; 4])
    }
    pub fn top(v: f32) -> Self {
        let v = v.max(0.0).min(1.0);
        Self([v, v, 0.0, 0.0])
    }
    pub fn bottom(v: f32) -> Self {
        let v = v.max(0.0).min(1.0);
        Self([0.0, 0.0, v, v])
    }
    // ...
}
fn percent_rounded_to_corner(
    mut query: Query<
        (
            &mut CornerI,
            &mut CornerII,
            &mut CornerIII,
            &mut CornerIV,
            &mut PanelCornerRounding,
            &Position<LogicalContext>,
            &Area<LogicalContext>,
        ),
        Or<(
            Changed<PanelCornerRounding>,
            Changed<Position<LogicalContext>>,
            Changed<Area<LogicalContext>>,
        )>,
    >,
    scale_factor: Res<ScaleFactor>,
) {
    for (mut i, mut ii, mut iii, mut iv, mut percents, pos, area) in query.iter_mut() {
        let section = Section::new(*pos, *area).to_device(scale_factor.value());
        let half_smallest = section.height().min(section.width()) / 2f32;
        let delta = half_smallest * percents.0[0];
        let position = Position::numerical((section.right() - delta, section.y() + delta));
        *i = CornerI([position.x(), position.y(), delta]);
        let delta = half_smallest * percents.0[1];
        let position = Position::numerical((section.x() + delta, section.y() + delta));
        *ii = CornerII([position.x(), position.y(), delta]);
        let delta = half_smallest * percents.0[2];
        let position = Position::numerical((section.x() + delta, section.bottom() - delta));
        *iii = CornerIII([position.x(), position.y(), delta]);
        let delta = half_smallest * percents.0[3];
        let position = Position::numerical((section.right() - delta, section.bottom() - delta));
        *iv = CornerIV([position.x(), position.y(), delta]);
    }
}
#[repr(C)]
#[derive(Component, Copy, Clone, Pod, Zeroable, PartialEq, Default, Debug)]
pub(crate) struct CornerI(pub(crate) [f32; 3]);
#[repr(C)]
#[derive(Component, Copy, Clone, Pod, Zeroable, PartialEq, Default, Debug)]
pub(crate) struct CornerII(pub(crate) [f32; 3]);
#[repr(C)]
#[derive(Component, Copy, Clone, Pod, Zeroable, PartialEq, Default, Debug)]
pub(crate) struct CornerIII(pub(crate) [f32; 3]);
#[repr(C)]
#[derive(Component, Copy, Clone, Pod, Zeroable, PartialEq, Default, Debug)]
pub(crate) struct CornerIV(pub(crate) [f32; 3]);
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

impl Render for Panel {
    type DirectiveGroupKey = i32;
    const RENDER_PHASE: RenderPhase = RenderPhase::Alpha(0);
    type Resources = PanelResources;
    fn create_resources(ginkgo: &Ginkgo) -> Self::Resources {
        let shader = ginkgo.create_shader(include_wgsl!("panel.wgsl"));
        let vertex_buffer = ginkgo.create_vertex_buffer(VERTICES);
        let bind_group_layout = ginkgo.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("panel-bind-group-layout"),
            entries: &[Ginkgo::bind_group_layout_entry(0)
                .at_stages(ShaderStages::VERTEX)
                .uniform_entry()],
        });
        let bind_group = ginkgo.create_bind_group(&BindGroupDescriptor {
            label: Some("panel-bind-group"),
            layout: &bind_group_layout,
            entries: &[ginkgo.viewport_bind_group_entry(0)],
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
                    Ginkgo::vertex_buffer_layout::<GpuSection>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![1 => Float32x4],
                    ),
                    Ginkgo::vertex_buffer_layout::<Layer>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![2 => Float32],
                    ),
                    Ginkgo::vertex_buffer_layout::<Color>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![3 => Float32x4],
                    ),
                    Ginkgo::vertex_buffer_layout::<CornerI>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![4 => Float32x3],
                    ),
                    Ginkgo::vertex_buffer_layout::<CornerII>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![5 => Float32x3],
                    ),
                    Ginkgo::vertex_buffer_layout::<CornerIII>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![6 => Float32x3],
                    ),
                    Ginkgo::vertex_buffer_layout::<CornerIV>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![7 => Float32x3],
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
            .with_attribute::<GpuSection>(ginkgo)
            .with_attribute::<Layer>(ginkgo)
            .with_attribute::<Color>(ginkgo)
            .with_attribute::<CornerI>(ginkgo)
            .with_attribute::<CornerII>(ginkgo)
            .with_attribute::<CornerIII>(ginkgo)
            .with_attribute::<CornerIV>(ginkgo);
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
        for packet in queue_handle.read_adds::<Self, GpuSection>() {
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
        for packet in queue_handle.read_adds::<Self, CornerI>() {
            renderer
                .resource_handle
                .instances
                .checked_write(packet.entity, packet.value);
        }
        for packet in queue_handle.read_adds::<Self, CornerII>() {
            renderer
                .resource_handle
                .instances
                .checked_write(packet.entity, packet.value);
        }
        for packet in queue_handle.read_adds::<Self, CornerIII>() {
            renderer
                .resource_handle
                .instances
                .checked_write(packet.entity, packet.value);
        }
        for packet in queue_handle.read_adds::<Self, CornerIV>() {
            renderer
                .resource_handle
                .instances
                .checked_write(packet.entity, packet.value);
        }
        let should_record = renderer.resource_handle.instances.resolve_changes(ginkgo);
        should_record
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
                    .buffer::<GpuSection>()
                    .slice(..),
            );
            recorder.0.set_vertex_buffer(
                2,
                renderer
                    .resource_handle
                    .instances
                    .buffer::<Layer>()
                    .slice(..),
            );
            recorder.0.set_vertex_buffer(
                3,
                renderer
                    .resource_handle
                    .instances
                    .buffer::<Color>()
                    .slice(..),
            );
            recorder.0.set_vertex_buffer(
                4,
                renderer
                    .resource_handle
                    .instances
                    .buffer::<CornerI>()
                    .slice(..),
            );
            recorder.0.set_vertex_buffer(
                5,
                renderer
                    .resource_handle
                    .instances
                    .buffer::<CornerII>()
                    .slice(..),
            );
            recorder.0.set_vertex_buffer(
                6,
                renderer
                    .resource_handle
                    .instances
                    .buffer::<CornerIII>()
                    .slice(..),
            );
            recorder.0.set_vertex_buffer(
                7,
                renderer
                    .resource_handle
                    .instances
                    .buffer::<CornerIV>()
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
