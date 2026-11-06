//! trellis -- one planning sheet for the people, the meetings and the specs of a launch.
//!
//! *DWG TR-001*: Fernwood Studio's plan for the *orchard launch*, drawn as an engineering drawing
//! is, in graphite on vellum. People, meetings, specs, milestones and notes are its symbols and their links
//! its line types; a double-booking is boxed in red pencil, with a revision delta; where the plan
//! stands is its title block and revision block; time is its scale bar; the instruments lie over it
//! as a drafting machine's do. Everything is answered by a server that is not there, late, through a courier.

#[path = "../../../assets/gen/trellis/icons.rs"]
#[allow(dead_code)]
mod icons;

mod api;
mod arrange;
mod board;
mod clash;
mod dock;
mod drawers;
mod drawing;
mod inspector;
mod kit;
mod node;
mod rail;
mod sheet;
mod theme;
mod timeline;
mod undo;
mod wires;

use foliage::{
    Area, Boxed, Cap, Claim, Elevation, Grove, Grow, Key, Keystroke, Layout, Leaf, Line, Location, Panel, Place,
    Point, Pollen, Position, Root, Source, Stem, Timing, Tween, left, top,
};
use lichen::{Hover, Voice, drawn};

use arboretum::sprout::Courier;

use crate::api::{Busy, Rejected, Saved};
use crate::board::{Board, Id, Kind, Slot};
use crate::clash::Clashes;
use crate::dock::Dock;
use crate::drawers::Drawers;
use crate::drawing::{Drawing, Said, Tool};
use crate::icons::Icons;
use crate::inspector::Inspector;
use crate::rail::Rail;
use crate::sheet::Sheet;
use crate::theme::Faces;
use crate::timeline::Timeline;
use crate::undo::History;

type Loaded = (Board, u32, Vec<(u32, &'static str)>);

/// How long after the last change the board is filed, and how long the server takes.
const SETTLE_MS: u64 = 1200;
const FILE_MS: u64 = 900;
const CALENDAR_MS: u64 = 1300;
/// The time the session starts at on the sheet's own clock, in minutes after midnight: there is
/// no wall clock to ask, so the drawing keeps its own.
const CLOCK: u64 = 14 * 60 + 2;
/// Where the needle stands when the sheet opens: Monday, 09:15.
const NEEDLE: f32 = 2.5;

/// The leader from the inspector's bubble to the card it details.
/// Where a leader runs: from the plate, to its knee, to the card.
type Ends = ((f32, f32), (f32, f32), (f32, f32));

struct Leader {
    shoulder: Leaf,
    reach: Leaf,
    dot: Leaf,
    shown: bool,
    last: Option<Ends>,
}

struct Trellis {
    sheet: Sheet,
    drawing: Drawing,
    clashes: Clashes,
    rail: Rail,
    dock: Dock,
    timeline: Timeline,
    inspector: Inspector,
    drawers: Drawers,
    leader: Leader,
    hover: Hover,
    board: Board,
    history: History,
    revision: u32,
    revisions: Vec<(u32, String)>,
    loading: Courier<Loaded>,
    saving: Courier<(Vec<String>, Result<Saved, Rejected>)>,
    fixing: Courier<(usize, Id, Result<Slot, Busy>)>,
    moving: Courier<(Id, Result<Slot, Busy>)>,
    settle: Option<Tween>,
    loaded: bool,
    layout: Layout,
    tool: Tool,
    found: Option<Vec<Id>>,
    turn: usize,
    seen: Option<((f32, f32), f32, Area)>,
}

impl Root for Trellis {
    fn take_root(grove: &mut Grove) -> Self {
        grove.repaint(theme::scheme());
        let icons = grove.marks::<Icons>();
        let faces = Faces {
            regular: grove.font(include_bytes!("../../../assets/fonts/ibm-plex-mono/IBMPlexMono-Regular.ttf")),
            bold: grove.font(include_bytes!("../../../assets/fonts/ibm-plex-mono/IBMPlexMono-SemiBold.ttf")),
        };
        let window = grove.plant(Panel::new().color(theme::paper()).at(Location::new()));
        let sheet = Sheet::grow(grove, window, &faces, &icons);
        sheet.plot(grove);
        let drawing = Drawing::grow(grove, sheet.paper, &faces);
        let clashes = Clashes::grow(grove, sheet.paper, &faces, &icons);
        // The leader runs over the paper and under the instruments.
        let line = |grove: &mut Grove, weight: f32| {
            grove.branch(
                window,
                Line::new()
                    .between(Point::new(0.px(), 0.px()), Point::new(0.px(), 0.px()))
                    .weight(weight)
                    .color(theme::line())
                    .cap(Cap::Round)
                    .intangible()
                    .elevate(Elevation::up(20))
                    .visible(false),
            )
        };
        let leader = Leader {
            shoulder: line(grove, 1.25),
            reach: line(grove, 1.25),
            dot: grove.branch(
                window,
                kit::ring(1.5, theme::line())
                    .color(theme::paper())
                    .intangible()
                    .elevate(Elevation::up(21))
                    .visible(false)
                    .at(Location::new()),
            ),
            shown: false,
            last: None,
        };
        let instruments = grove.branch(window, Stem::new().intangible().elevate(Elevation::up(2)).at(Location::new()));
        let rail = Rail::grow(grove, instruments, &faces, &icons);
        let dock = Dock::grow(grove, instruments, &faces, &icons);
        let mut timeline = Timeline::grow(grove, instruments, &faces, &icons);
        let inspector = Inspector::grow(grove, instruments, &faces, &icons);
        let layer = grove.branch(window, Stem::new().intangible().elevate(Elevation::up(40)).at(Location::new()));
        let drawers = Drawers::grow(grove, layer, sheet.viewport, &faces, &icons);
        let hover = Hover::new(grove);
        timeline.needle(grove, NEEDLE);
        timeline.say.tell(grove, "fetching TR-001…", Voice::Quiet);
        // The board is the server's: asked for at once, and drawn when it comes.
        let mut loading = Courier::new(0x007e_1115);
        loading.send(grove, 900, api::load());
        let mut trellis = Trellis {
            sheet,
            drawing,
            clashes,
            rail,
            dock,
            timeline,
            inspector,
            drawers,
            leader,
            hover,
            board: Board::empty(),
            history: History::default(),
            revision: 0,
            revisions: Vec::new(),
            loading,
            saving: Courier::new(0x5a7e),
            fixing: Courier::new(0xf1c5),
            moving: Courier::new(0x0c10c),
            settle: None,
            loaded: false,
            layout: Layout::of(grove.viewport()),
            tool: Tool::Select,
            found: None,
            turn: 0,
            seen: None,
        };
        trellis.place(grove);
        trellis.rail.undoable(grove, false);
        trellis.rail.clashes(grove, 0);
        trellis
    }

    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        // A callout beside its card where the clear window has room for both, under it where not.
        let (l, _, r, _) = self.open_clear(grove);
        self.clashes.place(grove, clash::under(r - l));
        if pollen.resized().is_some() {
            let layout = Layout::of(grove.viewport());
            if layout != self.layout {
                self.layout = layout;
                self.place(grove);
            }
        }
        self.deliveries(grove, &pollen);
        let drawer_open = self.drawers.open();
        let mut asked_drawers = self.drawers.frame(grove, &pollen);
        let drawer_closed = asked_drawers.contains(&drawers::Asked::Closed) || (drawer_open && !self.drawers.open());
        for asked in asked_drawers.drain(..) {
            match asked {
                drawers::Asked::Add(kind) => self.add(grove, kind),
                drawers::Asked::Go(id) => self.go(grove, id),
                drawers::Asked::Search(words) => {
                    let found = self.search(grove, &words);
                    self.drawers.list(grove, &self.board, &found);
                    self.rail.search(grove, &words);
                    self.rail.found(grove, if words.trim().is_empty() { None } else { Some(found.len()) });
                }
                drawers::Asked::Closed => {}
            }
        }
        // The paper.
        let moved = self.sheet.frame(grove, &pollen);
        if moved.tapped || moved.grabbed {
            if moved.tapped {
                self.select(grove, None);
            }
            self.dock_open(grove, false);
        }
        self.hover.frame(grove, !self.drawers.open() && !self.drawing.handling() && !self.sheet.moving());
        let mut keys: Vec<Keystroke> = Vec::new();
        let said = self.drawing.frame(grove, &pollen, &mut self.board, &mut self.history, &self.hover, self.tool);
        for said in said {
            match said {
                Said::Selected(id) => self.select(grove, id),
                Said::Inspect(id) => {
                    self.select(grove, Some(id));
                    self.inspector.focus(grove);
                }
                Said::Changed => self.changed(grove),
                Said::Linked(link) => self.timeline.say.tell(grove, &format!("linked · {}", link.name()), Voice::Hint),
                Said::Refused(why) => self.timeline.say.tell(grove, why, Voice::Warn),
                Said::Delete(id) => self.delete(grove, id),
                Said::Keys(k) => keys.extend(k),
            }
        }
        // The instruments.
        for asked in self.rail.frame(grove, &pollen) {
            match asked {
                rail::Asked::Tool(tool) => {
                    self.tool = tool;
                    self.rail.choose(grove, tool);
                    let words = match tool {
                        Tool::Select => "select · drag a card to move it",
                        Tool::Connect => "connect · drag card to card",
                    };
                    self.timeline.say.tell(grove, words, Voice::Quiet);
                }
                rail::Asked::Add(kind) => self.add(grove, kind),
                rail::Asked::AddMenu => {
                    self.dock_open(grove, false);
                    self.drawers.adding(grove);
                }
                rail::Asked::Arrange => self.arrange(grove),
                rail::Asked::Clashes => self.next_clash(grove),
                rail::Asked::Undo => self.undo(grove),
                rail::Asked::Zoom(step) => self.zoom(grove, step),
                rail::Asked::View => {
                    let open = !self.dock.open;
                    if open {
                        self.close_inspector(grove);
                    }
                    self.dock_open(grove, open);
                }
                rail::Asked::Find => {
                    self.dock_open(grove, false);
                    self.drawers.finding(grove);
                }
                rail::Asked::Search(words) => {
                    let found = self.search(grove, &words);
                    self.rail.found(grove, if words.trim().is_empty() { None } else { Some(found.len()) });
                    self.drawers.search(grove, &words);
                    self.drawers.list(grove, &self.board, &found);
                }
                rail::Asked::Go(words) => {
                    if let Some(&id) = self.board.search(&words).first() {
                        self.go(grove, id);
                    } else if !words.trim().is_empty() {
                        self.timeline.say.tell(grove, "nothing on the sheet by that", Voice::Warn);
                    }
                }
                rail::Asked::Keys(k) => keys.extend(k),
            }
        }
        for asked in self.dock.frame(grove, &pollen) {
            match asked {
                dock::Asked::Layer(n, on) => {
                    self.drawing.layer(grove, n, on);
                    self.inspector_follows(grove);
                    self.plot(grove);
                    self.mark_clashes(grove);
                }
                dock::Asked::Look(at) => {
                    let (cx, cy) = self.middle(grove);
                    self.sheet.bring(grove, at, (cx, cy), 0);
                }
                dock::Asked::Zoom(step) => self.zoom(grove, step),
            }
        }
        let (needled, asked) = self.timeline.frame(grove, &pollen);
        if needled {
            let active = self.timeline.active(&self.board);
            self.drawing.activate(grove, &active);
        }
        for timeline::Asked::Move(id, slot) in asked {
            let code = self.board.get(id).map(|n| n.code.clone()).unwrap_or_default();
            self.timeline.say.tell(grove, &format!("holding {} for {code}…", slot.start_label()), Voice::Quiet);
            self.moving.send(grove, CALENDAR_MS, (id, api::reschedule(&self.board, id, slot)));
        }
        for clash::Asked::Fix { n, meeting, slot } in self.clashes.frame(&pollen) {
            self.clashes.asking(grove, &self.board, n);
            self.fixing.send(grove, CALENDAR_MS, (n, meeting, api::reschedule(&self.board, meeting, slot)));
        }
        for asked in self.inspector.frame(grove, &pollen, &mut self.board, &mut self.history) {
            match asked {
                inspector::Asked::Close => {
                    self.close_inspector(grove);
                    self.drawing.select(grove, None);
                }
                inspector::Asked::Edited => {
                    self.drawing.refresh(grove, &self.board);
                    self.changed(grove);
                }
                inspector::Asked::Delete(id) => self.delete(grove, id),
            }
        }
        // The sheet's own keys, from wherever focus is that is not a field.
        keys.extend_from_slice(pollen.root_keys());
        keys.extend_from_slice(pollen.keys(self.sheet.table));
        for leaf in self.dock.plates().into_iter().chain([self.timeline.plate()]).chain(self.inspector.keyed()) {
            keys.extend_from_slice(pollen.keys(leaf));
        }
        self.keys(grove, &keys);
        // Escape, where no drawer took it for its own.
        if pollen.escaped() && !drawer_open && !drawer_closed {
            self.escape(grove);
        }
        self.sheet.signpost(grove, &pollen);
        self.follow(grove);
    }
}

impl Trellis {
    /// Lays every instrument out for the window as it is now.
    fn place(&mut self, grove: &mut Grove) {
        let layout = self.layout;
        self.sheet.place(grove, layout);
        self.rail.place(grove, layout);
        self.dock.place(grove, layout);
        self.timeline.place(grove);
        self.inspector.place(grove, layout);
        self.seen = None;
        self.leader.last = None;
    }

    /// What has come back from the server this frame.
    fn deliveries(&mut self, grove: &mut Grove, pollen: &Pollen) {
        for (_, (board, revision, history)) in self.loading.frame(pollen) {
            self.board = board;
            self.revision = revision;
            self.revisions = history.into_iter().map(|(r, w)| (r, w.to_string())).collect();
            self.sheet.revised(grove, self.revision, &self.revisions);
            self.status(grove, &format!("issued · rev {}", api::letter(self.revision)));
            self.drawing.populate(grove, &self.board);
            for leaf in self.drawing.cards.iter().map(|(_, c)| c.root) {
                self.hover.watch(leaf);
            }
            self.timeline.say.tell(grove, &format!("rev {} · {} on the sheet", api::letter(self.revision), self.board.nodes.len()), Voice::Quiet);
            self.loaded = true;
            // The pencil comes out once the cards are down.
            self.clashes.lead = 1000;
            self.after(grove);
            let active = self.timeline.active(&self.board);
            self.drawing.activate(grove, &active);
        }
        for (_, (edits, answer)) in self.saving.frame(pollen) {
            match answer {
                Ok(saved) => {
                    self.revision = saved.revision;
                    self.revisions.push((saved.revision, saved.note));
                    self.sheet.revised(grove, self.revision, &self.revisions);
                    let filed = format!("saved · rev {} · {}", api::letter(self.revision), self.clock(grove));
                    self.status(grove, &filed);
                    self.timeline.say.tell(grove, &format!("filed rev {}", api::letter(self.revision)), Voice::Hint);
                    self.inspector.filed(grove, &self.board);
                }
                Err(Rejected(why)) => {
                    self.history.refile(edits);
                    self.status(grove, &format!("not filed · {why}"));
                    self.timeline.say.tell(grove, &theme::clip(&format!("rejected · {why}"), 30), Voice::Warn);
                }
            }
        }
        for (_, (n, meeting, answer)) in self.fixing.frame(pollen) {
            match answer {
                Ok(slot) => {
                    self.clashes.answered(grove, &self.board, n, None);
                    self.reschedule(grove, meeting, slot);
                    // A clash put right is a revision of its own, filed at once.
                    self.file(grove);
                }
                Err(busy) => {
                    self.clashes.answered(grove, &self.board, n, Some((busy.words(), busy.instead)));
                    self.timeline.say.tell(grove, &theme::clip(&busy.words(), 30), Voice::Warn);
                }
            }
        }
        for (_, (meeting, answer)) in self.moving.frame(pollen) {
            match answer {
                Ok(slot) => {
                    self.timeline.waiting = None;
                    self.reschedule(grove, meeting, slot);
                }
                Err(busy) => {
                    self.timeline.snap_back(grove);
                    self.timeline.say.tell(grove, &theme::clip(&busy.words(), 30), Voice::Warn);
                }
            }
        }
        if let Some(settle) = self.settle
            && pollen.finished(settle)
        {
            self.settle = None;
            self.file(grove);
        }
    }

    /// Moves `meeting` to `slot` on the board, as the server agreed.
    fn reschedule(&mut self, grove: &mut Grove, meeting: Id, slot: Slot) {
        let code = self.board.get(meeting).map(|n| n.code.clone()).unwrap_or_default();
        self.history.keep(&self.board, format!("{code} to {}", slot.start_label()));
        if let Some(node) = self.board.get_mut(meeting) {
            node.slot = Some(slot);
        }
        self.drawing.refresh(grove, &self.board);
        self.timeline.say.tell(grove, &format!("{code} moved to {}", slot.start_label()), Voice::Hint);
        self.changed(grove);
    }

    /// Everything that follows from the board changing: what clashes, the scale bar, the key plan,
    /// the rail -- and the board filed, once things have settled.
    fn changed(&mut self, grove: &mut Grove) {
        self.after(grove);
        if self.history.unfiled() {
            if let Some(settle) = self.settle.take() {
                grove.stop(settle);
            }
            self.settle = Some(grove.timer(Timing::ms(SETTLE_MS)));
            self.status(grove, "changed · not yet filed");
        }
    }

    fn after(&mut self, grove: &mut Grove) {
        self.mark_clashes(grove);
        self.rail.undoable(grove, self.history.can_undo());
        self.plot(grove);
        let active = self.timeline.active(&self.board);
        self.drawing.activate(grove, &active);
        if self.inspector.open {
            match self.inspector.id.filter(|id| self.board.get(*id).is_some()) {
                Some(_) => {}
                None => self.close_inspector(grove),
            }
        }
    }

    fn mark_clashes(&mut self, grove: &mut Grove) {
        let cards: Vec<(Id, Leaf)> = self
            .drawing
            .cards
            .iter()
            .filter(|(_, c)| c.shown)
            .map(|(id, c)| (*id, c.root))
            .collect();
        self.clashes
            .sync(grove, &self.board, &api::letter(self.revision + 1), |id| cards.iter().find(|(i, _)| *i == id).map(|(_, leaf)| *leaf));
        let (meetings, wires) = self.clashes.involved(&self.board);
        self.drawing.clashing = wires;
        self.drawing.relight(grove);
        self.timeline.sync(grove, &self.board, &meetings);
        self.rail.clashes(grove, self.clashes.all.len());
    }

    fn plot(&mut self, grove: &mut Grove) {
        let cards: Vec<(Id, bool, (f32, f32))> = self.drawing.cards.iter().map(|(id, c)| (*id, c.shown, c.at)).collect();
        self.dock.plot(
            grove,
            &self.board,
            |id| cards.iter().any(|(i, s, _)| *i == id && *s),
            |id| cards.iter().find(|(i, _, _)| *i == id).map(|(_, _, at)| *at),
        );
    }

    /// Files whatever has changed with the server.
    fn file(&mut self, grove: &mut Grove) {
        if self.saving.busy() {
            self.settle = Some(grove.timer(Timing::ms(SETTLE_MS)));
            return;
        }
        let edits = self.history.take();
        if edits.is_empty() {
            return;
        }
        let answer = api::sync(&self.board, self.revision, &edits);
        let words = match edits.len() {
            1 => "filing 1 change…".to_string(),
            n => format!("filing {n} changes…"),
        };
        self.status(grove, &words);
        self.saving.send(grove, FILE_MS, (edits, answer));
    }

    fn status(&mut self, grove: &mut Grove, words: &str) {
        self.sheet.status(grove, &theme::clip(words, 40));
    }

    /// The sheet's own clock, as `14:07`.
    fn clock(&self, grove: &Grove) -> String {
        let minutes = CLOCK + grove.elapsed().as_secs() / 60;
        format!("{:02}:{:02}", (minutes / 60) % 24, minutes % 60)
    }

    /// Picks out `id`'s card and opens its record, or lets both go.
    fn select(&mut self, grove: &mut Grove, id: Option<Id>) {
        self.drawing.select(grove, id);
        match self.drawing.selected {
            Some(id) => {
                self.dock_open(grove, false);
                self.inspector.show(grove, &self.board, id);
                self.leader.last = None;
                if !self.drawing.handling() {
                    self.reveal(grove, id);
                }
            }
            None => self.close_inspector(grove),
        }
    }

    fn close_inspector(&mut self, grove: &mut Grove) {
        self.inspector.close(grove);
        self.leader.last = None;
    }

    /// Keeps the inspector to a card that is still shown.
    fn inspector_follows(&mut self, grove: &mut Grove) {
        if self.inspector.open && self.drawing.selected.is_none() {
            self.close_inspector(grove);
        }
    }

    fn dock_open(&mut self, grove: &mut Grove, open: bool) {
        self.dock.show(grove, open);
        self.rail.open(grove, self.dock.open, false);
    }

    /// The window's clear middle: what the instruments leave of it.
    fn clear(&self, grove: &Grove) -> (f32, f32, f32, f32) {
        let view = grove.viewport();
        let left = match theme::dock(self.layout) {
            0.0 => 0.0,
            dock => theme::GUTTER * 2.0 + dock,
        };
        let right = match (theme::docked(self.layout), self.inspector.open) {
            (Some(width), true) => view.width - theme::GUTTER * 2.0 - width,
            _ => view.width,
        };
        (left, theme::TOP, right.max(left + 40.0), (view.height - theme::BOTTOM).max(theme::TOP + 40.0))
    }

    /// What of the window is left clear once a risen inspector is counted too.
    fn open_clear(&self, grove: &Grove) -> (f32, f32, f32, f32) {
        let (l, t, r, b) = self.clear(grove);
        let b = match (self.inspector.open, self.inspector.docked()) {
            (true, false) => (b - self.inspector.extent() - theme::GUTTER).max(t + 60.0),
            _ => b,
        };
        (l, t, r, b)
    }

    fn middle(&self, grove: &Grove) -> (f32, f32) {
        let (l, t, r, b) = self.open_clear(grove);
        ((l + r) / 2.0, (t + b) / 2.0)
    }

    /// Slides the paper just far enough that `id`'s card stands clear of the instruments -- with
    /// the callout beside it, if it is a meeting a clash would move and there is room for both.
    fn reveal(&mut self, grove: &mut Grove, id: Id) {
        let Some(card) = self.drawing.card(id) else {
            return;
        };
        let clear = self.open_clear(grove);
        let (w, h) = card.size();
        let called = self.sheet.scale >= theme::OVERVIEW && self.clashes.all.iter().take(clash::SHOWN).any(|c| c.moves == id);
        let both = match self.clashes.under() {
            false => (w + clash::CALLOUT_GAP + clash::CALLOUT.0, h.max(clash::CALLOUT.1)),
            true => (w, h + clash::UNDER_GAP + clash::CALLOUT.1),
        };
        let size = match called && both.0 + 40.0 <= clear.2 - clear.0 {
            true => both,
            false => (w, h),
        };
        self.sheet.reveal(grove, card.at, size, clear, 480);
    }

    /// Brings `id`'s card into the middle of the view, picks it out, and rings it.
    fn go(&mut self, grove: &mut Grove, id: Id) {
        let Some(card) = self.drawing.card(id) else {
            return;
        };
        if !card.shown {
            let kind = card.kind.index();
            self.drawing.layer(grove, kind, true);
            self.dock.set(grove, kind, true);
        }
        self.select(grove, Some(id));
        let Some(card) = self.drawing.card(id) else {
            return;
        };
        let (w, h) = card.size();
        let scale = self.sheet.scale;
        let at = (card.at.0 + w / 2.0 / scale, card.at.1 + h / 2.0 / scale);
        let middle = self.middle(grove);
        self.sheet.bring(grove, at, middle, 700);
        self.drawing.find(grove, id, 520);
    }

    fn search(&mut self, grove: &mut Grove, words: &str) -> Vec<Id> {
        let found = self.board.search(words);
        let sift = !words.trim().is_empty();
        self.drawing.sift(grove, sift.then_some(found.as_slice()));
        self.found = sift.then(|| found.clone());
        found
    }

    fn add(&mut self, grove: &mut Grove, kind: Kind) {
        let middle = self.middle(grove);
        let at = self.sheet.to_sheet(grove, Position::new(middle.0, middle.1));
        let (bounds, reserve) = (self.bounds(), self.reserve());
        let id = self.drawing.add(grove, &mut self.board, &mut self.history, kind, at, &bounds, &reserve);
        if let Some(card) = self.drawing.card(id) {
            self.hover.watch(card.root);
        }
        self.changed(grove);
        self.select(grove, Some(id));
        self.inspector.focus(grove);
        let code = self.board.get(id).map(|n| n.code.clone()).unwrap_or_default();
        self.timeline.say.tell(grove, &format!("added {code} · name it"), Voice::Hint);
    }

    fn delete(&mut self, grove: &mut Grove, id: Id) {
        let code = self.board.get(id).map(|n| n.code.clone()).unwrap_or_default();
        self.drawing.remove(grove, &mut self.board, &mut self.history, id);
        self.close_inspector(grove);
        self.changed(grove);
        self.timeline.say.tell(grove, &format!("deleted {code} · undo restores"), Voice::Quiet);
    }

    /// The room the markup takes round each card it is drawn about, on the sheet: what arranging
    /// keeps clear, and what a card added keeps clear of.
    fn reserve(&self) -> Vec<(Id, drawing::Margins)> {
        // A clash's boxes stand round both its meetings, and its callout to the right of the one
        // it moves: each wants its room, in the window's pixels, which the sheet's are not -- the
        // most it takes of the sheet at any scale it is drawn at.
        let (brief, full) = theme::least();
        let mut reserve: Vec<(Id, drawing::Margins)> = Vec::new();
        let mut want = |id: Id, room: drawing::Margins| match reserve.iter_mut().find(|(i, _)| *i == id) {
            Some((_, m)) => *m = (m.0.max(room.0), m.1.max(room.1), m.2.max(room.2), m.3.max(room.3)),
            None => reserve.push((id, room)),
        };
        let at = |(l, t, r, b): (f32, f32, f32, f32)| (l / brief, t / brief, r / brief, b / brief);
        let tall = Kind::Meeting.footprint().1;
        // The callout is drawn only in full: beside the card, or under it, as wide as its box, as
        // the window has room.
        let (callout, below) = match self.clashes.under() {
            false => (clash::CALLOUT.0 + clash::CALLOUT_GAP, clash::CALLOUT.1 - tall),
            true => (0.0, clash::UNDER_GAP + clash::CALLOUT.1),
        };
        for c in self.clashes.all.iter().take(clash::SHOWN) {
            want(c.keeps, at(clash::room(false)));
            let (l, t, r, b) = at(clash::room(true));
            want(c.moves, (l, t, r.max(callout / full), b.max(below / full)));
        }
        reserve
    }

    fn arrange(&mut self, grove: &mut Grove) {
        let bounds = self.bounds();
        let reserve = self.reserve();
        self.drawing.arrange(grove, &mut self.board, &mut self.history, &bounds, &reserve);
        self.timeline.say.tell(grove, "arranging the sheet…", Voice::Quiet);
    }

    fn undo(&mut self, grove: &mut Grove) {
        let Some((board, what)) = self.history.undo() else {
            self.timeline.say.tell(grove, "nothing to undo", Voice::Quiet);
            return;
        };
        self.board = board;
        self.drawing.refresh(grove, &self.board);
        if let Some(id) = self.inspector.id.filter(|id| self.board.get(*id).is_some()) {
            self.inspector.show(grove, &self.board, id);
        }
        self.changed(grove);
        self.timeline.say.tell(grove, &theme::clip(&format!("undid {what}"), 30), Voice::Quiet);
    }

    /// Takes the view to the next clash, in turn.
    fn next_clash(&mut self, grove: &mut Grove) {
        let all = self.clashes.all.clone();
        if all.is_empty() {
            self.timeline.say.tell(grove, "no clashes", Voice::Hint);
            return;
        }
        let clash = &all[self.turn % all.len()];
        self.turn += 1;
        let words = self.board.clash_words(clash);
        // A clash is put right from its callout, which an overview does not draw.
        if self.sheet.scale < theme::OVERVIEW {
            self.zoom_to(grove, 1.0);
        }
        self.go(grove, clash.moves);
        // And the callout is brought in with its meeting, the two -- and the box's delta -- in the
        // middle of the clear window: side by side, or the callout under the meeting where the
        // window has no room for both across.
        if let Some(card) = self.drawing.card(clash.moves) {
            let (w, h) = card.size();
            let (l, _, r, _) = self.open_clear(grove);
            let (across, down) = match self.clashes.under() {
                false => (w + clash::CALLOUT_GAP + clash::CALLOUT.0, 0.0),
                // The card stands above the middle by half what the callout under it takes more
                // than the delta over it.
                true => (w + clash::SIDE, (clash::UNDER_GAP + clash::CALLOUT.1 - clash::room(true).1) / 2.0),
            };
            let left = (l + (r - l - across - clash::REACH) / 2.0).max(l + 6.0) + clash::REACH;
            let scale = self.sheet.scale;
            let at = (card.at.0 + w / 2.0 / scale, card.at.1 + h / 2.0 / scale);
            let middle = self.middle(grove);
            self.sheet.bring(grove, at, (left + w / 2.0, middle.1 - down), 700);
        }
        self.timeline.say.tell(grove, &words, Voice::Warn);
    }

    fn zoom(&mut self, grove: &mut Grove, step: i32) {
        let now = theme::SCALES.iter().position(|s| (*s - self.sheet.scale).abs() < 1e-3).unwrap_or(2) as i32;
        let to = (now + step).clamp(0, theme::SCALES.len() as i32 - 1) as usize;
        self.zoom_to(grove, theme::SCALES[to]);
    }

    /// Reads the sheet at `scale`, about the middle of the clear window.
    fn zoom_to(&mut self, grove: &mut Grove, scale: f32) {
        if (scale - self.sheet.scale).abs() < 1e-3 {
            return;
        }
        let middle = self.middle(grove);
        self.sheet.zoom(grove, scale, middle);
        self.drawing.rescale(grove, scale);
        self.rail.scale(grove, scale);
        self.dock.scale(grove, scale);
        self.plot(grove);
        self.clashes.overview(grove, scale < theme::OVERVIEW);
        self.mark_clashes(grove);
    }

    fn keys(&mut self, grove: &mut Grove, keys: &[Keystroke]) {
        for stroke in keys {
            match (stroke.key, stroke.modifiers.control) {
                (Key::Typed('z' | 'Z'), true) => self.undo(grove),
                (Key::Typed('+' | '='), false) => self.zoom(grove, 1),
                (Key::Typed('-'), false) => self.zoom(grove, -1),
                (Key::Typed('c'), false) => {
                    self.tool = Tool::Connect;
                    self.rail.choose(grove, Tool::Connect);
                }
                (Key::Typed('v'), false) => {
                    self.tool = Tool::Select;
                    self.rail.choose(grove, Tool::Select);
                }
                _ => {}
            }
        }
    }

    fn escape(&mut self, grove: &mut Grove) {
        if self.dock.open {
            self.dock_open(grove, false);
        } else if self.inspector.open || self.drawing.selected.is_some() {
            self.select(grove, None);
        } else if self.found.is_some() {
            self.rail.search(grove, "");
            self.rail.found(grove, None);
            self.drawers.search(grove, "");
            self.search(grove, "");
        }
    }

    /// Where arranging may put cards: the drawing, less its blocks.
    fn bounds(&self) -> arrange::Bounds {
        let (w, h) = theme::SHEET;
        let m = theme::INNER + 24.0;
        let pad = |(l, t, r, b): (f32, f32, f32, f32)| (l - 8.0, t - 8.0, r + 8.0, b + 8.0);
        arrange::Bounds {
            left: m,
            top: m,
            right: w - m,
            bottom: h - m,
            // The blocks as they stand read smallest, where they take the most of the sheet.
            keep_out: vec![pad(theme::title_block(theme::least().0)), pad(theme::revision_block(theme::least().0))],
        }
    }

    /// What follows the view each frame: the key plan's frame, and the inspector's leader.
    fn follow(&mut self, grove: &mut Grove) {
        let offset = self.sheet.offset(grove);
        let view = grove.viewport();
        let seen = (offset, self.sheet.scale, view);
        if self.seen != Some(seen) {
            self.seen = Some(seen);
            let (l, t, r, b) = self.clear(grove);
            let a = self.sheet.to_sheet(grove, Position::new(l, t));
            let z = self.sheet.to_sheet(grove, Position::new(r, b));
            self.dock.view(grove, (a.0, a.1, z.0, z.1));
            self.leader.last = None;
        }
        self.lead(grove);
    }

    /// Runs the leader from the inspector's bubble to its card, or takes it away.
    fn lead(&mut self, grove: &mut Grove) {
        let target = self
            .inspector
            .id
            .filter(|_| self.inspector.open)
            .and_then(|id| self.drawing.card(id))
            .filter(|c| c.shown)
            .map(|c| c.root);
        let ends = match (target.and_then(|t| drawn(grove, t)), drawn(grove, self.inspector.bubble), drawn(grove, self.inspector.plate())) {
            (Some(card), Some(bubble), Some(plate)) => {
                let docked = self.inspector.docked();
                let (start, knee) = match docked {
                    true => {
                        let y = bubble.center().y;
                        ((plate.left(), y), (plate.left() - 22.0, y))
                    }
                    false => {
                        let x = bubble.center().x;
                        ((x, plate.top()), (x, plate.top() - 22.0))
                    }
                };
                let near = (knee.0.clamp(card.left(), card.right()), knee.1.clamp(card.top(), card.bottom()));
                let (l, t, r, b) = self.clear(grove);
                let inside = near.0 > l - 4.0 && near.0 < r + 4.0 && near.1 > t - 4.0 && near.1 < b + 4.0;
                let under = plate.contains(Position::new(near.0, near.1));
                (inside && !under).then_some((start, knee, near))
            }
            _ => None,
        };
        if ends == self.leader.last && ends.is_some() == self.leader.shown {
            return;
        }
        self.leader.last = ends;
        match ends {
            Some((start, knee, near)) => {
                let p = |a: (f32, f32)| Point::new(a.0.px(), a.1.px());
                grove.between(self.leader.shoulder, p(start), p(knee));
                grove.between(self.leader.reach, p(knee), p(near));
                grove.at(
                    self.leader.dot,
                    Location::new().xs(
                        left((near.0 - 4.0).px()).width(8.px()),
                        top((near.1 - 4.0).px()).height(8.px()),
                    ),
                );
                if !self.leader.shown {
                    self.leader.shown = true;
                    for leaf in [self.leader.shoulder, self.leader.reach, self.leader.dot] {
                        grove.visible(leaf, true);
                    }
                }
            }
            None => {
                if self.leader.shown {
                    self.leader.shown = false;
                    for leaf in [self.leader.shoulder, self.leader.reach, self.leader.dot] {
                        grove.visible(leaf, false);
                    }
                }
            }
        }
    }
}

fn main() {
    arboretum::boot::<Trellis>("trellis", Area::new(1280.0, 800.0), |foliage| {
        foliage.tune(Claim {
            horizontal: 6.0,
            vertical: 6.0,
        });
    });
}
