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
//! # The stack is shared
//!
//! There is one stack, across every renderer, ordered back to front by rank -- which is what alpha
//! blending requires. It is walked here, and a renderer holds no opinion about it: each keeps its
//! own slots sorted by rank, and this merges them.
//!
//! Two things come out of that one walk, and they must come out of the same one. Each instance is
//! given a **depth** from its position in it, so the depth test holds the order for fragments the
//! draw order alone would not, and a repaint of the same content lands on the same result. And the
//! walk is cut into **spans**: maximal runs sharing a renderer and a clip. A run of one renderer's
//! slots is contiguous in that renderer's buffer, because the merge meets them in slot order, so a
//! span is one draw of one range.
//!
//! Clipping is applied to the pass as a scissor rather than carried in an instance and tested per
//! fragment, so it costs nothing per element and every renderer has it without a line of shader.

use tracing::field::Empty;
use tracing::trace_span;

use crate::ash::panel::Panels;
use crate::ash::text::Texts;
use crate::color::Color;
use crate::coordinate::Section;
use crate::elevation::ResolvedElevation;
use crate::elm::Elm;
use crate::ginkgo::Ginkgo;
use crate::ginkgo::depth::Depth;
use crate::text::font::Fonts;

mod atlas;
mod instances;
mod panel;
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
    panels: Panels,
    texts: Texts,
    /// The draws, in order. One per run of the stack sharing a renderer and a clip.
    spans: Vec<Span>,
    /// The stack itself, kept between walks for its capacity.
    stack: Vec<Slot>,
}

/// One run of the stack: which renderer draws it, what it is scissored to, and which of that
/// renderer's slots it covers.
#[derive(Copy, Clone, Debug)]
struct Span {
    renderer: Renderer,
    clip: Section,
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
    Text,
}

impl Ash {
    pub(crate) fn new(ginkgo: &Ginkgo) -> Self {
        Self {
            panels: Panels::new(ginkgo),
            texts: Texts::new(ginkgo),
            spans: Vec::new(),
            stack: Vec::new(),
        }
    }

    /// Takes this frame's batches and puts them on the GPU.
    ///
    /// Runs once for every extraction, and reads what extraction produced without consuming it: a
    /// batch is a statement of what the backend should be holding, so applying one twice reaches
    /// the same holding as applying it once. What must not happen is applying one zero times.
    /// `fonts` is here and not in the batch because cutting a glyph is the backend's: what
    /// extraction states is the cell and the character, and what a face makes of that -- at this
    /// display's density -- is the only part that has to be re-answered when the display changes.
    pub(crate) fn absorb(&mut self, elm: &Elm, fonts: &Fonts, ginkgo: &Ginkgo) {
        let span = trace_span!("absorb", written = Empty, withdrawn = Empty, glyphs = Empty);
        let _entered = span.enter();
        span.record(
            "written",
            elm.panels.written.len() + elm.texts.written.len(),
        );
        span.record(
            "withdrawn",
            elm.panels.withdrawn.len() + elm.texts.withdrawn.len(),
        );
        for wanted in &elm.panels.written {
            self.panels
                .instances
                .write(wanted.key, wanted.rank, wanted.clip, wanted.instance);
        }
        for key in &elm.panels.withdrawn {
            self.panels.instances.withdraw(*key);
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
        self.panels.instances.flush(ginkgo.device(), ginkgo.queue());
        self.texts.flush(ginkgo.device(), ginkgo.queue());
        span.record("glyphs", self.texts.len());
        // Both, and not the first that says so: a walk skipped because the other renderer answered
        // first would leave that one's new slots without a depth.
        let panels = self.panels.instances.disturbed();
        let texts = self.texts.disturbed();
        if panels || texts {
            self.restack(ginkgo);
        }
    }

    /// Walks the one stack: gives every instance its depth, and cuts the draw into spans.
    ///
    /// Runs only when something in it moved -- a slot appeared, went, changed rank, or changed what
    /// it is clipped to. A frame that only recoloured what was already there leaves the stack as it
    /// was, and this does not run.
    fn restack(&mut self, ginkgo: &Ginkgo) {
        let _span = trace_span!("restack").entered();
        self.stack.clear();
        self.stack.extend(
            self.panels
                .instances
                .ranks()
                .iter()
                .enumerate()
                .map(|(slot, rank)| Slot {
                    rank: *rank,
                    renderer: Renderer::Panel,
                    slot: slot as u32,
                }),
        );
        // A run contributes one entry however many characters it has, which is what keeps this the
        // size of the tree rather than the size of the text on it.
        self.stack
            .extend(self.texts.ranks().iter().enumerate().map(|(slot, rank)| {
                Slot {
                    rank: *rank,
                    renderer: Renderer::Text,
                    slot: slot as u32,
                }
            }));
        self.stack.sort_unstable();
        let total = self.stack.len();
        self.spans.clear();
        for (position, entry) in self.stack.iter().enumerate() {
            let depth = Depth::of(position, total);
            let clip = match entry.renderer {
                Renderer::Panel => {
                    self.panels.instances.set_depth(entry.slot, depth);
                    self.panels.instances.clip(entry.slot)
                }
                Renderer::Text => {
                    self.texts.set_depth(entry.slot, depth);
                    self.texts.clip(entry.slot)
                }
            };
            match self.spans.last_mut() {
                Some(span)
                    if span.renderer == entry.renderer
                        && span.clip == clip
                        && span.to == entry.slot =>
                {
                    span.to = entry.slot + 1;
                }
                _ => self.spans.push(Span {
                    renderer: entry.renderer,
                    clip,
                    from: entry.slot,
                    to: entry.slot + 1,
                }),
            }
        }
        self.panels.instances.flush_depths(ginkgo.queue());
        self.texts.flush_depths(ginkgo.queue());
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
                match span.renderer {
                    Renderer::Panel => self.panels.draw(pass, span.from..span.to),
                    Renderer::Text => self.texts.draw(pass, span.from..span.to),
                }
            }
        });
    }
}
