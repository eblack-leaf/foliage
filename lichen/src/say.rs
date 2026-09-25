//! A say: the line that answers a press.
//!
//! The middle one of the three ways something asks to be noticed. A [`Ping`](crate::Ping) goes by
//! itself and a [`Badge`](crate::Badge) stays until cleared; a say holds until the next change,
//! which takes it away -- what a confirm found wanting, that a thing was queued, that a record was
//! kept. So what it says is always about the last thing done, and never outlives it.
//!
//! One line, set quietly beside the presses it answers.

use foliage::{
    Boxed, Elevation, Fill, Font, Grove, Grow, Leaf, Location, Motion, Palette, Place, Text,
};

use crate::chip::caption;
use crate::tone;

/// How a say is set.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Voice {
    /// In the grey between: what happened, said and let be.
    #[default]
    Quiet,
    /// In the system's hue: what went right and is worth a glance.
    Hint,
    /// In the caution hue: what wants putting right.
    Warn,
}

impl Voice {
    fn ink(self) -> Fill {
        match self {
            Voice::Quiet => tone::INERT.ink,
            Voice::Hint => Fill::Role(tone::HINT),
            Voice::Warn => Fill::Role(Palette::Caution.mark()),
        }
    }
}

/// A say, grown, and what it last said.
pub struct Say {
    leaf: Leaf,
    said: String,
    voice: Voice,
}

impl Say {
    /// Grows a say under `under`, standing where `at` says, set in `font` if one is given --
    /// an italic, to set it apart from what it stands beside. Saying nothing.
    pub fn grow(grove: &mut Grove, under: Leaf, at: Location, font: Option<Font>) -> Self {
        let text = Text::new("")
            .at(at)
            .elevate(Elevation::up(1))
            .intangible()
            .color(Voice::Quiet.ink())
            .font_size(caption());
        let text = match font {
            Some(font) => text.font(font),
            None => text,
        };
        let leaf = grove.branch(under, text);
        Self {
            leaf,
            said: String::new(),
            voice: Voice::Quiet,
        }
    }

    /// The line, to anchor to or to hide.
    pub fn leaf(&self) -> Leaf {
        self.leaf
    }

    /// What it last said.
    pub fn said(&self) -> &str {
        &self.said
    }

    /// Says `words`, quietly.
    pub fn set(&mut self, grove: &mut Grove, words: &str) {
        self.tell(grove, words, Voice::Quiet);
    }

    /// Says `words` in `voice`.
    pub fn tell(&mut self, grove: &mut Grove, words: &str, voice: Voice) {
        if voice != self.voice {
            self.voice = voice;
            grove.animate(self.leaf, Motion::from(voice.ink()), tone::timing());
        }
        if words != self.said {
            self.said = words.to_string();
            grove.text(self.leaf, words);
        }
    }

    /// Says nothing: the next change has come.
    pub fn clear(&mut self, grove: &mut Grove) {
        self.set(grove, "");
    }
}
