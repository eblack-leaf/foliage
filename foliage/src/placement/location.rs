//! Where an element sits, per breakpoint.

use bevy_ecs::component::Component;

use crate::layout::{Layout, Short};
use crate::placement::breakpoints::{Breakpoints, Override};
use crate::placement::role::{Config, Horizontal, Vertical, left, top};
use crate::placement::source::Source;

/// An element's placement: one [`Horizontal`] and one [`Vertical`], per breakpoint.
///
/// The two axes are distinct types and are given in that order, so they cannot be written the wrong
/// way round.
///
/// ```ignore
/// Location::new(
///     left(20.px()).right(100.pct() - 16.px()),
///     top(anchor().bottom() + 8.px()).height(content()),
/// )
/// .md(left(2.col()).right(3.col()), top(0.px()).height(120.px()))
/// ```
///
/// # A value is a source and a role
///
/// The two are independent, which is what lets any source fill any coordinate. A **source** is
/// where a number comes from -- [`20.px()`](crate::Source::px), [`50.pct()`](crate::Source::pct),
/// [`2.col()`](crate::Source::col), [`8.letters()`](crate::Source::letters),
/// [`anchor()`](crate::anchor)`.right()`, [`content()`](crate::content), and arithmetic on any of
/// them. A **role** is what it means for this element: left, right, top, bottom, width, height,
/// centre.
///
/// The role is written first, because it says what the value describes before the value has to be
/// parsed: "my left is my anchor's right" is read in the order it is understood. The pairing is a
/// type rather than a rule -- each opener returns a value carrying only the completions legal with
/// it, so an axis has exactly four spellings and the illegal ones cannot be written at all.
///
/// ```ignore
/// left(anchor().right()).width(140.px())
/// left(20.px()).right(100.pct() - 16.px())
/// center_x(50.pct()).width(140.px())
/// top(0.px()).height(content())
/// ```
///
/// Every edge role is a coordinate in the parent's space, including `right` and `bottom`, so
/// sixteen in from the right is `right(100.pct() - 16.px())`. Insets would read more nicely for
/// that one case, but an anchor's edges are already positions, so an inset would put two coordinate
/// spaces in one expression.
///
/// # Width flows down. Height flows up.
///
/// Text wrapping makes height depend on width, and width comes from layout, so a container sized to
/// its contents is a cycle in the general case. It is bounded here at two passes, with no iteration
/// to convergence: the horizontal axis resolves for the whole tree, each text run wraps at its
/// now-known width, and then the vertical axis resolves and reads the heights that came out of it.
///
/// A monospaced run's widest unwrapped width is its character count times the cell width -- exact,
/// free, and available before any layout has happened. That is what makes the down-pass complete,
/// and why only the up-pass needs a real measurement.
///
/// The order is visible in the types: a [`VerticalLength`](crate::VerticalLength) is refused by a
/// horizontal role, because the horizontal pass runs first and cannot read a height.
#[derive(Component, Clone, Debug, PartialEq)]
pub struct Location(pub(crate) Breakpoints<Axes>);

impl Location {
    /// A placement used at every breakpoint. Add exceptions with [`sm`](Location::sm) upward.
    pub fn new(horizontal: Horizontal, vertical: Vertical) -> Self {
        Self(Breakpoints::new(Axes {
            horizontal: horizontal.0,
            vertical: vertical.0,
        }))
    }

    /// Overrides the placement from the `sm` breakpoint up.
    pub fn sm(self, horizontal: Horizontal, vertical: Vertical) -> Self {
        self.set(Override::Sm, horizontal, vertical)
    }

    /// Overrides the placement from the `md` breakpoint up.
    pub fn md(self, horizontal: Horizontal, vertical: Vertical) -> Self {
        self.set(Override::Md, horizontal, vertical)
    }

    /// Overrides the placement from the `lg` breakpoint up.
    pub fn lg(self, horizontal: Horizontal, vertical: Vertical) -> Self {
        self.set(Override::Lg, horizontal, vertical)
    }

    /// Overrides the placement at the `xl` breakpoint.
    pub fn xl(self, horizontal: Horizontal, vertical: Vertical) -> Self {
        self.set(Override::Xl, horizontal, vertical)
    }

    /// Overrides the placement whenever the viewport is vertically cramped, whatever its width.
    pub fn short(self, horizontal: Horizontal, vertical: Vertical) -> Self {
        self.set(Override::Short, horizontal, vertical)
    }

    fn set(mut self, at: Override, horizontal: Horizontal, vertical: Vertical) -> Self {
        self.0.set(
            at,
            Axes {
                horizontal: horizontal.0,
                vertical: vertical.0,
            },
        );
        self
    }

    pub(crate) fn axes(&self, layout: Layout, short: Short) -> &Axes {
        self.0.at(layout, short)
    }
}

impl Default for Location {
    /// The whole of the parent's box.
    fn default() -> Self {
        Self::new(
            left(0.px()).right(100.pct()),
            top(0.px()).bottom(100.pct()),
        )
    }
}

impl From<(Horizontal, Vertical)> for Location {
    fn from((horizontal, vertical): (Horizontal, Vertical)) -> Self {
        Self::new(horizontal, vertical)
    }
}

/// One breakpoint's placement: both axes, each pinned down.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Axes {
    pub(crate) horizontal: Config,
    pub(crate) vertical: Config,
}
