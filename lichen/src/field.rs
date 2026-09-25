//! A field: a name in a cell, a bar, and a value -- the chip's shape, offered for what a form
//! reads rather than what it presses, so a form stands as one set of things with the presses that
//! review it. Two kinds. A [`Field`] holds words, typed or pasted in: a line of them, or more. A
//! [`Pick`] holds one of a fixed few, laid out as badges with the chosen one lit, so a thing that
//! is only ever one of a few is never a line to type.
//!
//! The dressing is [`dress`], and it is shared: anything else an app stands in a row of this
//! shape is dressed by the same line, so the name cells and the bars down a column are one column
//! whatever is beside them.
//!
//! # Provenance, in ink
//!
//! What a field wears says where its value came from. Filled from something that is not the
//! person -- a paste, an import, what a model made of a page -- it is read in the grey between, as
//! a chip with nothing to press for is: there, and plainly not the person's own. It can be edited
//! all the same, and the first edit makes it theirs, read in ink like anything typed from nothing.
//! So a glance down a form says which of it came from elsewhere and which the person said, and a
//! second fill goes by the same reading: what was typed stands, and the fill says the rest. A pick
//! says the same with its badge -- the fill's choice, or the default, is a grey badge; the
//! person's is lit.
//!
//! A field over a record already written reads the same two origins the other way round -- see
//! [`Holds`] -- because there the record is what is true and an edit is what is not written yet.
//!
//! The bar between the name and the value is where focus shows, since the engine draws nothing
//! for it: lit in the accent while the field holds focus, which is the caret's colour on a field
//! with a caret.

use foliage::{
    Boxed, Elevation, Fill, Grove, Grow, Key, Leaf, Location, Motion, Palette, Panel, Place,
    Pollen, Sap, Section, Source, Stem, Text, TextArea, TextInput, Vein, center_x, center_y,
    content, left, top,
};

use crate::chip::caption;
use crate::measure::measure;
use crate::tone::{self, Reach, Tone};

/// Where a field's value came from.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Origin {
    /// Nowhere: nothing is in it, or nothing but the default.
    Empty,
    /// Not the person: a fill's word on a draft, the record's own on a record.
    Imported,
    /// The person: typed, or pressed.
    Typed,
}

/// What a field holds, which is which way round its two origins read.
///
/// A draft's words are provisional, and the person's are the good ones: what a fill said reads in
/// the grey between, and what was typed reads in ink. A record's are the other way round -- the
/// record is what is true, and a word changed here is true of nothing until it is written -- so
/// what the record says reads in ink, and what has been changed reads in the accent, which is also
/// what the press it arms is filled with.
///
/// One mechanism, two readings. Which one a field is is said once, where it is grown.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Holds {
    #[default]
    Draft,
    Record,
}

impl Origin {
    /// What a field with a value from here wears, for what the field holds.
    pub fn tone(self, holds: Holds) -> Tone {
        match (self, holds) {
            (Origin::Imported, Holds::Draft) => tone::INERT,
            (Origin::Typed, Holds::Record) => tone::CHANGED,
            _ => tone::REST,
        }
    }
}

/// A field for words, grown, and where its words came from.
pub struct Field {
    /// What stands where the field was placed, to anchor to.
    field: Leaf,
    /// What holds the words, takes the press, and holds focus.
    input: Leaf,
    label: Leaf,
    bar: Leaf,
    origin: Origin,
    holds: Holds,
}

impl Field {
    /// Grows a field under `under`, standing where `at` says, named `name` in a cell `cell`
    /// letters wide. One line, or as many as its box holds if `tall`. Empty, and holding a draft.
    pub fn grow(
        grove: &mut Grove,
        under: Leaf,
        at: Location,
        cell: f32,
        name: &str,
        tall: bool,
    ) -> Self {
        let m = measure();
        let field = grove.branch(
            under,
            Stem::new()
                .at(at)
                .elevate(Elevation::up(1))
                .font_size(caption())
                .intangible(),
        );
        let (_, label, bar) = dress(grove, field, cell, name);
        let across = left(cell.letters() + (1.0 + m.gap).px()).right(100.pct() - m.gap.px());
        // The value stands in front of the back, and its first line level with the name: a line
        // is centred as the name is, and a tall one starts where the name does and runs to the
        // notch.
        let input = match tall {
            false => grove.branch(
                field,
                TextInput::new()
                    .color(tone::REST.ink)
                    .at(Location::new().xs(across, top(0.px()).bottom(100.pct())))
                    .elevate(Elevation::up(2))
                    .font_size(caption()),
            ),
            true => grove.branch(
                field,
                TextArea::new()
                    .color(tone::REST.ink)
                    .at(Location::new().xs(
                        across,
                        top((m.height / 2.0).px() - 0.5.letters()).bottom(100.pct() - m.notch.px()),
                    ))
                    .elevate(Elevation::up(2))
                    .font_size(caption()),
            ),
        };
        Self {
            field,
            input,
            label,
            bar,
            origin: Origin::Empty,
            holds: Holds::Draft,
        }
    }

    /// Says the field holds a record rather than a draft, which is which way round it reads where
    /// its words came from. Said where it is grown, before anything is put in it.
    pub fn holds(&mut self, holds: Holds) {
        self.holds = holds;
    }

    /// The field itself, to anchor to.
    pub fn leaf(&self) -> Leaf {
        self.field
    }

    /// What holds the words, focus, and the caret.
    pub fn input(&self) -> Leaf {
        self.input
    }

    /// The name in the cell, to be written over with `Grow::text`.
    pub fn label(&self) -> Leaf {
        self.label
    }

    /// Where its words came from.
    pub fn origin(&self) -> Origin {
        self.origin
    }

    /// What it says.
    pub fn value(&self, grove: &Grove) -> String {
        match grove.tap(self.input, Vein::Text) {
            Some(Sap::Text(value)) => value,
            _ => String::new(),
        }
    }

    /// Carries the field for a frame: focus shows on the bar, and an edit makes the words the
    /// person's -- or nothing, if they took the last of them out. Whether the words moved.
    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) -> bool {
        if pollen.focused(self.input) {
            mark(grove, self.bar, true);
        }
        if pollen.unfocused(self.input) {
            mark(grove, self.bar, false);
        }
        if !pollen.edited(self.input) {
            return false;
        }
        let was = self.origin;
        self.origin = match self.value(grove).trim().is_empty() {
            true => Origin::Empty,
            false => Origin::Typed,
        };
        if self.origin != was {
            self.wear(grove);
        }
        true
    }

    /// Puts a word that is not the person's in the field, unless the person's is there: what was
    /// typed stands.
    pub fn fill(&mut self, grove: &mut Grove, text: &str) {
        if self.origin == Origin::Typed {
            return;
        }
        self.put(grove, text);
    }

    /// Puts a word that is not the person's in the field whatever is there -- what a record says,
    /// put back over an edit that was let go.
    pub fn put(&mut self, grove: &mut Grove, text: &str) {
        grove.text(self.input, text);
        self.origin = match text.trim().is_empty() {
            true => Origin::Empty,
            false => Origin::Imported,
        };
        self.wear(grove);
    }

    /// Empties the field.
    pub fn clear(&mut self, grove: &mut Grove) {
        grove.text(self.input, "");
        self.origin = Origin::Empty;
        self.wear(grove);
    }

    /// Puts focus in the field, so the caret says where to type.
    pub fn focus(&self, grove: &mut Grove) {
        grove.focus(self.input);
    }

    /// Dresses the field for where its words came from: the name and the words in the tone's
    /// ink.
    fn wear(&self, grove: &mut Grove) {
        let ink = self.origin.tone(self.holds).ink;
        grove.animate(self.label, Motion::from(ink), tone::timing());
        grove.animate(self.input, Motion::from(ink), tone::timing());
    }
}

impl Reach for Field {
    /// Puts the field in reach, or out of it. What its words are is not touched either way -- a
    /// field passed over is not a field emptied.
    ///
    /// A draft out of reach is read in the grey between, the way an inert chip is: there is
    /// nothing to type into it and nothing in it that is anyone's. A record out of reach still
    /// says what the record says, so it goes on reading as it did -- what changes is that it
    /// cannot be typed into, which is the whole of what being out of reach means there. So it is
    /// made read-only rather than disabled: it still takes a drag, to scroll a value longer than
    /// its box into view, and a hold, to select out of it.
    fn reach(&self, grove: &mut Grove, within: bool) {
        match (within, self.holds) {
            (_, Holds::Record) => {
                grove.enable(self.field);
                grove.read_only(self.input, !within);
            }
            (true, Holds::Draft) => grove.enable(self.field),
            (false, Holds::Draft) => grove.disable(self.field),
        }
        let ink = match (within, self.holds) {
            (false, Holds::Draft) => tone::INERT.ink,
            _ => self.origin.tone(self.holds).ink,
        };
        grove.animate(self.label, Motion::from(ink), tone::timing());
        grove.animate(self.input, Motion::from(ink), tone::timing());
    }
}

/// One badge of a pick: its back, and the name on it.
struct Badge {
    back: Leaf,
    name: Leaf,
}

/// A field for one of a fixed few, grown, and which is chosen.
pub struct Pick {
    /// What stands where the pick was placed, takes the press, and holds focus.
    pick: Leaf,
    label: Leaf,
    bar: Leaf,
    badges: Vec<Badge>,
    /// The few, spelt as they are stored. The first is the default.
    names: &'static [&'static str],
    chosen: usize,
    origin: Origin,
}

impl Pick {
    /// How tall a pick stands with its badges in `rows` rows: the notches, and the rows between.
    pub fn height(rows: usize) -> f32 {
        let m = measure();
        2.0 * m.notch + rows as f32 * m.badge() + (rows as f32 - 1.0) * m.between
    }

    /// Grows a pick under `under`, standing where `at` says, named `name` in a cell `cell`
    /// letters wide, offering `names` as badges laid out in `rows` rows, as many to a row as
    /// divides them evenly. The first is chosen, as the default.
    pub fn grow(
        grove: &mut Grove,
        under: Leaf,
        at: Location,
        cell: f32,
        name: &str,
        names: &'static [&'static str],
        rows: usize,
    ) -> Self {
        let m = measure();
        let pick = grove.branch(
            under,
            Stem::new()
                .at(at)
                .elevate(Elevation::up(1))
                .font_size(caption())
                .interactive(),
        );
        let (back, label, bar) = dress(grove, pick, cell, name);
        // Each badge as wide as its name and a letter's room either side, the way a chip is, and
        // set along its row from the bar.
        let per_row = names.len().div_ceil(rows.max(1));
        let mut badges = Vec::with_capacity(names.len());
        for (row, chunk) in names.chunks(per_row.max(1)).enumerate() {
            let mut along = 0.0;
            for (n, &name) in chunk.iter().enumerate() {
                let letters = name.chars().count() as f32 + 2.0;
                // Dressed as chosen by default for the first, and as the rest for the rest.
                let (fill, ink) = badge_tones(badges.is_empty(), Origin::Empty);
                let back = grove.branch(
                    back,
                    Panel::new()
                        .at(Location::new().xs(
                            left(
                                cell.letters()
                                    + (1.0 + m.gap + n as f32 * m.between).px()
                                    + along.letters(),
                            )
                            .width(letters.letters()),
                            top((m.notch + row as f32 * (m.badge() + m.between)).px())
                                .height(m.badge().px()),
                        ))
                        .elevate(Elevation::up(1))
                        .intangible()
                        .color(fill)
                        .rounding(tone::corner())
                        .font_size(caption()),
                );
                let name = grove.branch(
                    back,
                    Text::new(name)
                        .at(Location::new().xs(
                            center_x(50.pct()).width(content()),
                            center_y(50.pct()).height(1.letters()),
                        ))
                        .elevate(Elevation::up(1))
                        .intangible()
                        .color(ink)
                        .font_size(caption()),
                );
                along += letters;
                badges.push(Badge { back, name });
            }
        }
        Self {
            pick,
            label,
            bar,
            badges,
            names,
            chosen: 0,
            origin: Origin::Empty,
        }
    }

    /// The pick itself, to anchor to.
    pub fn leaf(&self) -> Leaf {
        self.pick
    }

    /// The name, to be written over with `Grow::text` -- a count in the cell, say, where what the
    /// pick offers is an order to read something in and the cell is where the something is
    /// counted.
    pub fn label(&self) -> Leaf {
        self.label
    }

    /// Where its choice came from.
    pub fn origin(&self) -> Origin {
        self.origin
    }

    /// Which is chosen, spelt as it is stored.
    pub fn chosen(&self) -> &'static str {
        self.names[self.chosen]
    }

    /// Which is chosen, by where it stands among the few.
    pub fn index(&self) -> usize {
        self.chosen
    }

    /// Carries the pick for a frame: focus shows on the bar; a press on a badge chooses it, and
    /// the arrow keys step the choice along, either way round. Whether the choice moved.
    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) -> bool {
        if pollen.focused(self.pick) {
            mark(grove, self.bar, true);
        }
        if pollen.unfocused(self.pick) {
            mark(grove, self.bar, false);
        }
        let count = self.badges.len();
        let mut to = None;
        if let Some(at) = pollen.clicked_at(self.pick) {
            to = self
                .badges
                .iter()
                .position(|badge| drawn(grove, badge.back).is_some_and(|back| back.contains(at)));
        }
        for stroke in pollen.keys(self.pick) {
            let from = to.unwrap_or(self.chosen);
            to = match stroke.key {
                Key::Left | Key::Up => Some((from + count - 1) % count),
                Key::Right | Key::Down => Some((from + 1) % count),
                _ => to,
            };
        }
        let Some(to) = to else {
            return false;
        };
        self.choose(grove, to, Origin::Typed);
        true
    }

    /// Chooses what a fill's word names, unless the person chose already: what was pressed
    /// stands. A word naming none of the few is the default, from nowhere.
    pub fn fill(&mut self, grove: &mut Grove, text: &str) {
        if self.origin == Origin::Typed {
            return;
        }
        match self
            .names
            .iter()
            .position(|name| name.eq_ignore_ascii_case(text.trim()))
        {
            Some(n) => self.choose(grove, n, Origin::Imported),
            None => self.choose(grove, 0, Origin::Empty),
        }
    }

    /// Chooses badge `n`, as a choice from `origin`, and dresses the pick for it.
    pub fn set(&mut self, grove: &mut Grove, n: usize, origin: Origin) {
        self.choose(grove, n.min(self.badges.len().saturating_sub(1)), origin);
    }

    /// Puts the pick back to its default.
    pub fn clear(&mut self, grove: &mut Grove) {
        self.choose(grove, 0, Origin::Empty);
    }

    /// Chooses badge `n`, as a choice from `origin`, and dresses the pick for it: the name in the
    /// tone's ink, the chosen badge lit or grey for whose choice it is, and the rest wells.
    fn choose(&mut self, grove: &mut Grove, n: usize, origin: Origin) {
        self.chosen = n;
        self.origin = origin;
        let timing = tone::timing();
        grove.animate(
            self.label,
            Motion::from(origin.tone(Holds::Draft).ink),
            timing,
        );
        for (k, badge) in self.badges.iter().enumerate() {
            let (fill, ink) = badge_tones(k == n, origin);
            grove.animate(badge.back, Motion::from(fill), timing);
            grove.animate(badge.name, Motion::from(ink), timing);
        }
    }
}

impl Reach for Pick {
    /// Puts the pick in reach, or out of it: what changes is whether it takes a press. It goes on
    /// showing which of the few is chosen either way, because that is what it is saying and not
    /// something it is offering.
    fn reach(&self, grove: &mut Grove, within: bool) {
        match within {
            true => grove.enable(self.pick),
            false => grove.disable(self.pick),
        }
    }
}

/// What a badge is filled with and its name read in: chosen by the person, it is lit, as a
/// switch that is on is; chosen by a fill or by default, the grey between; and not chosen, a
/// well of the ground's colour with its name in the grey between, so it reads as a place to
/// press and nothing more.
fn badge_tones(chosen: bool, origin: Origin) -> (Fill, Fill) {
    match (chosen, origin) {
        (true, Origin::Typed) => (Fill::Role(tone::LIT), tone::REST.ink),
        (true, _) => (Fill::Role(tone::BAR), tone::REST.ink),
        (false, _) => (tone::WELL.fill, tone::INERT.ink),
    }
}

/// Dresses `field` in the chip's shape: the back, the name centred in its cell and level with a
/// chip's, and the bar. The three, to hang more on and to change.
pub fn dress(grove: &mut Grove, field: Leaf, cell: f32, name: &str) -> (Leaf, Leaf, Leaf) {
    let m = measure();
    let back = grove.branch(
        field,
        Panel::new()
            .at(Location::new())
            .elevate(Elevation::up(1))
            .intangible()
            .color(tone::REST.fill)
            .rounding(tone::corner()),
    );
    let label = grove.branch(
        back,
        Text::new(name)
            .at(Location::new().xs(
                center_x((cell / 2.0).letters()).width(content()),
                top((m.height / 2.0).px() - 0.5.letters()).height(1.letters()),
            ))
            .elevate(Elevation::up(1))
            .intangible()
            .color(tone::REST.ink)
            .font_size(caption()),
    );
    let bar = grove.branch(
        back,
        Panel::new()
            .at(Location::new().xs(
                left(cell.letters()).width(1.px()),
                top(m.notch.px()).bottom(100.pct() - m.notch.px()),
            ))
            .elevate(Elevation::up(1))
            .intangible()
            .color(tone::BAR)
            .font_size(caption()),
    );
    (back, label, bar)
}

/// Lights a bar for focus, or puts it back.
pub fn mark(grove: &mut Grove, bar: Leaf, focused: bool) {
    let fill = match focused {
        true => Palette::Accent.mark(),
        false => tone::BAR,
    };
    grove.animate(bar, Motion::Palette(fill), tone::timing());
}

/// Where `leaf` was drawn, if it was. What a row of things to press is hit-tested against, where
/// an app lays out its own row and reads which of it a press landed on.
pub fn drawn(grove: &Grove, leaf: Leaf) -> Option<Section> {
    match grove.tap(leaf, Vein::Drawn) {
        Some(Sap::Section(section)) => Some(section),
        _ => None,
    }
}
