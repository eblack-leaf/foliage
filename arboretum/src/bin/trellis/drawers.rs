//! The drawers a narrow window keeps its tools in: one of the kinds of card to add, and one to
//! find a card by name or code, with what it finds listed to press.

use foliage::{Elevation, Grove, Grow, Leaf, Location, Pollen, Source, left, top};
use lichen::{Chip, Field, measure};

use arboretum::sprout::{Drawer, Edge};

use crate::board::{Board, Id, Kind};
use crate::icons::Icons;
use crate::kit;
use crate::theme::{self, Faces};

/// How many finds are listed.
const FINDS: usize = 6;

/// What a drawer asks of the sheet.
#[derive(Clone, Debug, PartialEq)]
pub enum Asked {
    Add(Kind),
    Go(Id),
    /// The search's words changed; what it finds is listed.
    Search(String),
    Closed,
}

fn mark(icons: &Icons, kind: Kind) -> foliage::Field {
    match kind {
        Kind::Person => icons.user,
        Kind::Meeting => icons.calendar,
        Kind::Spec => icons.file_text,
        Kind::Milestone => icons.flag,
        Kind::Note => icons.edit_3,
    }
}

pub struct Drawers {
    pub add: Drawer,
    adds: Vec<(Kind, Chip)>,
    pub find: Drawer,
    field: Field,
    finds: Vec<(Option<Id>, Chip)>,
    marks: [foliage::Field; 5],
    searched: String,
}

impl Drawers {
    pub fn grow(grove: &mut Grove, under: Leaf, page: Leaf, faces: &Faces, icons: &Icons) -> Self {
        let m = measure();
        let add = Drawer::grow(grove, under, page, Edge::Bottom, 40.0 + 24.0 + 2.0 * m.height + m.gap + 4.0);
        kit::heading(grove, add.body(), faces, "ADD TO THE SHEET", 0.0, 0.0);
        let kinds = [Kind::Person, Kind::Meeting, Kind::Spec, Kind::Note];
        let adds = kinds
            .iter()
            .enumerate()
            .map(|(n, &kind)| {
                let (col, row) = ((n % 2) as f32, (n / 2) as f32);
                let at = Location::new().xs(
                    left((col * 50.0).pct() + (col * m.gap / 2.0).px()).width(50.pct() - (m.gap / 2.0).px()),
                    top((24.0 + row * (m.height + m.gap)).px()).height(m.height.px()),
                );
                (kind, Chip::grow(grove, add.body(), at, Elevation::up(1), mark(icons, kind), kind.name()))
            })
            .collect();
        let find = Drawer::grow(grove, under, page, Edge::Bottom, 40.0 + 24.0 + (FINDS as f32 + 1.0) * (m.height + m.gap) + 4.0);
        kit::heading(grove, find.body(), faces, "FIND ON THE SHEET", 0.0, 0.0);
        let field = Field::grow(
            grove,
            find.body(),
            Location::new().xs(left(0.px()).right(100.pct()), top(24.px()).height(m.height.px())),
            6.0,
            "find",
            false,
        );
        field.hint(grove, "a name, a title, a code…");
        let finds = (0..FINDS)
            .map(|n| {
                let at = Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top((24.0 + (n as f32 + 1.0) * (m.height + m.gap)).px()).height(m.height.px()),
                );
                let chip = Chip::grow(grove, find.body(), at, Elevation::up(1), icons.search, "");
                grove.visible(chip.leaf(), false);
                (None, chip)
            })
            .collect();
        Self {
            add,
            adds,
            find,
            field,
            finds,
            marks: Kind::ALL.map(|k| mark(icons, k)),
            searched: String::new(),
        }
    }

    pub fn open(&self) -> bool {
        self.add.is_open() || self.find.is_open()
    }

    /// Opens the find drawer with focus in its field.
    pub fn finding(&mut self, grove: &mut Grove) {
        self.add.close(grove);
        self.find.open(grove);
        self.field.focus(grove);
    }

    /// Opens the add drawer.
    pub fn adding(&mut self, grove: &mut Grove) {
        self.find.close(grove);
        self.add.open(grove);
    }

    /// Writes a search made on the rail into the field, so the two never disagree.
    pub fn search(&mut self, grove: &mut Grove, words: &str) {
        if words == self.searched {
            return;
        }
        self.searched = words.to_string();
        match words.is_empty() {
            true => self.field.clear(grove),
            false => self.field.put(grove, words),
        }
    }

    /// Lists what `found` names, best first.
    pub fn list(&mut self, grove: &mut Grove, board: &Board, found: &[Id]) {
        for (n, (id, chip)) in self.finds.iter_mut().enumerate() {
            match found.get(n).and_then(|f| board.get(*f)) {
                Some(node) => {
                    *id = Some(node.id);
                    let title = match node.kind {
                        Kind::Note => theme::clip(&node.title, 22),
                        _ => theme::clip(&node.title, 26),
                    };
                    grove.text(chip.label(), format!("{} · {title}", node.code));
                    chip.remark(grove, self.marks[node.kind.index()]);
                    grove.visible(chip.leaf(), true);
                }
                None => {
                    *id = None;
                    grove.visible(chip.leaf(), false);
                }
            }
        }
    }

    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) -> Vec<Asked> {
        let mut asked = Vec::new();
        if self.add.frame(grove, pollen).is_some() {
            asked.push(Asked::Closed);
        }
        if self.find.frame(grove, pollen).is_some() {
            asked.push(Asked::Closed);
        }
        for (kind, chip) in &self.adds {
            if chip.pressed(pollen) {
                self.add.close(grove);
                asked.push(Asked::Add(*kind));
            }
        }
        if self.field.frame(grove, pollen) {
            let words = self.field.value(grove);
            if words != self.searched {
                self.searched = words.clone();
                asked.push(Asked::Search(words));
            }
        }
        let first = self.finds.first().and_then(|(id, _)| *id);
        if pollen.submitted(self.field.input())
            && let Some(id) = first
        {
            self.find.close(grove);
            asked.push(Asked::Go(id));
        }
        for (id, chip) in &self.finds {
            if chip.pressed(pollen)
                && let Some(id) = id
            {
                self.find.close(grove);
                asked.push(Asked::Go(*id));
            }
        }
        asked
    }
}
