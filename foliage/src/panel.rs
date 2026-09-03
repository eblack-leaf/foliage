//! Panel -- a filled rectangle.

use bytemuck::{Pod, Zeroable};

use crate::color::Color;
use crate::coordinate::Section;
use crate::palette::Fill;
use crate::place::{Placement, Places};
use crate::rounding::Corners;

/// A filled rectangle with rounded corners.
///
/// A card, a backdrop, a divider and a button's body are all this with a different box, so it is
/// what most of a surface is composed from.
///
/// Where it sits, how it is divided, and what it anchors to read exactly as they do on a
/// [`Stem`](crate::Stem). What it holds beyond that is what it is filled with.
#[derive(Clone, Debug, Default)]
pub struct Panel {
    pub(crate) placement: Placement,
    pub(crate) fill: Fill,
    pub(crate) rounding: Corners,
}

impl Panel {
    /// A surface-filled rectangle with square corners, filling its trunk.
    pub fn new() -> Self {
        Self::default()
    }

    /// What it is filled with: a [`Palette`](crate::Palette) role, or a [`Color`] stated outright. Undeclared, it is
    /// [`Palette::Surface`](crate::Palette::Surface).
    ///
    /// A role is the ordinary answer, and a literal is an element saying it is not part of the
    /// scheme -- a [`repaint`](crate::Grow::repaint) moves the first and not the second.
    pub fn color(mut self, fill: impl Into<Fill>) -> Self {
        self.fill = fill.into();
        self
    }

    /// How its corners are rounded, per corner or all at once. Undeclared, they are square.
    pub fn rounding(mut self, rounding: impl Into<Corners>) -> Self {
        self.rounding = rounding.into();
        self
    }
}

impl Places for Panel {
    fn placement(&mut self) -> &mut Placement {
        &mut self.placement
    }
}

/// One panel, in the form the backend takes it.
///
/// The renderer's own instance, and the only form of one: it is `#[repr(C)]` over the types the
/// coordinate module guarantees the layout of, so this is both what extraction compares and what
/// the vertex buffer holds. There is no second, upload-shaped copy of it to keep in step.
///
/// Where the panel sits in the stack is not here. A rank belongs to every renderer alike, so
/// [`Instances`](crate::elm::Instances) carries it beside the instance rather than inside it, and
/// the backend turns it into a depth from the order it puts the instances in.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
pub(crate) struct PanelInstance {
    pub(crate) section: Section,
    pub(crate) color: Color,
    /// The radii the element's [`Corners`] resolved to against its own box, in logical pixels.
    pub(crate) radii: [f32; 4],
}

impl PanelInstance {
    /// What a panel resolved to, from what it was told and where it ended up.
    ///
    /// Takes the resolved state rather than reading it, because only the element's
    /// [`Chlorophyll`](crate::elm::Chlorophyll) says whether it is a panel at all -- a set of
    /// components that happens to look like one is not one.
    pub(crate) fn new(section: Section, color: Color, rounding: Corners) -> Self {
        Self {
            section,
            color,
            radii: rounding.radii(section),
        }
    }
}
