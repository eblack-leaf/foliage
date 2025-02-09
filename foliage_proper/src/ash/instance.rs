use crate::ash::node::Node;
use crate::ash::render::{GroupId, PipelineId};
use crate::ginkgo::Ginkgo;
use crate::{ResolvedElevation, Stem};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::ops::Range;

#[derive(Copy, Clone)]
pub(crate) struct Instance {
    #[allow(unused)]
    pub(crate) elevation: ResolvedElevation,
    #[allow(unused)]
    pub(crate) clip_context: Stem,
    pub(crate) id: InstanceId,
}

impl Instance {
    pub fn new(elevation: ResolvedElevation, clip_context: Stem, id: InstanceId) -> Self {
        Self {
            elevation,
            clip_context,
            id,
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[allow(unused)]
pub(crate) struct Swap {
    pub(crate) old: Order,
    pub(crate) id: InstanceId,
}

pub(crate) struct InstanceCoordinator {
    pub(crate) instances: Vec<Instance>,
    #[allow(unused)]
    pub(crate) cache: Vec<Instance>,
    #[allow(unused)]
    pub(crate) node_submit: HashSet<InstanceId>,
    #[allow(unused)]
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
            node_submit: HashSet::new(),
            id_gen: 0,
            gen_pool: Default::default(),
            capacity,
            needs_sort: false,
        }
    }
    pub(crate) fn add(&mut self, instance: Instance) {
        self.instances.push(instance);
        self.node_submit.insert(instance.id);
        self.needs_sort = true;
    }
    pub(crate) fn has_instance(&self, id: InstanceId) -> bool {
        self.instances.iter().any(|i| i.id == id)
    }
    pub(crate) fn update_elevation(&mut self, id: InstanceId, elevation: ResolvedElevation) {
        for instance in self.instances.iter_mut() {
            if instance.id == id {
                instance.elevation = elevation;
                self.node_submit.insert(id);
                self.needs_sort = true;
            }
        }
    }
    pub(crate) fn update_clip_context(&mut self, id: InstanceId, clip_context: Stem) {
        for instance in self.instances.iter_mut() {
            if instance.id == id {
                instance.clip_context = clip_context;
                self.node_submit.insert(id);
                self.needs_sort = true;
            }
        }
    }
    pub(crate) fn updated_nodes(&mut self, id: PipelineId, group_id: GroupId) -> Vec<Node> {
        let mut nodes = vec![];
        for changed in self.node_submit.drain().collect::<Vec<_>>() {
            let instance = self.instances.iter().find(|i| i.id == changed).unwrap();
            let order = self.order(changed);
            nodes.push(Node::new(
                instance.elevation,
                id,
                group_id,
                order,
                instance.clip_context,
                changed,
            ));
        }
        nodes
    }
    pub(crate) fn count(&self) -> u32 {
        self.instances.len() as u32
    }
    #[allow(unused)]
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
    #[allow(unused)]
    pub(crate) fn sort(&mut self) -> Vec<Swap> {
        let mut swaps = vec![];
        if !self.needs_sort {
            return swaps;
        }
        self.needs_sort = false;
        self.instances
            .sort_by(|a, b| match a.elevation.0.total_cmp(&b.elevation.0) {
                Ordering::Less => Ordering::Greater,
                Ordering::Equal => match a.clip_context.partial_cmp(&b.clip_context).unwrap() {
                    Ordering::Less => Ordering::Less,
                    Ordering::Equal => Ordering::Equal,
                    Ordering::Greater => Ordering::Greater,
                },
                Ordering::Greater => Ordering::Less,
            });
        for (new, instance) in self.instances.iter().enumerate() {
            if let Some(old) = self.cache.iter().position(|c| c.id == instance.id) {
                if new != old {
                    self.node_submit.insert(instance.id);
                    swaps.push(Swap {
                        old: old as Order,
                        id: instance.id,
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
    #[allow(unused)]
    pub(crate) fn swap(&mut self, swap: Swap) {
        let current = *self.cpu.get(swap.old as usize).unwrap();
        if self.queue.get(&swap.id).is_none() {
            self.queue(swap.id, current);
        }
    }
    pub(crate) fn write_cpu(&mut self, order: Order, data: I) {
        *self.cpu.get_mut(order as usize).unwrap() = data;
        if let Some(range) = self.write_range.as_mut() {
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
        *self.cpu.get_mut(order as usize).unwrap() = I::default();
    }
}

pub(crate) type Order = i32;
pub(crate) type InstanceId = i32;
