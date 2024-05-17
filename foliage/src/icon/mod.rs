use std::collections::HashMap;

use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Component;
use bytemuck::{Pod, Zeroable};
use wgpu::{
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor,
    PipelineLayoutDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderStages,
    TextureSampleType, TextureViewDimension, VertexState, VertexStepMode,
};

use crate::ash::{Render, RenderPhase, Renderer};
use crate::color::Color;
use crate::coordinate::layer::Layer;
use crate::coordinate::section::{GpuSection, Section};
use crate::coordinate::{Coordinates, LogicalContext};
use crate::differential::{Differential, RenderLink};
use crate::elm::RenderQueueHandle;
use crate::ginkgo::Ginkgo;
use crate::instances::Instances;

#[derive(Bundle)]
pub struct Icon {
    link: RenderLink,
    section: Section<LogicalContext>,
    layer: Differential<Layer>,
    gpu_section: Differential<GpuSection>,
    id: Differential<IconId>,
    color: Differential<Color>,
}
impl Icon {
    pub const SCALE: Coordinates = Coordinates::new(24f32, 24f32);
    pub fn new<I: Into<IconId>, L: Into<Layer>>(id: I, color: Color, l: L) -> Self {
        Self {
            link: RenderLink::new::<Icon>(),
            section: Default::default(),
            layer: Differential::new(l.into()),
            gpu_section: Differential::new(GpuSection::default()),
            id: Differential::new(id.into()),
            color: Differential::new(color),
        }
    }
}
#[derive(Hash, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Component)]
pub struct IconId(pub i32);
impl From<i32> for IconId {
    fn from(value: i32) -> Self {
        Self(value)
    }
}
pub(crate) struct IconGroup {
    bind_group: BindGroup,
    instances: Instances<Entity>,
}
pub(crate) struct IconResources {
    pipeline: RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    bind_group: BindGroup,
    icon_group_layout: BindGroupLayout,
    groups: HashMap<IconId, IconGroup>,
    entity_to_icon: HashMap<Entity, IconId>,
}
impl IconResources {
    pub(crate) fn group(&self, entity: Entity) -> &IconGroup {
        self.groups
            .get(self.entity_to_icon.get(&entity).unwrap())
            .unwrap()
    }
    pub(crate) fn group_mut(&mut self, entity: Entity) -> &mut IconGroup {
        self.groups
            .get_mut(self.entity_to_icon.get(&entity).unwrap())
            .unwrap()
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
impl Render for Icon {
    type DirectiveGroupKey = IconId;
    const RENDER_PHASE: RenderPhase = RenderPhase::Alpha(1);
    type Resources = IconResources;

    fn create_resources(ginkgo: &Ginkgo) -> Self::Resources {
        let shader = ginkgo.create_shader(include_wgsl!("icon.wgsl"));
        let vertex_buffer = ginkgo.create_vertex_buffer(VERTICES);
        let bind_group_layout = ginkgo.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("icon-bind-group-layout"),
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
            label: Some("icon-bind-group"),
            layout: &bind_group_layout,
            entries: &[
                ginkgo.viewport_bind_group_entry(0),
                Ginkgo::sampler_bind_group_entry(&sampler, 1),
            ],
        });
        let icon_group_layout = ginkgo.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("icon-group-bind-group-layout"),
            entries: &[Ginkgo::bind_group_layout_entry(0)
                .at_stages(ShaderStages::FRAGMENT)
                .texture_entry(
                    TextureViewDimension::D2,
                    TextureSampleType::Float { filterable: false },
                )],
        });
        let pipeline_layout = ginkgo.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("icon-pipeline-layout"),
            bind_group_layouts: &[&bind_group_layout, &icon_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = ginkgo.create_pipeline(&RenderPipelineDescriptor {
            label: Some("icon-render-pipeline"),
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
        IconResources {
            pipeline,
            vertex_buffer,
            bind_group,
            icon_group_layout,
            groups: HashMap::new(),
            entity_to_icon: Default::default(),
        }
    }

    fn prepare(
        renderer: &mut Renderer<Self>,
        queue_handle: &mut RenderQueueHandle,
        ginkgo: &Ginkgo,
    ) -> bool {
        for entity in queue_handle.read_removes::<Self>() {
            renderer.resource_handle.group_mut(entity).instances.remove(entity);
            renderer.resource_handle.entity_to_icon.remove(&entity);
        }
        for packet in queue_handle.read_adds::<Self, IconId>() {
            if !renderer.resource_handle.entity_to_icon.contains_key(&packet.entity) {

            }
        }
        true
    }

    fn record(renderer: &mut Renderer<Self>, ginkgo: &Ginkgo) {
        todo!()
    }
}
