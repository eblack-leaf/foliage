//! Palette -- the roles a fill is stated in.

use bevy_ecs::component::Component;

use crate::color::Color;

/// What a color is for, rather than what it is.
///
/// An element declares a role and the palette decides what it resolves to, so a treatment is stated
/// once and every element carrying that role follows when it changes. A literal has no way to be
/// changed together with the others, which is why one cannot be written here.
///
/// Resolution happens in extraction, against the element's declared role. Nothing on the element
/// holds a resolved color, so there is no copy to fall out of date.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum Palette {
    /// The ordinary fill, and what an element that says nothing takes.
    #[default]
    Surface,
    /// A surface in front of another: a card against the page it sits on.
    Raised,
    /// A quieter fill, for a division or a rule.
    Muted,
    /// The one emphatic color.
    Accent,
    /// What is read against a surface rather than drawn as one.
    Ink,
}

impl Palette {
    /// The color this role resolves to.
    ///
    /// One value per role. The ramp behind them -- a tint scale, and the scheme it is read against
    /// -- lands separately, and changes what a role resolves to without changing what any element
    /// declares.
    pub(crate) fn color(self) -> Color {
        match self {
            Palette::Surface => Color::rgb(0.09, 0.10, 0.12),
            Palette::Raised => Color::rgb(0.15, 0.16, 0.19),
            Palette::Muted => Color::rgb(0.28, 0.30, 0.34),
            Palette::Accent => Color::rgb(0.38, 0.71, 0.51),
            Palette::Ink => Color::rgb(0.93, 0.94, 0.96),
        }
    }
}
