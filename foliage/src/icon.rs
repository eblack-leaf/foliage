//! Icon -- a vector mark, drawn from a distance field.

use bevy_ecs::component::Component;
use tracing::info;

use crate::coordinate::Section;
use crate::color::Color;
use crate::elm::{Chlorophyll, Pigment};
use crate::op::Bud;
use crate::palette::{Fill, Palette};
use crate::place::{Boxed, Caller, Placement, Places};
use crate::seed::Buds;

/// A registered mark: one multi-channel distance field, at whatever resolution it was baked.
///
/// Handed out by [`Foliage::icon`](crate::Foliage::icon) and named on an element with
/// [`Icon::new`]. Opaque: there is nothing to be done with one but draw it.
///
/// A field rather than a bitmap, and that is the whole reason an icon is not a glyph. A glyph is
/// cut into coverage at a size, because text is composed at a handful of sizes and a cut is exact.
/// A mark is stretched to whatever box a layout hands it -- a 16px affordance and a 96px empty
/// state are the same artwork -- so it is stored once as a distance to its own edge and
/// reconstructed at any size, sharp at every one of them.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Field(pub(crate) u32);

/// A vector mark, filled like text and scaled like a shape.
///
/// Sized by its box like any other element; the artwork is fitted into the largest square that box
/// holds, so a non-square box leaves room around the mark rather than distorting it.
///
/// ```no_run
/// # use foliage::{Boxed, Field, Icon, Location, Palette, Source, left, top};
/// # fn f(check: Field) {
/// Icon::new(check)
///     .color(Palette::Accent)
///     .at(Location::new().xs(
///         left(0.px()).width(24.px()),
///         top(0.px()).height(24.px()),
///     ));
/// # }
/// ```
///
/// What it is filled with is entirely the element's: the field carries shape and no colour, exactly
/// as a glyph's coverage does. So an icon is recoloured, repainted and animated by the same writes a
/// panel and a run take.
#[derive(Clone, Debug)]
pub struct Icon {
    pub(crate) placement: Placement,
    pub(crate) field: Field,
    pub(crate) fill: Fill,
}

impl Icon {
    /// A mark drawing the registered `field`, filled with [`Palette::Ink`] -- what is read against a
    /// surface rather than drawn as one, which is what a mark beside a label is.
    pub fn new(field: Field) -> Self {
        Self {
            placement: Placement::default(),
            field,
            fill: Fill::Role(Palette::Ink),
        }
    }

    /// What the mark is filled with: a [`Palette`] role, or a [`Color`](crate::Color) stated
    /// outright.
    pub fn color(mut self, fill: impl Into<Fill>) -> Self {
        self.fill = fill.into();
        self
    }
}

impl Places for Icon {
    fn placement(&mut self) -> &mut Placement {
        &mut self.placement
    }
}

impl Boxed for Icon {}

impl Buds for Icon {
    fn bud(self, at: Caller) -> Bud {
        Bud {
            chlorophyll: Chlorophyll::Icon,
            pigment: Some(Pigment::Icon(IconPigment {
                field: self.field,
                fill: self.fill,
            })),
            placement: self.placement,
            at,
            ..Bud::bare()
        }
    }
}

/// What the icon renderer was told.
///
/// Grown alongside [`Chlorophyll::Icon`] and by nothing else, so an element carries both or neither.
#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub(crate) struct IconPigment {
    pub(crate) field: Field,
    pub(crate) fill: Fill,
}

/// One icon, as extraction states it.
///
/// Not the form the vertex buffer takes: where the field sits on the sheet, and how many screen
/// pixels its distance range spans at this size, are both the backend's -- the first because only
/// the backend packs the sheet, the second because it depends on the display's density. What
/// extraction supplies is the box, the fill, and which field.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct IconInstance {
    pub(crate) section: Section,
    pub(crate) color: Color,
    pub(crate) field: Field,
}

/// Every registered mark's field bytes.
///
/// Held on the engine rather than in the backend because a field is registered before there is a
/// device to hold it: an app names its marks at boot, and the surface arrives when the platform
/// resumes. The backend packs what is here onto its sheet the first time it draws one.
#[derive(Default)]
pub(crate) struct Fields {
    /// A slot per name. Empty where a name was handed out for a field that has not arrived yet: an
    /// element drawing one occupies its box and draws nothing until it does, exactly as a picture
    /// whose pixels are still coming does.
    marks: Vec<Option<Mark>>,
}

/// One registered mark: the field itself, and what the shader needs to read it.
pub(crate) struct Mark {
    /// The field, `side` by `side` texels of RGBA, row-major.
    pub(crate) field: Vec<u8>,
    pub(crate) side: u32,
    /// How many texels the baked distance range spans. What turns a sampled distance into a
    /// screen-space edge one pixel wide at whatever size the mark is drawn.
    pub(crate) range: f32,
}

impl Fields {
    /// Registers a mark and hands back the name elements choose it by.
    pub(crate) fn register(&mut self, field: &[u8], side: u32, range: f32) -> Field {
        assert!(
            side > 0 && field.len() as u32 >= side * side * 4,
            "an icon field is {side}x{side} texels of RGBA, which is {} bytes, and {} were given",
            side * side * 4,
            field.len(),
        );
        self.marks.push(Some(Mark {
            field: field.to_vec(),
            side,
            range: range.max(1.0),
        }));
        let name = Field(self.marks.len() as u32 - 1);
        info!(field = name.0, side, range, "icon registered");
        name
    }

    /// A name for a field that has not been read yet, so elements can name it now.
    pub(crate) fn pending(&mut self) -> Field {
        self.marks.push(None);
        Field(self.marks.len() as u32 - 1)
    }

    /// Fills a name handed out by [`pending`](Fields::pending).
    ///
    /// Refuses rather than asserts, for the reason a fetched font is refused rather than panicked
    /// on: what arrived from a path or a URL is not something the program stated.
    pub(crate) fn fill(
        &mut self,
        field: Field,
        bytes: &[u8],
        side: u32,
        range: f32,
    ) -> Result<(), String> {
        if side == 0 || (bytes.len() as u32) < side * side * 4 {
            return Err(format!(
                "a {side}x{side} field is {} bytes of RGBA, and {} arrived",
                side * side * 4,
                bytes.len()
            ));
        }
        let Some(slot) = self.marks.get_mut(field.0 as usize) else {
            return Err("no such field".to_string());
        };
        *slot = Some(Mark {
            field: bytes.to_vec(),
            side,
            range: range.max(1.0),
        });
        info!(field = field.0, side, range, "icon registered");
        Ok(())
    }

    /// The mark `field` names, or `None` where it named nothing or has yet to arrive.
    pub(crate) fn mark(&self, field: Field) -> Option<&Mark> {
        self.marks.get(field.0 as usize)?.as_ref()
    }
}
