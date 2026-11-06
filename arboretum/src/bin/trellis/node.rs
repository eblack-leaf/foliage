//! The cards on the sheet, one per node, each drawn in the drafting symbol for what it is: a person
//! as an initials stamp, a meeting as a slot card under a reversed time strip, a spec as a callout
//! headed by its code and stamped with its status, a milestone as a diamond, and a note as a dashed
//! box.
//!
//! A card is drawn in full or in brief. Read at half scale, the sheet is a map rather than a page,
//! so each card keeps only what it is called and draws everything else out of the way: nothing is
//! grown again, only placed and hidden.

use foliage::{
    Area, Axes, Boxed, Color, Elevation, Fill, Grove, Grow, Leaf, Location, Motion, Panel, Place,
    Polygon, Position, Source, Timing, center_x, center_y, content, left, right, top,
};

use lichen::{Dashes, Stamp};

use crate::board::{Kind, Node, Status};
use crate::kit::{self, letter};
use crate::theme::{self, Faces, size};

/// How a card is drawn.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Detail {
    Full,
    Brief,
}

/// One part of a card, and where it stands in full and in brief -- or that it is not drawn.
struct Part {
    leaf: Leaf,
    full: Option<Location>,
    brief: Option<Location>,
}

pub struct NodeView {
    pub kind: Kind,
    /// The card: what is pressed, dragged and focused.
    pub root: Leaf,
    parts: Vec<Part>,
    title: Leaf,
    sub: Option<Leaf>,
    initials: Option<Leaf>,
    strip: Option<(Leaf, Leaf)>,
    stamp: Option<Stamp>,
    /// A note's dashed outline, and its box, hidden with the card.
    dashes: Option<(Dashes, Leaf)>,
    /// Where the card stands now, on the sheet at full scale -- the board's word for it, or where
    /// a drag or an arrangement has it on the way.
    pub at: (f32, f32),
    pub detail: Detail,
    pub shown: bool,
    pub active: bool,
    pub dim: bool,
}

/// The status a stamp says, in the ink it says it in.
fn stamp_ink(status: Status) -> Color {
    match status {
        Status::Draft => theme::faint(),
        Status::Review => theme::marked(),
        Status::Approved => theme::line(),
    }
}

/// What a card says under its title.
fn sub_words(node: &Node, attending: usize) -> String {
    match node.kind {
        Kind::Person => theme::clip(&node.role, 19),
        Kind::Meeting => {
            let length = node.slot.map(|s| s.length()).unwrap_or("");
            match attending {
                0 => format!("nobody yet · {length}"),
                n => format!("{n} attending · {length}"),
            }
        }
        Kind::Milestone => format!("{} · {}", theme::clip(&node.title, 14), crate::board::DAYS[node.day as usize % 5]),
        _ => String::new(),
    }
}

fn title_words(node: &Node) -> String {
    match node.kind {
        Kind::Note => theme::clip(&node.title, 46),
        Kind::Milestone => node.code.clone(),
        Kind::Spec => theme::clip(&node.title.to_uppercase(), 24),
        _ => theme::clip(&node.title.to_uppercase(), 22),
    }
}

impl NodeView {
    pub fn grow(grove: &mut Grove, paper: Leaf, node: &Node, attending: usize, faces: &Faces) -> Self {
        let (w, h) = node.kind.footprint();
        let outlined = matches!(node.kind, Kind::Meeting | Kind::Spec);
        let card = Panel::new().color(theme::paper());
        let card = if outlined { card.outline(1.25, theme::line()) } else { card };
        let root = grove.branch(
            paper,
            card.interactive()
                .drags(Axes::Both)
                .elevate(Elevation::up(8))
                .opacity(0.0)
                .at(Location::new().xs(left(node.at.0.px()).width(w.px()), top(node.at.1.px()).height(h.px()))),
        );
        let ink = theme::line();
        let soft = theme::faint();
        let mut parts = Vec::new();
        let mut part = |leaf: Leaf, full: Option<Location>, brief: Option<Location>| {
            parts.push(Part { leaf, full, brief });
        };
        let at = |x: f32, y: f32| Location::new().xs(left(x.px()).width(content()), top(y.px()).height(1.letters()));
        let across = |x: f32, y: f32| {
            Location::new().xs(left(x.px()).right(100.pct() - 8.px()), top(y.px()).height(content()))
        };
        let mut sub = None;
        let mut initials = None;
        let mut strip = None;
        let mut stamp = None;
        let mut dashes = None;
        let title;
        match node.kind {
            Kind::Person => {
                let ring = grove.branch(
                    root,
                    kit::ring(1.25, ink)
                        .intangible()
                        .elevate(Elevation::up(1))
                        .at(Location::new().xs(left(6.px()).width(40.px()), center_y(50.pct()).height(40.px()))),
                );
                part(
                    ring,
                    Some(Location::new().xs(left(6.px()).width(40.px()), center_y(50.pct()).height(40.px()))),
                    Some(Location::new().xs(left(2.px()).width(26.px()), center_y(50.pct()).height(26.px()))),
                );
                initials = Some(letter(
                    grove,
                    ring,
                    node.initials(),
                    faces.bold,
                    theme::SUB,
                    ink,
                    Location::new().xs(center_x(50.pct()).width(content()), center_y(50.pct()).height(1.letters())),
                ));
                title = letter(grove, root, title_words(node), faces.bold, theme::TITLE, ink, at(54.0, 9.0));
                part(title, Some(at(54.0, 9.0)), Some(Location::new().xs(left(36.px()).width(content()), center_y(50.pct()).height(1.letters()))));
                let role = letter(grove, root, sub_words(node, attending), faces.regular, theme::SUB, soft, at(54.0, 28.0));
                part(role, Some(at(54.0, 28.0)), None);
                sub = Some(role);
            }
            Kind::Meeting => {
                let band = grove.branch(
                    root,
                    Panel::new()
                        .color(ink)
                        .intangible()
                        .elevate(Elevation::up(1))
                        .at(Location::new().xs(left(0.px()).right(100.pct()), top(0.px()).height(20.px()))),
                );
                part(
                    band,
                    Some(Location::new().xs(left(0.px()).right(100.pct()), top(0.px()).height(20.px()))),
                    Some(Location::new()),
                );
                let words = node.slot.map(|s| s.label()).unwrap_or_default();
                let when = letter(
                    grove,
                    band,
                    words,
                    faces.bold,
                    theme::SUB,
                    theme::paper(),
                    Location::new().xs(left(8.px()).width(content()), center_y(50.pct()).height(1.letters())),
                );
                strip = Some((band, when));
                title = letter(grove, root, title_words(node), faces.bold, theme::TITLE, ink, at(8.0, 27.0));
                part(title, Some(at(8.0, 27.0)), None);
                let count = letter(grove, root, sub_words(node, attending), faces.regular, theme::SUB, soft, at(8.0, 45.0));
                part(count, Some(at(8.0, 45.0)), None);
                sub = Some(count);
            }
            Kind::Spec => {
                let code = letter(grove, root, node.code.clone(), faces.bold, theme::TITLE, ink, at(8.0, 6.0));
                part(
                    code,
                    Some(at(8.0, 6.0)),
                    Some(Location::new().xs(left(8.px()).width(content()), center_y(50.pct()).height(1.letters()))),
                );
                for y in [24.0, 27.0] {
                    let rule = kit::rule(
                        grove,
                        root,
                        Location::new().xs(left(0.px()).right(100.pct()), top(y.px()).height(1.px())),
                        ink,
                    );
                    part(rule, Some(Location::new().xs(left(0.px()).right(100.pct()), top(y.px()).height(1.px()))), None);
                }
                title = letter(grove, root, title_words(node), faces.regular, theme::TITLE, ink, at(8.0, 35.0));
                part(title, Some(at(8.0, 35.0)), None);
                let seal = grove.branch(
                    root,
                    foliage::Stem::new()
                        .intangible()
                        .elevate(Elevation::up(1))
                        .at(Location::new().xs(right(100.pct() - 10.px()).width(70.px()), top(55.px()).height(15.px()))),
                );
                part(seal, Some(Location::new().xs(right(100.pct() - 10.px()).width(70.px()), top(55.px()).height(15.px()))), None);
                let mut s = Stamp::grow(
                    grove,
                    seal,
                    Location::new(),
                    node.status.word(),
                    size(9),
                    Some(faces.bold),
                    stamp_ink(node.status),
                );
                s.press(grove, 400);
                stamp = Some(s);
            }
            Kind::Milestone => {
                let holder = grove.branch(
                    root,
                    foliage::Stem::new()
                        .intangible()
                        .elevate(Elevation::up(1))
                        .at(Location::new().xs(left(4.px()).width(40.px()), center_y(50.pct()).height(40.px()))),
                );
                part(
                    holder,
                    Some(Location::new().xs(left(4.px()).width(40.px()), center_y(50.pct()).height(40.px()))),
                    Some(Location::new().xs(left(2.px()).width(26.px()), center_y(50.pct()).height(26.px()))),
                );
                // A ruled diamond with a solid one at its heart: the milestone's mark.
                grove.branch(
                    holder,
                    Polygon::new().sides(4.0).color(ink).intangible().at(Location::new()),
                );
                grove.branch(
                    holder,
                    Polygon::new()
                        .sides(4.0)
                        .color(theme::paper())
                        .intangible()
                        .elevate(Elevation::up(1))
                        .at(Location::new().xs(center_x(50.pct()).width(82.pct()), center_y(50.pct()).height(82.pct()))),
                );
                grove.branch(
                    holder,
                    Polygon::new()
                        .sides(4.0)
                        .color(ink)
                        .intangible()
                        .elevate(Elevation::up(2))
                        .at(Location::new().xs(center_x(50.pct()).width(34.pct()), center_y(50.pct()).height(34.pct()))),
                );
                title = letter(grove, root, title_words(node), faces.bold, theme::TITLE, ink, at(50.0, 9.0));
                part(title, Some(at(50.0, 9.0)), Some(Location::new().xs(left(34.px()).width(content()), center_y(50.pct()).height(1.letters()))));
                let line = letter(grove, root, sub_words(node, attending), faces.regular, theme::SUB, soft, at(50.0, 28.0));
                part(line, Some(at(50.0, 28.0)), None);
                sub = Some(line);
            }
            Kind::Note => {
                let code = letter(grove, root, node.code.clone(), faces.bold, theme::LABEL + 1, soft, at(8.0, 6.0));
                part(
                    code,
                    Some(at(8.0, 6.0)),
                    Some(Location::new().xs(left(8.px()).width(content()), center_y(50.pct()).height(1.letters()))),
                );
                title = letter(grove, root, title_words(node), faces.regular, theme::SUB, ink, across(8.0, 20.0));
                part(title, Some(across(8.0, 20.0)), None);
                let mut d = Dashes::new(Area::new(w, h))
                    .pitch(11.5)
                    .color(theme::faint())
                    .elevate(Elevation::up(8));
                let outline = d.grow(grove, paper);
                d.over(grove, root);
                dashes = Some((d, outline));
            }
        }
        Self {
            kind: node.kind,
            root,
            parts,
            title,
            sub,
            initials,
            strip,
            stamp,
            dashes,
            at: node.at,
            detail: Detail::Full,
            shown: false,
            active: false,
            dim: false,
        }
    }

    /// How large the card is drawn, in sheet pixels -- which do not scale, since its lettering
    /// does not.
    pub fn size(&self) -> (f32, f32) {
        match self.detail {
            Detail::Full => self.kind.footprint(),
            Detail::Brief => self.kind.brief(),
        }
    }

    /// The card's box in the sheet's pixels at `scale`: its corner scales, its size does not.
    pub fn drawn(&self, scale: f32) -> (f32, f32, f32, f32) {
        let (w, h) = self.size();
        let (x, y) = (self.at.0 * scale, self.at.1 * scale);
        (x, y, x + w, y + h)
    }

    /// Stands the card at `at` on the sheet, read at `scale`.
    pub fn place(&mut self, grove: &mut Grove, at: (f32, f32), scale: f32) {
        self.at = at;
        let (w, h) = self.size();
        grove.at(
            self.root,
            Location::new().xs(
                left((at.0 * scale).round().px()).width(w.px()),
                top((at.1 * scale).round().px()).height(h.px()),
            ),
        );
    }

    /// Draws the card in full or in brief.
    pub fn detail(&mut self, grove: &mut Grove, detail: Detail, scale: f32) {
        if self.detail == detail {
            return;
        }
        self.detail = detail;
        for part in &self.parts {
            let at = match detail {
                Detail::Full => &part.full,
                Detail::Brief => &part.brief,
            };
            match at {
                Some(location) => {
                    grove.at(part.leaf, location.clone());
                    grove.visible(part.leaf, true);
                }
                None => grove.visible(part.leaf, false),
            }
        }
        self.place(grove, self.at, scale);
    }

    /// Writes what the card says from `node`.
    pub fn refresh(&mut self, grove: &mut Grove, node: &Node, attending: usize) {
        grove.text(self.title, title_words(node));
        if let Some(sub) = self.sub {
            grove.text(sub, sub_words(node, attending));
        }
        if let Some(initials) = self.initials {
            grove.text(initials, node.initials());
        }
        if let Some((_, when)) = self.strip {
            grove.text(when, node.slot.map(|s| s.label()).unwrap_or_default());
        }
        if let Some(stamp) = &mut self.stamp {
            stamp.say(grove, node.status.word(), stamp_ink(node.status));
        }
    }

    /// Shows the card or hides it outright, with its dashes.
    pub fn show(&mut self, grove: &mut Grove, shown: bool) {
        if self.shown == shown {
            return;
        }
        self.shown = shown;
        grove.visible(self.root, shown);
        if shown {
            grove.opacity(self.root, if self.dim { 0.28 } else { 1.0 });
        }
        if let Some((_, outline)) = self.dashes {
            grove.visible(outline, shown);
        }
    }

    /// Brings the card onto the sheet, `after` ms from now: up from nothing, dropping a few
    /// pixels into place as a stamp is pressed.
    pub fn arrive(&mut self, grove: &mut Grove, after: u64) {
        self.shown = true;
        grove.visible(self.root, true);
        grove.opacity(self.root, 0.0);
        grove.shift(self.root, Position::new(0.0, -6.0));
        let timing = Timing::ms(320).after(after).ease(foliage::Ease::Decelerate);
        grove.animate(self.root, Motion::Opacity(1.0), timing);
        grove.animate(self.root, Motion::Shift(Position::default()), timing);
        if let Some((dashes, outline)) = &mut self.dashes {
            grove.visible(*outline, true);
            dashes.draw(grove, true);
        }
    }

    /// Lights a meeting's time strip with the highlighter while it is under the scale bar's
    /// needle.
    pub fn activate(&mut self, grove: &mut Grove, active: bool) {
        if self.active == active {
            return;
        }
        self.active = active;
        if let Some((band, when)) = self.strip {
            let timing = Timing::ms(220).ease(foliage::Ease::Decelerate);
            let (fill, words): (Fill, Fill) = match active {
                true => (theme::marker().into(), theme::line().into()),
                false => (theme::line().into(), theme::paper().into()),
            };
            grove.animate(band, Motion::from(fill), timing);
            grove.animate(when, Motion::from(words), timing);
        }
    }

    /// Quiets the card, while a search finds other things.
    pub fn quiet(&mut self, grove: &mut Grove, dim: bool) {
        if self.dim == dim {
            return;
        }
        self.dim = dim;
        if self.shown {
            grove.animate(
                self.root,
                Motion::Opacity(if dim { 0.28 } else { 1.0 }),
                Timing::ms(200).ease(foliage::Ease::Decelerate),
            );
        }
    }
}
