//! The text renderer: one entry in the stack per run, and one quad per glyph under it.
//!
//! # A run is one thing to the stack and many things here
//!
//! Every other renderer draws one instance per element, so its slots and its instances are the same
//! list. A run is not: it is **one** entry in the one stack -- at one rank, under one clip, at one
//! depth -- holding as many quads as it has characters. So this keeps two numberings. Slots are
//! runs, sorted by rank, and are what [`Ash`](crate::ash::Ash) walks and cuts spans on. Glyphs are
//! this renderer's own, laid out in slot order with each run's contiguous, so a span of runs is a
//! range of glyphs and one draw covers it.
//!
//! That is what keeps the shared stack the size of the tree rather than the size of the text on it,
//! and what keeps R6's total order over elements from having to say anything about characters.

use core::ops::Range;
use std::collections::HashMap;

use bytemuck::{Pod, Zeroable};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    Buffer, BufferAddress, BufferDescriptor, BufferUsages, Device, PipelineLayoutDescriptor, Queue,
    RenderPass, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource,
    VertexBufferLayout, VertexState, VertexStepMode,
};

use crate::ash::CORNERS;
use crate::ash::atlas::{Atlas, Cut};
use crate::color::Color;
use crate::coordinate::{Position, Section};
use crate::elevation::ResolvedElevation;
use crate::elm::{Key, Run};
use crate::ginkgo::Ginkgo;
use crate::text::font::Fonts;

/// How many glyphs there is room for before the buffers are grown.
const CAPACITY: u32 = 256;

/// One glyph, in the form the GPU takes it.
///
/// Built here rather than in extraction, because two of its three parts are the backend's own: where
/// the ink sits inside its cell and where it is on the sheet are both answers only the rasteriser
/// has. What extraction supplies is the cell and the character.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
struct GlyphInstance {
    /// The ink's box on the surface, in logical pixels.
    section: Section,
    /// What the run is filled with. Carried per glyph because that is what a vertex buffer is, and
    /// because a per-character tint is a fill over part of the run's own index space.
    color: Color,
    /// The ink's rect on the sheet, normalised: origin then size.
    sheet: [f32; 4],
}

/// The pipeline, the sheet, and every run the backend is holding.
pub(crate) struct Texts {
    pipeline: RenderPipeline,
    corners: Buffer,
    atlas: Atlas,
    held: HashMap<Key, Held>,
    /// Slot order: which run each slot is, and the rank and clip the stack is built from.
    order: Vec<Key>,
    ranks: Vec<ResolvedElevation>,
    clips: Vec<Section>,
    /// Which glyphs each slot covers.
    blocks: Vec<Range<u32>>,
    /// One depth per glyph, written from the run's own place in the stack.
    depths: Vec<f32>,
    glyphs: Buffer,
    depth: Buffer,
    capacity: u32,
    total: u32,
    /// Where one run's glyphs are built before they are taken. Kept for its capacity.
    scratch: Vec<GlyphInstance>,
    /// Whether the order itself changed, which moves every slot after the change.
    resort: bool,
    /// Whether anything the stack is built from changed: which slots there are, or what one is
    /// clipped to.
    disturbed: bool,
    /// Slots whose glyphs changed while the order did not.
    touched: Vec<u32>,
}

/// One run, and where its glyphs currently sit.
struct Held {
    glyphs: Vec<GlyphInstance>,
    rank: ResolvedElevation,
    clip: Section,
    slot: u32,
}

impl Texts {
    pub(crate) fn new(ginkgo: &Ginkgo) -> Self {
        let device = ginkgo.device();
        let atlas = Atlas::new(ginkgo);
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("glyph"),
            source: ShaderSource::Wgsl(include_str!("glyph.wgsl").into()),
        });
        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("glyph"),
            bind_group_layouts: &[Some(ginkgo.viewport_layout()), Some(atlas.layout())],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("glyph"),
            layout: Some(&layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vertex_entry"),
                compilation_options: Default::default(),
                buffers: &[
                    Some(VertexBufferLayout {
                        array_stride: size_of::<[f32; 2]>() as u64,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                    }),
                    Some(VertexBufferLayout {
                        array_stride: size_of::<GlyphInstance>() as u64,
                        step_mode: VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![
                            1 => Float32x4,
                            2 => Float32x4,
                            3 => Float32x4,
                        ],
                    }),
                    Some(VertexBufferLayout {
                        array_stride: size_of::<f32>() as u64,
                        step_mode: VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![4 => Float32],
                    }),
                ],
            },
            primitive: Ginkgo::triangles(),
            depth_stencil: Ginkgo::depth_state(),
            multisample: Ginkgo::samples(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment_entry"),
                compilation_options: Default::default(),
                targets: &ginkgo.blending(),
            }),
            multiview_mask: None,
            cache: None,
        });
        Self {
            pipeline,
            corners: device.create_buffer_init(&BufferInitDescriptor {
                label: Some("glyph-corners"),
                contents: bytemuck::cast_slice(&CORNERS),
                usage: BufferUsages::VERTEX,
            }),
            atlas,
            held: HashMap::new(),
            order: Vec::new(),
            ranks: Vec::new(),
            clips: Vec::new(),
            blocks: Vec::new(),
            depths: Vec::new(),
            glyphs: buffer(device, size_of::<GlyphInstance>() as u32 * CAPACITY),
            depth: buffer(device, size_of::<f32>() as u32 * CAPACITY),
            capacity: CAPACITY,
            total: 0,
            scratch: Vec::new(),
            resort: false,
            disturbed: false,
            touched: Vec::new(),
        }
    }

    /// Takes one run: cuts whatever of it has not been cut, and holds the quads it turned out to be.
    ///
    /// A run that kept its glyph count and its rank is written where it already sits, which is the
    /// ordinary case -- a run that moved, scrolled or was refilled has as many characters as it had.
    /// A run whose length changed moves every run after it, so the order is built again.
    pub(crate) fn write(&mut self, key: Key, run: &Run, fonts: &Fonts, ginkgo: &Ginkgo) {
        let mut scratch = core::mem::take(&mut self.scratch);
        scratch.clear();
        for glyph in &run.glyphs {
            let cut = Cut::new(run.font, run.size, ginkgo.scale(), glyph.character);
            let placed = self.atlas.place(fonts, ginkgo, cut);
            // A character the face draws nothing for takes no quad. It still advanced the wrap,
            // which happened above this line and is not undone by there being nothing to draw.
            if placed.area.width <= 0.0 || placed.area.height <= 0.0 {
                continue;
            }
            scratch.push(GlyphInstance {
                section: Section::new(
                    snapped(glyph.cell.position.moved(placed.offset), ginkgo.scale()),
                    placed.area,
                ),
                color: run.color,
                sheet: placed.uv,
            });
        }
        match self.held.get_mut(&key) {
            Some(held) => {
                let recut = held.clip != run.clip;
                held.clip = run.clip;
                let settled = held.rank == run.rank && held.glyphs.len() == scratch.len();
                held.rank = run.rank;
                held.glyphs.clear();
                held.glyphs.extend_from_slice(&scratch);
                match settled {
                    true => {
                        self.touched.push(held.slot);
                        if recut {
                            self.clips[held.slot as usize] = run.clip;
                            self.disturbed = true;
                        }
                    }
                    false => self.resort = true,
                }
            }
            None => {
                self.held.insert(
                    key,
                    Held {
                        glyphs: scratch.clone(),
                        rank: run.rank,
                        clip: run.clip,
                        slot: 0,
                    },
                );
                self.resort = true;
            }
        }
        self.scratch = scratch;
    }

    /// Drops one run from what is held.
    pub(crate) fn withdraw(&mut self, key: Key) {
        if self.held.remove(&key).is_some() {
            self.resort = true;
        }
    }

    /// Puts what changed onto the GPU.
    pub(crate) fn flush(&mut self, device: &Device, queue: &Queue) {
        if self.resort {
            self.reorder(device, queue);
            return;
        }
        self.touched.sort_unstable();
        self.touched.dedup();
        for slot in self.touched.drain(..) {
            let block = self.blocks[slot as usize].clone();
            if block.is_empty() {
                continue;
            }
            let held = &self.held[&self.order[slot as usize]];
            queue.write_buffer(
                &self.glyphs,
                block.start as BufferAddress * size_of::<GlyphInstance>() as BufferAddress,
                bytemuck::cast_slice(&held.glyphs),
            );
        }
    }

    /// Rebuilds the order back to front and rewrites every glyph.
    ///
    /// One write per run rather than one for the whole buffer: the runs are already the unit the
    /// glyphs are held in, and a flat mirror of them kept only to be uploaded in one call would be
    /// a second copy of every glyph on the page.
    fn reorder(&mut self, device: &Device, queue: &Queue) {
        self.resort = false;
        self.disturbed = true;
        self.touched.clear();
        let mut order = self
            .held
            .iter()
            .map(|(key, held)| (held.rank, *key))
            .collect::<Vec<_>>();
        // A rank orders back to front and is total, so this is one sort with no tie left in it.
        order.sort_unstable();
        self.order.clear();
        self.ranks.clear();
        self.clips.clear();
        self.blocks.clear();
        let mut at = 0;
        for (slot, (rank, key)) in order.into_iter().enumerate() {
            let held = self.held.get_mut(&key).expect("held");
            held.slot = slot as u32;
            let count = held.glyphs.len() as u32;
            self.order.push(key);
            self.ranks.push(rank);
            self.clips.push(held.clip);
            self.blocks.push(at..at + count);
            at += count;
        }
        self.total = at;
        self.depths.clear();
        self.depths.resize(at as usize, 0.0);
        if at > self.capacity {
            self.capacity = at.next_power_of_two();
            self.glyphs = buffer(device, size_of::<GlyphInstance>() as u32 * self.capacity);
            self.depth = buffer(device, size_of::<f32>() as u32 * self.capacity);
        }
        for slot in 0..self.order.len() {
            let held = &self.held[&self.order[slot]];
            if held.glyphs.is_empty() {
                continue;
            }
            queue.write_buffer(
                &self.glyphs,
                self.blocks[slot].start as BufferAddress
                    * size_of::<GlyphInstance>() as BufferAddress,
                bytemuck::cast_slice(&held.glyphs),
            );
        }
    }

    /// Whether the stack has to be walked again. Cleared by the walk.
    pub(crate) fn disturbed(&mut self) -> bool {
        core::mem::take(&mut self.disturbed)
    }

    /// The rank of each run, in slot order.
    pub(crate) fn ranks(&self) -> &[ResolvedElevation] {
        &self.ranks
    }

    /// What the run in `slot` is clipped to.
    pub(crate) fn clip(&self, slot: u32) -> Section {
        self.clips[slot as usize]
    }

    /// Where in the whole stack the run in `slot` sits, as a depth.
    ///
    /// Every glyph of it takes that one depth, because the run is one entry in the stack and its
    /// glyphs are all at that one place in it.
    pub(crate) fn set_depth(&mut self, slot: u32, depth: f32) {
        let block = self.blocks[slot as usize].clone();
        self.depths[block.start as usize..block.end as usize].fill(depth);
    }

    /// Puts the depths on the GPU, once every run has one.
    pub(crate) fn flush_depths(&self, queue: &Queue) {
        if self.depths.is_empty() {
            return;
        }
        queue.write_buffer(&self.depth, 0, bytemuck::cast_slice(&self.depths));
    }

    /// Draws one run of what is held.
    ///
    /// The range is a span of the stack and so is in *runs*. The glyphs of a run are contiguous and
    /// the runs of a span are contiguous, so the glyphs of a span are one range and one draw.
    pub(crate) fn draw(&self, pass: &mut RenderPass<'_>, span: Range<u32>) {
        let from = self.blocks[span.start as usize].start;
        let to = self.blocks[span.end as usize - 1].end;
        // A span of runs that are all empty -- every character of them a space, or none of them
        // drawn by the face -- is a draw of nothing.
        if from == to {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(1, self.atlas.binding(), &[]);
        pass.set_vertex_buffer(0, self.corners.slice(..));
        pass.set_vertex_buffer(1, self.glyphs.slice(..));
        pass.set_vertex_buffer(2, self.depth.slice(..));
        pass.draw(0..CORNERS.len() as u32, from..to);
    }

    /// How many glyphs are held. Reported to the trace.
    pub(crate) fn len(&self) -> u32 {
        self.total
    }
}

/// `at` moved to the nearest whole device pixel.
///
/// A cut is an exact number of device pixels on the sheet, and the quad it is drawn on is that many
/// wide and tall. Landed on a whole pixel, the two are one to one: every sample falls on a texel
/// centre and the glyph is the bitmap. Landed on a fraction of one -- and it always is, because a
/// baseline is a face's own fractional metric and a run's box is wherever the layout put it -- every
/// sample falls between texels, so the glyph is filtered against the clearance packed around it and
/// its outermost rows fade. That reads as a glyph trimmed along its edges rather than as a glyph
/// half a pixel low, which is what it is.
///
/// Snapping the whole ink rather than the baseline alone, because both halves of where a glyph
/// landed are fractional and it is their sum that has to fall on the grid. A line of a run shares
/// one baseline and one row of cells, so a line snaps as a line and stays level.
fn snapped(at: Position, scale: f32) -> Position {
    Position::new(
        (at.x * scale).round() / scale,
        (at.y * scale).round() / scale,
    )
}

fn buffer(device: &Device, size: u32) -> Buffer {
    device.create_buffer(&BufferDescriptor {
        label: Some("glyph"),
        size: size as BufferAddress,
        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
