use crate::ash::clip::{prepare_clip_section, ClipSection};
use crate::ash::differential::Elm;
use crate::ginkgo::{Ginkgo, ScaleFactor};
use crate::{
    Attachment, Color, Component, DeviceContext, DiffMarkers, Foliage, ResolvedElevation, Resource,
    Section, Text,
};
use bevy_ecs::prelude::IntoSystemConfigs;
use bevy_ecs::world::World;
use std::cmp::Ordering;
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
        let mut elm = Elm::new(world);
        let mut nodes = vec![];
        let mut to_remove = vec![];
        let text_nodes = Render::prepare(self.text.as_mut().unwrap(), &mut elm, ginkgo);
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
        for (node, idx) in to_replace {
            *self.nodes.get_mut(idx).unwrap() = node;
        }
        for node in to_add {
            self.nodes.push(node);
        }
        self.nodes.sort_by(|lhs, rhs| {
            if lhs.elevation < rhs.elevation {
                Ordering::Less
            } else if lhs.elevation > rhs.elevation {
                Ordering::Greater
            } else {
                if lhs.pipeline != rhs.pipeline {
                    Ordering::Less
                } else {
                    if lhs.group != rhs.group {
                        Ordering::Less
                    } else {
                        if lhs.clip_section != rhs.clip_section {
                            Ordering::Less
                        } else {
                            if lhs.order < rhs.order {
                                Ordering::Less
                            } else if lhs.order > rhs.order {
                                Ordering::Greater
                            } else {
                                Ordering::Equal
                            }
                        }
                    }
                }
            }
        });
        self.contiguous.clear();
        let mut index = 0;
        let mut contiguous = 1;
        let mut range_start = None;
        for node in self.nodes.iter() {
            let next = if let Some(n) = self.nodes.get(index + 1) {
                Some(n.clone())
            } else {
                None
            };
            index += 1;
            if let Some(next) = next {
                if node.pipeline == next.pipeline
                    && node.group == next.group
                    && node.order + 1 == next.order
                    && node.clip_section == next.clip_section
                {
                    contiguous += 1;
                    if range_start.is_none() {
                        range_start = Some(node.order);
                    }
                    continue;
                }
            }
            let start = range_start.take().unwrap_or(node.order);
            self.contiguous.push(ContiguousSpan {
                pipeline: node.pipeline,
                group: node.group,
                range: start..start + contiguous,
                clip_section: node.clip_section,
            });
        }
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
            color_attachments: &ginkgo.color_attachment(&view, Color::gray(950)),
            depth_stencil_attachment: ginkgo.depth_stencil_attachment(),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        for span in self.contiguous.iter() {
            let parameters = span.parameters(
                ginkgo.viewport().section(),
                ginkgo.configuration().scale_factor,
            );
            match span.pipeline {
                PipelineId::Text => {
                    Render::render(self.text.as_mut().unwrap(), &mut rpass, parameters);
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
    fn prepare(renderer: &mut Renderer<Self>, elm: &mut Elm, ginkgo: &Ginkgo) -> Nodes;
    fn render(renderer: &mut Renderer<Self>, render_pass: &mut RenderPass, parameters: Parameters);
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
    pub(crate) fn parameters(
        &self,
        view_section: Section<DeviceContext>,
        scale_factor: ScaleFactor,
    ) -> Parameters {
        let clip_section = if let Some(present) = self.clip_section.0 {
            present
                .to_device(scale_factor.value())
                .intersection(view_section)
        } else {
            None
        };
        Parameters {
            group: self.group,
            range: self.range.clone(),
            clip_section,
        }
    }
}
#[derive(Clone)]
pub(crate) struct Parameters {
    pub(crate) group: GroupId,
    pub(crate) range: Range<Order>,
    pub(crate) clip_section: Option<Section<DeviceContext>>,
}
#[derive(Copy, Clone)]
pub(crate) struct Node {
    pub(crate) elevation: ResolvedElevation,
    pub(crate) pipeline: PipelineId,
    pub(crate) group: GroupId,
    pub(crate) order: Order,
    pub(crate) clip_section: ClipSection,
    pub(crate) instance_id: InstanceId,
}
impl Node {
    pub(crate) fn new(
        elevation: ResolvedElevation,
        pipeline_id: PipelineId,
        group_id: GroupId,
        order: Order,
        clip_section: ClipSection,
        instance_id: InstanceId,
    ) -> Self {
        Self {
            elevation,
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
        self.updated.push(node);
    }
    pub(crate) fn remove(&mut self, remove_node: RemoveNode) {
        self.removed.push(remove_node);
    }
}
#[derive(Copy, Clone)]
pub(crate) struct Instance {
    pub(crate) elevation: ResolvedElevation,
    pub(crate) clip_section: ClipSection,
    pub(crate) id: InstanceId,
}
impl Instance {
    pub fn new(elevation: ResolvedElevation, clip_section: ClipSection, id: InstanceId) -> Self {
        Self {
            elevation,
            clip_section,
            id,
        }
    }
}
#[derive(Copy, Clone)]
pub(crate) struct Swap {
    pub(crate) old: Order,
    pub(crate) new: Order,
}
pub(crate) struct InstanceCoordinator {
    pub(crate) instances: Vec<Instance>,
    pub(crate) cache: Vec<Instance>,
    pub(crate) node_submit: Vec<Node>,
    pub(crate) id_gen: InstanceId,
    pub(crate) gen_pool: HashSet<InstanceId>,
    pub(crate) capacity: u32,
    pub(crate) needs_sort: bool,
}
impl InstanceCoordinator {
    pub(crate) fn new(capacity: u32) -> Self {
        Self {
            instances: vec![],
            cache: vec![],
            node_submit: vec![],
            id_gen: 0,
            gen_pool: Default::default(),
            capacity,
            needs_sort: false,
        }
    }
    pub(crate) fn add(&mut self, instance: Instance) {
        self.instances.push(instance);
        self.needs_sort = true;
    }
    pub(crate) fn has_instance(&self, id: InstanceId) -> bool {
        self.instances.iter().find(|i| i.id == id).is_some()
    }
    pub(crate) fn count(&self) -> u32 {
        self.instances.len() as u32
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
    pub(crate) fn grown(&mut self) -> Option<u32> {
        const REPEAT_ALLOCATION_AVOIDANCE: u32 = 2;
        if self.instances.len() > self.capacity as usize {
            let new = self.instances.len() as u32 + REPEAT_ALLOCATION_AVOIDANCE;
            self.capacity = new;
            return Some(new);
        }
        None
    }
    pub(crate) fn sort(&mut self) -> Vec<Swap> {
        let mut swaps = vec![];
        if !self.needs_sort {
            return swaps;
        }
        self.needs_sort = false;
        self.instances.sort_by(|a, b| {
            if a.elevation > b.elevation {
                Ordering::Greater
            } else if a.elevation < b.elevation {
                Ordering::Less
            } else {
                if a.clip_section != b.clip_section {
                    Ordering::Less
                } else {
                    Ordering::Equal
                }
            }
        });
        for (new, instance) in self.instances.iter().enumerate() {
            if let Some(old) = self.cache.iter().position(|c| c.id == instance.id) {
                if new != old {
                    swaps.push(Swap {
                        old: old as Order,
                        new: new as Order,
                    })
                }
            }
        }
        self.cache = self.instances.clone();
        swaps
    }
    pub(crate) fn order(&self, id: InstanceId) -> Order {
        self.instances.iter().position(|i| i.id == id).unwrap() as Order
    }
    pub(crate) fn remove(&mut self, order: Order) {
        self.instances.remove(order as usize);
        self.needs_sort = true;
    }
}
pub(crate) struct InstanceBuffer<I: bytemuck::Pod + bytemuck::Zeroable + Default> {
    pub(crate) cpu: Vec<I>,
    pub(crate) buffer: wgpu::Buffer,
    pub(crate) queue: HashMap<InstanceId, I>,
    pub(crate) write_range: Option<Range<usize>>,
    pub(crate) capacity: u32,
}
impl<I: bytemuck::Pod + bytemuck::Zeroable + Default> InstanceBuffer<I> {
    pub(crate) fn new(ginkgo: &Ginkgo, initial_capacity: u32) -> Self {
        let cpu = vec![I::default(); initial_capacity as usize];
        let buffer = ginkgo
            .context()
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("instance-buffer"),
                size: Ginkgo::memory_size::<I>(initial_capacity),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        Self {
            cpu,
            buffer,
            queue: HashMap::new(),
            write_range: None,
            capacity: initial_capacity,
        }
    }
    pub(crate) fn queue(&mut self, id: InstanceId, i: I) {
        self.queue.insert(id, i);
    }
    pub(crate) fn queued(&mut self) -> Vec<(InstanceId, I)> {
        self.queue.drain().collect::<Vec<_>>()
    }
    pub(crate) fn grow(&mut self, ginkgo: &Ginkgo, capacity: u32) {
        if capacity < self.capacity {
            return;
        }
        let mut cpu = self.cpu.drain(..).collect::<Vec<_>>();
        let mut queued = self.queue.drain().collect::<Vec<_>>();
        *self = Self::new(ginkgo, capacity);
        for (i, c) in cpu.drain(..).enumerate() {
            *self.cpu.get_mut(i).unwrap() = c;
        }
        for (id, i) in queued.drain(..) {
            self.queue.insert(id, i);
        }
        self.write_range.replace(0..self.cpu.len());
    }
    pub(crate) fn swap(&mut self, swap: Swap) {
        let current = self.cpu.get(swap.old as usize).unwrap().clone();
        self.queue(swap.new, current);
    }
    pub(crate) fn write_cpu(&mut self, order: Order, data: I) {
        *self.cpu.get_mut(order as usize).unwrap() = data;
        if let Some(mut range) = self.write_range.as_mut() {
            if range.start > order as usize {
                range.start = order as usize;
            }
            if range.end < order as usize + 1 {
                range.end = order as usize + 1;
            }
        } else {
            self.write_range.replace(order as usize..order as usize + 1);
        }
    }
    pub(crate) fn write_gpu(&mut self, ginkgo: &Ginkgo) {
        if let Some(range) = self.write_range.take() {
            let slice = &self.cpu[range.clone()];
            ginkgo.context().queue.write_buffer(
                &self.buffer,
                Ginkgo::memory_size::<I>(range.start as u32),
                bytemuck::cast_slice(slice),
            );
        }
    }
    pub(crate) fn remove(&mut self, order: Order) {
        self.cpu.remove(order as usize);
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
