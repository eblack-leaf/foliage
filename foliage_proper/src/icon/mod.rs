use std::collections::HashMap;

use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, IntoSystemConfigs};
use bevy_ecs::query::{Changed, With};
use bevy_ecs::system::Query;
use bytemuck::{Pod, Zeroable};
use wgpu::{
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor,
    PipelineLayoutDescriptor, RenderPass, RenderPipeline, RenderPipelineDescriptor, ShaderStages,
    TextureFormat, TextureSampleType, TextureViewDimension, VertexState, VertexStepMode,
};

use crate::ash::{ClippingContext, DrawRange, Render, Renderer};
use crate::color::Color;
use crate::coordinate::area::Area;
use crate::coordinate::elevation::RenderLayer;
use crate::coordinate::position::Position;
use crate::coordinate::section::{GpuSection, Section};
use crate::coordinate::{Coordinates, DeviceContext, LogicalContext};
use crate::differential::{Differential, RenderLink};
use crate::elm::{Elm, InternalStage, RenderQueueHandle};
use crate::ginkgo::Ginkgo;
use crate::instances::Instances;
use crate::leaf::{Remove, Visibility};
use crate::texture::Mips;
use crate::Root;

mod proc_gen;

impl Root for Icon {
    fn attach(elm: &mut Elm) {
        elm.enable_differential::<Self, IconId>();
        elm.enable_differential::<Self, GpuSection>();
        elm.enable_differential::<Self, RenderLayer>();
        elm.enable_differential::<Self, Color>();
        elm.enable_differential::<Self, IconData>();
        elm.scheduler
            .main
            .add_systems(icon_scale.in_set(InternalStage::Resolve));
    }
}
fn icon_scale(
    mut icons: Query<
        &mut Section<LogicalContext>,
        (Changed<Section<LogicalContext>>, With<IconId>),
    >,
) {
    for (mut section) in icons.iter_mut() {
        let old = section.area;
        let new = if old.width() == 0.0 || old.height() == 0.0 {
            Area::logical((0, 0))
        } else {
            let new = Area::logical(Icon::SCALE);
            let diff = (old - new).max((0, 0)) / Area::logical((2, 2));
            section.position += Position::logical(diff.coordinates);
            new
        };
        section.area = new;
    }
}
#[derive(Component, Clone, PartialEq)]
pub struct IconData(pub IconId, pub Vec<u8>);
#[derive(Bundle)]
pub struct IconRequest {
    data: Differential<IconData>,
    d: IconData,
    link: RenderLink,
    remove: Remove,
    visibility: Visibility,
}
impl IconRequest {
    pub fn new<I: Into<IconId>>(id: I, data: Vec<u8>) -> Self {
        Self {
            data: Differential::new(),
            d: IconData(id.into(), data),
            link: RenderLink::new::<Icon>(),
            remove: Default::default(),
            visibility: Default::default(),
        }
    }
}
#[derive(Bundle, Clone)]
pub struct Icon {
    link: RenderLink,
    layer: Differential<RenderLayer>,
    gpu_section: Differential<GpuSection>,
    gs: GpuSection,
    id: Differential<IconId>,
    i: IconId,
    color: Differential<Color>,
    c: Color,
}
impl Icon {
    pub const SCALE: Coordinates = Coordinates::new(24f32, 24f32);
    pub const TEXTURE_SCALE: Coordinates = Coordinates::new(96f32, 96f32);
    pub fn new<I: Into<IconId>>(id: I, color: Color) -> Self {
        Self {
            link: RenderLink::new::<Icon>(),
            layer: Differential::new(),
            gpu_section: Differential::new(),
            gs: Default::default(),
            id: Differential::new(),
            i: id.into(),
            color: Differential::new(),
            c: color,
        }
    }

    fn write_mips(renderer: &mut Renderer<Icon>, ginkgo: &Ginkgo, entity: Entity) {
        let scale_factor = ginkgo.configuration().scale_factor.value();
        if scale_factor == 3f32 {
            renderer
                .resource_handle
                .group_mut_from_entity(entity)
                .instances
                .checked_write(entity, Mips(0f32));
        } else if scale_factor == 2f32 {
            renderer
                .resource_handle
                .group_mut_from_entity(entity)
                .instances
                .checked_write(entity, Mips(1f32));
        } else {
            renderer
                .resource_handle
                .group_mut_from_entity(entity)
                .instances
                .checked_write(entity, Mips(2f32));
        }
    }
}
#[derive(Hash, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Component, Debug)]
pub struct IconId(pub i32);
impl From<i32> for IconId {
    fn from(value: i32) -> Self {
        Self(value)
    }
}
pub(crate) struct IconGroup {
    bind_group: BindGroup,
    instances: Instances<Entity>,
    should_record: bool,
}
pub struct IconResources {
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
    pub(crate) fn group_mut_from_entity(&mut self, entity: Entity) -> &mut IconGroup {
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
            bind_group_layouts: &[&icon_group_layout, &bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = ginkgo.create_pipeline(&RenderPipelineDescriptor {
            label: Some("icon-render-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Option::from("vertex_entry"),
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
                    Ginkgo::vertex_buffer_layout::<RenderLayer>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![2 => Float32],
                    ),
                    Ginkgo::vertex_buffer_layout::<Color>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![3 => Float32x4],
                    ),
                    Ginkgo::vertex_buffer_layout::<Mips>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![4 => Float32],
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
            cache: None,
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
    ) {
        for packet in queue_handle.read_adds::<Self, IconData>() {
            renderer.associate_directive_group(packet.value.0 .0, packet.value.0);
            let (_, view) = ginkgo.create_texture(
                TextureFormat::R8Unorm,
                Self::TEXTURE_SCALE,
                3,
                packet.value.1.as_slice(),
            );
            renderer.resource_handle.groups.insert(
                packet.value.0,
                IconGroup {
                    bind_group: ginkgo.create_bind_group(&BindGroupDescriptor {
                        label: Some("icon-group-bind-group"),
                        layout: &renderer.resource_handle.icon_group_layout,
                        entries: &[Ginkgo::texture_bind_group_entry(&view, 0)],
                    }),
                    instances: Instances::new(1)
                        .with_attribute::<GpuSection>(ginkgo)
                        .with_attribute::<RenderLayer>(ginkgo)
                        .with_attribute::<Color>(ginkgo)
                        .with_attribute::<Mips>(ginkgo),
                    should_record: false,
                },
            );
        }
        for entity in queue_handle.read_removes::<Self>() {
            if !renderer
                .resource_handle
                .entity_to_icon
                .contains_key(&entity)
            {
                continue;
            }
            renderer
                .resource_handle
                .group_mut_from_entity(entity)
                .instances
                .queue_remove(entity);
            renderer.resource_handle.entity_to_icon.remove(&entity);
        }
        for packet in queue_handle.read_adds::<Self, IconId>() {
            let old = renderer
                .resource_handle
                .entity_to_icon
                .insert(packet.entity, packet.value);
            renderer
                .resource_handle
                .group_mut_from_entity(packet.entity)
                .instances
                .add(packet.entity);
            if let Some(o) = old {
                if o != packet.value {
                    // send current cpu values over to new instance
                    let gpu_sec = renderer
                        .resource_handle
                        .groups
                        .get(&o)
                        .unwrap()
                        .instances
                        .get_attr::<GpuSection>(&packet.entity)
                        .unwrap();
                    renderer
                        .resource_handle
                        .group_mut_from_entity(packet.entity)
                        .instances
                        .checked_write(packet.entity, gpu_sec);
                    Self::write_mips(renderer, ginkgo, packet.entity);
                    let layer = renderer
                        .resource_handle
                        .groups
                        .get(&o)
                        .unwrap()
                        .instances
                        .get_attr::<RenderLayer>(&packet.entity)
                        .unwrap();
                    renderer
                        .resource_handle
                        .group_mut_from_entity(packet.entity)
                        .instances
                        .checked_write(packet.entity, layer);
                    let color = renderer
                        .resource_handle
                        .groups
                        .get(&o)
                        .unwrap()
                        .instances
                        .get_attr::<Color>(&packet.entity)
                        .unwrap();
                    renderer
                        .resource_handle
                        .group_mut_from_entity(packet.entity)
                        .instances
                        .checked_write(packet.entity, color);
                    renderer
                        .resource_handle
                        .groups
                        .get_mut(&o)
                        .unwrap()
                        .instances
                        .queue_remove(packet.entity);
                }
            }
        }
        // TODO update cpu len of each attribute?
        for packet in queue_handle.read_adds::<Self, GpuSection>() {
            let id = renderer
                .resource_handle
                .entity_to_icon
                .get(&packet.entity)
                .unwrap();
            renderer
                .resource_handle
                .group_mut_from_entity(packet.entity)
                .instances
                .checked_write(packet.entity, packet.value.rounded());
            Self::write_mips(renderer, ginkgo, packet.entity);
        }
        for packet in queue_handle.read_adds::<Self, ClippingContext>() {
            renderer
                .resource_handle
                .group_mut_from_entity(packet.entity)
                .instances
                .set_clipping_context(packet.entity, packet.value);
        }
        for packet in queue_handle.read_adds::<Self, RenderLayer>() {
            renderer
                .resource_handle
                .group_mut_from_entity(packet.entity)
                .instances
                .set_layer(packet.entity, packet.value);
            renderer
                .resource_handle
                .group_mut_from_entity(packet.entity)
                .instances
                .checked_write(packet.entity, packet.value);
        }
        for packet in queue_handle.read_adds::<Self, Color>() {
            renderer
                .resource_handle
                .group_mut_from_entity(packet.entity)
                .instances
                .checked_write(packet.entity, packet.value);
        }
        for (i, g) in renderer.resource_handle.groups.iter_mut() {
            if g.instances.resolve_changes(ginkgo) {
                renderer
                    .node_manager
                    .set_nodes(i.0, g.instances.render_nodes());
            }
        }
    }
    fn draw<'a>(
        renderer: &'a Renderer<Self>,
        group_key: Self::DirectiveGroupKey,
        alpha_range: DrawRange,
        clipping_section: Section<DeviceContext>,
        render_pass: &mut RenderPass<'a>,
    ) {
        let group = renderer.resource_handle.groups.get(&group_key).unwrap();
        render_pass.set_scissor_rect(
            clipping_section.left() as u32,
            clipping_section.top() as u32,
            clipping_section.width() as u32,
            clipping_section.height() as u32,
        );
        render_pass.set_pipeline(&renderer.resource_handle.pipeline);
        render_pass.set_bind_group(0, &group.bind_group, &[]);
        render_pass.set_bind_group(1, &renderer.resource_handle.bind_group, &[]);
        render_pass.set_vertex_buffer(0, renderer.resource_handle.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, group.instances.buffer::<GpuSection>().slice(..));
        render_pass.set_vertex_buffer(2, group.instances.buffer::<RenderLayer>().slice(..));
        render_pass.set_vertex_buffer(3, group.instances.buffer::<Color>().slice(..));
        render_pass.set_vertex_buffer(4, group.instances.buffer::<Mips>().slice(..));
        render_pass.draw(0..VERTICES.len() as u32, alpha_range.start..alpha_range.end);
    }
}
