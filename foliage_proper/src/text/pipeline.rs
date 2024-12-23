use crate::ash::clip::ClipSection;
use crate::ash::differential::{RenderQueue, RenderRemoveQueue};
use crate::ash::{
    GroupId, InstanceBuffer, InstanceId, Node, Nodes, Parameters, PipelineId, RemoveNode, Render,
    RenderGroup, Renderer,
};
use crate::ginkgo::{Ginkgo, VectorUniform};
use crate::text::glyph::{GlyphKey, GlyphOffset};
use crate::text::monospaced::MonospacedFont;
use crate::texture::{TextureAtlas, TextureCoordinates, Vertex, VERTICES};
use crate::{CReprColor, CReprSection, Layer, Text};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::World;
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
    pub(crate) coordinates: InstanceBuffer<TextureCoordinates>,
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
            coordinates: InstanceBuffer::new(ginkgo, initial_capacity),
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

    fn prepare(renderer: &mut Renderer<Self>, world: &mut World, ginkgo: &Ginkgo) -> Nodes {
        let mut nodes = Nodes::new();
        // read-attrs
        for entity in world
            .get_resource_mut::<RenderRemoveQueue<Self>>()
            .unwrap()
            .queue
            .drain()
        {
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
        for (entity, packet) in world
            .get_resource_mut::<RenderQueue<Self, Layer>>()
            .unwrap()
            .queue
            .drain()
        {
            // queue add/update
            if renderer.resources.entity_to_group.contains_key(&entity) {
                let group = &mut renderer
                    .groups
                    .get_mut(renderer.resources.entity_to_group.get(&entity).unwrap())
                    .unwrap()
                    .group;
                group.layer = packet;
                group.uniform.set(2, packet.value());
                group.update_node = true;
            } else {
                // adding new group
                let group = Group::new(ginkgo, packet, &renderer.resources.group_layout);
                renderer
                    .groups
                    .insert(entity.index() as GroupId, RenderGroup::new(group));
            }
        }
        // TODO other attributes
        // queue-writes @ instance-id (instance-coordinator generated w/ reuse pool)
        // sort instance-coordinator
        for (id, render_group) in renderer.groups.iter_mut() {
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
