//! The instances one renderer is holding, on the GPU.

use std::collections::HashMap;

use bytemuck::Pod;
use wgpu::{Buffer, BufferAddress, BufferDescriptor, BufferSlice, BufferUsages, Device, Queue};

use crate::coordinate::Section;
use crate::elevation::ResolvedElevation;
use crate::leaf::Leaf;

/// One renderer's instance buffers, kept in rank order.
///
/// Generic over the instance, because what a renderer sends is the renderer's own business. This
/// owns the upload and knows nothing about what is being uploaded.
///
/// # Two buffers
///
/// The renderer's own data is one, and the depth each instance is drawn at is the other. They are
/// separate because they change for different reasons: a depth comes from an instance's *position*
/// in the stack, so it is untouched while the stack is, and a value written to an instance already
/// in it does not disturb anyone else's. Holding depth inside the instance would make every
/// recolour a candidate for a full rewrite.
///
/// # What this does not decide
///
/// Where its instances sit among *everyone's* is [`Ash`](crate::ash::Ash)'s: the depths are written
/// from there, and so is the order the draws go in. This keeps its slots sorted by rank so that a
/// walk of the whole stack meets them in slot order, and reports the rank and clip of each so that
/// walk has something to sort and cut on.
pub(crate) struct Instances<I: Pod> {
    held: HashMap<Leaf, Held<I>>,
    /// The renderer's data, in slot order.
    data: Vec<I>,
    /// One depth per slot, written by `Ash` from the whole stack's order.
    depths: Vec<f32>,
    /// One rank per slot, which is what that order is built from.
    ranks: Vec<ResolvedElevation>,
    /// One clip per slot. CPU-side only: it is what the draw is cut on, and is never uploaded.
    clips: Vec<Section>,
    buffer: Buffer,
    depth: Buffer,
    capacity: u32,
    label: &'static str,
    /// Whether the order itself changed, which moves every slot after the change.
    resort: bool,
    /// Whether anything the stack is built from changed: which slots there are, or what one is
    /// clipped to.
    disturbed: bool,
    /// Slots whose value changed while the order did not.
    touched: Vec<u32>,
}

/// One instance, and where it currently sits.
struct Held<I> {
    instance: I,
    rank: ResolvedElevation,
    clip: Section,
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
            ranks: Vec::new(),
            clips: Vec::new(),
            buffer: buffer(device, label, size_of::<I>() as u32 * capacity),
            depth: buffer(device, label, size_of::<f32>() as u32 * capacity),
            capacity,
            label,
            resort: false,
            disturbed: false,
            touched: Vec::new(),
        }
    }

    /// Takes one instance from a batch: the value, the rank it is to be drawn at, and the clip it is
    /// drawn under.
    ///
    /// A rank that is unchanged leaves the order alone, so the ordinary write -- a move, a recolour,
    /// a rounding -- costs one slot rather than a re-sort. A clip that changed while the order did
    /// not costs the stack being cut again, and no upload at all.
    pub(crate) fn write(&mut self, leaf: Leaf, rank: ResolvedElevation, clip: Section, instance: I) {
        match self.held.get_mut(&leaf) {
            Some(held) => {
                held.instance = instance;
                let recut = held.clip != clip;
                held.clip = clip;
                if held.rank == rank {
                    let slot = held.slot as usize;
                    self.data[slot] = instance;
                    self.touched.push(slot as u32);
                    if recut {
                        self.clips[slot] = clip;
                        self.disturbed = true;
                    }
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
                        clip,
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
    /// A batch that added, removed, or moved anything through the stack rewrites the instance
    /// buffer, because a slot is a position in one order and every position after the change has
    /// moved. A batch that only rewrote values already in the order writes those slots and nothing
    /// else. A frame with no batch at all does nothing here.
    pub(crate) fn flush(&mut self, device: &Device, queue: &Queue) {
        if self.resort {
            self.reorder(device, queue);
            return;
        }
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

    /// Rebuilds the order back to front and rewrites the instance buffer.
    fn reorder(&mut self, device: &Device, queue: &Queue) {
        self.resort = false;
        self.disturbed = true;
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
        self.ranks.clear();
        self.clips.clear();
        self.depths.clear();
        self.depths.resize(total, 0.0);
        for (slot, (rank, leaf)) in order.into_iter().enumerate() {
            let held = self.held.get_mut(&leaf).expect("held");
            held.slot = slot as u32;
            self.data.push(held.instance);
            self.ranks.push(rank);
            self.clips.push(held.clip);
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
    }

    /// Whether the stack has to be walked again: a slot appeared, went, moved, or changed what it
    /// is clipped to. Cleared by the walk.
    pub(crate) fn disturbed(&mut self) -> bool {
        core::mem::take(&mut self.disturbed)
    }

    /// The rank of each slot, in slot order.
    pub(crate) fn ranks(&self) -> &[ResolvedElevation] {
        &self.ranks
    }

    /// What slot `slot` is clipped to.
    pub(crate) fn clip(&self, slot: u32) -> Section {
        self.clips[slot as usize]
    }

    /// Where in the whole stack slot `slot` sits, as a depth.
    pub(crate) fn set_depth(&mut self, slot: u32, depth: f32) {
        self.depths[slot as usize] = depth;
    }

    /// Puts the depths on the GPU, once the whole stack has been walked and every slot has one.
    pub(crate) fn flush_depths(&self, queue: &Queue) {
        if self.depths.is_empty() {
            return;
        }
        queue.write_buffer(&self.depth, 0, bytemuck::cast_slice(&self.depths));
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
