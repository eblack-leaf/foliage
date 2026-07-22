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
    /// `id -> row` mirror of `instances` so per-item lookups are O(1) instead of a linear
    /// scan — `order()`/`has_instance()` are called once per queued attribute per frame, which
    /// made frame preparation quadratic in instance count before this map existed.
    pub(crate) orders: HashMap<InstanceId, Order>,
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
            orders: HashMap::new(),
            node_submit: HashSet::new(),
            id_gen: 0,
            gen_pool: Default::default(),
            capacity,
            needs_sort: false,
        }
    }
    pub(crate) fn add(&mut self, instance: Instance) {
        tracing::trace!(id = instance.id, "instance-coordinator: add");
        self.orders
            .insert(instance.id, self.instances.len() as Order);
        self.instances.push(instance);
        self.node_submit.insert(instance.id);
        self.needs_sort = true;
    }
    pub(crate) fn has_instance(&self, id: InstanceId) -> bool {
        self.orders.contains_key(&id)
    }
    pub(crate) fn update_elevation(&mut self, id: InstanceId, elevation: ResolvedElevation) {
        if let Some(order) = self.orders.get(&id) {
            self.instances[*order as usize].elevation = elevation;
            self.node_submit.insert(id);
            self.needs_sort = true;
        }
    }
    pub(crate) fn update_clip_context(&mut self, id: InstanceId, clip_context: Stem) {
        if let Some(order) = self.orders.get(&id) {
            self.instances[*order as usize].clip_context = clip_context;
            self.node_submit.insert(id);
            self.needs_sort = true;
        }
    }
    pub(crate) fn updated_nodes(&mut self, id: PipelineId, group_id: GroupId) -> Vec<Node> {
        let mut nodes = vec![];
        for changed in self.node_submit.drain().collect::<Vec<_>>() {
            let order = *self.orders.get(&changed).unwrap();
            let instance = self.instances[order as usize];
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
            // geometric growth amortizes reallocation when instances stream in one-by-one
            // (every grow re-uploads the whole buffer)
            let len = self.instances.len() as u32;
            let new = (self.capacity * 3 / 2).max(len + REPEAT_ALLOCATION_AVOIDANCE);
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
        self.instances.sort_by(|a, b| {
            match a.elevation.front_to_back(&b.elevation) {
                Ordering::Equal => a.clip_context.partial_cmp(&b.clip_context).unwrap(),
                ord => ord,
            }
        });
        let old_orders = self
            .cache
            .iter()
            .enumerate()
            .map(|(order, c)| (c.id, order))
            .collect::<HashMap<_, _>>();
        self.orders.clear();
        for (new, instance) in self.instances.iter().enumerate() {
            self.orders.insert(instance.id, new as Order);
            if let Some(old) = old_orders.get(&instance.id) {
                if new != *old {
                    self.node_submit.insert(instance.id);
                    swaps.push(Swap {
                        old: *old as Order,
                        id: instance.id,
                    })
                }
            }
        }
        self.cache = self.instances.clone();
        swaps
    }
    pub(crate) fn order(&self, id: InstanceId) -> Order {
        *self.orders.get(&id).unwrap_or_else(|| {
            tracing::error!(
                id,
                known = ?self.orders.keys().collect::<Vec<_>>(),
                "instance-coordinator: no order for id"
            );
            panic!("no instance-order for id {}", id)
        })
    }
    pub(crate) fn remove(&mut self, order: Order) {
        // `swap_remove` moves the *last* element into the vacated slot instead of shifting
        // every subsequent element down by one -- the previous `Vec::remove` + "shift every
        // row after it, updating `orders` for each" was O(n) per removal, so removing n
        // instances in one batch (e.g. every glyph instance when a huge pasted block gets
        // deleted) was O(n^2): the actual cause of the multi-second-to-minutes freezes/crash
        // reported on deleting a large selection. Only the one relocated instance's `orders`
        // entry needs updating -- `orders` just needs to stay a dense 0..len() mapping to
        // GPU buffer slots, not any particular shifted order, since `sort()` (always run
        // after any remove, `needs_sort = true` below) fully re-establishes render order from
        // elevation regardless of this method's internal physical layout.
        let removed = self.instances.swap_remove(order as usize);
        tracing::trace!(id = removed.id, order, "instance-coordinator: remove");
        self.orders.remove(&removed.id);
        if let Some(relocated) = self.instances.get(order as usize) {
            self.orders.insert(relocated.id, order);
        }
        self.needs_sort = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ResolvedElevation;

    fn instance(id: InstanceId, elevation: f32) -> Instance {
        Instance::new(ResolvedElevation::new(elevation), Stem::default(), id)
    }

    #[test]
    fn add_makes_has_instance_true_and_flags_a_sort() {
        let mut c = InstanceCoordinator::new(4);
        assert!(!c.has_instance(1));
        c.add(instance(1, 0.0));
        assert!(c.has_instance(1));
        assert!(c.needs_sort);
    }

    #[test]
    fn generate_id_hands_out_sequential_ids_when_the_pool_is_empty() {
        let mut c = InstanceCoordinator::new(4);
        assert_eq!(c.generate_id(), 0);
        assert_eq!(c.generate_id(), 1);
        assert_eq!(c.generate_id(), 2);
    }

    #[test]
    fn generate_id_reuses_a_pooled_id_before_minting_a_new_one() {
        let mut c = InstanceCoordinator::new(4);
        let _ = c.generate_id(); // 0
        let _ = c.generate_id(); // 1
        c.gen_pool.insert(0);
        assert_eq!(c.generate_id(), 0, "a freed id should come back before minting id 2");
        assert_eq!(c.generate_id(), 2, "pool now empty -- back to sequential minting");
    }

    #[test]
    fn sort_orders_instances_so_a_smaller_raw_elevation_value_sorts_last() {
        // mirrors `ResolvedElevation`'s own inverted `PartialOrd` (see `elevation.rs`'
        // tests): a *smaller* raw value means *more in front*, and the render list is
        // built back-to-front, so it needs to land *last* -- drawn last (highest
        // painter's-algorithm order) is exactly what "most in front" has to mean.
        let mut c = InstanceCoordinator::new(4);
        c.add(instance(10, 5.0)); // furthest back
        c.add(instance(20, 1.0)); // furthest in front (smallest raw value)
        c.add(instance(30, 3.0)); // middle
        c.sort();
        let order: Vec<InstanceId> = c.instances.iter().map(|i| i.id).collect();
        assert_eq!(order, vec![10, 30, 20], "back-to-front: 5.0, 3.0, then 1.0 last");
    }

    #[test]
    fn sort_is_a_no_op_until_needs_sort_is_set() {
        let mut c = InstanceCoordinator::new(4);
        c.add(instance(1, 5.0));
        c.sort();
        assert!(!c.needs_sort, "sanity: sort() itself clears the flag");
        let swaps = c.sort();
        assert!(swaps.is_empty(), "nothing changed since the last sort -- no swaps to report");
    }

    #[test]
    fn remove_reindexes_the_relocated_instance_and_drops_the_removed_one() {
        // the exact property the `swap_remove`-based rewrite (see `remove`'s own doc
        // comment -- this used to be an O(n) `Vec::remove` per call, causing real
        // multi-second freezes deleting a large selection) has to preserve: every
        // *surviving* id still resolves to a valid, correct order afterward.
        let mut c = InstanceCoordinator::new(4);
        c.add(instance(1, 0.0));
        c.add(instance(2, 0.0));
        c.add(instance(3, 0.0));
        let order_of_2 = c.order(2);
        c.remove(c.order(1)); // remove the first one; swap_remove relocates the last (3) into its slot
        assert!(!c.has_instance(1));
        assert!(c.has_instance(2));
        assert!(c.has_instance(3));
        assert_eq!(c.order(2), order_of_2, "the untouched middle instance's order shouldn't move");
        assert_eq!(c.instances.len(), 2);
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
        if !self.queue.contains_key(&swap.id) {
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
