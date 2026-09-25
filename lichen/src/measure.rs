//! How large the look is drawn: one knob, and everything else follows from it.
//!
//! Every part lichen grows is sized from a [`Measure`], and a measure is chosen by a [`Density`]
//! rather than stated, so an app says how dense it wants to be and cannot say anything else about
//! size. Two apps at the same density are the same size throughout; that is the point of there
//! being a knob and not a set of them.
//!
//! The density is chosen once, before anything is grown -- see [`density`] -- and read by every
//! part as it is grown. Choosing it again changes what is grown after, and nothing already grown.

use std::sync::atomic::{AtomicU8, Ordering};

/// How dense the look is drawn.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Density {
    /// The size the look was drawn at: a pointer, and room to spare.
    #[default]
    Regular,
    /// Tighter, for a small screen or a crowded one. Everything a step smaller, and the text a
    /// size down.
    Compact,
}

/// Every size the look is drawn at, at one density.
///
/// Pixels, except [`caption`](Measure::caption), which is a font size.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Measure {
    /// How tall a chip is, and every row of the chip's shape: a field, a pick's bar, a switch.
    pub height: f32,
    /// How far a bar stops short of the edges of what it divides.
    pub notch: f32,
    /// How large a mark in a chip's cell is drawn.
    pub glyph: f32,
    /// The one text size the parts are set at.
    pub caption: u32,
    /// The room between two things side by side, or one above the other: a field's bar and its
    /// value, two chips in a row, two rows.
    pub gap: f32,
    /// How far something stands in from the edges of what holds it.
    pub inset: f32,
    /// The room between two badges of a pick, and between two rows of them.
    pub between: f32,
    /// How large a ping's mark is drawn.
    pub ping: f32,
    /// How large the dot a badge is drawn as.
    pub dot: f32,
    /// A switch's track, wide and tall; the knob on it, square; and how far in from the track's
    /// ends the knob stands.
    pub track: (f32, f32),
    pub knob: f32,
    pub knob_in: f32,
}

impl Measure {
    /// How tall a badge of a pick is: a bar's height, so a row of badges stands level with the
    /// bar beside it.
    pub fn badge(&self) -> f32 {
        self.height - 2.0 * self.notch
    }

    /// The measure `density` is drawn at.
    pub const fn at(density: Density) -> Self {
        match density {
            Density::Regular => Self {
                height: 34.0,
                notch: 7.0,
                glyph: 18.0,
                caption: 13,
                gap: 8.0,
                inset: 12.0,
                between: 6.0,
                ping: 12.0,
                dot: 8.0,
                track: (34.0, 16.0),
                knob: 12.0,
                knob_in: 2.0,
            },
            Density::Compact => Self {
                height: 28.0,
                notch: 6.0,
                glyph: 15.0,
                caption: 12,
                gap: 6.0,
                inset: 9.0,
                between: 5.0,
                ping: 10.0,
                dot: 7.0,
                track: (28.0, 14.0),
                knob: 10.0,
                knob_in: 2.0,
            },
        }
    }
}

/// The density chosen, as the one byte it takes. Regular until said otherwise.
static DENSITY: AtomicU8 = AtomicU8::new(0);

/// Chooses how dense the look is drawn. Once, in `take_root`, before anything is grown.
pub fn density(density: Density) {
    let byte = match density {
        Density::Regular => 0,
        Density::Compact => 1,
    };
    DENSITY.store(byte, Ordering::Relaxed);
}

/// The measure everything is grown at: the one the chosen [`Density`] is drawn at.
pub fn measure() -> Measure {
    Measure::at(match DENSITY.load(Ordering::Relaxed) {
        1 => Density::Compact,
        _ => Density::Regular,
    })
}
