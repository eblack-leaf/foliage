use crate::ash::differential::RenderQueueHandle;
use crate::ash::instance::{Instance, InstanceBuffer, InstanceId};
use crate::ash::node::{Nodes, RemoveNode};
use crate::ash::render::{GroupId, Parameters, PipelineId, Render, RenderGroup, Renderer};
use crate::ginkgo::Ginkgo;
use crate::icon::Icon;
use crate::opacity::BlendedOpacity;
use crate::texture::Mips;
use crate::{
    CReprColor, CReprSection, Color, Coordinates, IconMemory, Logical, ResolvedElevation, Section,
    Stem,
};
use bevy_ecs::entity::Entity;
use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;
use wgpu::{
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor,
    PipelineLayoutDescriptor, RenderPass, RenderPipelineDescriptor, ShaderStages, TextureFormat,
    TextureSampleType, TextureViewDimension, VertexState, VertexStepMode,
};

pub(crate) struct Resources {
    entity_to_group: HashMap<Entity, GroupId>,
    group_layout: BindGroupLayout,
}
pub(crate) struct Group {
    bind_group: BindGroup,
    sections: InstanceBuffer<CReprSection>,
    elevations: InstanceBuffer<ResolvedElevation>,
    colors: InstanceBuffer<CReprColor>,
    mips: InstanceBuffer<Mips>,
    opacities: InstanceBuffer<BlendedOpacity>,
    mip_count: u32,
}
/// Picks the sharpest mip level that still covers `scale_factor`, in a chain where level 0
/// is the largest/mip-0 (`base`) and each subsequent level is exactly half the previous
/// (`base >> level`, the shape wgpu's texture upload requires) -- generalizes what was a
/// hardcoded 3-level 96/48/24 threshold ladder to any `mip_count`.
fn mip_level_for_scale(mip_count: u32, scale_factor: f32) -> f32 {
    let rounded = scale_factor.round().max(1.0) as u32;
    let mut level = mip_count.saturating_sub(1);
    let mut covered = 1u32;
    while covered < rounded && level > 0 {
        level -= 1;
        covered <<= 1;
    }
    level as f32
}
impl Render for Icon {
    type Group = Group;
    type Resources = Resources;

    fn renderer(ginkgo: &Ginkgo) -> Renderer<Self> {
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
                    .sampler_entry(false),
            ],
        });
        let sampler = ginkgo.create_sampler(false);
        let bind_group = ginkgo.create_bind_group(&BindGroupDescriptor {
            label: Some("icon-bind-group"),
            layout: &bind_group_layout,
            entries: &[
                ginkgo.viewport_bind_group_entry(0),
                Ginkgo::sampler_bind_group_entry(&sampler, 1),
            ],
        });
        let group_layout = ginkgo.create_bind_group_layout(&BindGroupLayoutDescriptor {
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
            bind_group_layouts: &[Some(&group_layout), Some(&bind_group_layout)],
            immediate_size: 0,
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
                    Ginkgo::vertex_buffer_layout::<CReprSection>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![1 => Float32x4],
                    ),
                    Ginkgo::vertex_buffer_layout::<ResolvedElevation>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![2 => Float32],
                    ),
                    Ginkgo::vertex_buffer_layout::<CReprColor>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![3 => Float32x4],
                    ),
                    Ginkgo::vertex_buffer_layout::<Mips>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![4 => Float32],
                    ),
                    Ginkgo::vertex_buffer_layout::<BlendedOpacity>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![5 => Float32],
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
            multiview_mask: None,
            cache: None,
        });
        Renderer {
            pipeline,
            vertex_buffer,
            bind_group,
            groups: Default::default(),
            resources: Resources {
                entity_to_group: HashMap::new(),
                group_layout,
            },
        }
    }

    fn prepare(
        renderer: &mut Renderer<Self>,
        queues: &mut RenderQueueHandle,
        ginkgo: &Ginkgo,
    ) -> Nodes {
        tracing::trace!("pipeline: icon prepare");
        let mut nodes = Nodes::new();
        for entity in queues.removes::<Icon>() {
            if let Some(gid) = renderer.resources.entity_to_group.remove(&entity) {
                let group = renderer.groups.get_mut(&gid).unwrap();
                let id = entity.index().index() as InstanceId;
                let order = group.coordinator.order(id);
                group.coordinator.remove(order);
                nodes.remove(RemoveNode::new(PipelineId::Icon, gid, id));
                queues.remove_attr::<Icon, Icon>(entity);
                queues.remove_attr::<Icon, Section<Logical>>(entity);
                queues.remove_attr::<Icon, ResolvedElevation>(entity);
                queues.remove_attr::<Icon, Stem>(entity);
                queues.remove_attr::<Icon, Color>(entity);
                queues.remove_attr::<Icon, BlendedOpacity>(entity);
            }
        }
        for (_, mem) in queues.attribute::<Icon, IconMemory>() {
            let (_, view) = ginkgo.create_texture(
                TextureFormat::R8Unorm,
                mem.texture_scale,
                mem.mip_count,
                mem.resolved_bytes(),
            );
            let group = Group {
                bind_group: ginkgo.create_bind_group(&BindGroupDescriptor {
                    label: Some("icon-group-bind-group"),
                    layout: &renderer.resources.group_layout,
                    entries: &[Ginkgo::texture_bind_group_entry(&view, 0)],
                }),
                sections: InstanceBuffer::new(ginkgo, 1),
                elevations: InstanceBuffer::new(ginkgo, 1),
                colors: InstanceBuffer::new(ginkgo, 1),
                mips: InstanceBuffer::new(ginkgo, 1),
                opacities: InstanceBuffer::new(ginkgo, 1),
                mip_count: mem.mip_count,
            };
            renderer.groups.insert(mem.id, RenderGroup::new(group));
        }
        for (entity, icon) in queues.attribute::<Icon, Icon>() {
            let id = entity.index().index() as InstanceId;
            if let Some(gid) = renderer.resources.entity_to_group.remove(&entity) {
                let group = renderer.groups.get_mut(&gid).unwrap();
                let order = group.coordinator.order(id);
                group.coordinator.remove(order);
            }
            renderer.resources.entity_to_group.insert(entity, icon.id);
            if let Some(group) = renderer.groups.get_mut(&icon.id) {
                if !group.coordinator.has_instance(id) {
                    group.coordinator.add(Instance::new(
                        ResolvedElevation::default(),
                        Stem::default(),
                        id,
                    ));
                }
            } else {
                panic!("icon-memory non-existent")
            }
        }
        for (entity, section) in queues.attribute::<Icon, Section<Logical>>() {
            let gid = renderer.resources.entity_to_group.get(&entity).unwrap();
            let id = entity.index().index() as InstanceId;
            let group = renderer.groups.get_mut(gid).unwrap();
            let sf = ginkgo.configuration().scale_factor;
            group
                .group
                .sections
                .queue(id, section.to_physical(sf.value()).rounded().c_repr());
            let level = mip_level_for_scale(group.group.mip_count, sf.value());
            group.group.mips.queue(id, Mips(level));
        }
        for (entity, elevation) in queues.attribute::<Icon, ResolvedElevation>() {
            let gid = renderer.resources.entity_to_group.get(&entity).unwrap();
            let id = entity.index().index() as InstanceId;
            let group = renderer.groups.get_mut(gid).unwrap();
            group.coordinator.update_elevation(id, elevation);
            group.group.elevations.queue(id, elevation);
        }
        for (entity, clip) in queues.attribute::<Icon, Stem>() {
            let gid = renderer.resources.entity_to_group.get(&entity).unwrap();
            let id = entity.index().index() as InstanceId;
            let group = renderer.groups.get_mut(gid).unwrap();
            group.coordinator.update_clip_context(id, clip);
        }
        for (entity, color) in queues.attribute::<Icon, Color>() {
            let gid = renderer.resources.entity_to_group.get(&entity).unwrap();
            let id = entity.index().index() as InstanceId;
            let group = renderer.groups.get_mut(gid).unwrap();
            group.group.colors.queue(id, color.c_repr());
        }
        for (entity, opacity) in queues.attribute::<Icon, BlendedOpacity>() {
            let gid = renderer.resources.entity_to_group.get(&entity).unwrap();
            let id = entity.index().index() as InstanceId;
            let group = renderer.groups.get_mut(gid).unwrap();
            group.group.opacities.queue(id, opacity);
        }
        for (gid, group) in renderer.groups.iter_mut() {
            if let Some(n) = group.coordinator.grown() {
                group.group.sections.grow(ginkgo, n);
                group.group.elevations.grow(ginkgo, n);
                group.group.colors.grow(ginkgo, n);
                group.group.mips.grow(ginkgo, n);
                group.group.opacities.grow(ginkgo, n);
            }
            for swap in group.coordinator.sort() {
                group.group.sections.swap(swap);
                group.group.elevations.swap(swap);
                group.group.colors.swap(swap);
                group.group.mips.swap(swap);
                group.group.opacities.swap(swap);
            }
            for (id, data) in group.group.sections.queued() {
                let order = group.coordinator.order(id);
                group.group.sections.write_cpu(order, data);
            }
            for (id, data) in group.group.elevations.queued() {
                let order = group.coordinator.order(id);
                group.group.elevations.write_cpu(order, data);
            }
            for (id, data) in group.group.colors.queued() {
                let order = group.coordinator.order(id);
                group.group.colors.write_cpu(order, data);
            }
            for (id, data) in group.group.mips.queued() {
                let order = group.coordinator.order(id);
                group.group.mips.write_cpu(order, data);
            }
            for (id, data) in group.group.opacities.queued() {
                let order = group.coordinator.order(id);
                group.group.opacities.write_cpu(order, data);
            }
            group.group.sections.write_gpu(ginkgo);
            group.group.elevations.write_gpu(ginkgo);
            group.group.colors.write_gpu(ginkgo);
            group.group.mips.write_gpu(ginkgo);
            group.group.opacities.write_gpu(ginkgo);
            for node in group.coordinator.updated_nodes(PipelineId::Icon, *gid) {
                nodes.update(node);
            }
        }
        nodes
    }

    fn render(renderer: &mut Renderer<Self>, render_pass: &mut RenderPass, parameters: Parameters) {
        let group = renderer.groups.get(&parameters.group).unwrap();
        render_pass.set_pipeline(&renderer.pipeline);
        render_pass.set_bind_group(0, &group.group.bind_group, &[]);
        render_pass.set_bind_group(1, &renderer.bind_group, &[]);
        render_pass.set_vertex_buffer(0, renderer.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, group.group.sections.buffer.slice(..));
        render_pass.set_vertex_buffer(2, group.group.elevations.buffer.slice(..));
        render_pass.set_vertex_buffer(3, group.group.colors.buffer.slice(..));
        render_pass.set_vertex_buffer(4, group.group.mips.buffer.slice(..));
        render_pass.set_vertex_buffer(5, group.group.opacities.buffer.slice(..));
        render_pass.draw(0..VERTICES.len() as u32, parameters.range);
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
