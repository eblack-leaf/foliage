use crate::ash::clip::{prepare_clip_section, ClipSection};
use crate::ginkgo::Ginkgo;
use crate::{Attachment, Color, Component, DiffMarkers, Foliage, Layer, Resource, Text};
use bevy_ecs::prelude::IntoSystemConfigs;
use bevy_ecs::world::World;
use std::collections::{HashMap, HashSet};
use std::ops::Range;
use wgpu::{CommandEncoderDescriptor, RenderPass, RenderPassDescriptor, TextureViewDescriptor};

pub(crate) mod clip;
pub(crate) mod differential;

impl Attachment for Ash {
    fn attach(foliage: &mut Foliage) {
        foliage
            .diff
            .add_systems(prepare_clip_section.in_set(DiffMarkers::Prepare));
    }
}
pub(crate) struct Ash {
    pub(crate) drawn: bool,
    pub(crate) nodes: Vec<Node>,
    pub(crate) contiguous: Vec<ContiguousSpan>,
    pub(crate) text: Option<Renderer<Text>>,
}
impl Default for Ash {
    fn default() -> Self {
        Self::new()
    }
}
impl Ash {
    pub(crate) fn new() -> Self {
        Self {
            drawn: false,
            nodes: vec![],
            contiguous: vec![],
            text: None,
        }
    }
    pub(crate) fn initialize(&mut self, ginkgo: &Ginkgo) {
        self.text.replace(Text::renderer(ginkgo));
        // TODO other renderers
    }
    pub(crate) fn prepare(&mut self, world: &mut World, ginkgo: &Ginkgo) {
        let mut nodes = vec![];
        let mut to_remove = vec![];
        let text_nodes = Render::prepare(self.text.as_mut().unwrap(), world, ginkgo);
        nodes.extend(text_nodes.updated);
        to_remove.extend(text_nodes.removed);
        // TODO extend other renderers
        if nodes.is_empty() && to_remove.is_empty() {
            return;
        }
        let mut idxs = to_remove
            .iter()
            .filter_map(|rn| {
                if let Some(idx) = self.nodes.iter().position(|n| {
                    n.pipeline == rn.pipeline_id
                        && n.group == rn.group_id
                        && n.instance_id == rn.instance_id
                }) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        idxs.sort();
        idxs.reverse();
        for idx in idxs {
            self.nodes.remove(idx);
        }
        let mut to_replace = vec![];
        let mut to_add = vec![];
        for node in nodes {
            if let Some(idx) = self.nodes.iter().position(|n| {
                n.pipeline == node.pipeline
                    && n.group == node.group
                    && n.instance_id == node.instance_id
            }) {
                to_replace.push((node, idx));
            } else {
                to_add.push(node);
            }
        }
        // process to_replace + to_add
        for (node, idx) in to_replace {
            *self.nodes.get_mut(idx).unwrap() = node;
        }
        for node in to_add {
            self.nodes.push(node);
        }
        // sort
        // remake contiguous
    }
    pub(crate) fn render(&mut self, ginkgo: &Ginkgo) {
        let surface_texture = ginkgo.surface_texture();
        let view = surface_texture
            .texture
            .create_view(&TextureViewDescriptor::default());
        let mut encoder =
            ginkgo
                .context()
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("present-encoder"),
                });
        let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("render-pass"),
            color_attachments: &ginkgo.color_attachment(&view, Color::gray(50)),
            depth_stencil_attachment: ginkgo.depth_stencil_attachment(),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        for span in self.contiguous.iter() {
            match span.pipeline {
                PipelineId::Text => {
                    Render::render(
                        self.text.as_mut().unwrap(),
                        &mut rpass,
                        ginkgo,
                        span.parameters(),
                    );
                }
                PipelineId::Icon => {}
                PipelineId::Shape => {}
                PipelineId::Panel => {}
                PipelineId::Image => {}
            }
        }
        drop(rpass);
        ginkgo
            .context()
            .queue
            .submit(std::iter::once(encoder.finish()));
        surface_texture.present();
    }
}
pub(crate) trait Render
where
    Self: Sized,
{
    type Group;
    type Resources;
    fn renderer(ginkgo: &Ginkgo) -> Renderer<Self>;
    fn prepare(renderer: &mut Renderer<Self>, world: &mut World, ginkgo: &Ginkgo) -> Nodes;
    fn render(
        renderer: &mut Renderer<Self>,
        render_pass: &mut RenderPass,
        ginkgo: &Ginkgo,
        parameters: Parameters,
    );
}
pub(crate) type Order = i32;
pub(crate) type InstanceId = i32;
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub(crate) enum PipelineId {
    Text,
    Icon,
    Shape,
    Panel,
    Image,
}
pub(crate) type GroupId = i32;
#[derive(Clone)]
pub(crate) struct ContiguousSpan {
    pub(crate) pipeline: PipelineId,
    pub(crate) group: GroupId,
    pub(crate) range: Range<Order>,
    pub(crate) clip_section: ClipSection,
}
impl ContiguousSpan {
    pub(crate) fn parameters(&self) -> Parameters {
        Parameters {
            group: self.group,
            range: self.range.clone(),
            clip_section: self.clip_section,
        }
    }
}
#[derive(Clone)]
pub(crate) struct Parameters {
    pub(crate) group: GroupId,
    pub(crate) range: Range<Order>,
    pub(crate) clip_section: ClipSection,
}
#[derive(Copy, Clone)]
pub(crate) struct Node {
    pub(crate) layer: Layer,
    pub(crate) pipeline: PipelineId,
    pub(crate) group: GroupId,
    pub(crate) order: Order,
    pub(crate) clip_section: ClipSection,
    pub(crate) instance_id: InstanceId,
}
impl Node {
    pub(crate) fn new(
        layer: Layer,
        pipeline_id: PipelineId,
        group_id: GroupId,
        order: Order,
        clip_section: ClipSection,
        instance_id: InstanceId,
    ) -> Self {
        Self {
            layer,
            pipeline: pipeline_id,
            group: group_id,
            order,
            clip_section,
            instance_id,
        }
    }
}
#[derive(Copy, Clone)]
pub(crate) struct RemoveNode {
    pub(crate) pipeline_id: PipelineId,
    pub(crate) group_id: GroupId,
    pub(crate) instance_id: InstanceId,
}
impl RemoveNode {
    pub fn new(pipeline_id: PipelineId, group_id: GroupId, instance_id: InstanceId) -> Self {
        Self {
            pipeline_id,
            group_id,
            instance_id,
        }
    }
}
pub(crate) struct Nodes {
    pub(crate) updated: Vec<Node>,
    pub(crate) removed: Vec<RemoveNode>,
}
impl Nodes {
    pub(crate) fn new() -> Self {
        Self {
            updated: vec![],
            removed: vec![],
        }
    }
    pub(crate) fn update(&mut self, node: Node) {
        todo!()
    }
    pub(crate) fn remove(&mut self, remove_node: RemoveNode) {
        todo!()
    }
}
#[derive(Copy, Clone)]
pub(crate) struct Instance {
    pub(crate) layer: Layer,
    pub(crate) clip_section: ClipSection,
    pub(crate) id: InstanceId,
}
#[derive(Copy, Clone)]
pub(crate) struct Swap {
    pub(crate) old: Order,
    pub(crate) new: Order,
}
pub(crate) struct InstanceCoordinator {
    pub(crate) instances: Vec<Instance>,
    pub(crate) cache: Vec<Instance>,
    pub(crate) swaps: Vec<Swap>,
    pub(crate) node_submit: Vec<Node>,
    pub(crate) id_gen: InstanceId,
    pub(crate) gen_pool: HashSet<InstanceId>,
}
impl InstanceCoordinator {
    pub(crate) fn new(capacity: u32) -> Self {
        todo!()
    }
    pub(crate) fn has_instance(&self, id: InstanceId) -> bool {
        todo!()
    }
    pub(crate) fn generate_id(&mut self) -> InstanceId {
        if self.gen_pool.is_empty() {
            let val = self.id_gen;
            self.id_gen += 1;
            val
        } else {
            let val = self.gen_pool.iter().last().copied().unwrap();
            self.gen_pool.remove(&val);
            val
        }
    }
}
pub(crate) struct InstanceBuffer<I: bytemuck::Pod + bytemuck::Zeroable> {
    pub(crate) cpu: Vec<I>,
    pub(crate) buffer: wgpu::Buffer,
    pub(crate) writes: HashMap<InstanceId, I>,
}
impl<I: bytemuck::Pod + bytemuck::Zeroable> InstanceBuffer<I> {
    pub(crate) fn new(ginkgo: &Ginkgo, initial_capacity: u32) -> Self {
        todo!()
    }
}
pub(crate) struct RenderGroup<R: Render> {
    pub(crate) coordinator: InstanceCoordinator,
    pub(crate) group: R::Group,
}
impl<R: Render> RenderGroup<R> {
    pub(crate) fn new(group: R::Group) -> Self {
        Self {
            coordinator: InstanceCoordinator::new(1),
            group,
        }
    }
}
pub(crate) struct Renderer<R: Render> {
    pub(crate) pipeline: wgpu::RenderPipeline,
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) groups: HashMap<GroupId, RenderGroup<R>>,
    pub(crate) resources: R::Resources,
}
