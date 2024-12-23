use crate::ash::clip::ClipSection;
use crate::ash::differential::Elm;
use crate::ash::{
    GroupId, Instance, InstanceBuffer, InstanceId, Node, Nodes, Parameters, PipelineId, RemoveNode,
    Render, RenderGroup, Renderer,
};
use crate::ginkgo::{Ginkgo, VectorUniform};
use crate::opacity::BlendedOpacity;
use crate::text::glyph::{GlyphKey, GlyphOffset, ResolvedColors, ResolvedGlyphs};
use crate::text::monospaced::MonospacedFont;
use crate::text::UniqueCharacters;
use crate::texture::{TextureAtlas, TextureCoordinates, Vertex, VERTICES};
use crate::{CReprColor, CReprSection, FontSize, Layer, LogicalContext, Section, Text};
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
    pub(crate) texture_atlas: TextureAtlas<GlyphKey, GlyphOffset, u8>,
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) update_node: bool,
    pub(crate) layer: Layer,
    pub(crate) clip_section: ClipSection,
    pub(crate) uniform: VectorUniform<f32>,
    pub(crate) sections: InstanceBuffer<CReprSection>,
    pub(crate) colors: InstanceBuffer<CReprColor>,
    pub(crate) tex_coords: InstanceBuffer<TextureCoordinates>,
    pub(crate) write_uniform: bool,
    pub(crate) unique_characters: UniqueCharacters,
}

impl Group {
    pub(crate) fn new(ginkgo: &Ginkgo, layer: Layer, layout: &wgpu::BindGroupLayout) -> Self {
        let initial_capacity = 15;
        let texture_atlas = TextureAtlas::new(
            ginkgo,
            (9, 19),
            initial_capacity,
            wgpu::TextureFormat::R8Unorm,
        );
        let uniform = VectorUniform::new(ginkgo.context(), [0.0, 0.0, layer.value(), 1.0]);
        let bind_group = ginkgo.create_bind_group(&BindGroupDescriptor {
            label: Some("text-group"),
            layout,
            entries: &[
                Ginkgo::texture_bind_group_entry(texture_atlas.view(), 0),
                Ginkgo::uniform_bind_group_entry(&uniform.uniform, 1),
            ],
        });
        Self {
            texture_atlas,
            bind_group,
            update_node: false,
            layer,
            clip_section: Default::default(),
            uniform,
            sections: InstanceBuffer::new(ginkgo, initial_capacity),
            colors: InstanceBuffer::new(ginkgo, initial_capacity),
            tex_coords: InstanceBuffer::new(ginkgo, initial_capacity),
            write_uniform: false,
            unique_characters: Default::default(),
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
    fn prepare(renderer: &mut Renderer<Self>, elm: &mut Elm, ginkgo: &Ginkgo) -> Nodes {
        let mut nodes = Nodes::new();
        // read-attrs
        for entity in elm.removes::<Text>() {
            // remove group
            if let Some(id) = renderer.resources.entity_to_group.get(&entity) {
                renderer.groups.remove(id);
            }
            nodes.remove(RemoveNode::new(
                PipelineId::Text,
                entity.index() as GroupId,
                ONE_NODE_PER_GROUP_OPTIMIZATION,
            ));
        }
        for (entity, packet) in elm.attribute::<Text, Layer>() {
            // queue add/update
            if renderer.resources.entity_to_group.contains_key(&entity) {
                let group = &mut renderer
                    .groups
                    .get_mut(renderer.resources.entity_to_group.get(&entity).unwrap())
                    .unwrap()
                    .group;
                group.layer = packet;
                group.uniform.set(2, packet.value());
                group.write_uniform = true;
                group.update_node = true;
            } else {
                // adding new group
                let group = Group::new(ginkgo, packet, &renderer.resources.group_layout);
                renderer
                    .groups
                    .insert(entity.index() as GroupId, RenderGroup::new(group));
            }
        }
        for (entity, packet) in elm.attribute::<Text, ClipSection>() {
            let id = renderer.resources.entity_to_group.get(&entity).unwrap();
            let group = &mut renderer.groups.get_mut(id).unwrap().group;
            group.clip_section = packet;
            group.update_node = true;
        }
        for (entity, packet) in elm.attribute::<Text, Section<LogicalContext>>() {
            let id = renderer.resources.entity_to_group.get(&entity).unwrap();
            let group = &mut renderer.groups.get_mut(id).unwrap().group;
            let position = packet
                .position
                .to_device(ginkgo.configuration().scale_factor.value());
            group.uniform.set(0, position.left());
            group.uniform.set(1, position.top());
            group.write_uniform = true;
        }
        for (entity, packet) in elm.attribute::<Text, BlendedOpacity>() {
            let id = renderer.resources.entity_to_group.get(&entity).unwrap();
            let group = &mut renderer.groups.get_mut(id).unwrap().group;
            group.uniform.set(3, packet.value);
            group.write_uniform = true;
        }
        for (entity, packet) in elm.attribute::<Text, UniqueCharacters>() {
            let id = renderer.resources.entity_to_group.get(&entity).unwrap();
            let group = &mut renderer.groups.get_mut(id).unwrap().group;
            group.unique_characters = packet; // prevents under-grow
        }
        for (entity, packet) in elm.attribute::<Text, FontSize>() {
            let id = renderer.resources.entity_to_group.get(&entity).unwrap();
            let group = &mut renderer.groups.get_mut(id).unwrap().group;
            // TODO block has changed so must remake atlas w/ stored unique-characters as capacity
        }
        for (entity, glyphs) in elm.attribute::<Text, ResolvedGlyphs>() {
            let id = renderer.resources.entity_to_group.get(&entity).unwrap();
            let group = renderer.groups.get_mut(id).unwrap();
            for glyph in glyphs.removed {
                group.coordinator.remove(glyph.offset as InstanceId);
                // TODO remove from instance-buffers?
                // TODO remove reference from atlas
            }
            for glyph in glyphs.updated {
                if !group.coordinator.has_instance(glyph.offset as InstanceId) {
                    group.coordinator.add(Instance::new(
                        Layer::new(0.0),
                        group.group.clip_section,
                        glyph.offset as InstanceId,
                    ));
                }
                // TODO add-entry / reference [depending on existence] to atlas (rasterization)
                // TODO then queue tex-coords from atlas (deferred because need grow atlas after all)
                group
                    .group
                    .sections
                    .queue(glyph.offset as InstanceId, glyph.section.c_repr());
            }
        }
        // TODO grow texture-atlas if needed (writes in process)
        // TODO handle queued tex-coords read (atlas) + queue (instance-buffer)
        for (entity, packet) in elm.attribute::<Self, ResolvedColors>() {
            let id = renderer.resources.entity_to_group.get(&entity).unwrap();
            let group = renderer.groups.get_mut(id).unwrap();
            for glyph_color in packet.colors {
                group
                    .group
                    .colors
                    .queue(glyph_color.offset as InstanceId, glyph_color.color.into());
            }
        }
        for (id, render_group) in renderer.groups.iter_mut() {
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
            render_group.group.sections.write_gpu();
            render_group.group.colors.write_gpu();
            render_group.group.tex_coords.write_gpu();
            // submit node
            if render_group.group.update_node {
                let node = Node::new(
                    render_group.group.layer,
                    PipelineId::Text,
                    *id,
                    0,
                    render_group.group.clip_section,
                    ONE_NODE_PER_GROUP_OPTIMIZATION,
                );
                nodes.update(node);
                render_group.group.update_node = false;
            }
        }
        nodes
    }

    fn render(
        renderer: &mut Renderer<Self>,
        render_pass: &mut RenderPass,
        ginkgo: &Ginkgo,
        parameters: Parameters,
    ) {
        todo!()
    }
}
