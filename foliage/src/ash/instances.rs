//! The instances one renderer is holding, on the GPU.

use core::ops::Range;

use bytemuck::Pod;
use wgpu::{Buffer, BufferAddress, BufferDescriptor, BufferSlice, BufferUsages, Device, Queue};

use crate::coordinate::Section;
use crate::elevation::ResolvedElevation;
use crate::elm::{Key, Table};

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
    slots: Slots<I>,
    buffer: Buffer,
    depth: Buffer,
    capacity: u32,
    label: &'static str,
}

impl<I: Pod> Instances<I> {
    /// Room for `capacity` instances, grown as it fills.
    pub(crate) fn new(device: &Device, label: &'static str, capacity: u32) -> Self {
        let capacity = capacity.max(1);
        Self {
            slots: Slots::default(),
            buffer: buffer(device, label, size_of::<I>() as u32 * capacity),
            depth: buffer(device, label, size_of::<f32>() as u32 * capacity),
            capacity,
            label,
        }
    }

    /// Takes one instance from a batch. See [`Slots::write`].
    pub(crate) fn write(
        &mut self,
        key: Key,
        rank: ResolvedElevation,
        clip: Section,
        group: u32,
        instance: I,
    ) {
        self.slots.write(key, rank, clip, group, instance);
    }

    /// Lets go of everything held, so the next batch states every instance there is.
    ///
    /// The GPU's buffers are kept for their room. What is in them is rewritten in full by the next
    /// flush, because the order has to be built again from whatever the batch holds.
    pub(crate) fn forget(&mut self) {
        self.slots.forget();
    }

    /// Drops one instance from what is held.
    pub(crate) fn withdraw(&mut self, key: Key) {
        self.slots.withdraw(key);
    }

    /// Puts what changed onto the GPU.
    ///
    /// A batch that added, removed, or moved anything through the stack rewrites the instance
    /// buffer, because a slot is a position in one order and every position after the change has
    /// moved. A batch that only rewrote values already in the order writes those slots and nothing
    /// else -- as runs of neighbouring slots, one write each, since a write costs something of its
    /// own whatever it carries and a frame that refilled part of a page touched a stretch of it --
    /// or, where it rewrote most of them, the whole buffer at once, which is what the runs would
    /// come to after sorting every slot to find them. A frame with no batch at all does nothing
    /// here.
    pub(crate) fn flush(&mut self, device: &Device, queue: &Queue) {
        let whole = match self.slots.resort {
            true => {
                self.slots.reorder();
                let total = self.slots.data.len() as u32;
                if total > self.capacity {
                    self.capacity = total.next_power_of_two();
                    self.buffer = buffer(device, self.label, size_of::<I>() as u32 * self.capacity);
                    self.depth =
                        buffer(device, self.label, size_of::<f32>() as u32 * self.capacity);
                }
                true
            }
            false => self.slots.touched.len() * 2 >= self.slots.data.len(),
        };
        if whole {
            self.slots.touched.clear();
            if !self.slots.data.is_empty() {
                queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.slots.data));
            }
            return;
        }
        let stride = size_of::<I>() as BufferAddress;
        for run in runs(&mut self.slots.touched) {
            queue.write_buffer(
                &self.buffer,
                run.start as BufferAddress * stride,
                bytemuck::cast_slice(&self.slots.data[run]),
            );
        }
        self.slots.touched.clear();
    }

    /// Whether anything the stack is built from changed since this was last asked.
    pub(crate) fn disturbed(&mut self) -> bool {
        core::mem::take(&mut self.slots.disturbed)
    }

    /// The rank of each slot, in slot order.
    pub(crate) fn ranks(&self) -> &[ResolvedElevation] {
        &self.slots.ranks
    }

    /// What slot `slot` is clipped to.
    pub(crate) fn clip(&self, slot: u32) -> Section {
        self.slots.clips[slot as usize]
    }

    /// What slot `slot` binds. Zero for every renderer that binds one thing or nothing.
    pub(crate) fn group(&self, slot: u32) -> u32 {
        self.slots.groups[slot as usize]
    }

    /// Where in the whole stack slot `slot` sits, as a depth.
    pub(crate) fn set_depth(&mut self, slot: u32, depth: f32) {
        self.slots.depths[slot as usize] = depth;
    }

    /// Puts the depths on the GPU, once the whole stack has been walked and every slot has one.
    pub(crate) fn flush_depths(&self, queue: &Queue) {
        if self.slots.depths.is_empty() {
            return;
        }
        queue.write_buffer(&self.depth, 0, bytemuck::cast_slice(&self.slots.depths));
    }

    pub(crate) fn data(&self) -> BufferSlice<'_> {
        self.buffer.slice(..)
    }

    pub(crate) fn depths(&self) -> BufferSlice<'_> {
        self.depth.slice(..)
    }
}

/// What one renderer holds, in slot order, apart from the buffers it is put on: every instance,
/// where it sits, and what changed since the last flush.
///
/// Kept sorted by rank, then by name. An order is only ever repaired, never sorted whole: what
/// stays keeps its place relative to the rest, so it is already in order, and only what arrived or
/// changed rank is sorted before the two are merged. A frame that adds a hundred thousand instances
/// to as many already held sorts the hundred thousand.
struct Slots<I> {
    held: Table<Held<I>>,
    /// Which instance each slot holds.
    keys: Vec<Key>,
    /// The renderer's data, in slot order.
    data: Vec<I>,
    /// One depth per slot, written by `Ash` from the whole stack's order.
    depths: Vec<f32>,
    /// One rank per slot, which is what that order is built from.
    ranks: Vec<ResolvedElevation>,
    /// One clip per slot. CPU-side only: it is what the draw is cut on, and is never uploaded.
    clips: Vec<Section>,
    /// What each slot binds, for the one renderer whose instances do not all bind the same thing.
    /// CPU-side only, and zero everywhere except pictures -- a binding is not per-instance state,
    /// it is what a span is cut on.
    groups: Vec<u32>,
    /// Slots whose instance went, or moved to another rank, since the order was last made.
    vacated: Vec<u32>,
    /// What has no slot yet: arrived, or moved to another rank. Placed by the next order.
    arrived: Vec<Key>,
    /// Whether the order itself changed, which moves every slot after the change.
    resort: bool,
    /// Whether anything the stack is built from changed: which slots there are, or what one is
    /// clipped to.
    disturbed: bool,
    /// Slots whose value changed while the order did not.
    touched: Vec<u32>,
}

/// One instance, and where it currently sits: [`UNPLACED`] until an order places it.
struct Held<I> {
    instance: I,
    rank: ResolvedElevation,
    clip: Section,
    group: u32,
    slot: u32,
}

/// The slot of an instance no order has placed yet.
const UNPLACED: u32 = u32::MAX;

impl<I> Default for Slots<I> {
    fn default() -> Self {
        Self {
            held: Table::default(),
            keys: Vec::new(),
            data: Vec::new(),
            depths: Vec::new(),
            ranks: Vec::new(),
            clips: Vec::new(),
            groups: Vec::new(),
            vacated: Vec::new(),
            arrived: Vec::new(),
            resort: false,
            disturbed: false,
            touched: Vec::new(),
        }
    }
}

impl<I: Copy> Slots<I> {
    /// Takes one instance from a batch: the value, the rank it is to be drawn at, and the clip it is
    /// drawn under.
    ///
    /// A rank that is unchanged leaves the order alone, so the ordinary write -- a move, a recolour,
    /// a rounding -- costs one slot rather than a new order. A clip that changed while the order
    /// did not costs the stack being cut again, and no upload at all.
    fn write(&mut self, key: Key, rank: ResolvedElevation, clip: Section, group: u32, instance: I) {
        match self.held.get_mut(key) {
            Some(held) => {
                held.instance = instance;
                // Both are what the stack is cut on rather than what is uploaded, so either one
                // changing costs the walk again and no upload of its own.
                let recut = held.clip != clip || held.group != group;
                held.clip = clip;
                held.group = group;
                if held.slot == UNPLACED {
                    // Waiting to be placed, which takes everything from what is held.
                    held.rank = rank;
                } else if held.rank == rank {
                    let slot = held.slot as usize;
                    self.data[slot] = instance;
                    self.touched.push(slot as u32);
                    if recut {
                        self.clips[slot] = clip;
                        self.groups[slot] = group;
                        self.disturbed = true;
                    }
                } else {
                    held.rank = rank;
                    self.vacated.push(held.slot);
                    held.slot = UNPLACED;
                    self.arrived.push(key);
                    self.resort = true;
                }
            }
            None => {
                let held = Held {
                    instance,
                    rank,
                    clip,
                    group,
                    slot: UNPLACED,
                };
                // A name withered and another grown into its index in the same batch, with the
                // write ahead of the withdrawal: the one it displaced goes now, as it would have.
                if let Some((_, displaced)) = self.held.insert(key, held)
                    && displaced.slot != UNPLACED
                {
                    self.vacated.push(displaced.slot);
                }
                self.arrived.push(key);
                self.resort = true;
            }
        }
    }

    fn forget(&mut self) {
        self.held.clear();
        self.keys.clear();
        self.data.clear();
        self.ranks.clear();
        self.clips.clear();
        self.groups.clear();
        self.depths.clear();
        self.vacated.clear();
        self.arrived.clear();
        self.touched.clear();
        self.resort = true;
    }

    fn withdraw(&mut self, key: Key) {
        if let Some(held) = self.held.remove(key) {
            if held.slot != UNPLACED {
                self.vacated.push(held.slot);
            }
            self.resort = true;
        }
    }

    /// Makes the order again: what stays, in the order it was, merged with what arrived, sorted.
    /// Every slot an instance moved to is written back to what is held for it, and only those.
    fn reorder(&mut self) {
        self.resort = false;
        self.disturbed = true;
        self.touched.clear();
        let mut vacant = vec![false; self.keys.len()];
        for &slot in &self.vacated {
            vacant[slot as usize] = true;
        }
        self.vacated.clear();
        // An instance can arrive twice in one frame -- written, withdrawn and written again -- and
        // one that arrived can go again before it is placed. What is held says which are still
        // waiting, and at what rank.
        let mut arrived: Vec<(ResolvedElevation, Key)> = self
            .arrived
            .drain(..)
            .filter_map(|key| {
                let held = self.held.get(key)?;
                (held.slot == UNPLACED).then_some((held.rank, key))
            })
            .collect();
        arrived.sort_unstable();
        arrived.dedup();
        let total = vacant.iter().filter(|vacant| !**vacant).count() + arrived.len();
        let mut keys = Vec::with_capacity(total);
        let mut data = Vec::with_capacity(total);
        let mut ranks = Vec::with_capacity(total);
        let mut clips = Vec::with_capacity(total);
        let mut groups = Vec::with_capacity(total);
        let mut stays = (0..self.keys.len())
            .filter(|slot| !vacant[*slot])
            .peekable();
        let mut arriving = arrived.into_iter().peekable();
        loop {
            let from_stays = match (stays.peek(), arriving.peek()) {
                (Some(&slot), Some(&(rank, key))) => {
                    (self.ranks[slot], self.keys[slot]) < (rank, key)
                }
                (Some(_), None) => true,
                (None, Some(_)) => false,
                (None, None) => break,
            };
            let at = keys.len() as u32;
            if from_stays {
                let Some(slot) = stays.next() else { break };
                let key = self.keys[slot];
                if slot as u32 != at
                    && let Some(held) = self.held.get_mut(key)
                {
                    held.slot = at;
                }
                keys.push(key);
                data.push(self.data[slot]);
                ranks.push(self.ranks[slot]);
                clips.push(self.clips[slot]);
                groups.push(self.groups[slot]);
            } else {
                let Some((rank, key)) = arriving.next() else {
                    break;
                };
                let Some(held) = self.held.get_mut(key) else {
                    continue;
                };
                held.slot = at;
                keys.push(key);
                data.push(held.instance);
                ranks.push(rank);
                clips.push(held.clip);
                groups.push(held.group);
            }
        }
        self.keys = keys;
        self.data = data;
        self.ranks = ranks;
        self.clips = clips;
        self.groups = groups;
        self.depths.clear();
        self.depths.resize(total, 0.0);
    }
}

/// How many untouched slots a write reaches across rather than being cut in two. Rewriting a slot
/// with what it already holds is harmless, and cheaper than a write of its own.
const BRIDGE: u32 = 16;

/// The touched slots as runs to write, in order: neighbours, and slots no further apart than
/// [`BRIDGE`], are one run.
pub(crate) fn runs(touched: &mut [u32]) -> impl Iterator<Item = Range<usize>> + '_ {
    touched.sort_unstable();
    let mut at = 0;
    core::iter::from_fn(move || {
        let start = *touched.get(at)?;
        let mut end = start;
        while let Some(&next) = touched.get(at) {
            if next > end + BRIDGE {
                break;
            }
            end = end.max(next);
            at += 1;
        }
        Some(start as usize..end as usize + 1)
    })
}

fn buffer(device: &Device, label: &'static str, size: u32) -> Buffer {
    device.create_buffer(&BufferDescriptor {
        label: Some(label),
        size: size as BufferAddress,
        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rank(stack: i32, growth: u64) -> ResolvedElevation {
        ResolvedElevation { stack, growth }
    }

    /// The key of the name at `index`, at its first generation.
    fn named(index: u32) -> Key {
        Key(u64::from(index))
    }

    /// A name withered and another grown into its index, in one batch with the write first, leaves
    /// the new one held and the old one's slot let go -- and the withdrawal that follows names
    /// something no longer held.
    #[test]
    fn a_name_grown_into_an_index_displaces_what_withered_from_it() {
        let mut slots: Slots<u32> = Slots::default();
        for n in 0..4 {
            slots.write(named(n), rank(1, n as u64), Section::default(), 0, n);
        }
        slots.reorder();
        let withered = named(2);
        let grown = Key(withered.0 + (1 << 32));
        slots.write(grown, rank(1, 9), Section::default(), 0, 9);
        slots.withdraw(withered);
        slots.reorder();
        assert_eq!(slots.keys, vec![named(0), named(1), named(3), grown]);
        assert_eq!(slots.data, vec![0, 1, 3, 9]);
        assert_eq!(slots.held.len(), 4);
        assert!(slots.held.get(withered).is_none());
    }

    /// Every slot in rank-then-name order, every held instance where its slot says, and holding
    /// what was last written to it.
    fn consistent(slots: &Slots<u32>, expected: &[(Key, ResolvedElevation, u32)]) {
        let mut sorted = expected.to_vec();
        sorted.sort_by_key(|(key, rank, _)| (*rank, *key));
        let got: Vec<(Key, ResolvedElevation, u32)> = (0..slots.keys.len())
            .map(|slot| (slots.keys[slot], slots.ranks[slot], slots.data[slot]))
            .collect();
        assert_eq!(got, sorted);
        for (slot, key) in slots.keys.iter().enumerate() {
            assert_eq!(
                slots.held.get(*key).map(|held| held.slot),
                Some(slot as u32)
            );
        }
        assert_eq!(slots.held.len(), expected.len());
        assert_eq!(slots.depths.len(), expected.len());
    }

    #[test]
    fn a_repaired_order_is_the_order_sorting_everything_makes() {
        let mut slots: Slots<u32> = Slots::default();
        let mut expected: Vec<(Key, ResolvedElevation, u32)> = Vec::new();
        let mut seed = 7u64;
        let mut next = move |below: u64| {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (seed >> 33) % below
        };
        for frame in 0..60 {
            for _ in 0..40 {
                let key = named(next(120) as u32);
                let value = (frame * 1000) as u32 + next(1000) as u32;
                match next(5) {
                    // Gone.
                    0 => {
                        slots.withdraw(key);
                        expected.retain(|(held, _, _)| *held != key);
                    }
                    // Written, at a rank that changes now and then.
                    _ => {
                        let at = rank(next(4) as i32, key.0);
                        slots.write(key, at, Section::default(), 0, value);
                        match expected.iter_mut().find(|(held, _, _)| *held == key) {
                            Some(held) => *held = (key, at, value),
                            None => expected.push((key, at, value)),
                        }
                    }
                }
            }
            if slots.resort {
                slots.reorder();
            }
            consistent(&slots, &expected);
        }
    }

    #[test]
    fn what_stays_keeps_its_slot_where_nothing_before_it_moved() {
        let mut slots: Slots<u32> = Slots::default();
        for n in 0..10 {
            slots.write(named(n as u32), rank(1, n), Section::default(), 0, n as u32);
        }
        slots.reorder();
        slots.withdraw(named(7));
        slots.write(named(20), rank(2, 20), Section::default(), 0, 20);
        slots.reorder();
        assert_eq!(slots.held.get(named(3)).map(|held| held.slot), Some(3));
        assert_eq!(slots.held.get(named(8)).map(|held| held.slot), Some(7));
        assert_eq!(slots.held.get(named(20)).map(|held| held.slot), Some(9));
    }
}
