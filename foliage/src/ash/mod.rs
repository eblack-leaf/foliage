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
//! # The stack
//!
//! Instances are drawn back to front, which is what alpha blending requires, and each is given a
//! depth from its place in that order. The order and the depth are the same statement made twice --
//! the depth test then holds it for fragments the draw order alone would not, and a repaint of the
//! same content lands on the same result.

use tracing::field::Empty;
use tracing::trace_span;

use crate::ash::panel::Panels;
use crate::color::Color;
use crate::elm::Elm;
use crate::ginkgo::Ginkgo;

mod instances;
mod panel;

/// The renderers, and what each is holding.
pub(crate) struct Ash {
    panels: Panels,
}

impl Ash {
    pub(crate) fn new(ginkgo: &Ginkgo) -> Self {
        Self {
            panels: Panels::new(ginkgo),
        }
    }

    /// Takes this frame's batches and puts them on the GPU.
    ///
    /// Runs once for every extraction, and reads what extraction produced without consuming it: a
    /// batch is a statement of what the backend should be holding, so applying one twice reaches
    /// the same holding as applying it once. What must not happen is applying one zero times.
    pub(crate) fn absorb(&mut self, elm: &Elm, ginkgo: &Ginkgo) {
        let span = trace_span!("absorb", written = Empty, withdrawn = Empty);
        let _entered = span.enter();
        span.record("written", elm.panels.written.len());
        span.record("withdrawn", elm.panels.withdrawn.len());
        for wanted in &elm.panels.written {
            self.panels
                .instances
                .write(wanted.leaf, wanted.rank, wanted.instance);
        }
        for leaf in &elm.panels.withdrawn {
            self.panels.instances.withdraw(*leaf);
        }
        self.panels.instances.flush(ginkgo.device(), ginkgo.queue());
    }

    /// Step 9. Paints what is held.
    ///
    /// `clear` is what the surface is cleared to. Nothing of the engine's own sits behind the tree,
    /// so it is the app's own ground -- whatever [`Palette::Surface`](crate::Palette::Surface)
    /// currently resolves to.
    pub(crate) fn draw(&self, ginkgo: &Ginkgo, clear: Color) {
        let _span = trace_span!("draw", panels = self.panels.instances.count()).entered();
        ginkgo.draw(clear, |pass| self.panels.draw(pass));
    }
}
