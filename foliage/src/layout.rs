//! Breakpoints: the responsive state a placement is read against.
//!
//! Both are properties of the surface rather than of any element, so there is one answer for the
//! whole tree at any moment. They are recomputed at intake, from the viewport, and every element
//! resolves against the same pair for the rest of the frame.

use crate::coordinate::Area;

/// The width breakpoint in force.
///
/// A [`Location`](crate::Location) and a [`Grid`](crate::Grid) may state a configuration per
/// breakpoint, and each falls back to the nearest smaller one that was given, so only `xs` is ever
/// required.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Layout {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
}

impl Layout {
    /// Lower bound of [`Sm`](Layout::Sm), in logical pixels. Below this is [`Xs`](Layout::Xs).
    pub const SM: f32 = 420.0;
    /// Lower bound of [`Md`](Layout::Md).
    pub const MD: f32 = 600.0;
    /// Lower bound of [`Lg`](Layout::Lg).
    pub const LG: f32 = 840.0;
    /// Lower bound of [`Xl`](Layout::Xl).
    pub const XL: f32 = 1200.0;

    /// The breakpoint a viewport of `viewport` falls in.
    pub fn of(viewport: Area) -> Self {
        if viewport.width >= Self::XL {
            Self::Xl
        } else if viewport.width >= Self::LG {
            Self::Lg
        } else if viewport.width >= Self::MD {
            Self::Md
        } else if viewport.width >= Self::SM {
            Self::Sm
        } else {
            Self::Xs
        }
    }
}

/// Whether the viewport is vertically cramped.
///
/// Deliberately orthogonal to [`Layout`] rather than a sixth breakpoint: width and height are
/// independent, and `Layout`'s fallback chain is a total order that only means anything along one
/// axis. A phone held landscape is genuinely `Md`-wide *and* cramped-tall, and one enum has to
/// discard half of that.
///
/// A configuration keyed to [`Short::Yes`] wins over the width-derived one. Nothing without such a
/// configuration is affected, so this changes no layout until a placement opts in.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Short {
    Yes,
    No,
}

impl Short {
    /// Below this height, in logical pixels, the viewport becomes [`Short::Yes`].
    pub const ENTER: f32 = 400.0;
    /// At or above this height it returns to [`Short::No`].
    ///
    /// The gap between the two is a deadband, and it is asymmetric on purpose. A single threshold
    /// thrashes: on mobile web the address bar hides and shows as the page scrolls, so a viewport
    /// resting near the boundary would cross it on every scroll and re-resolve the whole tree. It
    /// leans toward `Yes` because a false `Yes` is a layout more conservative than it needed to be,
    /// while a false `No` is content running off the bottom of the screen.
    pub const EXIT: f32 = 440.0;

    /// The next state, given the current one. Between [`ENTER`](Short::ENTER) and
    /// [`EXIT`](Short::EXIT) the answer is whatever it already was.
    pub fn next(self, viewport: Area) -> Self {
        match self {
            Self::No if viewport.height < Self::ENTER => Self::Yes,
            Self::Yes if viewport.height >= Self::EXIT => Self::No,
            current => current,
        }
    }
}
