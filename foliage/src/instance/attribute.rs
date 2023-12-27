use std::collections::HashMap;
use std::hash::Hash;

use crate::ash::render_packet::RenderPacket;
use anymap::AnyMap;
use bytemuck::{Pod, Zeroable};
use serde::Deserialize;

use crate::ginkgo::Ginkgo;
use crate::instance::{Index, InstanceCoordinator, InstanceOrdering};

pub(crate) struct AttributeFn<Key: Hash + Eq> {
    pub(crate) create: Box<fn(&mut AnyMap, &mut AnyMap, &Ginkgo, u32)>,
    pub(crate) write: Box<fn(&InstanceOrdering<Key>, &mut AnyMap, &mut AnyMap, &Ginkgo, u32)>,
    pub(crate) grow: Box<fn(&mut AnyMap, &Ginkgo, u32, u32)>,
    pub(crate) remove: Box<fn(&mut AnyMap, &Vec<(Key, Index)>, u32)>,
    pub(crate) reorder: Box<fn(&InstanceOrdering<Key>, &mut AnyMap, &mut AnyMap)>,
    pub(crate) queue_packet: Box<fn(&mut AnyMap, Key, &mut RenderPacket)>,
}

impl<Key: Hash + Eq + Clone + 'static> AttributeFn<Key> {
    pub(crate) fn for_attribute<
        T: Default + Clone + Pod + Zeroable + 'static + for<'a> Deserialize<'a>,
    >() -> Self {
        Self {
            create: Box::new(InstanceCoordinator::<Key>::create_wrapper::<T>),
            write: Box::new(InstanceCoordinator::<Key>::write_wrapper::<T>),
            grow: Box::new(InstanceCoordinator::<Key>::grow_wrapper::<T>),
            remove: Box::new(InstanceCoordinator::<Key>::remove_wrapper::<T>),
            reorder: Box::new(InstanceCoordinator::<Key>::reorder_wrapper::<T>),
            queue_packet: Box::new(InstanceCoordinator::<Key>::queue_packet_wrapper::<T>),
        }
    }
}

type WriteRange = Option<(u32, u32)>;

pub(crate) struct InstanceAttribute<Key: Hash + Eq + Clone + 'static, T> {
    pub(crate) key_to_t: HashMap<Key, T>,
    cpu: Vec<T>,
    pub(crate) gpu: wgpu::Buffer,
    write_range: WriteRange,
}

impl<Key: Hash + Eq + Clone + 'static, T: Default + Clone + Pod + Zeroable>
    InstanceAttribute<Key, T>
{
    pub(crate) fn new(ginkgo: &Ginkgo, count: u32) -> Self {
        let data = vec![T::default(); count as usize];
        let buffer = Self::gpu_buffer(ginkgo, count);
        Self {
            key_to_t: HashMap::new(),
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
        queue: InstanceAttributeIndexedWriteQueue<Key, T>,
        ginkgo: &Ginkgo,
        instances: u32,
    ) {
        let mut needs_write = false;
        for (key, index, data) in queue.0 {
            if self.write_range.is_none() {
                self.write_range.replace((instances, instances));
            }
            if let Some((start, end)) = self.write_range.as_mut() {
                if index < *start {
                    *start = index;
                }
            }
            *self.cpu.get_mut(index as usize).unwrap() = data;
            self.key_to_t.insert(key, data);
            needs_write = true;
        }
        if needs_write {
            if let Some(range) = self.write_range.take() {
                if range.0 > range.1 {
                    return;
                }
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
    pub(crate) fn remove(&mut self, indices: &Vec<(Key, Index)>, instances: u32) {
        if !indices.is_empty() {
            let write_start = indices.last().unwrap().1;
            for (key, index) in indices.iter() {
                self.key_to_t.remove(key);
            }
            self.write_range
                .replace((write_start.checked_sub(1).unwrap_or_default(), instances));
        }
    }
    pub(crate) fn grow(&mut self, ginkgo: &Ginkgo, count: u32, instances: u32) {
        self.cpu.resize(count as usize, T::default());
        self.gpu = Self::gpu_buffer(ginkgo, count);
        self.write_range.replace((0, instances));
    }
}

pub(crate) struct InstanceAttributeWriteQueue<Key, T>(pub(crate) HashMap<Key, T>);

impl<Key: Hash + Eq + PartialEq + Clone + 'static, T> InstanceAttributeWriteQueue<Key, T> {
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
    }
    pub(crate) fn indexed(
        &mut self,
        ordering: &InstanceOrdering<Key>,
    ) -> InstanceAttributeIndexedWriteQueue<Key, T> {
        InstanceAttributeIndexedWriteQueue(
            self.0
                .drain()
                .map(|(key, t)| {
                    let index = ordering.index(&key).unwrap();
                    (key, index, t)
                })
                .collect::<Vec<(Key, Index, T)>>(),
        )
    }
}

#[derive(Default)]
pub(crate) struct InstanceAttributeIndexedWriteQueue<Key: Hash + Eq + 'static, T>(
    pub(crate) Vec<(Key, Index, T)>,
);