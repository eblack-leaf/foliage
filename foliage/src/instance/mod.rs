use std::collections::HashSet;
use std::hash::Hash;

use anymap::AnyMap;
use bytemuck::{Pod, Zeroable};

use attribute::{AttributeFn, InstanceAttribute, InstanceAttributeWriteQueue};

use crate::ginkgo::Ginkgo;

pub mod attribute;

pub type Index = u32;

#[derive(Default)]
pub struct InstanceCoordinatorBuilder<Key: Hash + Eq> {
    instance_fns: Vec<AttributeFn<Key>>,
    capacity: u32,
}

impl<Key: Hash + Eq + 'static> InstanceCoordinatorBuilder<Key> {
    pub fn new(capacity: u32) -> Self {
        Self {
            instance_fns: vec![],
            capacity,
        }
    }
    pub fn with_attribute<T: Default + Clone + Pod + Zeroable + 'static>(mut self) -> Self {
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
}

pub(crate) struct InstanceOrdering<Key>(pub(crate) Vec<Key>);

impl<Key: PartialEq> InstanceOrdering<Key> {
    pub(crate) fn index(&self, key: Key) -> Option<Index> {
        for (index, a) in self.0.iter().enumerate() {
            if *a == key {
                return Some(index as Index);
            }
        }
        None
    }
}

impl<Key: Hash + Eq + 'static> InstanceCoordinator<Key> {
    pub fn prepare(&mut self, ginkgo: &Ginkgo) -> bool {
        let mut should_record = false;
        if let Some(removed) = self.removed_indices() {
            if !removed.is_empty() {
                should_record = true;
            }
            Self::inner_remove(&self.attribute_fns, &mut self.attributes, &removed);
        }
        for add in self.adds.drain().collect::<Vec<Key>>() {
            self.insert(add);
            should_record = true;
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
    pub fn queue_write<T: 'static>(&mut self, key: Key, t: T) {
        self.attribute_writes
            .get_mut::<InstanceAttributeWriteQueue<Key, T>>()
            .unwrap()
            .0
            .insert(key, t);
    }
    pub fn has_instances(&self) -> bool {
        self.instances() > 0
    }
    pub fn instances(&self) -> u32 {
        self.ordering.0.len() as u32
    }
    pub fn buffer<T: 'static>(&self) -> &wgpu::Buffer {
        &self.attributes.get::<InstanceAttribute<T>>().unwrap().gpu
    }
    fn new(ginkgo: &Ginkgo, attribute_fns: Vec<AttributeFn<Key>>, capacity: u32) -> Self {
        let (attributes, attribute_writes) =
            Self::establish_attributes(ginkgo, &attribute_fns, capacity);
        Self {
            ordering: InstanceOrdering(vec![]),
            adds: HashSet::new(),
            removes: HashSet::new(),
            current_gpu_capacity: capacity,
            attributes,
            attribute_writes,
            attribute_fns,
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
        removed: &Vec<Index>,
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
        attributes.insert(InstanceAttribute::<T>::new(ginkgo, count));
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
            .get_mut::<InstanceAttribute<T>>()
            .unwrap()
            .write_from_queue(queue, ginkgo);
    }
    fn grow_wrapper<T: Default + Clone + Pod + Zeroable + 'static>(
        attributes: &mut AnyMap,
        ginkgo: &Ginkgo,
        new_capacity: u32,
    ) {
        attributes
            .get_mut::<InstanceAttribute<T>>()
            .unwrap()
            .grow(ginkgo, new_capacity);
    }
    fn remove_wrapper<T: Default + Clone + Pod + Zeroable + 'static>(
        attributes: &mut AnyMap,
        removed: &Vec<Index>,
    ) {
        attributes
            .get_mut::<InstanceAttribute<T>>()
            .unwrap()
            .remove(removed);
    }
    fn insert(&mut self, key: Key) {
        self.ordering.0.push(key);
    }
    fn should_grow(&mut self) -> Option<u32> {
        if self.ordering.0.len() as u32 > self.current_gpu_capacity {
            self.current_gpu_capacity = self.ordering.0.len() as u32;
            return Some(self.current_gpu_capacity);
        }
        None
    }
    fn removed_indices(&mut self) -> Option<Vec<Index>> {
        if !self.removes.is_empty() {
            let mut removed_indices = self
                .removes
                .drain()
                .map(|key| self.ordering.index(key).unwrap())
                .collect::<Vec<Index>>();
            removed_indices.sort();
            removed_indices.reverse();
            for index in removed_indices.iter() {
                self.ordering.0.remove(*index as usize);
            }
            return Some(removed_indices);
        }
        None
    }
}
