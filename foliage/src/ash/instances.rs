//! The instances one renderer is holding, on the GPU.

use std::collections::HashMap;

use bytemuck::Pod;
use wgpu::{Buffer, BufferAddress, BufferDescriptor, BufferSlice, BufferUsages, Device, Queue};

use crate::elevation::ResolvedElevation;
use crate::ginkgo::depth::Depth;
use crate::leaf::Leaf;

/// One renderer's instance buffers, kept in draw order.
///
/// Generic over the instance, because what a renderer sends is the renderer's own business. This
/// owns the ordering and the upload, and knows nothing about what is being ordered beyond its rank.
///
/// # Two buffers
///
/// The renderer's own data is one, and the depth each instance is drawn at is the other. They are
/// separate because they change for different reasons: a depth comes from an instance's *position*
/// in the order, so it is untouched while the order is, and a value written to an instance already
/// in the order does not disturb anyone else's. Holding depth inside the instance would make every
/// recolour a candidate for a full rewrite.
pub(crate) struct Instances<I: Pod> {
    held: HashMap<Leaf, Held<I>>,
    /// The renderer's data, in slot order.
    data: Vec<I>,
    /// One depth per slot, front-most last.
    depths: Vec<f32>,
    buffer: Buffer,
    depth: Buffer,
    capacity: u32,
    label: &'static str,
    /// Whether the order itself changed, which moves every slot after the change.
    resort: bool,
    /// Slots whose value changed while the order did not.
    touched: Vec<u32>,
}

/// One instance, and where it currently sits.
struct Held<I> {
    instance: I,
    rank: ResolvedElevation,
    slot: u32,
}

impl<I: Pod> Instances<I> {
    /// Room for `capacity` instances, grown as it fills.
    pub(crate) fn new(device: &Device, label: &'static str, capacity: u32) -> Self {
        let capacity = capacity.max(1);
        Self {
            held: HashMap::new(),
            data: Vec::new(),
            depths: Vec::new(),
            buffer: buffer(device, label, size_of::<I>() as u32 * capacity),
            depth: buffer(device, label, size_of::<f32>() as u32 * capacity),
            capacity,
            label,
            resort: false,
            touched: Vec::new(),
        }
    }

    /// Takes one instance from a batch: the value, and the rank it is to be drawn at.
    ///
    /// A rank that is unchanged leaves the order alone, so the ordinary write -- a move, a recolour,
    /// a rounding -- costs one slot rather than a re-sort.
    pub(crate) fn write(&mut self, leaf: Leaf, rank: ResolvedElevation, instance: I) {
        match self.held.get_mut(&leaf) {
            Some(held) => {
                held.instance = instance;
                if held.rank == rank {
                    self.data[held.slot as usize] = instance;
                    self.touched.push(held.slot);
                } else {
                    held.rank = rank;
                    self.resort = true;
                }
            }
            None => {
                self.held.insert(
                    leaf,
                    Held {
                        instance,
                        rank,
                        slot: 0,
                    },
                );
                self.resort = true;
            }
        }
    }

    /// Drops one instance from what is held.
    pub(crate) fn withdraw(&mut self, leaf: Leaf) {
        if self.held.remove(&leaf).is_some() {
            self.resort = true;
        }
    }

    /// Puts what changed onto the GPU.
    ///
    /// A batch that added, removed, or moved anything through the stack rewrites both buffers,
    /// because a slot is a position in one order and every position after the change has moved.
    /// A batch that only rewrote values already in the order writes those slots and nothing else.
    /// A frame with no batch at all does nothing here.
    pub(crate) fn flush(&mut self, device: &Device, queue: &Queue) {
        if self.resort {
            self.reorder(device, queue);
        } else {
            self.touched.sort_unstable();
            self.touched.dedup();
            for slot in self.touched.drain(..) {
                let stride = size_of::<I>() as BufferAddress;
                queue.write_buffer(
                    &self.buffer,
                    slot as BufferAddress * stride,
                    bytemuck::bytes_of(&self.data[slot as usize]),
                );
            }
        }
    }

    /// Rebuilds the order back to front and rewrites both buffers.
    fn reorder(&mut self, device: &Device, queue: &Queue) {
        self.resort = false;
        self.touched.clear();
        let mut order = self
            .held
            .iter()
            .map(|(leaf, held)| (held.rank, *leaf))
            .collect::<Vec<_>>();
        // A rank orders back to front and is total -- its allocation counter separates two elements
        // that accumulated to the same elevation -- so this is one sort with no tie left in it, and
        // two identical runs order identically.
        order.sort_unstable();
        let total = order.len();
        self.data.clear();
        self.depths.clear();
        for (slot, (_, leaf)) in order.into_iter().enumerate() {
            let held = self.held.get_mut(&leaf).expect("held");
            held.slot = slot as u32;
            self.data.push(held.instance);
            self.depths.push(Depth::of(slot, total));
        }
        if total as u32 > self.capacity {
            self.capacity = (total as u32).next_power_of_two();
            self.buffer = buffer(device, self.label, size_of::<I>() as u32 * self.capacity);
            self.depth = buffer(device, self.label, size_of::<f32>() as u32 * self.capacity);
        }
        if total == 0 {
            return;
        }
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.data));
        queue.write_buffer(&self.depth, 0, bytemuck::cast_slice(&self.depths));
    }

    /// How many instances are drawn.
    pub(crate) fn count(&self) -> u32 {
        self.data.len() as u32
    }

    pub(crate) fn data(&self) -> BufferSlice<'_> {
        self.buffer.slice(..)
    }

    pub(crate) fn depths(&self) -> BufferSlice<'_> {
        self.depth.slice(..)
    }
}

fn buffer(device: &Device, label: &'static str, size: u32) -> Buffer {
    device.create_buffer(&BufferDescriptor {
        label: Some(label),
        size: size as BufferAddress,
        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
