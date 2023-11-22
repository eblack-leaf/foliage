use crate::ginkgo::Ginkgo;
use anymap::AnyMap;
use bytemuck::{Pod, Zeroable};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

pub type Index = u32;
pub(crate) struct AttributeFn<Key: Hash + Eq> {
    create: Box<fn(&mut AnyMap, &Ginkgo, u32)>,
    write: Box<fn(&InstanceOrdering<Key>, &mut AnyMap, &mut AnyMap, &Ginkgo)>,
    grow: Box<fn(&mut AnyMap, &Ginkgo, u32)>,
    remove: Box<fn(&mut AnyMap, &Vec<Index>)>,
}
impl<Key: Hash + Eq + 'static> AttributeFn<Key> {
    pub(crate) fn for_attribute<T: Default + Clone + Pod + Zeroable + 'static>() -> Self {
        Self {
            create: Box::new(InstanceCoordinator::<Key>::create_wrapper::<T>),
            write: Box::new(InstanceCoordinator::<Key>::write_wrapper::<T>),
            grow: Box::new(InstanceCoordinator::<Key>::grow_wrapper::<T>),
            remove: Box::new(InstanceCoordinator::<Key>::remove_wrapper::<T>),
        }
    }
}
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
    pub fn build(mut self, ginkgo: &Ginkgo) -> InstanceCoordinator<Key> {
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
        let mut index = 0;
        for a in self.0.iter() {
            if *a == key {
                return Some(index);
            }
            index += 1;
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
        self.attributes
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
        Self {
            ordering: InstanceOrdering(vec![]),
            adds: HashSet::new(),
            removes: HashSet::new(),
            current_gpu_capacity: capacity,
            attributes: Self::establish_attributes(ginkgo, &attribute_fns, capacity),
            attribute_writes: AnyMap::new(),
            attribute_fns,
        }
    }
    fn inner_write(
        ordering: &InstanceOrdering<Key>,
        attribute_fns: &Vec<AttributeFn<Key>>,
        attributes: &mut AnyMap,
        attribute_writes: &mut AnyMap,
        ginkgo: &Ginkgo,
    ) {
        for attr_fn in attribute_fns.iter() {
            (attr_fn.write)(ordering, attributes, attribute_writes, ginkgo);
        }
    }
    fn inner_grow(
        attribute_fns: &Vec<AttributeFn<Key>>,
        attributes: &mut AnyMap,
        ginkgo: &Ginkgo,
        capacity: u32,
    ) {
        for attr_fn in attribute_fns.iter() {
            (attr_fn.grow)(attributes, ginkgo, capacity);
        }
    }
    fn inner_remove(
        attribute_fns: &Vec<AttributeFn<Key>>,
        attributes: &mut AnyMap,
        removed: &Vec<Index>,
    ) {
        for attr_fn in attribute_fns.iter() {
            (attr_fn.remove)(attributes, removed);
        }
    }
    fn establish_attributes(
        ginkgo: &Ginkgo,
        attribute_fns: &Vec<AttributeFn<Key>>,
        capacity: u32,
    ) -> AnyMap {
        let mut map = AnyMap::new();
        Self::inner_establish(attribute_fns, &mut map, ginkgo, capacity);
        map
    }
    fn inner_establish(
        attribute_fns: &Vec<AttributeFn<Key>>,
        attributes: &mut AnyMap,
        ginkgo: &Ginkgo,
        capacity: u32,
    ) {
        for attr_fn in attribute_fns.iter() {
            (attr_fn.create)(attributes, ginkgo, capacity);
        }
    }
    fn create_wrapper<T: Default + Clone + Pod + Zeroable + 'static>(
        attributes: &mut AnyMap,
        ginkgo: &Ginkgo,
        count: u32,
    ) {
        attributes.insert(InstanceAttribute::<T>::new(ginkgo, count));
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
type WriteRange = Option<(u32, u32)>;
struct InstanceAttribute<T> {
    cpu: Vec<T>,
    gpu: wgpu::Buffer,
    write_range: WriteRange,
}

impl<T: Default + Clone + Pod + Zeroable> InstanceAttribute<T> {
    fn new(ginkgo: &Ginkgo, count: u32) -> Self {
        let data = vec![T::default(); count as usize];
        let buffer = Self::gpu_buffer(ginkgo, count);
        Self {
            cpu: data,
            gpu: buffer,
            write_range: None,
        }
    }

    fn gpu_buffer(ginkgo: &Ginkgo, count: u32) -> wgpu::Buffer {
        ginkgo.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("instance-attribute-buffer"),
            size: Ginkgo::buffer_address::<T>(count),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }
    pub(crate) fn write_from_queue(
        &mut self,
        queue: InstanceAttributeIndexedWriteQueue<T>,
        ginkgo: &Ginkgo,
    ) {
        let mut needs_write = false;
        for (index, data) in queue.0 {
            if let Some((start, end)) = self.write_range.as_mut() {
                if index < *start {
                    *start = index;
                } else if index > *end {
                    *end = index;
                }
            }
            self.cpu.insert(index as usize, data);
            needs_write = true;
        }
        if needs_write {
            if let Some(range) = self.write_range.take() {
                let slice = &self.cpu[range.0 as usize..(range.1 + 1) as usize];
                let start_address = Ginkgo::buffer_address::<T>(range.0);
                ginkgo.queue.as_ref().unwrap().write_buffer(
                    &self.gpu,
                    start_address,
                    bytemuck::cast_slice(slice),
                );
            }
        }
    }
    fn remove(&mut self, indices: &Vec<Index>) {
        if !indices.is_empty() {
            let write_start = *indices.last().unwrap();
            for index in indices.iter() {
                self.cpu.remove(*index as usize);
            }
            self.write_range.replace((write_start, self.end()));
        }
    }
    fn grow(&mut self, ginkgo: &Ginkgo, count: u32) {
        self.cpu.resize(count as usize, T::default());
        self.gpu = Self::gpu_buffer(ginkgo, count);
        self.write_range.replace((0, self.end()));
    }
    fn end(&self) -> u32 {
        (self.cpu.len() - 1) as u32
    }
}
#[derive(Default)]
pub(crate) struct InstanceAttributeWriteQueue<Key, T>(pub(crate) HashMap<Key, T>);
impl<Key: PartialEq, T> InstanceAttributeWriteQueue<Key, T> {
    pub(crate) fn indexed(
        &mut self,
        ordering: &InstanceOrdering<Key>,
    ) -> InstanceAttributeIndexedWriteQueue<T> {
        InstanceAttributeIndexedWriteQueue(
            self.0
                .drain()
                .map(|(key, t)| {
                    let index = ordering.index(key).unwrap();
                    (index, t)
                })
                .collect::<Vec<(Index, T)>>(),
        )
    }
}
#[derive(Default)]
pub(crate) struct InstanceAttributeIndexedWriteQueue<T>(pub(crate) Vec<(Index, T)>);
