use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

use bevy_ecs::world::World;
use bytemuck::{Pod, Zeroable};
use wgpu::{BufferDescriptor, BufferUsages};

use crate::ginkgo::Ginkgo;

pub struct Instances<Key: Hash + Eq + Copy + Clone> {
    world: World,
    capacity: u32,
    map: HashMap<Key, usize>,
    order: Vec<Key>,
    removal_fns: Vec<fn(&mut World, usize)>,
    grow_fns: Vec<fn(&mut World, u32, &Ginkgo)>,
    cpu_to_gpu: Vec<fn(&mut World, &Ginkgo)>,
    update_needed: bool,
    removal_queue: HashSet<Key>,
}

impl<Key: Hash + Eq + Copy + Clone> Instances<Key> {
    pub fn get_attr<A: Pod + Zeroable + Default>(&self, key: Key) -> A {
        let index = *self.map.get(&key).unwrap();
        *self
            .world
            .get_non_send_resource::<Attribute<A>>()
            .unwrap()
            .cpu
            .get(index)
            .expect("unmapped key")
    }
    pub fn num_instances(&self) -> u32 {
        self.order.len() as u32
    }
    pub fn checked_write<A: Pod + Zeroable + Default + Debug>(&mut self, key: Key, a: A) {
        if !self.has_key(&key) {
            self.add(key);
        }
        self.queue_write(key, a);
    }
    pub fn buffer<A: Pod + Zeroable + Default>(&self) -> &wgpu::Buffer {
        &self
            .world
            .get_non_send_resource::<Attribute<A>>()
            .expect("attribute")
            .gpu
    }
    pub fn new(initial_capacity: u32) -> Self {
        Self {
            world: World::default(),
            capacity: initial_capacity,
            map: HashMap::new(),
            order: vec![],
            removal_fns: vec![],
            grow_fns: vec![],
            cpu_to_gpu: vec![],
            update_needed: false,
            removal_queue: HashSet::new(),
        }
    }
    pub fn with_attribute<A: Pod + Zeroable + Default + Debug>(mut self, ginkgo: &Ginkgo) -> Self {
        self.world
            .insert_non_send_resource(Attribute::<A>::new(ginkgo, self.capacity));
        self.removal_fns.push(|w, i| {
            w.get_non_send_resource_mut::<Attribute<A>>()
                .expect("attribute")
                .remove(i);
        });
        self.grow_fns.push(|w, c, g| {
            w.get_non_send_resource_mut::<Attribute<A>>()
                .expect("attribute")
                .grow(g, c);
        });
        self.cpu_to_gpu.push(|w, g| {
            w.get_non_send_resource_mut::<Attribute<A>>()
                .expect("attribute")
                .write_cpu_to_gpu(g);
        });
        self
    }
    pub fn clear(&mut self) -> Vec<Key> {
        let mut removed = vec![];
        let cloned = self.order.clone();
        for e in cloned {
            removed.push(e);
            self.queue_remove(e);
        }
        self.process_removals();
        removed
    }
    pub fn queue_remove(&mut self, key: Key) {
        if self.has_key(&key) {
            self.removal_queue.insert(key);
        }
    }
    pub(crate) fn remove(&mut self, index: usize) {
        self.order.remove(index);
        for r_fn in self.removal_fns.iter() {
            (r_fn)(&mut self.world, index);
            self.update_needed = true;
        }
    }
    pub fn add(&mut self, key: Key) {
        if !self.has_key(&key) {
            let index = self.order.len();
            self.order.push(key);
            self.map.insert(key, index);
            self.update_needed = true;
        }
    }
    pub fn has_key(&self, k: &Key) -> bool {
        self.map.contains_key(k)
    }
    pub(crate) fn process_removals(&mut self) {
        let removed = self.removal_queue.drain().collect::<Vec<Key>>();
        let mut orders = removed
            .iter()
            .map(|r| self.map.remove(r).unwrap())
            .collect::<Vec<usize>>();
        orders.sort();
        orders.reverse();
        for o in orders {
            self.remove(o);
        }
    }
    pub fn resolve_changes(&mut self, ginkgo: &Ginkgo) -> bool {
        let mut grown = false;
        self.process_removals();
        let ordering = self
            .order
            .iter()
            .enumerate()
            .map(|(i, k)| (i, *k))
            .collect::<Vec<(usize, Key)>>();
        for (i, k) in ordering {
            self.map.insert(k, i);
        }
        let order_len = self.order.len() as u32;
        if order_len > self.capacity {
            for g_fn in self.grow_fns.iter() {
                (g_fn)(&mut self.world, order_len, ginkgo);
            }
            self.capacity = order_len;
            grown = true;
        }
        let update_needed = self.update_needed;
        self.update_needed = false;
        self.write_cpu_to_gpu(ginkgo);
        grown || update_needed
    }
    fn queue_write<A: Pod + Zeroable + Default + Debug>(&mut self, key: Key, a: A) {
        let index = *self.map.get(&key).expect("key");
        self.world
            .get_non_send_resource_mut::<Attribute<A>>()
            .expect("attribute-setup")
            .queue_write(index, a);
    }
    pub fn write_cpu_to_gpu(&mut self, ginkgo: &Ginkgo) {
        for w_fn in self.cpu_to_gpu.iter() {
            (w_fn)(&mut self.world, ginkgo);
        }
    }
}

struct Attribute<A: Pod + Zeroable + Default> {
    cpu: Vec<A>,
    gpu: wgpu::Buffer,
    write_needed: bool,
}

impl<A: Pod + Zeroable + Default + Debug> Attribute<A> {
    fn new(ginkgo: &Ginkgo, capacity: u32) -> Self {
        let size = Ginkgo::memory_size::<A>(capacity);
        Self {
            cpu: vec![A::default(); capacity as usize],
            gpu: ginkgo.context().device.create_buffer(&BufferDescriptor {
                label: Some("attribute-buffer"),
                size,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            write_needed: false,
        }
    }
    fn remove(&mut self, index: usize) {
        // *self.cpu.get_mut(index).expect("index") = A::default();
        self.cpu.remove(index);
        self.write_needed = true;
    }
    fn queue_write(&mut self, index: usize, a: A) {
        if self.cpu.len() <= index {
            self.cpu
                .resize(index.checked_add(1).unwrap_or_default(), A::default());
        }
        *self.cpu.get_mut(index).expect("index") = a;
        self.write_needed = true;
    }
    fn grow(&mut self, ginkgo: &Ginkgo, c: u32) {
        let cpu = self.cpu.drain(..).collect::<Vec<A>>();
        *self = Self::new(ginkgo, c);
        self.cpu = cpu;
        self.write_needed = true;
    }
    fn write_cpu_to_gpu(&mut self, ginkgo: &Ginkgo) {
        if self.write_needed {
            let slice = &self.cpu[..];
            ginkgo
                .context()
                .queue
                .write_buffer(&self.gpu, 0, bytemuck::cast_slice(slice));
            self.write_needed = false;
        }
    }
}
