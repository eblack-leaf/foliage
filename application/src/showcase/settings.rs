//! Settings and an account: what the person is called, what reaches them, and what cannot be undone.

use foliage::{
    Boxed, Color, Corners, Elevation, Font, Grove, Grow, Leaf, Location, Motion, Palette, Panel,
    Place, Pollen, Rounding, Source, Text, center_y, content, left, right, top,
};
use lichen::{Chip, Confirm, Holds, Origin, Pick, Press, Say, Step, Switch, Voice, gate, measure};

use crate::icons::Icons;
use crate::parts::{self, Stack, at, in_lane, lane};

/// What the profile holds, as the record has it.
const PROFILE: [(&str, &str); 3] = [
    ("name", "Ada Moss"),
    ("email", "ada@example.com"),
    ("handle", "@ada"),
];

/// What can reach the person.
const NOTIFY: [&str; 3] = ["email", "push", "weekly digest"];

/// How the page can look, and what each looks like: its ground and the ink on it.
const LOOKS: [&str; 3] = ["dark", "light", "system"];
const PREVIEW: [(Color, Color); 3] = [
    (Color::rgb(0.08, 0.09, 0.10), Color::rgb(0.90, 0.92, 0.94)),
    (Color::rgb(0.96, 0.96, 0.97), Color::rgb(0.10, 0.11, 0.13)),
    (Color::rgb(0.30, 0.32, 0.35), Color::rgb(0.96, 0.96, 0.97)),
];

/// Where the account is signed in, and when it was last seen there. The first is this one.
const SESSIONS: [&str; 3] = [
    "laptop · lisbon · now",
    "phone · porto · 2h ago",
    "tablet · berlin · 3d ago",
];

/// What turning two-factor on goes through.
const TWO_FACTOR: [&str; 3] = ["scan", "verify", "on"];

pub(crate) struct Settings {
    profile: Vec<lichen::Field>,
    save: Chip,
    saved: Say,
    notify: Vec<Switch>,
    look: Pick,
    preview: (Leaf, Leaf),
    sessions: Vec<(Leaf, Chip, bool)>,
    two_factor: Vec<Chip>,
    enrolled: [bool; TWO_FACTOR.len()],
    password: lichen::Field,
    strength: Leaf,
    strength_said: Leaf,
    language: Pick,
    delete: Confirm,
    deleted: Say,
}

impl Settings {
    pub(crate) fn grow(grove: &mut Grove, page: Leaf, icons: &Icons, italic: Font) -> Self {
        let m = measure();
        let mut stack = Stack::new(page);
        stack.heading(
            grove,
            "settings & account",
            "A record the person can change: what is written reads in ink, and what they have \
             changed and not yet saved reads in the accent.",
            italic,
        );

        let body = stack.card(grove, "profile", lane(PROFILE.len() + 1) - m.gap);
        let profile = PROFILE
            .iter()
            .enumerate()
            .map(|(n, &(name, value))| {
                let mut field = lichen::Field::grow(
                    grove,
                    body,
                    in_lane(left(0.px()).width(100.pct()), n),
                    7.0,
                    name,
                    false,
                );
                field.holds(Holds::Record);
                field.put(grove, value);
                field
            })
            .collect();
        let mut save = parts::chip(
            grove,
            body,
            in_lane(left(0.px()).width(Chip::width("save")), PROFILE.len()),
            icons.check,
            "save",
        );
        save.arm(grove, Press::Inert);
        let saved = Say::grow(
            grove,
            body,
            Location::new().xs(
                left(Chip::width("save") + m.gap.px()).right(100.pct()),
                center_y((lane(PROFILE.len()) + m.height / 2.0).px()).height(1.letters()),
            ),
            Some(italic),
        );

        let body = stack.card(grove, "notifications", lane(NOTIFY.len()) - m.gap);
        let notify = NOTIFY
            .iter()
            .enumerate()
            .map(|(n, &name)| {
                let mut switch = Switch::grow(
                    grove,
                    body,
                    in_lane(left(0.px()).width(Switch::width(name)), n),
                    Elevation::up(1),
                    name,
                );
                switch.set(grove, n != 1);
                switch
            })
            .collect();

        // How the page looks, and a swatch of it.
        let body = stack.card(grove, "appearance", lane(1) + 48.0);
        let look = Pick::grow(
            grove,
            body,
            in_lane(left(0.px()).width(100.pct()), 0),
            6.0,
            "theme",
            &LOOKS,
            1,
        );
        let (ground, ink) = PREVIEW[0];
        let swatch = grove.branch(
            body,
            Panel::new()
                .color(ground)
                .rounding(Corners::all(Rounding::Sm))
                .intangible()
                .elevate(Elevation::up(1))
                .at(at(left(0.px()).width(100.pct()), lane(1), 48.0)),
        );
        let sample = grove.branch(
            swatch,
            Text::new("the quick brown fox")
                .color(ink)
                .font_size(lichen::caption())
                .intangible()
                .elevate(Elevation::up(1))
                .at(Location::new().xs(
                    left(14.px()).width(content()),
                    center_y(50.pct()).height(1.letters()),
                )),
        );

        // Where the account is signed in: every other place can be signed out of, and this one
        // cannot be from here.
        let body = stack.card(grove, "sessions", lane(SESSIONS.len()) - m.gap);
        let sessions = SESSIONS
            .iter()
            .enumerate()
            .map(|(n, &said)| {
                let this = n == 0;
                let name = match this {
                    true => "current",
                    false => "sign out",
                };
                let row = parts::words(
                    grove,
                    body,
                    said,
                    Location::new().xs(
                        left(0.px()).right(100.pct() - Chip::width("signed out") - m.gap.px()),
                        center_y((lane(n) + m.height / 2.0).px()).height(1.letters()),
                    ),
                    lichen::REST.ink,
                );
                let mut chip = parts::chip(
                    grove,
                    body,
                    in_lane(right(100.pct()).width(Chip::width("signed out")), n),
                    match this {
                        true => icons.smartphone,
                        false => icons.log_out,
                    },
                    name,
                );
                if this {
                    chip.arm(grove, Press::Inert);
                }
                (row, chip, !this)
            })
            .collect();

        let body = stack.card(grove, "two-factor", lane(1) + 16.0);
        let marks = [icons.smartphone, icons.key, icons.shield];
        let two_factor = TWO_FACTOR
            .iter()
            .zip(marks)
            .enumerate()
            .map(|(n, (&name, mark))| {
                let third = 100.0 / TWO_FACTOR.len() as f32;
                let across = left((n as f32 * third).pct() + (n as f32 * m.gap / 3.0).px())
                    .width(third.pct() - (2.0 * m.gap / 3.0).px());
                parts::chip(grove, body, in_lane(across, 0), mark, name)
            })
            .collect();

        // A new password, and how strong it reads as it is typed.
        let body = stack.card(grove, "password", lane(1) + 30.0);
        let password = lichen::Field::grow(
            grove,
            body,
            in_lane(left(0.px()).width(100.pct()), 0),
            7.0,
            "new",
            false,
        );
        let track = grove.branch(
            body,
            Panel::new()
                .color(Palette::Surface)
                .rounding(Corners::all(Rounding::Full))
                .intangible()
                .elevate(Elevation::up(1))
                .at(at(left(0.px()).width(100.pct()), lane(1), 6.0)),
        );
        let strength = grove.branch(
            track,
            Panel::new()
                .color(Palette::Danger)
                .rounding(Corners::all(Rounding::Full))
                .intangible()
                .elevate(Elevation::up(1))
                .at(measured(0)),
        );
        let strength_said = parts::line(
            grove,
            body,
            "type one to see how strong it is",
            0.0,
            lane(1) + 14.0,
            lichen::INERT.ink,
        );

        let body = stack.card(grove, "language", m.height);
        let language = Pick::grow(
            grove,
            body,
            Location::new(),
            7.0,
            "language",
            &["english", "português", "deutsch"],
            1,
        );

        let body = stack.card(grove, "delete account", lane(1) + 16.0);
        let mut delete = Confirm::grow(
            grove,
            body,
            in_lane(left(0.px()).width(Chip::width("delete account")), 0),
            in_lane(right(100.pct()).width(Chip::width("delete")), 0),
            (icons.trash_2, "delete account"),
            (icons.x, "keep"),
            (icons.trash_2, "delete"),
        );
        delete.open(grove, true);
        let deleted = Say::grow(
            grove,
            body,
            at(left(0.px()).width(100.pct()), lane(1), 16.0),
            Some(italic),
        );

        let mut settings = Self {
            profile,
            save,
            saved,
            notify,
            look,
            preview: (swatch, sample),
            sessions,
            two_factor,
            enrolled: [false; TWO_FACTOR.len()],
            password,
            strength,
            strength_said,
            language,
            delete,
            deleted,
        };
        settings.enroll(grove);
        settings
    }

    pub(crate) fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) {
        let mut edited = false;
        for field in &mut self.profile {
            edited |= field.frame(grove, pollen);
        }
        if edited {
            let changed = self
                .profile
                .iter()
                .any(|field| field.origin() == Origin::Typed);
            self.save.arm(grove, Press::armed_if(changed));
            self.saved.clear(grove);
        }
        if self.save.pressed(pollen) {
            // What was typed is the record now: put back as what the record says.
            for field in &mut self.profile {
                let value = field.value(grove);
                field.put(grove, &value);
            }
            self.save.arm(grove, Press::Inert);
            self.saved.tell(grove, "saved.", Voice::Hint);
        }
        for switch in &mut self.notify {
            switch.frame(grove, pollen);
        }
        if self.look.frame(grove, pollen) {
            let (ground, ink) = PREVIEW[self.look.index()];
            grove.animate(self.preview.0, Motion::Color(ground), lichen::timing());
            grove.animate(self.preview.1, Motion::Color(ink), lichen::timing());
        }
        for (row, chip, live) in &mut self.sessions {
            if *live && chip.pressed(pollen) {
                *live = false;
                chip.arm(grove, Press::Inert);
                grove.text(chip.label(), "signed out");
                grove.animate(*row, Motion::from(lichen::INERT.ink), lichen::timing());
            }
        }
        if let Some(n) = self.two_factor.iter().position(|chip| chip.pressed(pollen)) {
            match self.enrolled.iter().all(|&done| done) {
                true => self.enrolled = [false; TWO_FACTOR.len()],
                false => self.enrolled[n] = true,
            }
            self.enroll(grove);
        }
        if self.password.frame(grove, pollen) {
            let typed = self.password.value(grove);
            let score = strength(&typed);
            let (fill, said) = match score {
                0 => (Palette::Danger, "type one to see how strong it is"),
                1 | 2 => (Palette::Danger, "weak"),
                3 => (Palette::Caution, "fair"),
                _ => (Palette::Positive, "strong"),
            };
            grove.animate(
                self.strength,
                Motion::Location(measured(score)),
                lichen::timing(),
            );
            grove.animate(self.strength, fill.into(), lichen::timing());
            grove.text(self.strength_said, said);
        }
        self.language.frame(grove, pollen);
        match self.delete.frame(grove, pollen) {
            Some(Step::Confirmed) => {
                self.deleted
                    .tell(grove, "it would be gone now. it is not.", Voice::Warn)
            }
            Some(Step::Kept) => self.deleted.set(grove, "kept."),
            Some(Step::Armed) => self.deleted.clear(grove),
            None => {}
        }
    }

    /// Dresses the two-factor steps for which are done.
    fn enroll(&mut self, grove: &mut Grove) {
        let all = self.enrolled.iter().all(|&done| done);
        let presses = gate(&self.enrolled);
        for (n, (chip, press)) in self.two_factor.iter_mut().zip(presses).enumerate() {
            let press = match (all, n == TWO_FACTOR.len() - 1) {
                (true, true) => Press::Chosen,
                _ => press,
            };
            chip.arm(grove, press);
        }
    }
}

/// How strong a password reads, `0` for none to `5`: its length, in steps of four letters up to
/// three, and one each for a digit and for something that is neither a letter nor a digit.
fn strength(password: &str) -> usize {
    if password.is_empty() {
        return 0;
    }
    let long = (password.chars().count() / 4).clamp(1, 3);
    let digit = password.chars().any(|c| c.is_ascii_digit()) as usize;
    let other = password.chars().any(|c| !c.is_alphanumeric()) as usize;
    long + digit + other
}

/// The strength meter's fill at `score` of five.
fn measured(score: usize) -> Location {
    Location::new().xs(
        left(0.px()).width((score as f32 * 20.0).pct()),
        top(0.px()).bottom(100.pct()),
    )
}
