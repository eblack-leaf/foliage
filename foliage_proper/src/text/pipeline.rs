use crate::ash::clip::ClipSection;
use crate::ash::differential::RenderQueueHandle;
use crate::ash::instance::{Instance, InstanceBuffer, InstanceId};
use crate::ash::node::{Node, Nodes, RemoveNode};
use crate::ash::render::{GroupId, Parameters, PipelineId, Render, RenderGroup, Renderer};
use crate::ginkgo::{Ginkgo, VectorUniform};
use crate::opacity::BlendedOpacity;
use crate::text::glyph::{GlyphKey, GlyphOffset, ResolvedColors, ResolvedGlyphs};
use crate::text::monospaced::MonospacedFont;
use crate::text::{ResolvedFontSize, TextBounds, UniqueCharacters};
use crate::texture::{AtlasEntry, TextureAtlas, TextureCoordinates, Vertex, VERTICES};
use crate::{CReprColor, CReprSection, Logical, Physical, ResolvedElevation, Section, Text};
use bevy_ecs::entity::Entity;
use std::collections::HashMap;
use wgpu::{
    include_wgsl, BindGroupDescriptor, BindGroupLayoutDescriptor, PipelineLayoutDescriptor,
    RenderPass, RenderPipelineDescriptor, ShaderStages, TextureSampleType, TextureViewDimension,
    VertexState, VertexStepMode,
};

pub(crate) struct Resources {
    pub(crate) entity_to_group: HashMap<Entity, GroupId>,
    pub(crate) group_layout: wgpu::BindGroupLayout,
    pub(crate) font: MonospacedFont,
}

pub(crate) struct Group {
    pub(crate) texture_atlas: Option<TextureAtlas<GlyphKey, GlyphOffset, u8>>,
    pub(crate) bind_group: Option<wgpu::BindGroup>,
    pub(crate) update_node: bool,
    pub(crate) elevation: ResolvedElevation,
    pub(crate) clip_section: ClipSection,
    pub(crate) uniform: VectorUniform<f32>,
    pub(crate) sections: InstanceBuffer<CReprSection>,
    pub(crate) colors: InstanceBuffer<CReprColor>,
    pub(crate) tex_coords: InstanceBuffer<TextureCoordinates>,
    pub(crate) write_uniform: bool,
    pub(crate) unique_characters: UniqueCharacters,
    pub(crate) font_size: ResolvedFontSize,
    pub(crate) queued_tex_reads: Vec<(GlyphKey, InstanceId)>,
    pub(crate) bounds: Section<Physical>,
}

impl Group {
    pub(crate) fn new(ginkgo: &Ginkgo, elevation: ResolvedElevation) -> Self {
        let initial_capacity = 15;
        let uniform = VectorUniform::new(ginkgo.context(), [0.0, 0.0, elevation.value(), 1.0]);
        Self {
            texture_atlas: None,
            bind_group: None,
            update_node: false,
            elevation,
            clip_section: Default::default(),
            uniform,
            sections: InstanceBuffer::new(ginkgo, initial_capacity),
            colors: InstanceBuffer::new(ginkgo, initial_capacity),
            tex_coords: InstanceBuffer::new(ginkgo, initial_capacity),
            write_uniform: false,
            unique_characters: Default::default(),
            font_size: Default::default(),
            queued_tex_reads: vec![],
            bounds: Default::default(),
        }
    }
}
const ONE_NODE_PER_GROUP_OPTIMIZATION: InstanceId = 0;
impl Render for Text {
    type Group = Group;
    type Resources = Resources;
    fn renderer(ginkgo: &Ginkgo) -> Renderer<Self> {
        let shader = ginkgo.create_shader(include_wgsl!("text.wgsl"));
        let vertex_buffer = ginkgo.create_vertex_buffer(VERTICES);
        let bind_group_layout = ginkgo.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("text-bind-group-layout"),
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
            label: Some("text-bind-group"),
            layout: &bind_group_layout,
            entries: &[
                ginkgo.viewport_bind_group_entry(0),
                Ginkgo::sampler_bind_group_entry(&sampler, 1),
            ],
        });
        let group_layout = ginkgo.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("text-group-bind-group-layout"),
            entries: &[
                Ginkgo::bind_group_layout_entry(0)
                    .at_stages(ShaderStages::FRAGMENT)
                    .texture_entry(
                        TextureViewDimension::D2,
                        TextureSampleType::Float { filterable: false },
                    ),
                Ginkgo::bind_group_layout_entry(1)
                    .at_stages(ShaderStages::VERTEX)
                    .uniform_entry(),
            ],
        });
        let pipeline_layout = ginkgo.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("text-pipeline-layout"),
            bind_group_layouts: &[&group_layout, &bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = ginkgo.create_pipeline(&RenderPipelineDescriptor {
            label: Some("text-render-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Option::from("vertex_entry"),
                compilation_options: Default::default(),
                buffers: &[
                    Ginkgo::vertex_buffer_layout::<Vertex>(
                        VertexStepMode::Vertex,
                        &wgpu::vertex_attr_array![0 => Float32x2, 1 => Uint32x2],
                    ),
                    Ginkgo::vertex_buffer_layout::<CReprSection>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![2 => Float32x4],
                    ),
                    Ginkgo::vertex_buffer_layout::<CReprColor>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![3 => Float32x4],
                    ),
                    Ginkgo::vertex_buffer_layout::<TextureCoordinates>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![4 => Float32x4],
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
            vertex_buffer,
            bind_group,
            groups: Default::default(),
            resources: Resources {
                entity_to_group: Default::default(),
                group_layout,
                font: MonospacedFont::new(Text::OPT_SCALE),
            },
        }
    }
    fn prepare(
        renderer: &mut Renderer<Self>,
        queues: &mut RenderQueueHandle,
        ginkgo: &Ginkgo,
    ) -> Nodes {
        let mut nodes = Nodes::new();
        // read-attrs
        for entity in queues.removes::<Text>() {
            // remove group
            if let Some(id) = renderer.resources.entity_to_group.remove(&entity) {
                renderer.groups.remove(&id);
            }
            nodes.remove(RemoveNode::new(
                PipelineId::Text,
                entity.index() as GroupId,
                ONE_NODE_PER_GROUP_OPTIMIZATION,
            ));
        }
        for (entity, packet) in queues.attribute::<Text, ResolvedElevation>() {
            // queue add/update
            if renderer.resources.entity_to_group.contains_key(&entity) {
                let id = renderer.resources.entity_to_group.get(&entity).unwrap();
                // OMITTED for optimization renderer.groups.get_mut(id).unwrap().coordinator.needs_sort = true;
                let group = &mut renderer.groups.get_mut(id).unwrap().group;
                group.elevation = packet;
                group.uniform.set(2, packet.value());
                group.write_uniform = true;
                group.update_node = true;
            } else {
                // adding new group
                let group = Group::new(ginkgo, packet);
                renderer
                    .groups
                    .insert(entity.index() as GroupId, RenderGroup::new(group));
                renderer
                    .resources
                    .entity_to_group
                    .insert(entity, entity.index() as GroupId);
            }
        }
        for (entity, packet) in queues.attribute::<Text, ClipSection>() {
            let id = renderer.resources.entity_to_group.get(&entity).unwrap();
            // OMITTED for optimization renderer.groups.get_mut(id).unwrap().coordinator.needs_sort = true;
            let group = &mut renderer.groups.get_mut(id).unwrap().group;
            group.clip_section = packet;
            group.update_node = true;
        }
        for (entity, packet) in queues.attribute::<Text, TextBounds>() {
            let id = renderer.resources.entity_to_group.get(&entity).unwrap();
            let group = &mut renderer.groups.get_mut(id).unwrap().group;
            group.bounds = packet.bounds;
            group.update_node = true;
        }
        for (entity, packet) in queues.attribute::<Text, Section<Logical>>() {
            let id = renderer.resources.entity_to_group.get(&entity).unwrap();
            let group = &mut renderer.groups.get_mut(id).unwrap().group;
            let position = packet
                .position
                .to_device(ginkgo.configuration().scale_factor.value());
            group.uniform.set(0, position.left().round());
            group.uniform.set(1, position.top().round());
            group.write_uniform = true;
        }
        for (entity, packet) in queues.attribute::<Text, BlendedOpacity>() {
            let id = renderer.resources.entity_to_group.get(&entity).unwrap();
            let group = &mut renderer.groups.get_mut(id).unwrap().group;
            group.uniform.set(3, packet.value);
            group.write_uniform = true;
        }
        for (entity, packet) in queues.attribute::<Text, UniqueCharacters>() {
            let id = renderer.resources.entity_to_group.get(&entity).unwrap();
            let group = &mut renderer.groups.get_mut(id).unwrap().group;
            group.unique_characters = packet; // prevents under-growth
        }
        for (entity, packet) in queues.attribute::<Text, ResolvedFontSize>() {
            let id = renderer.resources.entity_to_group.get(&entity).unwrap();
            let group = &mut renderer.groups.get_mut(id).unwrap().group;
            group.font_size = packet;
            group.texture_atlas.replace(TextureAtlas::new(
                ginkgo,
                renderer.resources.font.character_block(packet.value),
                group.unique_characters.0,
                wgpu::TextureFormat::R8Unorm,
            ));
            let bind_group = ginkgo.create_bind_group(&BindGroupDescriptor {
                label: Some("text-group"),
                layout: &renderer.resources.group_layout,
                entries: &[
                    Ginkgo::texture_bind_group_entry(
                        group.texture_atlas.as_ref().unwrap().view(),
                        0,
                    ),
                    Ginkgo::uniform_bind_group_entry(&group.uniform.uniform, 1),
                ],
            });
            group.bind_group.replace(bind_group);
        }
        for (entity, glyphs) in queues.attribute::<Text, ResolvedGlyphs>() {
            let id = renderer.resources.entity_to_group.get(&entity).unwrap();
            let group = renderer.groups.get_mut(id).unwrap();
            for glyph in glyphs.removed {
                if group.coordinator.has_instance(glyph.offset as InstanceId) {
                    let order = group.coordinator.order(glyph.offset as InstanceId);
                    group.coordinator.remove(order);
                    group.group.sections.remove(order);
                    group.group.colors.remove(order);
                    group.group.tex_coords.remove(order);
                    group
                        .group
                        .texture_atlas
                        .as_mut()
                        .unwrap()
                        .remove_reference(glyph.key, glyph.offset);
                    // MISSING skipping 0 reference entry removal for optimization
                }
            }
            for glyph in glyphs.updated {
                if !group.coordinator.has_instance(glyph.offset as InstanceId) {
                    group.coordinator.add(Instance::new(
                        ResolvedElevation::new(0.0),
                        group.group.clip_section,
                        glyph.offset as InstanceId,
                    ));
                }
                if !group
                    .group
                    .texture_atlas
                    .as_ref()
                    .unwrap()
                    .has_key(glyph.key)
                {
                    let (metrics, rasterization) = renderer.resources.font.0.rasterize_indexed(
                        glyph.key.glyph_index,
                        group.group.font_size.value as f32,
                    );
                    let entry = AtlasEntry::new(rasterization, (metrics.width, metrics.height));
                    group
                        .group
                        .texture_atlas
                        .as_mut()
                        .unwrap()
                        .add_entry(glyph.key, entry);
                    group
                        .group
                        .queued_tex_reads
                        .push((glyph.key, glyph.offset as InstanceId));
                } else {
                    let tex_coords = group
                        .group
                        .texture_atlas
                        .as_ref()
                        .unwrap()
                        .tex_coordinates(glyph.key);
                    group
                        .group
                        .tex_coords
                        .queue(glyph.offset as InstanceId, tex_coords);
                }
                group
                    .group
                    .texture_atlas
                    .as_mut()
                    .unwrap()
                    .add_reference(glyph.key, glyph.offset);
                group
                    .group
                    .sections
                    .queue(glyph.offset as InstanceId, glyph.section.c_repr());
            }
        }
        for (id, group) in renderer.groups.iter_mut() {
            let (changed, grown) = group.group.texture_atlas.as_mut().unwrap().resolve(ginkgo);
            for key in changed {
                let (metrics, rasterization) = renderer
                    .resources
                    .font
                    .0
                    .rasterize_indexed(key.glyph_index, group.group.font_size.value as f32);
                let entry = AtlasEntry::new(rasterization, (metrics.width, metrics.height));
                for updated in group
                    .group
                    .texture_atlas
                    .as_mut()
                    .unwrap()
                    .write_entry(ginkgo, key, entry)
                {
                    group
                        .group
                        .tex_coords
                        .queue(updated.key as InstanceId, updated.tex_coords);
                }
            }
            if grown {
                let bind_group = ginkgo.create_bind_group(&BindGroupDescriptor {
                    label: Some("text-group"),
                    layout: &renderer.resources.group_layout,
                    entries: &[
                        Ginkgo::texture_bind_group_entry(
                            group.group.texture_atlas.as_ref().unwrap().view(),
                            0,
                        ),
                        Ginkgo::uniform_bind_group_entry(&group.group.uniform.uniform, 1),
                    ],
                });
                group.group.bind_group.replace(bind_group);
            }
        }
        for (id, group) in renderer.groups.iter_mut() {
            for (key, id) in group.group.queued_tex_reads.drain(..).collect::<Vec<_>>() {
                let tex_coords = group
                    .group
                    .texture_atlas
                    .as_ref()
                    .unwrap()
                    .tex_coordinates(key);
                group.group.tex_coords.queue(id, tex_coords);
            }
        }
        for (entity, packet) in queues.attribute::<Self, ResolvedColors>() {
            let id = renderer.resources.entity_to_group.get(&entity).unwrap();
            let group = renderer.groups.get_mut(id).unwrap();
            for glyph_color in packet.colors {
                group
                    .group
                    .colors
                    .queue(glyph_color.offset as InstanceId, glyph_color.color.into());
            }
        }
        for (group_id, render_group) in renderer.groups.iter_mut() {
            if render_group.group.write_uniform {
                render_group.group.uniform.write(ginkgo.context());
                render_group.group.write_uniform = false;
            }
            if let Some(capacity) = render_group.coordinator.grown() {
                render_group.group.sections.grow(ginkgo, capacity);
                render_group.group.colors.grow(ginkgo, capacity);
                render_group.group.tex_coords.grow(ginkgo, capacity);
            }
            // MISSING sort instances to get order [not needed for text]
            // MISSING handle swaps because of sorting [need to queue-write of attrs of swapped]
            for (id, data) in render_group.group.sections.queued() {
                let order = render_group.coordinator.order(id);
                render_group.group.sections.write_cpu(order, data);
            }
            for (id, data) in render_group.group.colors.queued() {
                let order = render_group.coordinator.order(id);
                render_group.group.colors.write_cpu(order, data);
            }
            for (id, data) in render_group.group.tex_coords.queued() {
                let order = render_group.coordinator.order(id);
                render_group.group.tex_coords.write_cpu(order, data);
            }
            // flush each instance buffer to gpu
            render_group.group.sections.write_gpu(ginkgo);
            render_group.group.colors.write_gpu(ginkgo);
            render_group.group.tex_coords.write_gpu(ginkgo);
            // submit node
            if render_group.group.update_node {
                let bounds = render_group
                    .group
                    .bounds
                    .to_logical(ginkgo.configuration().scale_factor.value());
                let resolved = if let Some(cs) = render_group.group.clip_section.0 {
                    cs.intersection(bounds).unwrap()
                } else {
                    bounds
                };
                let node = Node::new(
                    render_group.group.elevation,
                    PipelineId::Text,
                    *group_id,
                    0,
                    ClipSection(Some(resolved)),
                    ONE_NODE_PER_GROUP_OPTIMIZATION,
                );
                nodes.update(node);
                render_group.group.update_node = false;
            }
        }
        nodes
    }

    fn render(renderer: &mut Renderer<Self>, render_pass: &mut RenderPass, parameters: Parameters) {
        let group = renderer.groups.get(&parameters.group).unwrap();
        if let Some(clip) = parameters.clip_section {
            render_pass.set_scissor_rect(
                clip.left() as u32,
                clip.top() as u32,
                clip.width() as u32,
                clip.height() as u32,
            );
        }
        render_pass.set_pipeline(&renderer.pipeline);
        render_pass.set_bind_group(0, &group.group.bind_group, &[]);
        render_pass.set_bind_group(1, &renderer.bind_group, &[]);
        render_pass.set_vertex_buffer(0, renderer.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, group.group.sections.buffer.slice(..));
        render_pass.set_vertex_buffer(2, group.group.colors.buffer.slice(..));
        render_pass.set_vertex_buffer(3, group.group.tex_coords.buffer.slice(..));
        render_pass.draw(0..VERTICES.len() as u32, 0..group.coordinator.count());
    }
}
