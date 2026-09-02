//! Palette -- the roles a fill is stated in, and the scheme that answers them.

use bevy_ecs::component::Component;

use crate::color::Color;

/// What a color is for, rather than what it is.
///
/// An element declares a role and the [`Scheme`] decides what it resolves to, so a treatment is
/// stated once and every element carrying that role follows when it changes. A literal has no way to
/// be changed together with the others, which is why one cannot be written here.
///
/// Resolution happens in extraction, against the role the element declared. Nothing on the element
/// holds a resolved color, so there is no copy to fall out of date and repainting is one write.
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
    /// Where a [`Scheme`] holds this role's color.
    fn index(self) -> usize {
        match self {
            Palette::Surface => 0,
            Palette::Raised => 1,
            Palette::Muted => 2,
            Palette::Accent => 3,
            Palette::Ink => 4,
        }
    }
}

/// What each [`Palette`] role resolves to.
///
/// One value per role, written at boot or at any frame after it with
/// [`repaint`](crate::Grow::repaint). Changing it changes every element carrying an affected role
/// and nothing else: extraction resolves the role each frame and compares the result, so the
/// elements that moved are exactly the ones that were painted in a color that changed.
///
/// The ramp that will sit behind this -- a tint scale per role, and a light and dark reading of it
/// -- lands separately. It changes what a role resolves to, never what an element declares.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Scheme([Color; 5]);

impl Scheme {
    /// The scheme every role resolves to until one is given: a dark neutral ground with a green
    /// accent.
    pub fn new() -> Self {
        Self::default()
    }

    /// What one role resolves to.
    pub fn set(mut self, role: Palette, color: Color) -> Self {
        self.0[role.index()] = color;
        self
    }

    /// The color `role` resolves to.
    pub fn color(&self, role: Palette) -> Color {
        self.0[role.index()]
    }
}

impl Default for Scheme {
    fn default() -> Self {
        Self([
            Color::rgb(0.09, 0.10, 0.12),
            Color::rgb(0.15, 0.16, 0.19),
            Color::rgb(0.28, 0.30, 0.34),
            Color::rgb(0.38, 0.71, 0.51),
            Color::rgb(0.93, 0.94, 0.96),
        ])
    }
}
