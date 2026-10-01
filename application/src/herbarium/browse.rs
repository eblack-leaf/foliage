//! The browse section: every entry, and whichever one is open under the list.
//!
//! The controls are a field that narrows the list by its words -- all of them, in the title,
//! username, url or folder, any case -- and how many that leaves. What they open is the list at
//! the top, five rows of it at a time and a press on a row to open it, and under it the pane of
//! whichever is open. What they open scrolls, past the pane's foot. When what was open goes --
//! narrowed out, or deleted -- the first of the list stands open in its place.
//!
//! # The pane
//!
//! It reads down: the entry's title, username, url and folder; its password and its OTP seed,
//! each with copy at its end; its notes; a line for what kind it is and how old its password is;
//! show and edit; and at the foot, delete, tucked behind its mark. At rest the two secrets are
//! masked stand-ins, so which the entry has reads at a glance; `show` puts them on the screen.
//! Copy puts one on the clipboard without showing it. `edit` puts every field in reach, what is
//! changed reads in the hue, and save is the press to make while anything differs; letting go of
//! edit puts back what the entry says.

use foliage::{
    Axes, Boxed, Elevation, Font, Grove, Grow, Leaf, Location, Motion, Palette, Panel, Place,
    Pollen, Source, Stem, Text, anchor, bottom, center_y, left, right, top,
};
use lichen::{Chip, Confirm, Field, Holds, Press, Reach, Say, Step, Switch, Voice, measure, words};

use super::CLEARS;
use super::vault::{Entry, Secret, Vault};
use crate::icons::Icons;

/// The room between two things that are not one thing.
const GAP: f32 = 8.0;
/// The cell the find field's name is set in, in letters.
const FIND_CELL: f32 = 6.0;
/// The cell every name in the pane is set in, in letters.
const CELL: f32 = 10.0;
/// The room between one row of the list and the next.
const APART: f32 = 6.0;
/// How many rows of the list show at once.
const LIST: usize = 5;
/// The bar down a row's left edge, lit on the one that is open.
const EDGE: f32 = 2.0;
/// The username column, in letters.
const USER_COLUMN: f32 = 16.0;
/// How many rows the notes are given.
const NOTES_ROWS: usize = 2;
/// How many rows of the pane's fields and presses stand above its foot.
const ROWS: usize = 8 + NOTES_ROWS;

/// What a masked secret holds while it is not shown.
const HIDDEN: &str = "********";

/// What the section did this frame, for the root to answer.
#[derive(Default)]
pub(crate) struct Did {
    /// The vault changed: everything drawn from it is drawn again.
    pub(crate) changed: bool,
    /// A secret to put on the clipboard.
    pub(crate) copied: Option<String>,
}

/// The browse section, grown.
pub(crate) struct Browse {
    find: Field,
    count: Leaf,
    list: Leaf,
    rows: Vec<Line>,
    quiet: Leaf,
    italic: Font,
    /// The entries the find leaves standing, by key, in list order.
    shown: Vec<usize>,
    /// Which is open.
    chosen: Option<usize>,
    pane: Pane,
}

/// One row of the list.
struct Line {
    row: Leaf,
    back: Leaf,
    edge: Leaf,
    title: Leaf,
    user: Leaf,
}

/// The open entry.
struct Pane {
    title: Field,
    username: Field,
    url: Field,
    folder: Field,
    password: Field,
    otp: Field,
    notes: Field,
    copy_password: Chip,
    copy_otp: Chip,
    about: Leaf,
    show: Switch,
    edit: Switch,
    save: Chip,
    erase: Confirm,
    says: Say,
    /// Which entry it is showing, and what it said when it was opened or last saved.
    showing: Option<(usize, Entry)>,
}

impl Browse {
    /// Grows the controls on `sheet` and the list and the pane on `details`.
    pub(crate) fn grow(
        grove: &mut Grove,
        sheet: Leaf,
        details: Leaf,
        icons: &Icons,
        italic: Font,
    ) -> Self {
        let height = measure().height;
        // The pair, centred down the sheet: the field, and the count under it.
        let find = Field::grow(
            grove,
            sheet,
            Location::new().xs(
                left(0.px()).right(100.pct()),
                top(50.pct() - (((height + GAP) / 2.0).px() + 0.5.letters())).height(height.px()),
            ),
            FIND_CELL,
            "find",
            false,
        );
        let count = grove.branch(
            sheet,
            Text::new("")
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(anchor().bottom() + GAP.px()).height(1.letters()),
                ))
                .anchored(find.leaf())
                .elevate(Elevation::up(1))
                .intangible()
                .color(lichen::INERT.ink)
                .font(italic)
                .font_size(lichen::caption()),
        );
        let list = grove.branch(
            details,
            Stem::new()
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(0.px()).height(list_height().px()),
                ))
                .elevate(Elevation::up(1))
                .font_size(lichen::caption())
                .scrolls(Axes::Vertical)
                .focus_scope(),
        );
        let quiet = grove.branch(
            list,
            Text::new("")
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(0.px()).height(1.letters()),
                ))
                .elevate(Elevation::up(2))
                .intangible()
                .color(lichen::INERT.ink)
                .font(italic)
                .font_size(lichen::caption()),
        );
        let side = grove.branch(
            details,
            Stem::new()
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top((list_height() + GAP).px()).height(Pane::height().px()),
                ))
                .elevate(Elevation::up(1))
                .font_size(lichen::caption())
                .intangible()
                .focus_scope(),
        );
        let pane = Pane::grow(grove, side, icons, italic);
        Self {
            find,
            count,
            list,
            rows: Vec::new(),
            quiet,
            italic,
            shown: Vec::new(),
            chosen: None,
            pane,
        }
    }

    /// Carries the section for a frame: the find narrows the list, a press on a row opens it, and
    /// the pane takes its own presses.
    pub(crate) fn frame(&mut self, grove: &mut Grove, pollen: &Pollen, vault: &mut Vault) -> Did {
        if self.find.frame(grove, pollen) {
            self.redraw(grove, vault);
        }
        let pressed = self.rows.iter().position(|line| pollen.activated(line.row));
        if let Some(key) = pressed.and_then(|n| self.shown.get(n).copied()) {
            self.open(grove, vault, key);
        }
        let did = self.pane.frame(grove, pollen, vault);
        if did.changed {
            self.redraw(grove, vault);
        }
        did
    }

    /// Everything the section draws from the vault: the count, the list as the find leaves it,
    /// and the pane of whichever is open -- the first of the list when what was open has gone.
    pub(crate) fn redraw(&mut self, grove: &mut Grove, vault: &Vault) {
        let find = self.find.value(grove);
        self.shown = vault.find(&find);
        let count = match (find.trim().is_empty(), vault.len()) {
            (true, 1) => "1 entry".to_string(),
            (true, all) => format!("{all} entries"),
            (false, all) => format!("{} of {all}", self.shown.len()),
        };
        grove.text(self.count, count);
        let quiet = match vault.len() {
            0 => "no entries",
            _ => "nothing matches",
        };
        grove.text(self.quiet, quiet);
        grove.opacity(self.quiet, self.shown.is_empty() as u8 as f32);
        while self.rows.len() < self.shown.len() {
            let line = Line::grow(grove, self.list, self.rows.len(), self.italic);
            self.rows.push(line);
        }
        let open = self
            .chosen
            .filter(|chosen| self.shown.contains(chosen))
            .or_else(|| self.shown.first().copied());
        self.chosen = open;
        for (n, line) in self.rows.iter().enumerate() {
            match self
                .shown
                .get(n)
                .and_then(|&key| vault.entry(key).map(|entry| (key, entry)))
            {
                Some((key, entry)) => line.show(grove, entry, Some(key) == open),
                None => line.hide(grove),
            }
        }
        self.pane
            .show(grove, open.and_then(|key| Some((key, vault.entry(key)?))));
    }

    /// Opens `key`: the list says which is open, and the pane fills with it.
    fn open(&mut self, grove: &mut Grove, vault: &Vault, key: usize) {
        if Some(key) == self.chosen {
            return;
        }
        self.chosen = Some(key);
        for (n, line) in self.rows.iter().enumerate() {
            line.mark(grove, self.shown.get(n) == Some(&key));
        }
        self.pane
            .show(grove, vault.entry(key).map(|entry| (key, entry)));
    }

    /// The vault was locked: nothing opened stays on the screen.
    pub(crate) fn lock(&mut self, grove: &mut Grove, vault: &Vault) {
        self.find.clear(grove);
        self.pane.showing = None;
        self.redraw(grove, vault);
    }
}

impl Line {
    /// Grows row `n` of the list: its place is its number, so a row is grown once and never
    /// moved.
    fn grow(grove: &mut Grove, list: Leaf, n: usize, italic: Font) -> Self {
        let row = grove.branch(
            list,
            Stem::new()
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top((n as f32 * (measure().height + APART)).px()).height(measure().height.px()),
                ))
                .elevate(Elevation::up(1))
                .font_size(lichen::caption())
                .interactive(),
        );
        let back = grove.branch(
            row,
            Panel::new()
                .at(Location::new())
                .elevate(Elevation::up(1))
                .intangible()
                .color(lichen::WELL.fill)
                .rounding(lichen::corner()),
        );
        let edge = grove.branch(
            back,
            Panel::new()
                .at(Location::new().xs(
                    left(0.px()).width(EDGE.px()),
                    center_y(50.pct()).height(measure().badge().px()),
                ))
                .elevate(Elevation::up(1))
                .intangible()
                .opacity(0.0)
                .color(Palette::Accent.mark()),
        );
        let user = grove.branch(
            row,
            Text::new("")
                .at(Location::new().xs(
                    right(100.pct() - GAP.px()).width(USER_COLUMN.letters()),
                    center_y(50.pct()).height(1.letters()),
                ))
                .elevate(Elevation::up(2))
                .intangible()
                .color(lichen::INERT.ink)
                .font(italic)
                .font_size(lichen::caption()),
        );
        let title = grove.branch(
            row,
            Text::new("")
                .at(Location::new().xs(
                    left((EDGE + GAP).px()).right(anchor().left() - GAP.px()),
                    center_y(50.pct()).height(1.letters()),
                ))
                .anchored(user)
                .elevate(Elevation::up(2))
                .intangible()
                .color(lichen::REST.ink)
                .font_size(lichen::caption()),
        );
        Self {
            row,
            back,
            edge,
            title,
            user,
        }
    }

    fn show(&self, grove: &mut Grove, entry: &Entry, open: bool) {
        grove.visible(self.row, true);
        grove.text(self.title, entry.title.clone());
        grove.text(self.user, words::cut(&entry.username, USER_COLUMN as usize));
        self.mark(grove, open);
    }

    /// Dresses the line for whether it is the one open.
    fn mark(&self, grove: &mut Grove, open: bool) {
        let fill = match open {
            true => lichen::REST.fill,
            false => lichen::WELL.fill,
        };
        grove.animate(self.back, Motion::from(fill), lichen::timing());
        grove.animate(
            self.edge,
            Motion::Opacity(open as u8 as f32),
            lichen::timing(),
        );
    }

    fn hide(&self, grove: &mut Grove) {
        grove.visible(self.row, false);
    }
}

impl Pane {
    /// How tall the pane is: every row, and the foot under them.
    fn height() -> f32 {
        let m = measure().height;
        ROWS as f32 * (m + GAP) + m
    }

    fn grow(grove: &mut Grove, pane: Leaf, icons: &Icons, italic: Font) -> Self {
        let line = row(grove, pane, 0, 1);
        let title = plain(grove, line, "title");
        let line = row(grove, pane, 1, 1);
        let username = plain(grove, line, "username");
        let line = row(grove, pane, 2, 1);
        let url = plain(grove, line, "url");
        let line = row(grove, pane, 3, 1);
        let folder = plain(grove, line, "folder");
        let line = row(grove, pane, 4, 1);
        let copy_password = end(grove, line, None, icons.copy, "copy");
        let password = secret(grove, line, copy_password.leaf(), "password");
        let line = row(grove, pane, 5, 1);
        let copy_otp = end(grove, line, None, icons.copy, "copy");
        let otp = secret(grove, line, copy_otp.leaf(), "otp");
        let line = row(grove, pane, 6, NOTES_ROWS);
        let mut notes = Field::grow(grove, line, Location::new(), CELL, "notes", true);
        notes.holds(Holds::Record);
        notes.reach(grove, false);
        let line = row(grove, pane, 6 + NOTES_ROWS, 1);
        let about = grove.branch(
            line,
            Text::new("")
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    center_y(50.pct()).height(1.letters()),
                ))
                .elevate(Elevation::up(1))
                .intangible()
                .color(lichen::INERT.ink)
                .font(italic)
                .font_size(lichen::caption()),
        );
        // The edit row: the two toggles at the left, save at the right.
        let line = row(grove, pane, 7 + NOTES_ROWS, 1);
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
        let edit = Switch::grow(
            grove,
            line,
            Location::new().xs(
                left(anchor().right() + GAP.px()).width(Switch::width("edit")),
                top(0.px()).bottom(100.pct()),
            ),
            Elevation::up(1),
            "edit",
        );
        grove.anchor(edit.leaf(), show.leaf());
        let save = end(grove, line, None, icons.check, "save");
        // The foot: delete, which is two presses, tucked behind its mark, and what the pane says
        // beside it.
        let height = measure().height;
        let foot = grove.branch(
            pane,
            Stem::new()
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    bottom(100.pct()).height(height.px()),
                ))
                .elevate(Elevation::up(1))
                .intangible()
                .focus_scope(),
        );
        let erase = Confirm::marks(
            grove,
            foot,
            Location::new().xs(
                left(0.px()).width(Chip::square()),
                top(0.px()).bottom(100.pct()),
            ),
            Location::new().xs(
                right(100.pct()).width(Chip::width("delete")),
                top(0.px()).bottom(100.pct()),
            ),
            icons.trash_2,
            icons.x,
            (icons.trash_2, "delete"),
        );
        let says = Say::grow(
            grove,
            foot,
            Location::new().xs(
                left(anchor().right() + GAP.px())
                    .right(100.pct() - Chip::width("delete") - GAP.px()),
                center_y(50.pct()).height(1.letters()),
            ),
            Some(italic),
        );
        grove.anchor(says.leaf(), erase.ask().leaf());
        let mut pane = Self {
            title,
            username,
            url,
            folder,
            password,
            otp,
            notes,
            copy_password,
            copy_otp,
            about,
            show,
            edit,
            save,
            erase,
            says,
            showing: None,
        };
        pane.show(grove, None);
        pane
    }

    /// Shows `entry`, or nothing. The entry already showing keeps an edit under way.
    fn show(&mut self, grove: &mut Grove, entry: Option<(usize, &Entry)>) {
        let same = entry.map(|(key, _)| key) == self.showing.as_ref().map(|(key, _)| *key);
        if same && self.edit.on() {
            return;
        }
        self.showing = entry.map(|(key, entry)| (key, entry.clone()));
        if !same {
            if self.show.on() {
                self.show.set(grove, false);
            }
            self.erase.disarm(grove);
            self.says.tell(grove, "", Voice::Quiet);
        }
        self.editing(grove, false);
    }

    fn frame(&mut self, grove: &mut Grove, pollen: &Pollen, vault: &mut Vault) -> Did {
        let mut did = Did::default();
        let Some(key) = self.showing.as_ref().map(|(key, _)| *key) else {
            return did;
        };
        if self.show.frame(grove, pollen) {
            self.secrets(grove);
        }
        if self.edit.frame(grove, pollen) {
            let on = self.edit.on();
            self.editing(grove, on);
        }
        let mut moved = false;
        for field in self.fields() {
            moved |= field.frame(grove, pollen);
        }
        if moved {
            self.differs(grove);
        }
        for (pressed, secret) in [
            (self.copy_password.pressed(pollen), Secret::Password),
            (self.copy_otp.pressed(pollen), Secret::Otp),
        ] {
            if pressed && let Some(entry) = vault.entry(key) {
                let value = match secret {
                    Secret::Password => entry.password.clone(),
                    Secret::Otp => entry.otp.clone(),
                };
                vault.copied(key, secret);
                did.changed = true;
                did.copied = Some(value);
                let said = format!("{} copied · {}s", secret.name(), CLEARS.as_secs());
                self.says.tell(grove, &said, Voice::Hint);
            }
        }
        if self.save.pressed(pollen)
            && let Some((_, was)) = &self.showing
        {
            let entry = Entry {
                kind: was.kind.clone(),
                title: self.title.value(grove),
                username: self.username.value(grove),
                url: self.url.value(grove),
                folder: self.folder.value(grove),
                password: self.password.value(grove),
                otp: self.otp.value(grove),
                notes: self.notes.value(grove),
                days: was.days,
            };
            let changed = vault.save(key, entry);
            self.showing = vault.entry(key).map(|entry| (key, entry.clone()));
            self.editing(grove, false);
            let said = match changed {
                true => "saved",
                false => "nothing changed",
            };
            self.says.tell(grove, said, Voice::Hint);
            did.changed |= changed;
        }
        if let Some(Step::Confirmed) = self.erase.frame(grove, pollen) {
            vault.erase(key);
            self.showing = None;
            did.changed = true;
        }
        did
    }

    /// Every field, in reading order.
    fn fields(&mut self) -> [&mut Field; 7] {
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

    /// Starts an edit, or ends one: the fields in reach or out of it, and -- ending -- what the
    /// entry says put back.
    fn editing(&mut self, grove: &mut Grove, editing: bool) {
        if self.edit.on() != editing {
            self.edit.set(grove, editing);
        }
        for field in self.fields() {
            field.reach(grove, editing);
        }
        let open = self.showing.is_some();
        match self.showing.clone() {
            // An edit starts from what the entry says, not from the stand-in.
            Some((_, entry)) if editing => {
                self.password.put(grove, &entry.password);
                self.otp.put(grove, &entry.otp);
            }
            Some((_, entry)) => {
                self.title.put(grove, &entry.title);
                self.username.put(grove, &entry.username);
                self.url.put(grove, &entry.url);
                self.folder.put(grove, &entry.folder);
                self.notes.put(grove, &entry.notes);
                let mut about = vec![entry.kind.clone()];
                if entry.kind == "login" {
                    about.push(match entry.days {
                        0 => "password set today".to_string(),
                        1 => "password set yesterday".to_string(),
                        days => format!("password {days}d old"),
                    });
                }
                grove.text(self.about, about.join(" · "));
            }
            None => {
                for field in self.fields() {
                    field.clear(grove);
                }
                self.title.put(grove, "nothing open");
                grove.text(self.about, "");
            }
        }
        self.secrets(grove);
        let has = |pick: fn(&Entry) -> &String| {
            self.showing
                .as_ref()
                .is_some_and(|(_, entry)| !pick(entry).is_empty())
        };
        let (password, otp) = (has(|e| &e.password), has(|e| &e.otp));
        self.copy_password.arm(grove, Press::rest_if(password));
        self.copy_otp.arm(grove, Press::rest_if(otp));
        self.show.reach(grove, open);
        self.edit.reach(grove, open);
        self.erase.open(grove, open && !editing);
        self.save.arm(grove, Press::Inert);
    }

    /// The two secrets, as they stand: masked unless `show` is on; and outside an edit, what they
    /// are while shown and the stand-in while not, or nothing where the entry has none. An edit
    /// holds what was typed, so only the mask follows `show` while one is under way.
    fn secrets(&mut self, grove: &mut Grove) {
        let shown = self.show.on();
        let editing = self.edit.on();
        let Some((_, entry)) = self.showing.clone() else {
            return;
        };
        for (field, value) in [
            (&mut self.password, &entry.password),
            (&mut self.otp, &entry.otp),
        ] {
            grove.masked(field.input(), !shown);
            if editing {
                continue;
            }
            match (shown, value.is_empty()) {
                (_, true) => field.clear(grove),
                (true, false) => field.put(grove, value),
                (false, false) => field.put(grove, HIDDEN),
            }
        }
    }

    /// Whether what the fields say differs from the entry, and save armed for it.
    fn differs(&mut self, grove: &mut Grove) {
        let Some((_, entry)) = self.showing.clone() else {
            return self.save.arm(grove, Press::Inert);
        };
        let differs = self.title.value(grove) != entry.title
            || self.username.value(grove) != entry.username
            || self.url.value(grove) != entry.url
            || self.folder.value(grove) != entry.folder
            || self.password.value(grove) != entry.password
            || self.otp.value(grove) != entry.otp
            || self.notes.value(grove) != entry.notes;
        self.save
            .arm(grove, Press::armed_if(self.edit.on() && differs));
    }
}

/// How tall the list is: a few rows, scrolling for the rest, so the pane under it is in reach.
fn list_height() -> f32 {
    LIST as f32 * (measure().height + APART) - APART
}

/// How far down the pane row `n` stands.
fn down(n: usize) -> foliage::Length {
    (n as f32 * (measure().height + GAP)).px()
}

/// Row `n` of the pane, `rows` rows tall.
fn row(grove: &mut Grove, pane: Leaf, n: usize, rows: usize) -> Leaf {
    let height = rows as f32 * measure().height + (rows as f32 - 1.0) * GAP;
    grove.branch(
        pane,
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

/// A plain field filling its row: what a record says, out of reach until an edit.
fn plain(grove: &mut Grove, line: Leaf, name: &str) -> Field {
    let mut field = Field::grow(grove, line, Location::new(), CELL, name, false);
    field.holds(Holds::Record);
    field.reach(grove, false);
    field
}

/// A masked field filling its row up to `before`.
fn secret(grove: &mut Grove, line: Leaf, before: Leaf, name: &str) -> Field {
    let mut field = Field::grow(
        grove,
        line,
        Location::new().xs(
            left(0.px()).right(anchor().left() - GAP.px()),
            top(0.px()).bottom(100.pct()),
        ),
        CELL,
        name,
        false,
    );
    grove.anchor(field.leaf(), before);
    field.holds(Holds::Record);
    field.reach(grove, false);
    grove.masked(field.input(), true);
    field
}

/// A press at the end of a row: at its right edge, or left of `after`.
fn end(
    grove: &mut Grove,
    line: Leaf,
    after: Option<Leaf>,
    mark: foliage::Field,
    name: &str,
) -> Chip {
    let across = match after {
        Some(_) => right(anchor().left() - GAP.px()).width(Chip::width(name)),
        None => right(100.pct()).width(Chip::width(name)),
    };
    let mut chip = Chip::grow(
        grove,
        line,
        Location::new().xs(across, top(0.px()).bottom(100.pct())),
        Elevation::up(1),
        mark,
        name,
    );
    if let Some(after) = after {
        grove.anchor(chip.leaf(), after);
    }
    chip.arm(grove, Press::Inert);
    chip
}
