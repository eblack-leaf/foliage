//! A confirm: the press that cannot be taken back, made as two.
//!
//! The first press arms it: abort stands exactly where that press was made, and the press that
//! does it stands at the other end of the room, where nothing was a moment ago. A second press in
//! the same place is therefore the abort, and the only press that goes through is one the person
//! had to move to make.
//!
//! The ask and the commit wear the danger hue while the ask can be made, and the ask is inert
//! while it cannot; abort is at rest. Anything that moves the page under an arming should put it
//! back -- [`disarm`](Confirm::disarm) -- so an arming never outlives the moment it was made in.

use foliage::{Elevation, Field, Grove, Grow, Leaf, Location, Pollen};

use crate::chip::Chip;
use crate::tone::{Press, Reach};

/// What a press on a confirm came to, the frame it was made.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Step {
    /// The first press: armed, and abort standing where it was made.
    Armed,
    /// Abort: put back, and nothing done.
    Kept,
    /// The second press, at the other end: do it.
    Confirmed,
}

/// A confirm, grown, and whether it is armed.
pub struct Confirm {
    ask: Chip,
    abort: Chip,
    commit: Chip,
    /// Whether the ask can be made at all.
    open: bool,
    armed: bool,
}

impl Confirm {
    /// Grows a confirm under `under`: the ask at `here`, abort in exactly its place, and the
    /// commit at `there`. Each is a mark and a name. Closed -- the ask inert -- and unarmed.
    ///
    /// Placed by where they stand and anchored by the caller -- the ask and abort to the same
    /// thing, since they are the same place -- through [`ask`](Self::ask),
    /// [`abort`](Self::abort) and [`commit`](Self::commit).
    pub fn grow(
        grove: &mut Grove,
        under: Leaf,
        here: Location,
        there: Location,
        ask: (Field, &str),
        abort: (Field, &str),
        commit: (Field, &str),
    ) -> Self {
        let mut ask = Chip::grow(grove, under, here.clone(), Elevation::up(1), ask.0, ask.1);
        let mut abort = Chip::grow(grove, under, here, Elevation::up(2), abort.0, abort.1);
        let mut commit = Chip::grow(grove, under, there, Elevation::up(2), commit.0, commit.1);
        ask.arm(grove, Press::Inert);
        abort.arm(grove, Press::Rest);
        commit.arm(grove, Press::Danger);
        let mut confirm = Self {
            ask,
            abort,
            commit,
            open: false,
            armed: true,
        };
        confirm.disarm(grove);
        confirm
    }

    /// The first press, which stands alone while unarmed.
    pub fn ask(&self) -> &Chip {
        &self.ask
    }

    /// The way out, standing where the ask was while armed.
    pub fn abort(&self) -> &Chip {
        &self.abort
    }

    /// The press that goes through, standing at the other end while armed.
    pub fn commit(&self) -> &Chip {
        &self.commit
    }

    /// Whether it is armed.
    pub fn armed(&self) -> bool {
        self.armed
    }

    /// Opens the ask to a press, dressed in the danger hue, or closes it, inert. Closing it
    /// disarms it.
    pub fn open(&mut self, grove: &mut Grove, open: bool) {
        self.open = open;
        self.ask.arm(
            grove,
            match open {
                true => Press::Danger,
                false => Press::Inert,
            },
        );
        if !open {
            self.disarm(grove);
        }
    }

    /// Puts it back: the ask stands alone, and abort and the commit are gone.
    pub fn disarm(&mut self, grove: &mut Grove) {
        if self.armed {
            self.stand(grove, false);
        }
    }

    /// Carries it for a frame: what a press on it came to, if one was made.
    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) -> Option<Step> {
        if !self.armed && self.open && self.ask.pressed(pollen) {
            self.stand(grove, true);
            return Some(Step::Armed);
        }
        if self.armed && self.abort.pressed(pollen) {
            self.stand(grove, false);
            return Some(Step::Kept);
        }
        // Asked of the state as well as of the chip, which is out of reach and out of sight
        // unarmed: the one press that cannot be taken back is worth saying twice.
        if self.armed && self.commit.pressed(pollen) {
            self.stand(grove, false);
            return Some(Step::Confirmed);
        }
        None
    }

    fn stand(&mut self, grove: &mut Grove, armed: bool) {
        self.armed = armed;
        for (chip, stands) in [(&self.abort, armed), (&self.commit, armed)] {
            grove.visible(chip.leaf(), stands);
            chip.reach(grove, stands);
        }
        grove.visible(self.ask.leaf(), !armed);
        self.ask.reach(grove, !armed && self.open);
    }
}
