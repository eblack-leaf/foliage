use crate::ash::clip::ClipSection;
use crate::ash::differential::RenderQueueHandle;
use crate::ash::instance::{Instance, InstanceBuffer, InstanceId};
use crate::ash::node::{Nodes, RemoveNode};
use crate::ash::render::{GroupId, Parameters, PipelineId, Render, RenderGroup, Renderer};
use crate::ginkgo::Ginkgo;
use crate::image::{Image, ImageMemory, ImageWrite};
use crate::opacity::BlendedOpacity;
use crate::texture::TextureCoordinates;
use crate::{texture, Area, CReprSection, Numerical, Opacity, ResolvedElevation, Section};
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
    view: TextureView,
    bind_group: BindGroup,
    memory_extent: Area<Numerical>,
    image_extent: Area<Numerical>,
    texture_coordinates: TextureCoordinates,
    sections: InstanceBuffer<CReprSection>,
    elevations: InstanceBuffer<ResolvedElevation>,
    coords: InstanceBuffer<TextureCoordinates>,
    opaque: InstanceBuffer<Opacity>,
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
                    TextureSampleType::Float { filterable: false },
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
                    .sampler_entry(),
            ],
        });
        let sampler = ginkgo.create_sampler();
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
                let mut group = renderer.groups.get_mut(&group_id).unwrap();
                let order = group.coordinator.order(entity.index() as InstanceId);
                group.coordinator.remove(order);
                nodes.remove(RemoveNode::new(PipelineId::Image, group_id, order));
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
            let mut group = renderer.groups.get_mut(&image.image.memory_id).unwrap();
            ginkgo.context().queue.write_texture(
                ImageCopyTexture {
                    texture: &group.group.texture,
                    mip_level: 0,
                    origin: Origin3d::default(),
                    aspect: TextureAspect::All,
                },
                bytemuck::cast_slice(&image.image.data),
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
                let mut group = renderer.groups.get_mut(&gid).unwrap();
                let id = entity.index() as InstanceId;
                if !group.coordinator.has_instance(id) {
                    group
                        .coordinator
                        .add(Instance::new(elevation, ClipSection::default(), id));
                }
                // TODO queue-attribute
            }
        }
        // TODO other attributes
        nodes
    }

    fn render(renderer: &mut Renderer<Self>, render_pass: &mut RenderPass, parameters: Parameters) {
        todo!()
    }
}
