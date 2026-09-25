//! What a press wears, and what says a press is the one to make.
//!
//! The hue-arm. A press stands in one of a few states -- at rest, armed, inert, chosen, dangerous
//! -- and each state is a fill and an ink decided here, never at the callsite: an app says which
//! state a press is in, and the look says what that looks like. The accent is kept for the one
//! press to make, so a page with one thing to do on it says which; red is the press that cannot
//! be taken back; and everything else that has a state to show is grey, or inked in a colour.
//!
//! [`gate`] is what a chain of presses is built from: a run of steps, each done or not, as what
//! each step's press wears. What makes a step done, and what the chain is for, is the app's.

use foliage::{Corners, Ease, Fill, Grove, Palette, Rounding, Timing};

/// How long anything takes to change what it wears.
const WEAR_MS: u64 = 260;

/// A fill, and what is read against it. The two are one decision.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Tone {
    pub fill: Fill,
    pub ink: Fill,
}

impl Tone {
    /// `fill`, with what is read on it -- its [`on`](Palette::on).
    pub const fn on(fill: Palette) -> Self {
        Self {
            fill: Fill::Role(fill),
            ink: Fill::Role(fill.on()),
        }
    }
}

/// The states a press can stand in.
///
/// What each wears is [`tone`](Press::tone), and whether a press in it can be made is
/// [`within`](Press::within). Which state a press is in, and why, is the app's to say; what the
/// state means to a person reading the page is the look's, and is the same in every app.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Press {
    /// A press that can be made: the raised grey, in ink.
    #[default]
    Rest,
    /// The press to make here -- filled with the accent, which nothing at rest is.
    Armed,
    /// There, and plainly not a press: the raised grey, read in the grey between. Out of reach.
    Inert,
    /// The one of a set that is chosen: filled with the positive hue.
    Chosen,
    /// A press that cannot be taken back: filled with the danger hue. Told from the accent by its
    /// colour, never by a shape.
    Danger,
}

impl Press {
    /// What a press in this state wears.
    pub const fn tone(self) -> Tone {
        match self {
            Press::Rest => REST,
            Press::Armed => Tone::on(Palette::Accent),
            Press::Inert => INERT,
            Press::Chosen => Tone::on(Palette::Positive),
            Press::Danger => Tone::on(Palette::Danger),
        }
    }

    /// Whether a press in this state can be made. Only an inert one cannot.
    pub const fn within(self) -> bool {
        !matches!(self, Press::Inert)
    }

    /// `armed` if `ready`, and inert if not: the one question most presses ask.
    pub const fn armed_if(ready: bool) -> Self {
        match ready {
            true => Press::Armed,
            false => Press::Inert,
        }
    }

    /// At rest if `ready`, and inert if not.
    pub const fn rest_if(ready: bool) -> Self {
        match ready {
            true => Press::Rest,
            false => Press::Inert,
        }
    }
}

/// A press at rest.
pub const REST: Tone = Tone {
    fill: Fill::Role(Palette::Raised),
    ink: Fill::Role(Palette::Ink),
};

/// A press with nothing to press for.
pub const INERT: Tone = Tone {
    fill: Fill::Role(Palette::Raised),
    ink: Fill::Role(Palette::Muted),
};

/// A well: the ground's own colour, sunk into whatever stands it on. What a thing that is not the
/// chosen one of a set is -- a row of a list, a badge not picked -- so a set reads as one raised
/// thing with one of its places filled in and the rest sunk out of it.
pub const WELL: Tone = Tone {
    fill: Fill::Role(Palette::Surface),
    ink: Fill::Role(Palette::Ink),
};

/// A word that has been changed and not yet written: the raised grey, read in the accent. Ink
/// rather than fill, so it is a state shown in a colour and not the one press to make -- which is
/// what the fill is kept for, and which in this case is the press it arms.
pub const CHANGED: Tone = Tone {
    fill: Fill::Role(Palette::Raised),
    ink: Fill::Role(Palette::Accent.mark()),
};

/// The bar that divides the parts of a thing in the chip's shape.
pub const BAR: Palette = Palette::Muted;

/// What says a thing has just happened, or wants a person, without being the press to make: the
/// system's hue, as a mark.
pub const HINT: Palette = Palette::Signal.mark();

/// What a fill that is on is filled with: a switch's track, a badge the person picked. The
/// system's hue, as a ground.
pub const LIT: Palette = Palette::Signal;

/// The rounding every filled box takes.
pub fn corner() -> Corners {
    Corners::all(Rounding::Xs)
}

/// The pace at which anything changes what it wears.
pub fn timing() -> Timing {
    Timing::ms(WEAR_MS).ease(Ease::Decelerate)
}

/// What each step of a chain wears, given which are done: every step done is at rest, the first
/// not done is armed -- the press to make next -- and every step after it is inert, since it
/// cannot be made before the one before it.
///
/// ```
/// use lichen::{Press, gate};
/// let wears: Vec<Press> = gate(&[true, false, false]).collect();
/// assert_eq!(wears, [Press::Rest, Press::Armed, Press::Inert]);
/// ```
pub fn gate(done: &[bool]) -> impl Iterator<Item = Press> + '_ {
    let next = done.iter().position(|done| !done);
    done.iter().enumerate().map(move |(n, _)| match next {
        Some(next) if n == next => Press::Armed,
        Some(next) if n > next => Press::Inert,
        _ => Press::Rest,
    })
}

/// Something that can be put in reach and out of it.
pub trait Reach {
    /// Puts it in reach, or out of it.
    fn reach(&self, grove: &mut Grove, within: bool);
}

/// Puts every one of `parts` in reach, or out of it, together: a choice upstream engaging the
/// parts it opens, or putting them away.
pub fn engage(grove: &mut Grove, parts: &[&dyn Reach], within: bool) {
    for part in parts {
        part.reach(grove, within);
    }
}
