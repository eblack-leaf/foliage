//! Corner rounding, stated as a bracket rather than as a radius.

use bevy_ecs::component::Component;

use crate::coordinate::Section;

/// How much one corner is rounded.
///
/// A bracket rather than a pixel count, so one treatment is stated once and reads the same wherever
/// it is used. [`Xs`](Rounding::Xs) through [`Lg`](Rounding::Lg) are fixed radii, which is what makes
/// a small chip and a large card look like the same family; a radius taken as a fraction of the box
/// would grow into a distorted curve as the box did.
///
/// [`Full`](Rounding::Full) is the one bracket that scales, because a pill's radius is half its
/// shorter side by definition. Every other bracket is clamped to that same ceiling, so a box smaller
/// than its own bracket cannot ask for more curve than it has room for.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Rounding {
    /// Square corners.
    #[default]
    None,
    /// 4 logical pixels.
    Xs,
    /// 8 logical pixels.
    Sm,
    /// 12 logical pixels.
    Md,
    /// 16 logical pixels.
    Lg,
    /// Half the box's shorter side, which is a pill.
    Full,
}

impl Rounding {
    /// The radius this bracket resolves to for `section`, in logical pixels.
    fn radius(self, section: Section) -> f32 {
        let ceiling = section.width().min(section.height()) / 2.0;
        match self {
            Rounding::None => 0.0,
            Rounding::Xs => 4.0,
            Rounding::Sm => 8.0,
            Rounding::Md => 12.0,
            Rounding::Lg => 16.0,
            Rounding::Full => ceiling,
        }
        .min(ceiling)
        .max(0.0)
    }
}

/// One corner of a box.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Corner {
    /// The top-left corner.
    TopLeft,
    /// The top-right corner.
    TopRight,
    /// The bottom-right corner.
    BottomRight,
    /// The bottom-left corner.
    BottomLeft,
}

impl Corner {
    /// The four, in the order [`Corners`] holds them.
    const ALL: [Corner; 4] = [
        Corner::TopLeft,
        Corner::TopRight,
        Corner::BottomRight,
        Corner::BottomLeft,
    ];

    fn index(self) -> usize {
        match self {
            Corner::TopLeft => 0,
            Corner::TopRight => 1,
            Corner::BottomRight => 2,
            Corner::BottomLeft => 3,
        }
    }
}

/// One edge of a box, naming the two corners on it.
///
/// What a segmented control is made of: the first cell rounds its left, the last its right, and
/// everything between stays square, so the row reads as one shape rather than as a line of pills.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Side {
    /// The left edge: the top-left and bottom-left corners.
    Left,
    /// The right edge: the top-right and bottom-right corners.
    Right,
    /// The top edge: the top-left and top-right corners.
    Top,
    /// The bottom edge: the bottom-left and bottom-right corners.
    Bottom,
}

impl Side {
    fn corners(self) -> [Corner; 2] {
        match self {
            Side::Left => [Corner::TopLeft, Corner::BottomLeft],
            Side::Right => [Corner::TopRight, Corner::BottomRight],
            Side::Top => [Corner::TopLeft, Corner::TopRight],
            Side::Bottom => [Corner::BottomLeft, Corner::BottomRight],
        }
    }
}

/// An element's four corners, each with its own bracket.
///
/// Four rather than one because a corner is where two edges meet and adjacent elements have to
/// agree about it. Anything that abuts something else rounds the outside and leaves the join square.
///
/// A single [`Rounding`] converts into this, so the common case says one word:
/// `Panel::new().rounding(Rounding::Md)`.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Corners([Rounding; 4]);

impl Corners {
    /// Every corner square.
    pub fn none() -> Self {
        Self::all(Rounding::None)
    }

    /// Every corner at one bracket.
    pub fn all(rounding: Rounding) -> Self {
        Self([rounding; 4])
    }

    /// Rounds the two corners on `side`, leaving the rest as they are.
    pub fn side(mut self, side: Side, rounding: Rounding) -> Self {
        for corner in side.corners() {
            self.0[corner.index()] = rounding;
        }
        self
    }

    /// Rounds one corner, leaving the rest as they are.
    pub fn corner(mut self, corner: Corner, rounding: Rounding) -> Self {
        self.0[corner.index()] = rounding;
        self
    }

    /// What one corner is set to.
    pub fn of(self, corner: Corner) -> Rounding {
        self.0[corner.index()]
    }

    /// The four radii for `section`, in logical pixels.
    ///
    /// Ordered as [`Corner`] declares them -- top-left, top-right, bottom-right, bottom-left -- and
    /// that order is fixed, because it is what the shader indexes.
    pub(crate) fn radii(self, section: Section) -> [f32; 4] {
        Corner::ALL.map(|corner| self.of(corner).radius(section))
    }
}

impl From<Rounding> for Corners {
    fn from(rounding: Rounding) -> Self {
        Self::all(rounding)
    }
}
