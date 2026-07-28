use crate::ash::clip::ClipContext;
use crate::ash::differential::RenderQueueHandle;
use crate::ash::instance::{Instance, InstanceBuffer, InstanceId};
use crate::ash::node::{Nodes, RemoveNode};
use crate::ash::render::{GroupId, Parameters, PipelineId, Render, RenderGroup, Renderer};
use crate::ginkgo::Ginkgo;
use crate::image::{CropAdjustment, Image, ImageWrite};
use crate::opacity::BlendedOpacity;
use crate::texture::TextureCoordinates;
use crate::{
    Area, AssetKey, CReprSection, Logical, Numerical, ResolvedElevation, Section, Stem, texture,
};
use bevy_ecs::entity::Entity;
use std::collections::HashMap;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor,
    PipelineLayoutDescriptor, RenderPass, RenderPipelineDescriptor, ShaderStages, Texture, TextureSampleType,
    TextureView, TextureViewDimension, VertexState, VertexStepMode, include_wgsl,
};

pub(crate) struct Resources {
    group_layout: BindGroupLayout,
    entity_to_memory: HashMap<Entity, GroupId>,
    /// GPU group identity, allocated here and keyed by `AssetKey`, so callers never
    /// invent or track one themselves.
    key_to_group: HashMap<AssetKey, GroupId>,
    next_group: GroupId,
}
impl Resources {
    fn group_for(&mut self, key: AssetKey) -> GroupId {
        *self.key_to_group.entry(key).or_insert_with(|| {
            let id = self.next_group;
            self.next_group += 1;
            id
        })
    }
}
pub(crate) struct Group {
    /// Held for ownership, not read: dropping it destroys the GPU texture `bind_group`
    /// still points at. Same reason `view` below is kept.
    #[allow(dead_code)]
    texture: Texture,
    #[allow(dead_code)]
    view: TextureView,
    bind_group: BindGroup,
    /// The texture is always allocated to exactly the decoded image's real size now, so
    /// there's no separate declared-vs-real extent to track -- one field, not two.
    #[allow(dead_code)]
    extent: Area<Numerical>,
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
            bind_group_layouts: &[Some(&group_layout), Some(&bind_group_layout)],
            immediate_size: 0,
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
            multiview_mask: None,
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
                key_to_group: Default::default(),
                next_group: 0,
            },
        }
    }

    fn prepare(
        renderer: &mut Renderer<Self>,
        queues: &mut RenderQueueHandle,
        ginkgo: &Ginkgo,
    ) -> Nodes {
        tracing::trace!("pipeline: image prepare");
        let mut nodes = Nodes::new();
        for entity in queues.removes::<Image>() {
            if let Some(group_id) = renderer.resources.entity_to_memory.remove(&entity) {
                let group = renderer.groups.get_mut(&group_id).unwrap();
                let id = entity.index().index() as InstanceId;
                let order = group.coordinator.order(id);
                group.coordinator.remove(order);
                nodes.remove(RemoveNode::new(PipelineId::Image, group_id, id));
                queues.remove_attr::<Image, ResolvedElevation>(entity);
                queues.remove_attr::<Image, ClipContext>(entity);
                queues.remove_attr::<Image, CropAdjustment>(entity);
                queues.remove_attr::<Image, BlendedOpacity>(entity);
                queues.remove_attr::<Image, Section<Logical>>(entity);
                queues.remove_attr::<Image, ImageWrite>(entity);
            }
        }
        for (entity, image) in queues.attribute::<Image, ImageWrite>() {
            let group_id = renderer.resources.group_for(image.key);
            tracing::trace!(
                entity = ?entity,
                group = group_id,
                extent = ?image.extent,
                "image-pipeline: ImageWrite packet (allocates-or-reuses the group, populates entity_to_memory)"
            );
            if !renderer.groups.contains_key(&group_id) {
                // First entity to reach this AssetKey: allocate a texture sized exactly to
                // the real decoded image and upload in the same step -- no separate
                // pre-declared size, no window where the texture exists without real data.
                let (tex, view) = ginkgo.create_texture(
                    Image::FORMAT,
                    image.extent.coordinates,
                    1,
                    bytemuck::cast_slice(&image.data),
                );
                let g = Group {
                    texture: tex,
                    bind_group: ginkgo.create_bind_group(&BindGroupDescriptor {
                        label: Some("image-group-bind-group"),
                        layout: &renderer.resources.group_layout,
                        entries: &[Ginkgo::texture_bind_group_entry(&view, 0)],
                    }),
                    view,
                    extent: image.extent,
                    texture_coordinates: TextureCoordinates::from_section(
                        Section::new((0, 0), image.extent.coordinates),
                        image.extent.coordinates,
                    ),
                    sections: InstanceBuffer::new(ginkgo, 1),
                    elevations: InstanceBuffer::new(ginkgo, 1),
                    coords: InstanceBuffer::new(ginkgo, 1),
                    opaque: InstanceBuffer::new(ginkgo, 1),
                };
                renderer.groups.insert(group_id, RenderGroup::new(g));
            }
            // the texture was filled when the group was created -- a later viewer of the
            // same key just registers; its instance arrives whenever its elevation does
            renderer.resources.entity_to_memory.insert(entity, group_id);
        }
        for (entity, elevation) in queues.attribute::<Image, ResolvedElevation>() {
            tracing::trace!(
                entity = ?entity,
                has_memory = renderer.resources.entity_to_memory.contains_key(&entity),
                "image-pipeline: ResolvedElevation packet (gates add())"
            );
            if let Some(gid) = renderer.resources.entity_to_memory.get(&entity) {
                let group = renderer.groups.get_mut(&gid).unwrap();
                let id = entity.index().index() as InstanceId;
                if !group.coordinator.has_instance(id) {
                    group
                        .coordinator
                        .add(Instance::new(elevation, Stem::default(), id));
                } else {
                    group.coordinator.update_elevation(id, elevation);
                }
                group.group.elevations.queue(id, elevation);
            }
        }
        for (entity, clip) in queues.attribute::<Image, ClipContext>() {
            if let Some(gid) = renderer.resources.entity_to_memory.get(&entity) {
                let group = renderer.groups.get_mut(&gid).unwrap();
                let id = entity.index().index() as InstanceId;
                group.coordinator.update_clip_context(id, clip.0);
            }
        }
        for (entity, adjustments) in queues.attribute::<Image, CropAdjustment>() {
            if let Some(gid) = renderer.resources.entity_to_memory.get(&entity) {
                let group = renderer.groups.get_mut(&gid).unwrap();
                let id = entity.index().index() as InstanceId;
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
                let id = entity.index().index() as InstanceId;
                group.group.opaque.queue(id, opacity);
            }
        }
        for (entity, section) in queues.attribute::<Image, Section<Logical>>() {
            if let Some(gid) = renderer.resources.entity_to_memory.get(&entity) {
                let group = renderer.groups.get_mut(&gid).unwrap();
                let id = entity.index().index() as InstanceId;
                tracing::trace!(
                    entity = ?entity,
                    has_instance = group.coordinator.has_instance(id),
                    "image-pipeline: Section packet (queues unguarded by has_instance)"
                );
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
