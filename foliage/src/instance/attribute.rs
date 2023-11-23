use crate::ginkgo::Ginkgo;
use crate::instance::{Index, InstanceCoordinator, InstanceOrdering};
use anymap::AnyMap;
use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;
use std::hash::Hash;

pub(crate) struct AttributeFn<Key: Hash + Eq> {
    pub(crate) create: Box<fn(&mut AnyMap, &mut AnyMap, &Ginkgo, u32)>,
    pub(crate) write: Box<fn(&InstanceOrdering<Key>, &mut AnyMap, &mut AnyMap, &Ginkgo)>,
    pub(crate) grow: Box<fn(&mut AnyMap, &Ginkgo, u32)>,
    pub(crate) remove: Box<fn(&mut AnyMap, &Vec<Index>)>,
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

type WriteRange = Option<(u32, u32)>;

pub(crate) struct InstanceAttribute<T> {
    cpu: Vec<T>,
    pub(crate) gpu: wgpu::Buffer,
    write_range: WriteRange,
}

impl<T: Default + Clone + Pod + Zeroable> InstanceAttribute<T> {
    pub(crate) fn new(ginkgo: &Ginkgo, count: u32) -> Self {
        let data = vec![T::default(); count as usize];
        let buffer = Self::gpu_buffer(ginkgo, count);
        Self {
            cpu: data,
            gpu: buffer,
            write_range: Some((0, count - 1)),
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
            *self.cpu.get_mut(index as usize).unwrap() = data;
            needs_write = true;
        }
        if needs_write {
            if let Some(range) = self.write_range.take() {
                let slice = &self.cpu[range.0 as usize..=range.1 as usize];
                let start_address = Ginkgo::buffer_address::<T>(range.0);
                ginkgo.queue.as_ref().unwrap().write_buffer(
                    &self.gpu,
                    start_address,
                    bytemuck::cast_slice(slice),
                );
            }
        }
    }
    pub(crate) fn remove(&mut self, indices: &Vec<Index>) {
        if !indices.is_empty() {
            let write_start = *indices.last().unwrap();
            for index in indices.iter() {
                self.cpu.remove(*index as usize);
            }
            self.write_range.replace((write_start, self.end()));
        }
    }
    pub(crate) fn grow(&mut self, ginkgo: &Ginkgo, count: u32) {
        self.cpu.resize(count as usize, T::default());
        self.gpu = Self::gpu_buffer(ginkgo, count);
        self.write_range.replace((0, self.end()));
    }
    fn end(&self) -> u32 {
        (self.cpu.len() - 1) as u32
    }
}

pub(crate) struct InstanceAttributeWriteQueue<Key, T>(pub(crate) HashMap<Key, T>);

impl<Key: PartialEq, T> InstanceAttributeWriteQueue<Key, T> {
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
    }
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
