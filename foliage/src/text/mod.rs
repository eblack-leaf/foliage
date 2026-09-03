//! Text -- a run of monospaced glyphs, and the character cell everything else is measured in.
//!
//! # Width flows down. Height flows up.
//!
//! Wrapping makes a run's height depend on its width, and its width comes from the layout. That is
//! a cycle in the general case, and it is what stopped anything being sized to text that wraps.
//!
//! Monospace is what unties it. A run's **max-content width** -- the widest it would like to be,
//! unwrapped -- is its longest line's character count times the cell width: exact, free, and
//! available before any layout has happened at all. So the pass going down has everything it needs,
//! and only the pass coming up has to measure anything, by which time it has a width to measure
//! against.
//!
//! | | Pass | Produces |
//! |---|---|---|
//! | R1 | measure | the character cell, and max-content width |
//! | R2m | wrap | the lines the run takes at its resolved width, and so its height |
//!
//! Both are [`Rowan`](crate::rowan)'s. What this module owns is the vocabulary they are stated in:
//! which font, at what size, and what the run says.

pub(crate) mod font;
pub(crate) mod shape;

use bevy_ecs::component::Component;

use crate::elm::{Chlorophyll, Pigment};
use crate::op::Bud;
use crate::palette::{Fill, Palette};
use crate::place::{Caller, Placement, Places};
use crate::seed::Buds;

pub use font::{Font, FontSize};

/// A run of monospaced glyphs.
///
/// Where it sits, how it is divided and what it anchors to read exactly as they do on a
/// [`Stem`](crate::Stem). What it holds beyond that is what it says, what it is filled with, and
/// which font it composes in.
///
/// It is the one element whose box can be **measured rather than declared**. In a width role
/// [`content()`](crate::content) is the widest the run would like to be; in a height role it is how
/// tall the run turned out at the width the layout gave it:
///
/// ```no_run
/// # use foliage::{FontSize, Location, Palette, Source, Text, Place, content, left, top};
/// Text::new("as tall as it wraps to")
///     .color(Palette::Ink)
///     .font_size(FontSize::new().xs(14).lg(18))
///     .at(Location::new().xs(
///         left(0.px()).right(100.pct()),
///         top(0.px()).height(content()),
///     ));
/// ```
///
/// [`letters`](crate::Source::letters) is the other half, and it stays: where the count is genuinely
/// known ahead of time it costs nothing to resolve and says what it means. Measuring is for where it
/// is not.
#[derive(Clone, Debug, Default)]
pub struct Text {
    pub(crate) placement: Placement,
    pub(crate) value: String,
    pub(crate) fill: Fill,
}

impl Text {
    /// A run saying `value`, filled with [`Palette::Ink`] -- what is read against a surface rather
    /// than drawn as one.
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            placement: Placement::default(),
            value: value.into(),
            fill: Fill::Role(Palette::Ink),
        }
    }

    /// What the glyphs are filled with: a [`Palette`] role, or a [`Color`](crate::Color) stated
    /// outright.
    ///
    /// The same [`Fill`] a panel takes, so a run is repainted, animated and cancelled by the same
    /// writes -- and so a per-character tint, when it lands, is stated in the vocabulary the run
    /// already is rather than in a second one beside it.
    pub fn color(mut self, fill: impl Into<Fill>) -> Self {
        self.fill = fill.into();
        self
    }
}

impl Places for Text {
    fn placement(&mut self) -> &mut Placement {
        &mut self.placement
    }
}

impl Buds for Text {
    fn bud(mut self, at: Caller) -> Bud {
        // Every element may carry a typeface; a run of glyphs is the one that must, because there is
        // no reading of it at all without a font to read it in.
        self.placement.typeface.get_or_insert_default();
        Bud {
            chlorophyll: Chlorophyll::Text,
            pigment: Some(Pigment::Text(TextPigment { fill: self.fill })),
            lettering: Some(Lettering(self.value)),
            placement: self.placement,
            at,
        }
    }
}

/// What a run says.
///
/// Content rather than renderer state: it is what R1 shapes and what R2m wraps, so it is read by
/// passes that have nothing to do with drawing. [`text`](crate::Grow::text) writes it.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Lettering(pub(crate) String);

/// What the text renderer was told.
///
/// Grown alongside [`Chlorophyll::Text`] and by nothing else, so an element carries both or neither.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq)]
pub(crate) struct TextPigment {
    pub(crate) fill: Fill,
}
