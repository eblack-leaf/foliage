use crate::ginkgo::Ginkgo;
use anymap::AnyMap;
use bevy_ecs::entity::Entity;
use std::collections::{HashMap, HashSet};

pub type Index = u32;
pub type Key = u32; // use instead of Entity and make entity into key somehow
pub(crate) struct AttributeFn {
    create: Box<fn(&mut AnyMap, &Ginkgo, u32)>,
    write: Box<fn(&Vec<Entity>, &mut AnyMap, &mut AnyMap, &Ginkgo)>,
    grow: Box<fn(&mut AnyMap, &Ginkgo, u32)>,
    remove: Box<fn(&mut AnyMap, &Vec<Index>)>,
}
impl AttributeFn {
    pub(crate) fn for_attribute<T: Default + Clone + 'static>() -> Self {
        Self {
            create: Box::new(InstanceCoordinator::create_wrapper::<T>),
            write: Box::new(InstanceCoordinator::write_wrapper::<T>),
            grow: Box::new(InstanceCoordinator::grow_wrapper::<T>),
            remove: Box::new(InstanceCoordinator::remove_wrapper::<T>),
        }
    }
}
#[derive(Default)]
pub struct InstanceCoordinatorBuilder {
    instance_fns: Vec<AttributeFn>,
    capacity: u32,
}
impl InstanceCoordinatorBuilder {
    pub fn new(capacity: u32) -> Self {
        Self {
            instance_fns: vec![],
            capacity,
        }
    }
    pub fn with_attribute<T: Default + Clone + 'static>(mut self) -> Self {
        self.instance_fns.push(AttributeFn::for_attribute::<T>());
        self
    }
    pub fn build(mut self, ginkgo: &Ginkgo) -> InstanceCoordinator {
        InstanceCoordinator::new(ginkgo, self.instance_fns, self.capacity)
    }
}
pub struct InstanceCoordinator {
    ordering: Vec<Entity>,
    adds: HashSet<Entity>,
    removes: HashSet<Entity>,
    current_gpu_capacity: u32,
    attributes: AnyMap,
    attribute_writes: AnyMap,
    attribute_fns: Vec<AttributeFn>,
}

impl InstanceCoordinator {
    pub fn prepare(&mut self, ginkgo: &Ginkgo) -> bool {
        let mut should_record = false;
        if let Some(removed) = self.removed_indices() {
            Self::inner_remove(&self.attribute_fns, &mut self.attributes, &removed);
        }
        for add in self.adds.drain().collect::<Vec<Entity>>() {
            self.insert(add);
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
    pub fn queue_add(&mut self, entity: Entity) {
        self.adds.insert(entity);
    }
    pub fn queue_remove(&mut self, entity: Entity) {
        self.removes.insert(entity);
    }
    pub fn queue_write<T: 'static>(&mut self, entity: Entity, t: T) {
        self.attributes
            .get_mut::<InstanceAttributeWriteQueue<T>>()
            .unwrap()
            .0
            .insert(entity, t);
    }
    fn new(ginkgo: &Ginkgo, attribute_fns: Vec<AttributeFn>, capacity: u32) -> Self {
        Self {
            ordering: vec![],
            adds: HashSet::new(),
            removes: HashSet::new(),
            current_gpu_capacity: capacity,
            attributes: Self::establish_attributes(ginkgo, &attribute_fns, capacity),
            attribute_writes: AnyMap::new(),
            attribute_fns,
        }
    }
    fn inner_write(
        ordering: &Vec<Entity>,
        attribute_fns: &Vec<AttributeFn>,
        attributes: &mut AnyMap,
        attribute_writes: &mut AnyMap,
        ginkgo: &Ginkgo,
    ) {
        for attr_fn in attribute_fns.iter() {
            (attr_fn.write)(ordering, attributes, attribute_writes, ginkgo);
        }
    }
    fn inner_grow(
        attribute_fns: &Vec<AttributeFn>,
        attributes: &mut AnyMap,
        ginkgo: &Ginkgo,
        capacity: u32,
    ) {
        for attr_fn in attribute_fns.iter() {
            (attr_fn.grow)(attributes, ginkgo, capacity);
        }
    }
    fn inner_remove(
        attribute_fns: &Vec<AttributeFn>,
        attributes: &mut AnyMap,
        removed: &Vec<Index>,
    ) {
        for attr_fn in attribute_fns.iter() {
            (attr_fn.remove)(attributes, removed);
        }
    }
    fn establish_attributes(
        ginkgo: &Ginkgo,
        attribute_fns: &Vec<AttributeFn>,
        capacity: u32,
    ) -> AnyMap {
        let mut map = AnyMap::new();
        Self::inner_establish(attribute_fns, &mut map, ginkgo, capacity);
        map
    }
    fn inner_establish(
        attribute_fns: &Vec<AttributeFn>,
        attributes: &mut AnyMap,
        ginkgo: &Ginkgo,
        capacity: u32,
    ) {
        for attr_fn in attribute_fns.iter() {
            (attr_fn.create)(attributes, ginkgo, capacity);
        }
    }
    fn create_wrapper<T: Default + Clone + 'static>(
        attributes: &mut AnyMap,
        ginkgo: &Ginkgo,
        count: u32,
    ) {
        attributes.insert(InstanceAttribute::<T>::new(ginkgo, count));
    }
    fn write_wrapper<T: Default + Clone + 'static>(
        ordering: &Vec<Entity>,
        attributes: &mut AnyMap,
        attribute_writes: &mut AnyMap,
        ginkgo: &Ginkgo,
    ) {
        // TODO semantics of sending with index to writes
        let mut queue = attribute_writes
            .get_mut::<InstanceAttributeWriteQueue<T>>()
            .unwrap()
            .0
            .drain()
            .map(|(e, w)| {
                if let Some(index) = Self::index(ordering, e) {
                    return Some((index, w));
                }
                None
            })
            .collect::<Vec<Option<(Index, T)>>>();
        queue.retain(|a| a.is_some());
        let queue = queue
            .drain(..)
            .map(|a| a.unwrap())
            .collect::<Vec<(Index, T)>>();
        attributes
            .get_mut::<InstanceAttribute<T>>()
            .unwrap()
            .write_from_queue(queue, ginkgo);
    }
    fn grow_wrapper<T: Default + Clone + 'static>(
        attributes: &mut AnyMap,
        ginkgo: &Ginkgo,
        new_capacity: u32,
    ) {
        attributes
            .get_mut::<InstanceAttribute<T>>()
            .unwrap()
            .grow(ginkgo, new_capacity);
    }
    fn remove_wrapper<T: Default + Clone + 'static>(attributes: &mut AnyMap, removed: &Vec<Index>) {
        attributes
            .get_mut::<InstanceAttribute<T>>()
            .unwrap()
            .remove(removed);
    }
    fn index(ordering: &Vec<Entity>, entity: Entity) -> Option<u32> {
        let mut index = 0;
        for a in ordering.iter() {
            if *a == entity {
                return Some(index);
            }
            index += 1;
        }
        None
    }
    fn insert(&mut self, entity: Entity) {
        self.ordering.push(entity);
    }
    fn should_grow(&mut self) -> Option<u32> {
        if self.ordering.len() as u32 > self.current_gpu_capacity {
            self.current_gpu_capacity = self.ordering.len() as u32;
            return Some(self.current_gpu_capacity);
        }
        None
    }
    fn removed_indices(&mut self) -> Option<Vec<Index>> {
        if !self.removes.is_empty() {
            let mut removed_indices = self
                .removes
                .drain()
                .collect::<Vec<Entity>>()
                .drain(..)
                .map(|e| Self::index(&self.ordering, e).unwrap())
                .collect::<Vec<Index>>();
            removed_indices.sort();
            removed_indices.reverse();
            for index in removed_indices.iter() {
                self.ordering.remove(*index as usize);
            }
            return Some(removed_indices);
        }
        None
    }
}
struct InstanceAttribute<T> {
    cpu: Vec<T>,
    gpu: wgpu::Buffer,
}

impl<T: Default + Clone> InstanceAttribute<T> {
    fn new(ginkgo: &Ginkgo, count: u32) -> Self {
        Self {
            cpu: vec![T::default(); count as usize],
            gpu: ginkgo.device().create_buffer(&wgpu::BufferDescriptor {
                label: Some("instance-attribute-buffer"),
                size: Ginkgo::buffer_address::<T>(count),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        }
    }
    pub(crate) fn write_from_queue(&self, queue: Vec<(Index, T)>, ginkgo: &Ginkgo) {
        todo!()
    }
    fn remove(&mut self, indices: &Vec<Index>) {
        if !indices.is_empty() {
            let write_start = *indices.last().unwrap();
            for index in indices.iter() {
                self.cpu.remove(*index as usize);
            }
            // go from write_start to current cpu.len() - 1 as write range
            // only set write range here
        }
    }
    fn add(&mut self, index: Index, t: T) {
        todo!()
    }
    fn grow(&mut self, ginkgo: &Ginkgo, count: u32) {
        todo!()
    }
}
#[derive(Default)]
pub(crate) struct InstanceAttributeWriteQueue<T>(pub(crate) HashMap<Entity, T>);
