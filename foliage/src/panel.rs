//! Panel -- a filled rectangle.

use crate::color::Color;
use crate::coordinate::Section;
use crate::elevation::ResolvedElevation;
use crate::palette::Palette;
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
    pub(crate) color: Palette,
    pub(crate) rounding: Corners,
}

impl Panel {
    /// A surface-filled rectangle with square corners, filling its trunk.
    pub fn new() -> Self {
        Self::default()
    }

    /// The role it is filled with. Undeclared, it is [`Palette::Surface`].
    pub fn color(mut self, color: Palette) -> Self {
        self.color = color;
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
/// The renderer's own instance. What it is made of is the renderer's business; extraction only
/// compares one against what the backend is already holding.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct PanelInstance {
    pub(crate) section: Section,
    pub(crate) color: Color,
    /// The radii the element's [`Corners`] resolved to against its own box, in logical pixels.
    pub(crate) radii: [f32; 4],
    pub(crate) elevation: ResolvedElevation,
}

impl PanelInstance {
    /// What a panel resolved to, from what it was told and where it ended up.
    ///
    /// Takes the resolved state rather than reading it, because only the element's
    /// [`Chlorophyll`](crate::elm::Chlorophyll) says whether it is a panel at all -- a set of
    /// components that happens to look like one is not one.
    pub(crate) fn new(
        section: Section,
        color: Color,
        rounding: Corners,
        elevation: ResolvedElevation,
    ) -> Self {
        Self {
            section,
            color,
            radii: rounding.radii(section),
            elevation,
        }
    }
}
