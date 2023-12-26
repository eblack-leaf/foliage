use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use anymap::AnyMap;
use bytemuck::{Pod, Zeroable};
use serde::Deserialize;

use crate::ash::render_packet::RenderPacket;
use attribute::{AttributeFn, InstanceAttribute, InstanceAttributeWriteQueue};

use crate::coordinate::layer::Layer;
use crate::ginkgo::Ginkgo;

pub mod attribute;

pub type Index = u32;

#[derive(Default)]
pub struct InstanceCoordinatorBuilder<Key: Hash + Eq> {
    instance_fns: Vec<AttributeFn<Key>>,
    capacity: u32,
}

impl<Key: Hash + Eq + Clone + 'static> InstanceCoordinatorBuilder<Key> {
    pub fn new(capacity: u32) -> Self {
        Self {
            instance_fns: vec![],
            capacity,
        }
    }
    pub fn with_attribute<
        T: Default + Clone + Pod + Zeroable + 'static + for<'a> Deserialize<'a>,
    >(
        mut self,
    ) -> Self {
        self.instance_fns
            .push(AttributeFn::<Key>::for_attribute::<T>());
        self
    }
    pub fn build(self, ginkgo: &Ginkgo) -> InstanceCoordinator<Key> {
        InstanceCoordinator::new(ginkgo, self.instance_fns, self.capacity)
    }
}

pub struct InstanceCoordinator<Key: Hash + Eq> {
    ordering: InstanceOrdering<Key>,
    adds: HashSet<Key>,
    removes: HashSet<Key>,
    current_gpu_capacity: u32,
    attributes: AnyMap,
    attribute_writes: AnyMap,
    attribute_fns: Vec<AttributeFn<Key>>,
    needs_ordering: bool,
    layer_writes: HashMap<Key, Layer>,
}
pub(crate) struct InstanceOrdering<Key> {
    pub(crate) managed: HashMap<Key, Layer>,
    pub(crate) indices: HashMap<Key, Index>,
}
impl<Key> InstanceOrdering<Key> {
    pub(crate) fn new() -> Self {
        Self {
            managed: HashMap::new(),
            indices: HashMap::new(),
        }
    }
}
impl<Key: Hash + Eq + PartialEq> InstanceOrdering<Key> {
    pub(crate) fn index(&self, key: &Key) -> Option<Index> {
        self.indices.get(key).cloned()
    }
}

impl<Key: Hash + Eq + Clone + 'static> InstanceCoordinator<Key> {
    pub fn has_key(&self, key: &Key) -> bool {
        self.ordering.index(key).is_some()
    }
    pub fn prepare(&mut self, ginkgo: &Ginkgo) -> bool {
        let mut should_record = false;
        if let Some(removed) = self.removed_indices() {
            if !removed.is_empty() {
                should_record = true;
                self.needs_ordering = true;
            }
            Self::inner_remove(&self.attribute_fns, &mut self.attributes, &removed);
        }
        for key in self.adds.drain().collect::<Vec<Key>>() {
            self.ordering.managed.insert(key, Layer::default());
            self.needs_ordering = true;
            should_record = true;
        }
        // write layer_writes
        for (key, layer) in self.layer_writes.drain().collect::<Vec<(Key, Layer)>>() {
            self.ordering.managed.insert(key, layer);
            self.needs_ordering = true;
            should_record = true;
        }
        // sort by layer
        if self.needs_ordering {
            let mut order = self.ordering.managed.iter().map(|m| (m.0.clone(), *m.1)).collect::<Vec<(Key, Layer)>>();
            order.sort_by(|lhs, rhs| lhs.1.partial_cmp(&rhs.1).unwrap());
            order.reverse();
            self.ordering.indices.clear();
            for (index, (key, _)) in order.iter().enumerate() {
                self.ordering.indices.insert(key.clone(), index as Index);
            }
            Self::inner_reorder(
                &self.attribute_fns,
                &mut self.attributes,
                &mut self.attribute_writes,
                &self.ordering,
            );
            self.needs_ordering = false;
        }
        if let Some(count) = self.should_grow() {
            Self::inner_grow(&self.attribute_fns, &mut self.attributes, ginkgo, count);
            should_record = true;
        }
        Self::inner_write(
            &self.ordering,
            &self.attribute_fns,
            &mut self.attributes,
            &mut self.attribute_writes,
            ginkgo,
        );
        should_record
    }
    pub fn queue_add(&mut self, key: Key) {
        self.adds.insert(key);
    }
    pub fn queue_remove(&mut self, key: Key) {
        self.removes.insert(key);
    }
    pub fn queue_key_layer_change(&mut self, key: Key, layer: Layer) {
        self.layer_writes.insert(key, layer);
    }
    pub fn queue_write<T: Clone + 'static>(&mut self, key: Key, t: T) {
        if !self.removes.contains(&key) {
            self.attribute_writes
                .get_mut::<InstanceAttributeWriteQueue<Key, T>>()
                .unwrap()
                .0
                .insert(key, t);
        }
    }
    pub fn has_instances(&self) -> bool {
        self.instances() > 0
    }
    pub fn instances(&self) -> u32 {
        self.ordering.managed.len() as u32
    }
    pub fn buffer<T: 'static>(&self) -> &wgpu::Buffer {
        &self
            .attributes
            .get::<InstanceAttribute<Key, T>>()
            .unwrap()
            .gpu
    }
    pub fn queue_render_packet(&mut self, key: Key, render_packet: RenderPacket) {
        if !self.removes.contains(&key) {
            if let Some(layer) = render_packet.get::<Layer>() {
                self.queue_key_layer_change(key.clone(), layer);
            }
            Self::inner_queue_render_packet(
                &self.attribute_fns,
                &mut self.attribute_writes,
                key,
                render_packet,
            );
        }
    }
    fn inner_queue_render_packet(
        attr_fns: &[AttributeFn<Key>],
        attribute_writes: &mut AnyMap,
        key: Key,
        mut render_packet: RenderPacket,
    ) {
        for a_fn in attr_fns.iter() {
            (a_fn.queue_packet)(attribute_writes, key.clone(), &mut render_packet);
        }
    }
    fn queue_packet_wrapper<T: Clone + for<'a> Deserialize<'a> + 'static>(
        attribute_writes: &mut AnyMap,
        key: Key,
        render_packet: &mut RenderPacket,
    ) {
        if let Some(t) = render_packet.get::<T>() {
            attribute_writes
                .get_mut::<InstanceAttributeWriteQueue<Key, T>>()
                .unwrap()
                .0
                .insert(key, t);
        }
    }
    fn new(ginkgo: &Ginkgo, attribute_fns: Vec<AttributeFn<Key>>, capacity: u32) -> Self {
        let (attributes, attribute_writes) =
            Self::establish_attributes(ginkgo, &attribute_fns, capacity);
        Self {
            ordering: InstanceOrdering::new(),
            adds: HashSet::new(),
            removes: HashSet::new(),
            current_gpu_capacity: capacity,
            attributes,
            attribute_writes,
            attribute_fns,
            needs_ordering: true,
            layer_writes: HashMap::new(),
        }
    }
    fn inner_reorder(
        attribute_fns: &[AttributeFn<Key>],
        attributes: &mut AnyMap,
        attribute_writes: &mut AnyMap,
        ordering: &InstanceOrdering<Key>,
    ) {
        for attr_fn in attribute_fns.iter() {
            (attr_fn.reorder)(ordering, attributes, attribute_writes);
        }
    }
    fn reorder_wrapper<T: Clone + 'static>(
        ordering: &InstanceOrdering<Key>,
        attributes: &mut AnyMap,
        attribute_writes: &mut AnyMap,
    ) {
        for (key, _layer) in ordering.managed.iter() {
            if let Some(attr) = attributes
                .get::<InstanceAttribute<Key, T>>()
                .unwrap()
                .key_to_t
                .get(key)
            {
                if attribute_writes
                    .get::<InstanceAttributeWriteQueue<Key, T>>()
                    .unwrap()
                    .0
                    .get(key)
                    .is_none()
                {
                    attribute_writes
                        .get_mut::<InstanceAttributeWriteQueue<Key, T>>()
                        .unwrap()
                        .0
                        .insert(key.clone(), attr.clone());
                }
            }
        }
    }
    fn inner_write(
        ordering: &InstanceOrdering<Key>,
        attribute_fns: &[AttributeFn<Key>],
        attributes: &mut AnyMap,
        attribute_writes: &mut AnyMap,
        ginkgo: &Ginkgo,
    ) {
        for attr_fn in attribute_fns.iter() {
            (attr_fn.write)(ordering, attributes, attribute_writes, ginkgo);
        }
    }
    fn inner_grow(
        attribute_fns: &[AttributeFn<Key>],
        attributes: &mut AnyMap,
        ginkgo: &Ginkgo,
        capacity: u32,
    ) {
        for attr_fn in attribute_fns.iter() {
            (attr_fn.grow)(attributes, ginkgo, capacity);
        }
    }
    fn inner_remove(
        attribute_fns: &[AttributeFn<Key>],
        attributes: &mut AnyMap,
        removed: &Vec<(Key, Index)>,
    ) {
        for attr_fn in attribute_fns.iter() {
            (attr_fn.remove)(attributes, removed);
        }
    }
    fn establish_attributes(
        ginkgo: &Ginkgo,
        attribute_fns: &[AttributeFn<Key>],
        capacity: u32,
    ) -> (AnyMap, AnyMap) {
        let mut map = AnyMap::new();
        let mut write_map = AnyMap::new();
        Self::inner_establish(attribute_fns, &mut map, &mut write_map, ginkgo, capacity);
        (map, write_map)
    }
    fn inner_establish(
        attribute_fns: &[AttributeFn<Key>],
        attributes: &mut AnyMap,
        attribute_writes: &mut AnyMap,
        ginkgo: &Ginkgo,
        capacity: u32,
    ) {
        for attr_fn in attribute_fns.iter() {
            (attr_fn.create)(attributes, attribute_writes, ginkgo, capacity);
        }
    }
    fn create_wrapper<T: Default + Clone + Pod + Zeroable + 'static>(
        attributes: &mut AnyMap,
        attribute_writes: &mut AnyMap,
        ginkgo: &Ginkgo,
        count: u32,
    ) {
        attributes.insert(InstanceAttribute::<Key, T>::new(ginkgo, count));
        attribute_writes.insert(InstanceAttributeWriteQueue::<Key, T>::new());
    }
    fn write_wrapper<T: Default + Clone + Pod + Zeroable + 'static>(
        ordering: &InstanceOrdering<Key>,
        attributes: &mut AnyMap,
        attribute_writes: &mut AnyMap,
        ginkgo: &Ginkgo,
    ) {
        let queue = attribute_writes
            .get_mut::<InstanceAttributeWriteQueue<Key, T>>()
            .unwrap()
            .indexed(ordering);
        attributes
            .get_mut::<InstanceAttribute<Key, T>>()
            .unwrap()
            .write_from_queue(queue, ginkgo);
    }
    fn grow_wrapper<T: Default + Clone + Pod + Zeroable + 'static>(
        attributes: &mut AnyMap,
        ginkgo: &Ginkgo,
        new_capacity: u32,
    ) {
        attributes
            .get_mut::<InstanceAttribute<Key, T>>()
            .unwrap()
            .grow(ginkgo, new_capacity);
    }
    fn remove_wrapper<T: Default + Clone + Pod + Zeroable + 'static>(
        attributes: &mut AnyMap,
        removed: &Vec<(Key, Index)>,
    ) {
        attributes
            .get_mut::<InstanceAttribute<Key, T>>()
            .unwrap()
            .remove(removed);
    }
    fn should_grow(&mut self) -> Option<u32> {
        if self.ordering.managed.len() as u32 > self.current_gpu_capacity {
            self.current_gpu_capacity = self.ordering.managed.len() as u32;
            return Some(self.current_gpu_capacity);
        }
        None
    }
    fn removed_indices(&mut self) -> Option<Vec<(Key, Index)>> {
        if !self.removes.is_empty() {
            let mut removed_indices = self
                .removes
                .drain()
                .map(|key| {
                    let i = self.ordering.index(&key).unwrap();
                    (key, i)
                })
                .collect::<Vec<(Key, Index)>>();
            removed_indices.sort_by(|lhs, rhs| lhs.1.partial_cmp(&rhs.1).unwrap());
            removed_indices.reverse();
            for (key, index) in removed_indices.iter() {
                self.ordering.managed.remove(key);
            }
            return Some(removed_indices);
        }
        None
    }
}