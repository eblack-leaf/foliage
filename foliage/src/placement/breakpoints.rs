//! The responsive chain shared by [`Location`](crate::Location) and [`Grid`](crate::Grid).

use crate::layout::{Layout, Short};

/// One value per breakpoint, every one of them optional.
///
/// A breakpoint with no configuration of its own falls back to the nearest smaller one that has
/// them, so a placement that does not change with width is written once. The chain terminates in
/// `T`'s default rather than in a required `xs`, so an element that only differs above a certain
/// width states that width and nothing else.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Breakpoints<T> {
    xs: T,
    sm: Option<T>,
    md: Option<T>,
    lg: Option<T>,
    xl: Option<T>,
    short: Option<T>,
}

impl<T: Default> Breakpoints<T> {
    pub(crate) fn new() -> Self {
        Self {
            xs: T::default(),
            sm: None,
            md: None,
            lg: None,
            xl: None,
            short: None,
        }
    }
}

impl<T> Breakpoints<T> {
    pub(crate) fn set(&mut self, at: Override, value: T) {
        match at {
            Override::Xs => self.xs = value,
            Override::Sm => self.sm = Some(value),
            Override::Md => self.md = Some(value),
            Override::Lg => self.lg = Some(value),
            Override::Xl => self.xl = Some(value),
            Override::Short => self.short = Some(value),
        }
    }

    /// The configuration in force, falling back down the width breakpoints to `xs`.
    ///
    /// A `short` configuration wins outright when the viewport is cramped, because height and width
    /// are independent and the fallback chain only orders one of them.
    pub(crate) fn at(&self, layout: Layout, short: Short) -> &T {
        if short == Short::Yes
            && let Some(value) = &self.short
        {
            return value;
        }
        let chain = match layout {
            Layout::Xs => [None, None, None, None],
            Layout::Sm => [self.sm.as_ref(), None, None, None],
            Layout::Md => [self.md.as_ref(), self.sm.as_ref(), None, None],
            Layout::Lg => [self.lg.as_ref(), self.md.as_ref(), self.sm.as_ref(), None],
            Layout::Xl => [
                self.xl.as_ref(),
                self.lg.as_ref(),
                self.md.as_ref(),
                self.sm.as_ref(),
            ],
        };
        chain.into_iter().flatten().next().unwrap_or(&self.xs)
    }
}

/// Which breakpoint a configuration is given at. `Xs` is the base every chain falls back to, and
/// is a link like any other -- left unwritten, the base is the default.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum Override {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
    Short,
}
