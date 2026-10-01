//! The readout section: how the passwords are, what was done this visit, and the lock.
//!
//! In the credentials app this is the sync section, past the bow's neck, with the wire's readings
//! at its foot. A toy has no wire, so what stands here is the rest of it: two presses -- check,
//! which reads every password and says which are shared, old, short or missing, and lock, which
//! shuts the vault and stands the lock over the key again -- and a line for what the last of them
//! did. What they open is two lists, one over the other: what the last check found, and what was
//! done to the vault since the page opened.

use foliage::{
    Axes, Boxed, Elevation, Font, Grove, Grow, Leaf, Location, Place, Pollen, Source, Stem, Text,
    anchor, left, top,
};
use lichen::{Chip, Say, Voice, measure, words};

use super::vault::Line;
use crate::icons::Icons;

/// The room between two things that are not one thing, and between the two lists.
const GAP: f32 = 8.0;
const APART: f32 = 16.0;
/// How many letters a list's line shows: what a phone's column holds.
const COLUMN: usize = 40;
/// How far apart, in lines, one entry of a list stands from the next: its two lines and a gap.
const STEP: f32 = 3.0;

/// What the section asks the root for.
#[derive(Copy, Clone)]
pub(crate) enum Asked {
    /// Read every password and say how they are.
    Check,
    /// Shut the vault.
    Lock,
}

/// The readout section, grown.
pub(crate) struct Readout {
    check: Chip,
    lock: Chip,
    says: Say,
    health: List,
    visit: List,
}

/// A list with a head, a line to each thing in it, and what it says when it has nothing.
struct List {
    lines: Leaf,
    rows: Vec<(Leaf, Leaf)>,
    none: Leaf,
    italic: Font,
}

impl Readout {
    /// Grows the presses on `sheet` and the lists on `details`.
    pub(crate) fn grow(
        grove: &mut Grove,
        sheet: Leaf,
        details: Leaf,
        icons: &Icons,
        italic: Font,
    ) -> Self {
        let height = measure().height;
        let pair = height + GAP + 1.0 * 16.0;
        let check = Chip::grow(
            grove,
            sheet,
            Location::new().xs(
                left(0.px()).width(Chip::width("check")),
                top(50.pct() - (pair / 2.0).px()).height(height.px()),
            ),
            Elevation::up(1),
            icons.shield,
            "check",
        );
        let lock = Chip::grow(
            grove,
            sheet,
            Location::new().xs(
                left(anchor().right() + GAP.px()).width(Chip::width("lock")),
                top(50.pct() - (pair / 2.0).px()).height(height.px()),
            ),
            Elevation::up(1),
            icons.lock,
            "lock",
        );
        grove.anchor(lock.leaf(), check.leaf());
        let says = Say::grow(
            grove,
            sheet,
            Location::new().xs(
                left(0.px()).right(100.pct()),
                top(anchor().bottom() + GAP.px()).height(1.letters()),
            ),
            Some(italic),
        );
        grove.anchor(says.leaf(), check.leaf());
        let half = |n: usize| {
            let (from, to) = (n as f32 * 50.0, (n + 1) as f32 * 50.0);
            Location::new().xs(
                left(0.px()).right(100.pct()),
                top(from.pct() + (APART / 2.0).px()).bottom(to.pct() - (APART / 2.0).px()),
            )
        };
        let health = List::grow(grove, details, half(0), "health", "not checked", italic);
        let visit = List::grow(
            grove,
            details,
            half(1),
            "this visit",
            "nothing done yet",
            italic,
        );
        Self {
            check,
            lock,
            says,
            health,
            visit,
        }
    }

    /// Carries the presses for a frame: one says what it is doing and hands itself up.
    pub(crate) fn frame(&mut self, pollen: &Pollen) -> Option<Asked> {
        if self.check.pressed(pollen) {
            return Some(Asked::Check);
        }
        if self.lock.pressed(pollen) {
            return Some(Asked::Lock);
        }
        None
    }

    /// What the last check found.
    pub(crate) fn checked(&mut self, grove: &mut Grove, found: &[Line]) {
        self.health.show(grove, found);
        let said = match found.len() {
            0 => "nothing to say".to_string(),
            1 => "1 to look at".to_string(),
            n => format!("{n} to look at"),
        };
        self.says.tell(grove, &said, Voice::Hint);
    }

    /// What was done this visit, latest first.
    pub(crate) fn visited(&mut self, grove: &mut Grove, log: &[Line]) {
        self.visit.show(grove, log);
    }

    /// The vault was locked: what the check read is no longer anyone's to see.
    pub(crate) fn lock(&mut self, grove: &mut Grove) {
        self.health.show(grove, &[]);
        self.says.clear(grove);
    }
}

impl List {
    fn grow(
        grove: &mut Grove,
        details: Leaf,
        at: Location,
        head: &str,
        none: &str,
        italic: Font,
    ) -> Self {
        let room = grove.branch(
            details,
            Stem::new()
                .at(at)
                .elevate(Elevation::up(1))
                .font_size(lichen::caption())
                .intangible(),
        );
        let head = grove.branch(
            room,
            Text::new(head)
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(0.px()).height(1.letters()),
                ))
                .elevate(Elevation::up(1))
                .intangible()
                .color(lichen::REST.ink)
                .font_size(lichen::caption()),
        );
        let lines = grove.branch(
            room,
            Stem::new()
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(anchor().bottom() + GAP.px()).bottom(100.pct()),
                ))
                .anchored(head)
                .elevate(Elevation::up(1))
                .font_size(lichen::caption())
                .scrolls(Axes::Vertical),
        );
        let none = grove.branch(
            lines,
            Text::new(none)
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(0.px()).height(1.letters()),
                ))
                .elevate(Elevation::up(1))
                .intangible()
                .color(lichen::INERT.ink)
                .font(italic)
                .font_size(lichen::caption()),
        );
        Self {
            lines,
            rows: Vec::new(),
            none,
            italic,
        }
    }

    /// Shows `lines`, two lines to each: its name, and under it, in the quiet italic, what and
    /// more -- one under the other rather than side by side, so a phone's width does not wrap
    /// the reason into the next.
    fn show(&mut self, grove: &mut Grove, lines: &[Line]) {
        while self.rows.len() < lines.len() {
            let n = self.rows.len() as f32;
            let at = |down: f32| {
                Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top((n * STEP).letters() + down.letters()).height(1.letters()),
                )
            };
            let name = grove.branch(
                self.lines,
                Text::new("")
                    .at(at(0.0))
                    .elevate(Elevation::up(1))
                    .intangible()
                    .color(lichen::REST.ink)
                    .font_size(lichen::caption()),
            );
            let said = grove.branch(
                self.lines,
                Text::new("")
                    .at(at(1.2))
                    .elevate(Elevation::up(1))
                    .intangible()
                    .color(lichen::INERT.ink)
                    .font(self.italic)
                    .font_size(lichen::caption()),
            );
            self.rows.push((name, said));
        }
        for (n, &(name, said)) in self.rows.iter().enumerate() {
            let line = lines.get(n);
            grove.visible(name, line.is_some());
            grove.visible(said, line.is_some());
            if let Some(line) = line {
                grove.text(name, words::cut(&line.name, COLUMN));
                let more = match line.more.is_empty() {
                    true => line.what.clone(),
                    false => format!("{} · {}", line.what, line.more),
                };
                grove.text(said, words::cut(&more, COLUMN));
            }
        }
        grove.visible(self.none, lines.is_empty());
    }
}
