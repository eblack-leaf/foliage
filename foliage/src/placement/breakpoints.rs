//! The responsive chain shared by [`Location`](crate::Location), [`Trace`](crate::Trace),
//! [`Grid`](crate::Grid) and [`FontSize`](crate::FontSize).

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

/// The same chain, holding only what was stated: `xs`, and each override that was given.
///
/// For a `T` large enough that a slot per breakpoint is most of what an element holds. Every
/// element is grown with a placement whether it states one or not, so an element pays here for the
/// breakpoints it states and nothing for the ones it does not, whichever they are.
///
/// The overrides are kept in breakpoint order, so two chains stating the same values compare equal
/// however they were written.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Stated<T> {
    xs: T,
    overrides: Vec<(Override, T)>,
}

impl<T: Default> Stated<T> {
    pub(crate) fn new() -> Self {
        Self {
            xs: T::default(),
            overrides: Vec::new(),
        }
    }
}

impl<T> Stated<T> {
    pub(crate) fn set(&mut self, at: Override, value: T) {
        if at == Override::Xs {
            self.xs = value;
            return;
        }
        match self
            .overrides
            .binary_search_by_key(&at, |(stated, _)| *stated)
        {
            Ok(index) => self.overrides[index].1 = value,
            Err(index) => self.overrides.insert(index, (at, value)),
        }
    }

    /// What it states at every breakpoint, where it states one thing and overrides nothing.
    pub(crate) fn only(&self) -> Option<&T> {
        self.overrides.is_empty().then_some(&self.xs)
    }

    /// States `value` at every breakpoint, letting go of every override and keeping the room they
    /// were held in.
    pub(crate) fn reset(&mut self, value: T) {
        self.xs = value;
        self.overrides.clear();
    }

    /// What [`Breakpoints::at`] answers, for the same chain.
    pub(crate) fn at(&self, layout: Layout, short: Short) -> &T {
        let stated = |at: Override| {
            self.overrides
                .iter()
                .find(|(stated, _)| *stated == at)
                .map(|(_, value)| value)
        };
        if short == Short::Yes
            && let Some(value) = stated(Override::Short)
        {
            return value;
        }
        let chain: &[Override] = match layout {
            Layout::Xs => &[],
            Layout::Sm => &[Override::Sm],
            Layout::Md => &[Override::Md, Override::Sm],
            Layout::Lg => &[Override::Lg, Override::Md, Override::Sm],
            Layout::Xl => &[Override::Xl, Override::Lg, Override::Md, Override::Sm],
        };
        chain.iter().find_map(|at| stated(*at)).unwrap_or(&self.xs)
    }
}

/// Which breakpoint a configuration is given at. `Xs` is the base every chain falls back to, and
/// is a link like any other -- left unwritten, the base is the default.
///
/// Ordered as the breakpoints are, which is the order a [`Stated`] chain keeps its overrides in.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(crate) enum Override {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
    Short,
}
