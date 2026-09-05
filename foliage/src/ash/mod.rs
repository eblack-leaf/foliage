//! Ash -- the render backend.
//!
//! Step 9. What [`Elm`](crate::elm) decided had changed becomes what the GPU is holding, and what
//! the GPU is holding becomes a frame.
//!
//! # The batch is a contract
//!
//! `Elm`'s cache is *what the backend holds*. Every batch it produces has to be applied, because a
//! batch that is dropped leaves that cache claiming something the GPU does not have and nothing
//! afterwards will disagree with it. So absorbing a batch is separate from painting one: absorbing
//! is unconditional and happens in the same breath as the extraction that produced it, and a paint
//! that cannot acquire a surface loses a picture rather than a change.
//!
//! # This is where the density lives
//!
//! Everything above this line is written in logical pixels. Four of the six renderers cannot build
//! what the GPU draws without knowing how large a device pixel is -- a glyph is cut at a density, a
//! stroke's axis-aligned edges are snapped to whole ones, a mark's distance range is converted into
//! a screen-space one, a picture is uploaded at its own resolution -- so each takes what extraction
//! declared and derives the instance here. That is what keeps `Ginkgo`'s line honest: the scale
//! factor is applied below it and nowhere above.
//!
//! # The stack is shared
//!
//! There is one stack, across every renderer, ordered back to front by rank -- which is what alpha
//! blending requires. It is walked here, and a renderer holds no opinion about it: each keeps its
//! own slots sorted by rank, and this merges them.
//!
//! Two things come out of that one walk, and they must come out of the same one. Each instance is
//! given a **depth** from its position in it, so the depth test holds the order for fragments the
//! draw order alone would not, and a repaint of the same content lands on the same result. And the
//! walk is cut into **spans**: maximal runs sharing a renderer, a clip, and a binding. A run of one
//! renderer's slots is contiguous in that renderer's buffer, because the merge meets them in slot
//! order, so a span is one draw of one range.
//!
//! The binding is in that cut because a picture is one texture each: two images at neighbouring
//! ranks are two spans, and everything else in the engine binds one thing or nothing and so never
//! cuts on it.
//!
//! Clipping is applied to the pass as a scissor rather than carried in an instance and tested per
//! fragment, so it costs nothing per element and every renderer has it without a line of shader.

use tracing::field::Empty;
use tracing::trace_span;

use crate::ash::line::LineQuad;
use crate::ash::picture::{PictureQuad, Pictures};
use crate::ash::quad::{Quads, Recipe};
use crate::ash::sheet::{MarkQuad, Sheet};
use crate::ash::text::Texts;
use crate::color::Color;
use crate::coordinate::Section;
use crate::elevation::ResolvedElevation;
use crate::elm::Elm;
use crate::ginkgo::Ginkgo;
use crate::ginkgo::depth::Depth;
use crate::icon::Fields;
use crate::image::{ImageInstance, Plates};
use crate::panel::PanelInstance;
use crate::polygon::PolygonInstance;
use crate::text::font::Fonts;

mod atlas;
mod instances;
mod line;
mod picture;
mod quad;
mod sheet;
mod text;

/// The unit quad, as two triangles.
///
/// The whole of the geometry every renderer here draws over: a rounded corner is a distance field
/// and a glyph is a sample of a sheet, so neither has anything to carve into a mesh. One constant
/// rather than one per renderer, because two quads that disagreed would be two renderers whose
/// instances mean subtly different things.
pub(crate) const CORNERS: [[f32; 2]; 6] = [
    [0.0, 0.0],
    [1.0, 0.0],
    [0.0, 1.0],
    [1.0, 0.0],
    [1.0, 1.0],
    [0.0, 1.0],
];

/// The renderers, what each is holding, and the one stack over all of them.
pub(crate) struct Ash {
    panels: Quads<PanelInstance>,
    polygons: Quads<PolygonInstance>,
    lines: Quads<LineQuad>,
    icons: Quads<MarkQuad>,
    images: Quads<PictureQuad>,
    texts: Texts,
    /// Every registered mark's field, packed on one texture.
    sheet: Sheet,
    /// Every uploaded picture, one texture each.
    pictures: Pictures,
    /// The draws, in order. One per run of the stack sharing a renderer, a clip and a binding.
    spans: Vec<Span>,
    /// The stack itself, kept between walks for its capacity.
    stack: Vec<Slot>,
}

/// One run of the stack: which renderer draws it, what it is scissored to, what it binds, and which
/// of that renderer's slots it covers.
#[derive(Copy, Clone, Debug)]
struct Span {
    renderer: Renderer,
    clip: Section,
    group: u32,
    from: u32,
    to: u32,
}

/// One instance's place in the stack.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Slot {
    /// First, because this is what the stack is ordered by. A rank is total, so the sort leaves no
    /// tie and two identical runs draw identically.
    rank: ResolvedElevation,
    renderer: Renderer,
    slot: u32,
}

/// Which renderer holds an instance.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Renderer {
    Panel,
    Polygon,
    Line,
    Icon,
    Image,
    Text,
}

impl Ash {
    pub(crate) fn new(ginkgo: &Ginkgo) -> Self {
        let sheet = Sheet::new(ginkgo);
        let pictures = Pictures::new(ginkgo);
        let rounded = include_str!("sdf.wgsl");
        Self {
            panels: Quads::new(
                ginkgo,
                Recipe {
                    label: "panel",
                    shader: format!("{rounded}{}", include_str!("panel.wgsl")),
                    attributes: &PANEL,
                    depth: 4,
                    sampled: false,
                },
                None,
                16,
            ),
            polygons: Quads::new(
                ginkgo,
                Recipe {
                    label: "polygon",
                    shader: include_str!("polygon.wgsl").to_string(),
                    attributes: &POLYGON,
                    depth: 4,
                    sampled: false,
                },
                None,
                8,
            ),
            lines: Quads::new(
                ginkgo,
                Recipe {
                    label: "line",
                    shader: include_str!("line.wgsl").to_string(),
                    attributes: &LINE,
                    depth: 4,
                    sampled: false,
                },
                None,
                8,
            ),
            icons: Quads::new(
                ginkgo,
                Recipe {
                    label: "icon",
                    shader: include_str!("icon.wgsl").to_string(),
                    attributes: &ICON,
                    depth: 5,
                    sampled: true,
                },
                Some(sheet.layout()),
                8,
            ),
            images: Quads::new(
                ginkgo,
                Recipe {
                    label: "image",
                    shader: format!("{rounded}{}", include_str!("image.wgsl")),
                    attributes: &IMAGE,
                    depth: 5,
                    sampled: true,
                },
                Some(pictures.layout()),
                8,
            ),
            texts: Texts::new(ginkgo),
            sheet,
            pictures,
            spans: Vec::new(),
            stack: Vec::new(),
        }
    }

    /// Takes this frame's batches and puts them on the GPU.
    ///
    /// Runs once for every extraction, and reads what extraction produced without consuming it: a
    /// batch is a statement of what the backend should be holding, so applying one twice reaches
    /// the same holding as applying it once. What must not happen is applying one zero times.
    ///
    /// `fonts`, `fields` and `plates` are here and not in the batch for one reason: what a face, a
    /// distance field or a decoded picture *becomes* at this display's density is the only part
    /// that has to be re-answered when the display changes, and it is the part extraction cannot
    /// state.
    pub(crate) fn absorb(
        &mut self,
        elm: &Elm,
        fonts: &Fonts,
        fields: &Fields,
        plates: &Plates,
        ginkgo: &Ginkgo,
    ) {
        let span = trace_span!("absorb", written = Empty, withdrawn = Empty, glyphs = Empty);
        let _entered = span.enter();
        let (written, withdrawn) = elm.moved();
        span.record("written", written);
        span.record("withdrawn", withdrawn);
        let scale = ginkgo.scale();
        for wanted in &elm.panels.written {
            self.panels
                .instances
                .write(wanted.key, wanted.rank, wanted.clip, 0, wanted.instance);
        }
        for key in &elm.panels.withdrawn {
            self.panels.instances.withdraw(*key);
        }
        for wanted in &elm.polygons.written {
            self.polygons
                .instances
                .write(wanted.key, wanted.rank, wanted.clip, 0, wanted.instance);
        }
        for key in &elm.polygons.withdrawn {
            self.polygons.instances.withdraw(*key);
        }
        for wanted in &elm.lines.written {
            self.lines.instances.write(
                wanted.key,
                wanted.rank,
                wanted.clip,
                0,
                LineQuad::new(wanted.instance, scale),
            );
        }
        for key in &elm.lines.withdrawn {
            self.lines.instances.withdraw(*key);
        }
        for wanted in &elm.icons.written {
            // Packing on first sight, so a mark costs one upload for the life of the program and a
            // frame that draws one already packed touches no texture.
            let packed = self.sheet.place(fields, ginkgo, wanted.instance.field);
            self.icons.instances.write(
                wanted.key,
                wanted.rank,
                wanted.clip,
                0,
                MarkQuad::new(wanted.instance, packed, scale),
            );
        }
        for key in &elm.icons.withdrawn {
            self.icons.instances.withdraw(*key);
        }
        for wanted in &elm.images.written {
            self.upload(plates, ginkgo, wanted.instance);
            self.images.instances.write(
                wanted.key,
                wanted.rank,
                wanted.clip,
                // The one renderer whose binding is per instance, so the one that puts anything
                // but zero here.
                wanted.instance.plate.0,
                PictureQuad {
                    section: wanted.instance.section,
                    crop: wanted.instance.crop,
                    radii: wanted.instance.radii,
                    opacity: wanted.instance.opacity,
                },
            );
        }
        for key in &elm.images.withdrawn {
            self.images.instances.withdraw(*key);
        }
        for key in &elm.texts.written {
            // Written means the backend is not holding it as it now stands, so what is held for it
            // is what it should hold. A key with nothing behind it cannot happen and is not covered
            // for: the batch and the holding are produced by the one extraction.
            if let Some(run) = elm.texts.run(*key) {
                self.texts.write(*key, run, fonts, ginkgo);
            }
        }
        for key in &elm.texts.withdrawn {
            self.texts.withdraw(*key);
        }
        let device = ginkgo.device();
        let queue = ginkgo.queue();
        self.panels.instances.flush(device, queue);
        self.polygons.instances.flush(device, queue);
        self.lines.instances.flush(device, queue);
        self.icons.instances.flush(device, queue);
        self.images.instances.flush(device, queue);
        self.texts.flush(device, queue);
        span.record("glyphs", self.texts.len());
        // Every one of them, and not the first that says so: a walk skipped because another
        // renderer answered first would leave that one's new slots without a depth.
        let disturbed = [
            self.panels.instances.disturbed(),
            self.polygons.instances.disturbed(),
            self.lines.instances.disturbed(),
            self.icons.instances.disturbed(),
            self.images.instances.disturbed(),
            self.texts.disturbed(),
        ];
        if disturbed.into_iter().any(|moved| moved) {
            self.restack(ginkgo);
        }
    }

    /// Puts a picture on the GPU if this is the first instance to want it, or if its pixels were
    /// written again.
    fn upload(&mut self, plates: &Plates, ginkgo: &Ginkgo, instance: ImageInstance) {
        if self.pictures.binding(instance.plate).is_some() {
            return;
        }
        let Some(picture) = plates.picture(instance.plate) else {
            return;
        };
        self.pictures.upload(
            ginkgo,
            instance.plate,
            &picture.pixels,
            (picture.size.width as u32, picture.size.height as u32),
        );
    }

    /// Walks the one stack: gives every instance its depth, and cuts the draw into spans.
    ///
    /// Runs only when something in it moved -- a slot appeared, went, changed rank, or changed what
    /// it is clipped to or what it binds. A frame that only recoloured what was already there
    /// leaves the stack as it was, and this does not run.
    fn restack(&mut self, ginkgo: &Ginkgo) {
        let _span = trace_span!("restack").entered();
        self.stack.clear();
        let gather = |ranks: &[ResolvedElevation], renderer: Renderer, stack: &mut Vec<Slot>| {
            stack.extend(ranks.iter().enumerate().map(|(slot, rank)| Slot {
                rank: *rank,
                renderer,
                slot: slot as u32,
            }));
        };
        gather(
            self.panels.instances.ranks(),
            Renderer::Panel,
            &mut self.stack,
        );
        gather(
            self.polygons.instances.ranks(),
            Renderer::Polygon,
            &mut self.stack,
        );
        gather(
            self.lines.instances.ranks(),
            Renderer::Line,
            &mut self.stack,
        );
        gather(
            self.icons.instances.ranks(),
            Renderer::Icon,
            &mut self.stack,
        );
        gather(
            self.images.instances.ranks(),
            Renderer::Image,
            &mut self.stack,
        );
        // A run contributes one entry however many characters it has, which is what keeps this the
        // size of the tree rather than the size of the text on it.
        gather(self.texts.ranks(), Renderer::Text, &mut self.stack);
        self.stack.sort_unstable();
        let total = self.stack.len();
        self.spans.clear();
        for (position, entry) in self.stack.iter().enumerate() {
            let depth = Depth::of(position, total);
            let (clip, group) = match entry.renderer {
                Renderer::Panel => {
                    self.panels.instances.set_depth(entry.slot, depth);
                    (self.panels.instances.clip(entry.slot), 0)
                }
                Renderer::Polygon => {
                    self.polygons.instances.set_depth(entry.slot, depth);
                    (self.polygons.instances.clip(entry.slot), 0)
                }
                Renderer::Line => {
                    self.lines.instances.set_depth(entry.slot, depth);
                    (self.lines.instances.clip(entry.slot), 0)
                }
                Renderer::Icon => {
                    self.icons.instances.set_depth(entry.slot, depth);
                    (self.icons.instances.clip(entry.slot), 0)
                }
                Renderer::Image => {
                    self.images.instances.set_depth(entry.slot, depth);
                    (
                        self.images.instances.clip(entry.slot),
                        self.images.instances.group(entry.slot),
                    )
                }
                Renderer::Text => {
                    self.texts.set_depth(entry.slot, depth);
                    (self.texts.clip(entry.slot), 0)
                }
            };
            match self.spans.last_mut() {
                Some(span)
                    if span.renderer == entry.renderer
                        && span.clip == clip
                        && span.group == group
                        && span.to == entry.slot =>
                {
                    span.to = entry.slot + 1;
                }
                _ => self.spans.push(Span {
                    renderer: entry.renderer,
                    clip,
                    group,
                    from: entry.slot,
                    to: entry.slot + 1,
                }),
            }
        }
        let queue = ginkgo.queue();
        self.panels.instances.flush_depths(queue);
        self.polygons.instances.flush_depths(queue);
        self.lines.instances.flush_depths(queue);
        self.icons.instances.flush_depths(queue);
        self.images.instances.flush_depths(queue);
        self.texts.flush_depths(queue);
    }

    /// Step 9. Paints what is held.
    ///
    /// `clear` is what the surface is cleared to. Nothing of the engine's own sits behind the tree,
    /// so it is the app's own ground -- whatever [`Palette::Surface`](crate::Palette::Surface)
    /// currently resolves to.
    pub(crate) fn draw(&self, ginkgo: &Ginkgo, clear: Color) {
        let _span = trace_span!(
            "draw",
            instances = self.stack.len(),
            spans = self.spans.len()
        )
        .entered();
        ginkgo.draw(clear, |pass| {
            for span in &self.spans {
                let (left, top, width, height) = ginkgo.scissor(span.clip);
                // A region scrolled entirely off the surface has nothing to paint, and a zero-sized
                // scissor is not a way of saying so.
                if width == 0 || height == 0 {
                    continue;
                }
                pass.set_scissor_rect(left, top, width, height);
                let range = span.from..span.to;
                match span.renderer {
                    Renderer::Panel => self.panels.draw(pass, range, None),
                    Renderer::Polygon => self.polygons.draw(pass, range, None),
                    Renderer::Line => self.lines.draw(pass, range, None),
                    Renderer::Icon => self.icons.draw(pass, range, Some(self.sheet.binding())),
                    // A picture whose texture is not held yet is skipped rather than drawn against
                    // someone else's: it appears on the frame its pixels arrive.
                    Renderer::Image => {
                        if let Some(binding) =
                            self.pictures.binding(crate::image::Plate(span.group))
                        {
                            self.images.draw(pass, range, Some(binding));
                        }
                    }
                    Renderer::Text => self.texts.draw(pass, range),
                }
            }
        });
    }
}

/// Each renderer's instance attributes, from shader location 1 upward. The depth follows at the
/// location named in each [`Recipe`], and is its own buffer.
const PANEL: [wgpu::VertexAttribute; 3] =
    wgpu::vertex_attr_array![1 => Float32x4, 2 => Float32x4, 3 => Float32x4];
const POLYGON: [wgpu::VertexAttribute; 3] =
    wgpu::vertex_attr_array![1 => Float32x4, 2 => Float32x4, 3 => Float32x3];
const LINE: [wgpu::VertexAttribute; 3] =
    wgpu::vertex_attr_array![1 => Float32x4, 2 => Float32x4, 3 => Float32x3];
const ICON: [wgpu::VertexAttribute; 4] =
    wgpu::vertex_attr_array![1 => Float32x4, 2 => Float32x4, 3 => Float32x4, 4 => Float32];
const IMAGE: [wgpu::VertexAttribute; 4] =
    wgpu::vertex_attr_array![1 => Float32x4, 2 => Float32x4, 3 => Float32x4, 4 => Float32];
