//! Where a stroked element's two ends are, per breakpoint.

use bevy_ecs::component::Component;

use crate::layout::{Layout, Short};
use crate::placement::breakpoints::{Breakpoints, Override};
use crate::placement::point::Point;
use crate::placement::source::Source;

/// A stroke's placement: two [`Point`]s, per breakpoint.
///
/// The point-mode [`Location`](crate::Location). A box states one axis pair per breakpoint and a
/// stroke states one pair of ends, and the two are written in the same chain -- so a rule that runs
/// along a row on a phone and down a column on a desk says both, and which is in force is the
/// viewport's to decide.
///
/// ```ignore
/// Trace::new()
///     .xs(Point::new(0.px(), 50.pct()), Point::new(100.pct(), 50.pct()))
///     .md(Point::new(50.pct(), 0.px()), Point::new(50.pct(), 100.pct()))
/// ```
///
/// Each link states one breakpoint, both ends together, so where a stroke runs at a given width is
/// read in one place. A breakpoint with nothing of its own takes the nearest smaller one that has,
/// and a chain that runs out falls back to a stroke of no length at its trunk's top-left corner --
/// two ends have no reading that corresponds to "the whole of the parent's box".
///
/// Responsive at the level of the pair rather than the point, for the reason a [`Point`] is a
/// value: one is constructed complete and read as a position, and a point that meant something
/// different at every width would have no [`x`](Point::x) to hand a marker's box.
#[derive(Component, Clone, Debug, PartialEq)]
pub struct Trace(pub(crate) Breakpoints<Ends>);

impl Trace {
    /// An empty trace: a stroke of no length at its trunk's top-left corner, at every breakpoint.
    ///
    /// Each of [`xs`](Trace::xs) upward states one breakpoint's ends, and a breakpoint with none of
    /// its own takes the nearest smaller one that has.
    pub fn new() -> Self {
        Self(Breakpoints::new())
    }

    /// States the ends from the smallest breakpoint up, which is to say everywhere that a larger
    /// one does not override them.
    pub fn xs(self, from: Point, to: Point) -> Self {
        self.set(Override::Xs, from, to)
    }

    /// Overrides the ends from the `sm` breakpoint up.
    pub fn sm(self, from: Point, to: Point) -> Self {
        self.set(Override::Sm, from, to)
    }

    /// Overrides the ends from the `md` breakpoint up.
    pub fn md(self, from: Point, to: Point) -> Self {
        self.set(Override::Md, from, to)
    }

    /// Overrides the ends from the `lg` breakpoint up.
    pub fn lg(self, from: Point, to: Point) -> Self {
        self.set(Override::Lg, from, to)
    }

    /// Overrides the ends at the `xl` breakpoint.
    pub fn xl(self, from: Point, to: Point) -> Self {
        self.set(Override::Xl, from, to)
    }

    /// Overrides the ends whenever the viewport is vertically cramped, whatever its width.
    pub fn short(self, from: Point, to: Point) -> Self {
        self.set(Override::Short, from, to)
    }

    fn set(mut self, at: Override, from: Point, to: Point) -> Self {
        self.0.set(at, Ends { from, to });
        self
    }

    pub(crate) fn ends(&self, layout: Layout, short: Short) -> &Ends {
        self.0.at(layout, short)
    }
}

impl Default for Trace {
    /// A stroke of no length at its trunk's top-left corner.
    fn default() -> Self {
        Self::new()
    }
}

/// One breakpoint's ends: where a stroke starts and where it stops, as the grammar states them.
///
/// The declared pair, and not where the two ends landed -- that is
/// [`Stretched`](crate::line::Stretched), which is what [`Vein::Ends`](crate::Vein::Ends) reads.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Ends {
    pub(crate) from: Point,
    pub(crate) to: Point,
}

impl Default for Ends {
    /// The trunk's top-left corner, to itself.
    fn default() -> Self {
        Self {
            from: Point::new(0.px(), 0.px()),
            to: Point::new(0.px(), 0.px()),
        }
    }
}
