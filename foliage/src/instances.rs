use std::collections::HashMap;
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
    removal_fns: Vec<Box<fn(&mut World, usize)>>,
    grow_fns: Vec<Box<fn(&mut World, u32, &Ginkgo)>>,
    cpu_to_gpu: Vec<Box<fn(&mut World, &Ginkgo)>>,
    update_needed: bool,
}

impl<Key: Hash + Eq + Copy + Clone> Instances<Key> {
    pub fn num_instances(&self) -> u32 {
        self.order.len() as u32
    }
    pub fn checked_write<A: Pod + Zeroable + Default + Debug>(&mut self, key: Key, a: A) {
        if !self.has_key(&key) {
            self.add(key.clone());
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
        }
    }
    pub fn with_attribute<A: Pod + Zeroable + Default + Debug>(mut self, ginkgo: &Ginkgo) -> Self {
        self.world
            .insert_non_send_resource(Attribute::<A>::new(ginkgo, self.capacity));
        self.removal_fns.push(Box::new(|w, i| {
            w.get_non_send_resource_mut::<Attribute<A>>()
                .expect("attribute")
                .remove(i);
        }));
        self.grow_fns.push(Box::new(|w, c, g| {
            w.get_non_send_resource_mut::<Attribute<A>>()
                .expect("attribute")
                .grow(g, c);
        }));
        self.cpu_to_gpu.push(Box::new(|w, g| {
            w.get_non_send_resource_mut::<Attribute<A>>()
                .expect("attribute")
                .write_cpu_to_gpu(g);
        }));
        self
    }
    pub fn remove(&mut self, key: Key) {
        let index = self.map.remove(&key).expect("not-found");
        self.order.remove(index);
        for r_fn in self.removal_fns.iter() {
            (r_fn)(&mut self.world, index);
            self.update_needed = true;
        }
    }
    pub fn add(&mut self, key: Key) {
        let index = self.order.len();
        self.order.push(key);
        self.map.insert(key, index);
        self.update_needed = true;
    }
    pub fn has_key(&self, k: &Key) -> bool {
        self.map.contains_key(k)
    }
    pub fn resolve_changes(&mut self, ginkgo: &Ginkgo) -> bool {
        let mut grown = false;
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
    pub fn queue_write<A: Pod + Zeroable + Default + Debug>(&mut self, key: Key, a: A) {
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
        Self {
            cpu: vec![A::default(); capacity as usize],
            gpu: ginkgo.context().device.create_buffer(&BufferDescriptor {
                label: Some("attribute-buffer"),
                size: Ginkgo::memory_size::<A>(capacity),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            write_needed: false,
        }
    }
    fn remove(&mut self, index: usize) {
        self.cpu.remove(index);
        self.write_needed = true;
    }
    fn queue_write(&mut self, index: usize, a: A) {
        println!(
            "queueing write for {} w/ len: {} @ a: {:?}",
            index,
            self.cpu.len(),
            a
        );
        *self.cpu.get_mut(index).expect("index") = a;
        self.write_needed = true;
    }
    fn grow(&mut self, ginkgo: &Ginkgo, c: u32) {
        let cpu = self.cpu.drain(..).collect::<Vec<A>>();
        *self = Self::new(ginkgo, c);
        self.cpu.extend(cpu);
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
