//! The intake section: a new entry, typed or generated.
//!
//! The controls are what kind the entry is, and what the generator makes: how long, and whether
//! with symbols. What they open is the form: title, username, url and folder; the password,
//! masked, with `new` at its end to generate one; the OTP seed, masked; and the notes. Under them,
//! `show` to see the two secrets, `clear`, and `add`, which is the press to make once there is a
//! title. What was typed is the person's, and a generated password is the generator's until it is
//! typed over, which the fields say by how they read.

use foliage::{
    Boxed, Elevation, Font, Grove, Grow, Leaf, Location, Place, Pollen, Source, Stem, anchor, left,
    right, top,
};
use lichen::{Chip, Field, Origin, Pick, Press, Say, Switch, Voice, measure, words};

use super::vault::{Entry, KINDS, Vault};
use crate::icons::Icons;

/// The room between two things that are not one thing.
const GAP: f32 = 8.0;
/// The cell the controls' names are set in, in letters.
const CONTROL_CELL: f32 = 7.0;
/// The cell the form's names are set in, in letters.
const CELL: f32 = 10.0;
/// How many rows the notes are given.
const NOTES_ROWS: usize = 3;
/// How many letters the line under the form shows.
const TOLD: usize = 40;

/// The lengths the generator is offered at, and the one it starts on.
const LENGTHS: [&str; 3] = ["16", "24", "32"];
const LENGTH: usize = 1;

/// The intake section, grown.
pub(crate) struct Intake {
    kind: Pick,
    length: Pick,
    symbols: Switch,
    title: Field,
    username: Field,
    url: Field,
    folder: Field,
    password: Field,
    otp: Field,
    notes: Field,
    new: Chip,
    show: Switch,
    clear: Chip,
    add: Chip,
    told: Say,
}

impl Intake {
    /// Grows the controls on `sheet` and the form on `details`.
    pub(crate) fn grow(
        grove: &mut Grove,
        sheet: Leaf,
        details: Leaf,
        icons: &Icons,
        italic: Font,
    ) -> Self {
        let height = measure().height;
        let pick = Pick::height(1);
        // The stack, centred down the sheet: two picks and the switch.
        let stack = 2.0 * pick + height + 2.0 * GAP;
        let kind = Pick::grow(
            grove,
            sheet,
            Location::new().xs(
                left(0.px()).right(100.pct()),
                top(50.pct() - (stack / 2.0).px()).height(pick.px()),
            ),
            CONTROL_CELL,
            "kind",
            &KINDS,
            1,
        );
        let mut length = Pick::grow(
            grove,
            sheet,
            Location::new().xs(
                left(0.px()).right(100.pct()),
                top(anchor().bottom() + GAP.px()).height(pick.px()),
            ),
            CONTROL_CELL,
            "length",
            &LENGTHS,
            1,
        );
        grove.anchor(length.leaf(), kind.leaf());
        length.set(grove, LENGTH, Origin::Empty);
        let mut symbols = Switch::grow(
            grove,
            sheet,
            Location::new().xs(
                left(0.px()).width(Switch::width("symbols")),
                top(anchor().bottom() + GAP.px()).height(height.px()),
            ),
            Elevation::up(1),
            "symbols",
        );
        grove.anchor(symbols.leaf(), length.leaf());
        symbols.set(grove, true);

        let line = row(grove, details, 0, 1);
        let title = plain(grove, line, "title");
        let line = row(grove, details, 1, 1);
        let username = plain(grove, line, "username");
        let line = row(grove, details, 2, 1);
        let url = plain(grove, line, "url");
        let line = row(grove, details, 3, 1);
        let folder = plain(grove, line, "folder");
        let line = row(grove, details, 4, 1);
        let mut new = Chip::grow(
            grove,
            line,
            Location::new().xs(
                right(100.pct()).width(Chip::width("new")),
                top(0.px()).bottom(100.pct()),
            ),
            Elevation::up(1),
            icons.zap,
            "new",
        );
        new.arm(grove, Press::Rest);
        let password = masked(
            grove,
            line,
            Location::new().xs(
                left(0.px()).right(anchor().left() - GAP.px()),
                top(0.px()).bottom(100.pct()),
            ),
            "password",
        );
        grove.anchor(password.leaf(), new.leaf());
        let line = row(grove, details, 5, 1);
        let otp = masked(grove, line, Location::new(), "otp");
        let line = row(grove, details, 6, NOTES_ROWS);
        let notes = Field::grow(grove, line, Location::new(), CELL, "notes", true);

        // The foot: show at the left, clear and add at the right, add outermost; and under it,
        // what the form says.
        let line = row(grove, details, 6 + NOTES_ROWS, 1);
        let show = Switch::grow(
            grove,
            line,
            Location::new().xs(
                left(0.px()).width(Switch::width("show")),
                top(0.px()).bottom(100.pct()),
            ),
            Elevation::up(1),
            "show",
        );
        let mut add = Chip::grow(
            grove,
            line,
            Location::new().xs(
                right(100.pct()).width(Chip::width("add")),
                top(0.px()).bottom(100.pct()),
            ),
            Elevation::up(1),
            icons.plus,
            "add",
        );
        add.arm(grove, Press::Inert);
        let mut clear = Chip::grow(
            grove,
            line,
            Location::new().xs(
                right(anchor().left() - GAP.px()).width(Chip::width("clear")),
                top(0.px()).bottom(100.pct()),
            ),
            Elevation::up(1),
            icons.x,
            "clear",
        );
        grove.anchor(clear.leaf(), add.leaf());
        clear.arm(grove, Press::Rest);
        let line = row(grove, details, 7 + NOTES_ROWS, 1);
        let told = Say::grow(
            grove,
            line,
            Location::new().xs(
                left(0.px()).right(100.pct()),
                top(0.px()).height(1.letters()),
            ),
            Some(italic),
        );
        Self {
            kind,
            length,
            symbols,
            title,
            username,
            url,
            folder,
            password,
            otp,
            notes,
            new,
            show,
            clear,
            add,
            told,
        }
    }

    /// Carries the section for a frame. Whether an entry was added.
    pub(crate) fn frame(&mut self, grove: &mut Grove, pollen: &Pollen, vault: &mut Vault) -> bool {
        self.kind.frame(grove, pollen);
        self.length.frame(grove, pollen);
        self.symbols.frame(grove, pollen);
        let mut moved = false;
        for field in self.form() {
            moved |= field.frame(grove, pollen);
        }
        if moved {
            self.told.tell(grove, "", Voice::Quiet);
            self.ready(grove);
        }
        if self.show.frame(grove, pollen) {
            let masked = !self.show.on();
            for field in [&self.password, &self.otp] {
                grove.masked(field.input(), masked);
            }
        }
        if self.new.pressed(pollen) {
            let length = LENGTHS[self.length.index()].parse().unwrap_or(24);
            let made = vault.generate(length, self.symbols.on());
            self.password.fill(grove, &made);
            self.ready(grove);
        }
        if self.clear.pressed(pollen) {
            self.empty(grove);
        }
        if self.add.pressed(pollen) {
            let title = self.title.value(grove).trim().to_string();
            if title.is_empty() {
                return false;
            }
            vault.add(Entry {
                kind: self.kind.chosen().to_string(),
                title: title.clone(),
                username: self.username.value(grove),
                url: self.url.value(grove),
                folder: self.folder.value(grove),
                password: self.password.value(grove),
                otp: self.otp.value(grove),
                notes: self.notes.value(grove),
                days: 0,
            });
            self.empty(grove);
            self.told.tell(
                grove,
                &words::cut(&format!("added: {title}"), TOLD),
                Voice::Hint,
            );
            return true;
        }
        false
    }

    /// The vault was locked: nothing typed stays.
    pub(crate) fn lock(&mut self, grove: &mut Grove) {
        self.empty(grove);
        self.told.clear(grove);
    }

    /// Every field of the form, in reading order.
    fn form(&mut self) -> [&mut Field; 7] {
        [
            &mut self.title,
            &mut self.username,
            &mut self.url,
            &mut self.folder,
            &mut self.password,
            &mut self.otp,
            &mut self.notes,
        ]
    }

    /// Empties the form, and masks it again.
    fn empty(&mut self, grove: &mut Grove) {
        for field in self.form() {
            field.clear(grove);
        }
        if self.show.on() {
            self.show.set(grove, false);
        }
        for field in [&self.password, &self.otp] {
            grove.masked(field.input(), true);
        }
        // Not `ready`: a field cleared this frame still reads what it held until the clear lands,
        // and an emptied form has no title.
        self.add.arm(grove, Press::Inert);
    }

    /// Add is the press to make once there is a title.
    fn ready(&mut self, grove: &mut Grove) {
        let titled = !self.title.value(grove).trim().is_empty();
        self.add.arm(grove, Press::armed_if(titled));
    }
}

/// How far down the form row `n` stands.
fn down(n: usize) -> foliage::Length {
    (n as f32 * (measure().height + GAP)).px()
}

/// Row `n` of the form, `rows` rows tall.
fn row(grove: &mut Grove, room: Leaf, n: usize, rows: usize) -> Leaf {
    let height = rows as f32 * measure().height + (rows as f32 - 1.0) * GAP;
    grove.branch(
        room,
        Stem::new()
            .at(Location::new().xs(
                left(0.px()).right(100.pct()),
                top(down(n)).height(height.px()),
            ))
            .elevate(Elevation::up(1))
            .font_size(lichen::caption())
            .intangible()
            .focus_scope(),
    )
}

/// A field filling its row.
fn plain(grove: &mut Grove, line: Leaf, name: &str) -> Field {
    Field::grow(grove, line, Location::new(), CELL, name, false)
}

/// A masked field, `at` in its row.
fn masked(grove: &mut Grove, line: Leaf, at: Location, name: &str) -> Field {
    let field = Field::grow(grove, line, at, CELL, name, false);
    grove.masked(field.input(), true);
    field
}
