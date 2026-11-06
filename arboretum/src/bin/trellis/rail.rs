//! The drafting rail: the instrument along the top that holds the tools.
//!
//! What it holds depends on how much room there is. Wide, every tool stands in it with a name on
//! the two that need one, and the search is a field; narrower, the names go; narrower still, the
//! four kinds of card fold into one press that opens a drawer of them, and the layers, the key plan
//! and the scale fold into another. Nothing ever wraps: each breakpoint's row is laid out in
//! pixels, and is narrower than the narrowest window it is used at.

use foliage::{Elevation, Grove, Grow, Layout, Leaf, Location, Pollen, Source, center_y, content, left, top};
use lichen::{Badge, Chip, Field, Holds, Press, Tone};


use crate::board::Kind;
use crate::drawing::Tool;
use crate::icons::Icons;
use crate::kit;
use crate::theme::{self, Faces};

/// What a press on the rail asks for.
#[derive(Clone, Debug, PartialEq)]
pub enum Asked {
    Tool(Tool),
    Add(Kind),
    AddMenu,
    Arrange,
    Clashes,
    Undo,
    Zoom(i32),
    View,
    Find,
    /// The search field's words changed.
    Search(String),
    /// Enter in the search field.
    Go(String),
    /// Keys pressed on the rail, for the sheet's shortcuts.
    Keys(Vec<foliage::Keystroke>),
}

/// How far in from the plate's edges the row stands, and between groups.
const PAD: f32 = 8.0;
const GAP: f32 = 6.0;
const BREAK: f32 = 13.0;

/// Something that stands in the rail.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Item {
    Select,
    Connect,
    Person,
    Meeting,
    Spec,
    Note,
    AddMenu,
    Arrange,
    ArrangeNamed,
    Clashes,
    ClashesNamed,
    Undo,
    ZoomOut,
    Scale,
    ZoomIn,
    View,
    Find,
    Field,
    Break,
}

/// The row each breakpoint stands.
fn row(layout: Layout) -> Vec<Item> {
    use Item::*;
    match layout {
        Layout::Xs | Layout::Sm => vec![Select, Connect, Break, AddMenu, Break, Arrange, Clashes, Undo, Break, View, Find],
        Layout::Md => vec![
            Select, Connect, Break, Person, Meeting, Spec, Note, Break, Arrange, Clashes, Undo, Break, ZoomOut, ZoomIn,
            Break, Find,
        ],
        Layout::Lg => vec![
            Select, Connect, Break, Person, Meeting, Spec, Note, Break, Arrange, Clashes, Undo, Break, ZoomOut, Scale,
            ZoomIn, Break, Field,
        ],
        Layout::Xl => vec![
            Select,
            Connect,
            Break,
            Person,
            Meeting,
            Spec,
            Note,
            Break,
            ArrangeNamed,
            ClashesNamed,
            Undo,
            Break,
            ZoomOut,
            Scale,
            ZoomIn,
            Break,
            Field,
        ],
    }
}

fn width(item: Item, layout: Layout) -> f32 {
    let m = lichen::measure();
    match item {
        Item::ArrangeNamed => theme::chip_px("arrange"),
        Item::ClashesNamed => theme::chip_px("clashes"),
        Item::Scale => 46.0,
        Item::Field => match layout {
            Layout::Xl => 236.0,
            _ => 188.0,
        },
        Item::Break => BREAK - GAP,
        _ => m.height,
    }
}

pub struct Rail {
    plate: Leaf,
    chips: Vec<(Item, Chip)>,
    tallies: Vec<Badge>,
    scale: Leaf,
    field: Field,
    breaks: Vec<Leaf>,
    tool: Tool,
    searched: String,
}

impl Rail {
    pub fn grow(grove: &mut Grove, under: Leaf, faces: &Faces, icons: &Icons) -> Self {
        let plate = kit::plate(grove, under, Elevation::up(30));
        let m = lichen::measure();
        let spot = Location::new().xs(left(0.px()).width(m.height.px()), top(PAD.px()).height(m.height.px()));
        let mut chips = Vec::new();
        let marks = [
            (Item::Select, icons.mouse_pointer),
            (Item::Connect, icons.git_branch),
            (Item::Person, icons.user),
            (Item::Meeting, icons.calendar),
            (Item::Spec, icons.file_text),
            (Item::Note, icons.edit_3),
            (Item::AddMenu, icons.plus),
            (Item::Arrange, icons.grid),
            (Item::Clashes, icons.alert_triangle),
            (Item::Undo, icons.rotate_ccw),
            (Item::ZoomOut, icons.zoom_out),
            (Item::ZoomIn, icons.zoom_in),
            (Item::View, icons.layers),
            (Item::Find, icons.search),
        ];
        for (item, mark) in marks {
            chips.push((item, Chip::mark(grove, plate, spot.clone(), Elevation::up(1), mark)));
        }
        for (item, mark, name) in [(Item::ArrangeNamed, icons.grid, "arrange"), (Item::ClashesNamed, icons.alert_triangle, "clashes")] {
            chips.push((item, Chip::grow(grove, plate, spot.clone(), Elevation::up(1), mark, name)));
        }
        let mut tallies = Vec::new();
        for (item, chip) in &chips {
            if matches!(item, Item::Clashes | Item::ClashesNamed) {
                tallies.push(Badge::counting(
                    grove,
                    chip.leaf(),
                    Tone {
                        fill: theme::pencil().into(),
                        ink: theme::plate().into(),
                    },
                ));
            }
        }
        let scale = kit::letter(
            grove,
            plate,
            "100%",
            faces.bold,
            theme::TITLE,
            theme::ink(),
            Location::new().xs(left(0.px()).width(content()), center_y(50.pct()).height(1.letters())),
        );
        let mut field = Field::grow(grove, plate, spot, 6.0, "find", false);
        field.holds(Holds::Draft);
        field.hint(grove, "a name, a code…");
        let breaks = (0..6)
            .map(|_| {
                kit::rule(
                    grove,
                    plate,
                    Location::new().xs(left(0.px()).width(1.px()), top(14.px()).height(22.px())),
                    theme::rule(),
                )
            })
            .collect();
        let mut rail = Self {
            plate,
            chips,
            tallies,
            scale,
            field,
            breaks,
            tool: Tool::Select,
            searched: String::new(),
        };
        rail.choose(grove, Tool::Select);
        rail
    }

    /// Lays the rail out for `layout`.
    pub fn place(&mut self, grove: &mut Grove, layout: Layout) {
        let m = lichen::measure();
        let items = row(layout);
        let mut x = PAD;
        let mut breaks = self.breaks.iter();
        let mut shown = Vec::new();
        for (n, item) in items.iter().enumerate() {
            let w = width(*item, layout);
            match item {
                Item::Break => {
                    if let Some(&rule) = breaks.next() {
                        grove.at(
                            rule,
                            Location::new().xs(left((x + w / 2.0 - 0.5).round().px()).width(1.px()), top((PAD + 6.0).px()).height((m.height - 12.0).px())),
                        );
                        grove.visible(rule, true);
                    }
                }
                Item::Scale => {
                    grove.at(
                        self.scale,
                        Location::new().xs(
                            foliage::center_x((x + w / 2.0).px()).width(content()),
                            center_y((PAD + m.height / 2.0).px()).height(1.letters()),
                        ),
                    );
                }
                Item::Field => {
                    grove.at(
                        self.field.leaf(),
                        Location::new().xs(left(x.px()).width(w.px()), top(PAD.px()).height(m.height.px())),
                    );
                }
                chip => {
                    if let Some((_, c)) = self.chips.iter().find(|(i, _)| i == chip) {
                        grove.at(
                            c.leaf(),
                            Location::new().xs(left(x.px()).width(w.px()), top(PAD.px()).height(m.height.px())),
                        );
                    }
                }
            }
            shown.push(*item);
            x += w;
            if n + 1 < items.len() {
                x += GAP;
            }
        }
        for rule in breaks {
            grove.visible(*rule, false);
        }
        for (item, chip) in &self.chips {
            grove.visible(chip.leaf(), shown.contains(item));
        }
        grove.visible(self.scale, shown.contains(&Item::Scale));
        grove.visible(self.field.leaf(), shown.contains(&Item::Field));
        let width = x + PAD;
        grove.at(
            self.plate,
            Location::new().xs(
                left(theme::GUTTER.px()).width(width.px()),
                top(theme::GUTTER.px()).height(theme::RAIL.px()),
            ),
        );
    }

    fn chip(&mut self, item: Item) -> Option<&mut Chip> {
        self.chips.iter_mut().find(|(i, _)| *i == item).map(|(_, c)| c)
    }

    /// Stands the tools for `tool`: the one in hand chosen, the other at rest.
    pub fn choose(&mut self, grove: &mut Grove, tool: Tool) {
        self.tool = tool;
        let (select, connect) = match tool {
            Tool::Select => (Press::Chosen, Press::Rest),
            Tool::Connect => (Press::Rest, Press::Chosen),
        };
        if let Some(chip) = self.chip(Item::Select) {
            chip.arm(grove, select);
        }
        if let Some(chip) = self.chip(Item::Connect) {
            chip.arm(grove, connect);
        }
    }

    /// Counts the clashes on the clash press, which is the press to make while there are any.
    pub fn clashes(&mut self, grove: &mut Grove, n: usize) {
        for tally in &mut self.tallies {
            tally.set(grove, n as u32);
        }
        for item in [Item::Clashes, Item::ClashesNamed] {
            if let Some(chip) = self.chip(item) {
                chip.arm(grove, if n > 0 { Press::Armed } else { Press::Inert });
            }
        }
    }

    /// Whether there is anything to undo.
    pub fn undoable(&mut self, grove: &mut Grove, can: bool) {
        if let Some(chip) = self.chip(Item::Undo) {
            chip.arm(grove, Press::rest_if(can));
        }
    }

    /// Says the scale the sheet is read at.
    pub fn scale(&self, grove: &mut Grove, scale: f32) {
        grove.text(self.scale, format!("{}%", (scale * 100.0).round()));
    }

    /// Lights the press that opens a drawer while it is open.
    pub fn open(&mut self, grove: &mut Grove, view: bool, add: bool) {
        if let Some(chip) = self.chip(Item::View) {
            chip.arm(grove, if view { Press::Chosen } else { Press::Rest });
        }
        if let Some(chip) = self.chip(Item::AddMenu) {
            chip.arm(grove, if add { Press::Chosen } else { Press::Rest });
        }
    }

    /// The search field's words, put back or emptied.
    pub fn search(&mut self, grove: &mut Grove, words: &str) {
        self.searched = words.to_string();
        match words.is_empty() {
            true => self.field.clear(grove),
            false => self.field.put(grove, words),
        }
    }

    /// Says how many a search found, in the field's own cell.
    pub fn found(&self, grove: &mut Grove, n: Option<usize>) {
        grove.text(
            self.field.label(),
            match n {
                Some(n) if n > 99 => "99+".to_string(),
                Some(n) => n.to_string(),
                None => "find".to_string(),
            },
        );
    }

    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) -> Vec<Asked> {
        let mut asked = Vec::new();
        for (item, chip) in &self.chips {
            if !chip.pressed(pollen) {
                continue;
            }
            asked.push(match item {
                Item::Select => Asked::Tool(Tool::Select),
                Item::Connect => Asked::Tool(Tool::Connect),
                Item::Person => Asked::Add(Kind::Person),
                Item::Meeting => Asked::Add(Kind::Meeting),
                Item::Spec => Asked::Add(Kind::Spec),
                Item::Note => Asked::Add(Kind::Note),
                Item::AddMenu => Asked::AddMenu,
                Item::Arrange | Item::ArrangeNamed => Asked::Arrange,
                Item::Clashes | Item::ClashesNamed => Asked::Clashes,
                Item::Undo => Asked::Undo,
                Item::ZoomOut => Asked::Zoom(-1),
                Item::ZoomIn => Asked::Zoom(1),
                Item::View => Asked::View,
                Item::Find => Asked::Find,
                _ => continue,
            });
        }
        let mut keys = pollen.keys(self.plate).to_vec();
        for (_, chip) in &self.chips {
            keys.extend_from_slice(pollen.keys(chip.leaf()));
        }
        if !keys.is_empty() {
            asked.push(Asked::Keys(keys));
        }
        if self.field.frame(grove, pollen) {
            let words = self.field.value(grove);
            if words != self.searched {
                self.searched = words.clone();
                asked.push(Asked::Search(words));
            }
        }
        if pollen.submitted(self.field.input()) {
            asked.push(Asked::Go(self.field.value(grove)));
        }
        asked
    }
}
