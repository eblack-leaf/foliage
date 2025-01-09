use crate::ash::differential::RenderQueueHandle;
use crate::ash::instance::{Instance, InstanceBuffer, InstanceId};
use crate::ash::node::{Nodes, RemoveNode};
use crate::ash::render::{GroupId, Parameters, PipelineId, Render, RenderGroup, Renderer};
use crate::ginkgo::Ginkgo;
use crate::image::{CropAdjustment, Image, ImageMemory, ImageWrite};
use crate::opacity::BlendedOpacity;
use crate::texture::TextureCoordinates;
use crate::{
    texture, Area, CReprSection, ClipContext, Logical, Numerical, ResolvedElevation, Section,
};
use bevy_ecs::entity::Entity;
use std::collections::HashMap;
use wgpu::{
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor,
    Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, PipelineLayoutDescriptor, RenderPass,
    RenderPipelineDescriptor, ShaderStages, Texture, TextureAspect, TextureSampleType, TextureView,
    TextureViewDimension, VertexState, VertexStepMode,
};

pub(crate) struct Resources {
    group_layout: BindGroupLayout,
    entity_to_memory: HashMap<Entity, GroupId>,
}
pub(crate) struct Group {
    texture: Texture,
    #[allow(unused)]
    view: TextureView,
    bind_group: BindGroup,
    memory_extent: Area<Numerical>,
    image_extent: Area<Numerical>,
    texture_coordinates: TextureCoordinates,
    sections: InstanceBuffer<CReprSection>,
    elevations: InstanceBuffer<ResolvedElevation>,
    coords: InstanceBuffer<TextureCoordinates>,
    opaque: InstanceBuffer<BlendedOpacity>,
}
impl Render for Image {
    type Group = Group;
    type Resources = Resources;

    fn renderer(ginkgo: &Ginkgo) -> Renderer<Self> {
        let shader = ginkgo.create_shader(include_wgsl!("image.wgsl"));
        let group_layout = ginkgo.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("image-group-bind-group-layout"),
            entries: &[Ginkgo::bind_group_layout_entry(0)
                .at_stages(ShaderStages::FRAGMENT)
                .texture_entry(
                    TextureViewDimension::D2,
                    TextureSampleType::Float { filterable: true },
                )],
        });
        let bind_group_layout = ginkgo.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("image-bind-group-layout"),
            entries: &[
                Ginkgo::bind_group_layout_entry(0)
                    .at_stages(ShaderStages::VERTEX)
                    .uniform_entry(),
                Ginkgo::bind_group_layout_entry(1)
                    .at_stages(ShaderStages::FRAGMENT)
                    .sampler_entry(true),
            ],
        });
        let sampler = ginkgo.create_sampler(true);
        let bind_group = ginkgo.create_bind_group(&BindGroupDescriptor {
            label: Some("image-bind-group"),
            layout: &bind_group_layout,
            entries: &[
                ginkgo.viewport_bind_group_entry(0),
                Ginkgo::sampler_bind_group_entry(&sampler, 1),
            ],
        });
        let pipeline_layout = ginkgo.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("image-pipeline-layout"),
            bind_group_layouts: &[&group_layout, &bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = ginkgo.create_pipeline(&RenderPipelineDescriptor {
            label: Some("image-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Option::from("vertex_entry"),
                compilation_options: Default::default(),
                buffers: &[
                    Ginkgo::vertex_buffer_layout::<texture::Vertex>(
                        VertexStepMode::Vertex,
                        &wgpu::vertex_attr_array![0 => Float32x2, 1 => Uint32x2],
                    ),
                    Ginkgo::vertex_buffer_layout::<CReprSection>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![2 => Float32x4],
                    ),
                    Ginkgo::vertex_buffer_layout::<ResolvedElevation>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![3 => Float32],
                    ),
                    Ginkgo::vertex_buffer_layout::<TextureCoordinates>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![4 => Float32x4],
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
            multiview: None,
            cache: None,
        });
        Renderer {
            pipeline,
            vertex_buffer: ginkgo.create_vertex_buffer(texture::VERTICES),
            bind_group,
            groups: Default::default(),
            resources: Resources {
                group_layout,
                entity_to_memory: Default::default(),
            },
        }
    }

    fn prepare(
        renderer: &mut Renderer<Self>,
        queues: &mut RenderQueueHandle,
        ginkgo: &Ginkgo,
    ) -> Nodes {
        let mut nodes = Nodes::new();
        for entity in queues.removes::<Image>() {
            if let Some(group_id) = renderer.resources.entity_to_memory.remove(&entity) {
                let group = renderer.groups.get_mut(&group_id).unwrap();
                let id = entity.index() as InstanceId;
                let order = group.coordinator.order(id);
                group.coordinator.remove(order);
                group.group.sections.remove(order);
                group.group.elevations.remove(order);
                group.group.coords.remove(order);
                group.group.opaque.remove(order);
                nodes.remove(RemoveNode::new(PipelineId::Image, group_id, id));
            }
        }
        for (_, memory) in queues.attribute::<Image, ImageMemory>() {
            let (tex, view) = ginkgo.create_texture(
                Image::FORMAT,
                memory.extent.coordinates,
                1,
                bytemuck::cast_slice(&vec![
                    0f32;
                    memory.extent.width() as usize
                        * memory.extent.height() as usize
                ]),
            );
            let g = Group {
                texture: tex,
                bind_group: ginkgo.create_bind_group(&BindGroupDescriptor {
                    label: Some("image-group-bind-group"),
                    layout: &renderer.resources.group_layout,
                    entries: &[Ginkgo::texture_bind_group_entry(&view, 0)],
                }),
                view,
                memory_extent: memory.extent,
                image_extent: Default::default(),
                texture_coordinates: Default::default(),
                sections: InstanceBuffer::new(ginkgo, 1),
                elevations: InstanceBuffer::new(ginkgo, 1),
                coords: InstanceBuffer::new(ginkgo, 1),
                opaque: InstanceBuffer::new(ginkgo, 1),
            };
            renderer
                .groups
                .insert(memory.memory_id, RenderGroup::new(g));
        }
        for (entity, image) in queues.attribute::<Image, ImageWrite>() {
            let group = renderer.groups.get_mut(&image.image.memory_id).unwrap();
            if image.extent != Area::default() {
                ginkgo.context().queue.write_texture(
                    ImageCopyTexture {
                        texture: &group.group.texture,
                        mip_level: 0,
                        origin: Origin3d::default(),
                        aspect: TextureAspect::All,
                    },
                    bytemuck::cast_slice(&image.data),
                    ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(image.extent.width() as u32 * size_of::<f32>() as u32),
                        rows_per_image: Some(image.extent.height() as u32),
                    },
                    Extent3d {
                        width: image.extent.width() as u32,
                        height: image.extent.height() as u32,
                        depth_or_array_layers: 1,
                    },
                );
            }
            group.group.image_extent = image.extent;
            group.group.texture_coordinates = TextureCoordinates::from_section(
                Section::new((0, 0), image.extent.coordinates),
                group.group.memory_extent.coordinates,
            );
            if renderer
                .resources
                .entity_to_memory
                .iter()
                .find(|(e, g)| **g == image.image.memory_id && **e != entity)
                .is_some()
            {
                panic!("overwriting existing image group with active entity")
            }
            renderer
                .resources
                .entity_to_memory
                .insert(entity, image.image.memory_id);
        }
        for (entity, elevation) in queues.attribute::<Image, ResolvedElevation>() {
            if let Some(gid) = renderer.resources.entity_to_memory.get(&entity) {
                let group = renderer.groups.get_mut(&gid).unwrap();
                let id = entity.index() as InstanceId;
                if !group.coordinator.has_instance(id) {
                    group
                        .coordinator
                        .add(Instance::new(elevation, ClipContext::default(), id));
                } else {
                    group.coordinator.update_elevation(id, elevation);
                }
                group.group.elevations.queue(id, elevation);
            }
        }
        for (entity, clip) in queues.attribute::<Image, ClipContext>() {
            if let Some(gid) = renderer.resources.entity_to_memory.get(&entity) {
                let group = renderer.groups.get_mut(&gid).unwrap();
                let id = entity.index() as InstanceId;
                group.coordinator.update_clip_context(id, clip);
            }
        }
        for (entity, adjustments) in queues.attribute::<Image, CropAdjustment>() {
            if let Some(gid) = renderer.resources.entity_to_memory.get(&entity) {
                let group = renderer.groups.get_mut(&gid).unwrap();
                let id = entity.index() as InstanceId;
                let base = group.group.texture_coordinates;
                if adjustments.adjustments == Section::default() {
                    group.group.coords.queue(id, base);
                } else {
                    let t =
                        base.top_left.a() + base.bottom_right.a() * adjustments.adjustments.left();
                    let l =
                        base.top_left.b() + base.bottom_right.b() * adjustments.adjustments.top();
                    let b = base.bottom_right.a()
                        - base.bottom_right.a() * adjustments.adjustments.width();
                    let r = base.bottom_right.b()
                        - base.bottom_right.b() * adjustments.adjustments.height();
                    let adjusted = TextureCoordinates::new((t, l), (b, r));
                    group.group.coords.queue(id, adjusted);
                }
            }
        }
        for (entity, opacity) in queues.attribute::<Image, BlendedOpacity>() {
            if let Some(gid) = renderer.resources.entity_to_memory.get(&entity) {
                let group = renderer.groups.get_mut(&gid).unwrap();
                let id = entity.index() as InstanceId;
                group.group.opaque.queue(id, opacity);
            }
        }
        for (entity, section) in queues.attribute::<Image, Section<Logical>>() {
            if let Some(gid) = renderer.resources.entity_to_memory.get(&entity) {
                let group = renderer.groups.get_mut(&gid).unwrap();
                let id = entity.index() as InstanceId;
                group.group.sections.queue(
                    id,
                    section
                        .to_physical(ginkgo.configuration().scale_factor.value())
                        .c_repr(),
                );
            }
        }
        for (gid, group) in renderer.groups.iter_mut() {
            if let Some(n) = group.coordinator.grown() {
                group.group.sections.grow(ginkgo, n);
                group.group.elevations.grow(ginkgo, n);
                group.group.coords.grow(ginkgo, n);
                group.group.opaque.grow(ginkgo, n);
            }
            for swap in group.coordinator.sort() {
                group.group.sections.swap(swap);
                group.group.elevations.swap(swap);
                group.group.coords.swap(swap);
                group.group.opaque.swap(swap);
            }
            for (id, data) in group.group.sections.queued() {
                let order = group.coordinator.order(id);
                group.group.sections.write_cpu(order, data);
            }
            for (id, data) in group.group.elevations.queued() {
                let order = group.coordinator.order(id);
                group.group.elevations.write_cpu(order, data);
            }
            for (id, data) in group.group.coords.queued() {
                let order = group.coordinator.order(id);
                group.group.coords.write_cpu(order, data);
            }
            for (id, data) in group.group.opaque.queued() {
                let order = group.coordinator.order(id);
                group.group.opaque.write_cpu(order, data);
            }
            group.group.sections.write_gpu(ginkgo);
            group.group.elevations.write_gpu(ginkgo);
            group.group.coords.write_gpu(ginkgo);
            group.group.opaque.write_gpu(ginkgo);
            for node in group.coordinator.updated_nodes(PipelineId::Image, *gid) {
                nodes.update(node);
            }
        }
        nodes
    }

    fn render(renderer: &mut Renderer<Self>, render_pass: &mut RenderPass, parameters: Parameters) {
        if let Some(clip) = parameters.clip_section {
            render_pass.set_scissor_rect(
                clip.left() as u32,
                clip.top() as u32,
                clip.right() as u32,
                clip.bottom() as u32,
            );
        }
        render_pass.set_pipeline(&renderer.pipeline);
        let group = renderer.groups.get(&parameters.group).unwrap();
        render_pass.set_bind_group(0, &group.group.bind_group, &[]);
        render_pass.set_bind_group(1, &renderer.bind_group, &[]);
        render_pass.set_vertex_buffer(0, renderer.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, group.group.sections.buffer.slice(..));
        render_pass.set_vertex_buffer(2, group.group.elevations.buffer.slice(..));
        render_pass.set_vertex_buffer(3, group.group.coords.buffer.slice(..));
        render_pass.set_vertex_buffer(4, group.group.opaque.buffer.slice(..));
        render_pass.draw(0..texture::VERTICES.len() as u32, parameters.range);
    }
}
