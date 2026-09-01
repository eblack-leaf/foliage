//! The coordinate system a parent offers its children.

use bevy_ecs::component::Component;

use crate::coordinate::Axis;
use crate::layout::{Layout, Short};
use crate::placement::breakpoints::{Breakpoints, Override};

/// How a parent's box is divided, per breakpoint.
///
/// What a child's [`col`](crate::Source::col) and [`row`](crate::Source::row) address. Every
/// element has one -- [`Grid::default()`] is a single column and a single row, which is what makes
/// an element that is simply a positioned box behave without saying anything -- and declaring one
/// replaces it.
///
/// A grid decides how children are laid out and nothing else. Whether an element scrolls is
/// declared separately.
#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct Grid(pub(crate) Breakpoints<Tracks>);

impl Grid {
    /// A grid used at every breakpoint. Add exceptions with [`sm`](Grid::sm) upward.
    pub fn new(columns: Columns, rows: Rows) -> Self {
        Self(Breakpoints::new(Tracks {
            columns: columns.0,
            rows: rows.0,
        }))
    }

    /// Overrides the division from the `sm` breakpoint up.
    pub fn sm(self, columns: Columns, rows: Rows) -> Self {
        self.set(Override::Sm, columns, rows)
    }

    /// Overrides the division from the `md` breakpoint up.
    pub fn md(self, columns: Columns, rows: Rows) -> Self {
        self.set(Override::Md, columns, rows)
    }

    /// Overrides the division from the `lg` breakpoint up.
    pub fn lg(self, columns: Columns, rows: Rows) -> Self {
        self.set(Override::Lg, columns, rows)
    }

    /// Overrides the division at the `xl` breakpoint.
    pub fn xl(self, columns: Columns, rows: Rows) -> Self {
        self.set(Override::Xl, columns, rows)
    }

    /// Overrides the division whenever the viewport is vertically cramped, whatever its width.
    pub fn short(self, columns: Columns, rows: Rows) -> Self {
        self.set(Override::Short, columns, rows)
    }

    fn set(mut self, at: Override, columns: Columns, rows: Rows) -> Self {
        self.0.set(
            at,
            Tracks {
                columns: columns.0,
                rows: rows.0,
            },
        );
        self
    }

    pub(crate) fn tracks(&self, layout: Layout, short: Short) -> Tracks {
        *self.0.at(layout, short)
    }
}

impl Default for Grid {
    /// One column and one row: the whole of the parent, which is what a child addressing
    /// `1.col()` and `1.row()` fills.
    fn default() -> Self {
        Self::new(1.columns(), 1.rows())
    }
}

/// One breakpoint's division, on both axes.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Tracks {
    pub(crate) columns: Track,
    pub(crate) rows: Track,
}

impl Tracks {
    pub(crate) fn on(&self, axis: Axis) -> Track {
        match axis {
            Axis::Horizontal => self.columns,
            Axis::Vertical => self.rows,
        }
    }
}

/// How one axis is divided, and the space between the tracks that come out of it.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Track {
    pub(crate) pitch: Pitch,
    pub(crate) gap: f32,
}

/// What sets a track's size.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Pitch {
    /// The axis is divided into this many equal tracks, gaps taken out first.
    Count(u32),
    /// Every track is this many logical pixels, however many fit.
    Px(f32),
    /// Every track is this many character cells, in the font of the element the grid is on -- not
    /// in the font of the children addressing it. A child then gets a real column address into a
    /// letter-pitched grid rather than a hand-computed offset.
    Letters(f32),
}

impl Track {
    /// How large one track is, given the extent being divided and the character cell on this axis.
    pub(crate) fn size(&self, extent: f32, cell: f32) -> f32 {
        match self.pitch {
            Pitch::Count(count) => {
                let count = count.max(1) as f32;
                (extent - self.gap * (count - 1.0)) / count
            }
            Pitch::Px(px) => px,
            Pitch::Letters(letters) => letters * cell,
        }
    }
}

/// The horizontal division of a [`Grid`].
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Columns(Track);

/// The vertical division of a [`Grid`].
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rows(Track);

macro_rules! division {
    ($name:ident) => {
        impl $name {
            /// Tracks of a fixed width, in logical pixels, however many fit.
            pub fn px(px: f32) -> Self {
                Self(Track {
                    pitch: Pitch::Px(px),
                    gap: 0.0,
                })
            }

            /// Tracks of a fixed width in character cells, in the font of the element this grid is
            /// on rather than that of the children addressing it.
            pub fn letters(letters: f32) -> Self {
                Self(Track {
                    pitch: Pitch::Letters(letters),
                    gap: 0.0,
                })
            }

            /// The space between adjacent tracks, in logical pixels.
            ///
            /// Between tracks only, so an axis of `n` tracks has `n - 1` gaps and no outer margin.
            pub fn gap(mut self, gap: f32) -> Self {
                self.0.gap = gap;
                self
            }
        }
    };
}

division!(Columns);
division!(Rows);

/// Plain numbers as a [`Grid`]'s division.
///
/// Kept apart from [`Source`](crate::Source) by type as well as by name: `3.columns()` divides a
/// parent into three, and `3.col()` addresses the third of them.
pub trait Divide: Sized {
    /// This many equal columns.
    fn columns(self) -> Columns;

    /// This many equal rows.
    fn rows(self) -> Rows;
}

macro_rules! divide {
    ($($number:ty),*) => {$(
        impl Divide for $number {
            fn columns(self) -> Columns {
                Columns(Track { pitch: Pitch::Count(self as u32), gap: 0.0 })
            }

            fn rows(self) -> Rows {
                Rows(Track { pitch: Pitch::Count(self as u32), gap: 0.0 })
            }
        }
    )*};
}

divide!(i32, u32, f32, usize);
