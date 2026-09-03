//! The box stack, and the one read taken of it.
//!
//! # What is at this point, and who receives it, are two questions
//!
//! At a pointer position the **box stack** is every element whose box contains that point and which
//! is not clipped away, hidden or fully transparent, ordered top-first by resolved elevation.
//!
//! **Membership is universal.** Nothing opts in. An element is in the stack because it is there,
//! which is geometry, and it is what makes occlusion per-point and automatic: there is no such
//! thing as an obscured element, only an element that is below another *at this point*. A panel
//! covering half a button is above it for those pixels and absent for the rest, with no special
//! handling anywhere.
//!
//! # The read does not search
//!
//! A gesture goes to the top of the stack -- the element nearest the viewer at that point, whatever
//! it draws and whether or not it receives. [`pass_through`](crate::Place::pass_through) is what
//! takes an element out of the stack for this purpose, so the element beneath it is the top. An
//! element that is at the top without receiving eats the gesture, and a backdrop, a sheet backing
//! and a menu's padding are all that without declaring anything to be it.
//!
//! Reading the top is the whole of it. The engine never continues downward looking for an element
//! willing to take the gesture, and never passes over one because it is undeclared or draws
//! nothing. A search would have to judge, at each element it passed, whether that element is *part
//! of* what it covers or a *layer over* it -- and at a point those are the same picture, so it
//! would answer by inference. Inference is wrong silently, and it is wrong twice: where a gesture
//! lands is where a following drag looks for its scrolling region, so a press attributed to the
//! wrong element scrolls the wrong region.

use crate::coordinate::{Position, Section};
use crate::elevation::ResolvedElevation;
use crate::interaction::Shape;
use crate::leaf::Leaf;

/// One element as the hit test sees it: where it is, what shape it is, and what it declared.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Region {
    pub(crate) leaf: Leaf,
    /// Where it was drawn.
    pub(crate) section: Section,
    /// What a scrolling ancestor left of it. A point outside this is outside the element, however
    /// far its own box extends.
    pub(crate) clip: Section,
    pub(crate) shape: Shape,
    /// Marked [`pass_through`](crate::Place::pass_through): in the stack, and never the top of it.
    pub(crate) transparent: bool,
    /// Declared [`interactive`](crate::Place::interactive).
    pub(crate) receives: bool,
    /// Disabled, in its own right or by an ancestor. Blocks, and gives nothing back.
    pub(crate) disabled: bool,
}

impl Region {
    /// Whether the element covers `point`.
    fn holds(&self, point: Position) -> bool {
        if !self.clip.contains(point) {
            return false;
        }
        match self.shape {
            Shape::Box => self.section.contains(point),
            // The ellipse inscribed in the box, so a round control does not take presses in the
            // square corners it does not draw.
            Shape::Round => {
                if self.section.is_empty() {
                    return false;
                }
                let center = self.section.center();
                let x = (point.x - center.x) / (self.section.width() / 2.0);
                let y = (point.y - center.y) / (self.section.height() / 2.0);
                x * x + y * y <= 1.0
            }
        }
    }
}

/// Every element that was drawn, front-most first.
///
/// Rebuilt by R8 each frame and read by the next frame's dispatch, which is what makes hit-testing
/// run against what was drawn: a pointer event was produced by a person looking at the screen, and
/// the screen is the previous frame's render.
#[derive(Default)]
pub(crate) struct Stack {
    regions: Vec<Region>,
}

impl Stack {
    /// Takes this frame's regions, ordering them front-most first.
    ///
    /// The order is total -- a resolved elevation carries allocation order as its tie-break -- so
    /// two identical runs read the same element at the same point.
    pub(crate) fn settle(&mut self, mut ranked: Vec<(ResolvedElevation, Region)>) {
        ranked.sort_by(|left, right| right.0.cmp(&left.0));
        self.regions.clear();
        self.regions
            .extend(ranked.into_iter().map(|(_, region)| region));
    }

    /// The top of the stack at `point`: the front-most element there that is not
    /// [`pass_through`](crate::Place::pass_through).
    ///
    /// One read, and the whole of the hit test.
    pub(crate) fn top(&self, point: Position) -> Option<Region> {
        self.regions
            .iter()
            .find(|region| !region.transparent && region.holds(point))
            .copied()
    }

    /// How many elements were in the stack. Reported to the trace, and nothing else reads it.
    pub(crate) fn len(&self) -> usize {
        self.regions.len()
    }
}
