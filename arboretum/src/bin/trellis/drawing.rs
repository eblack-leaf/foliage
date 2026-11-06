//! What is drawn on the sheet, and what a hand does to it: the cards and their wires, picking one
//! out, dragging it about, drawing a new link, arranging the lot, and showing or hiding a layer.
//!
//! The board is the truth and this is its picture. Every change a hand makes is written to the
//! board, with the board as it was kept for undoing, and reported upward so the sheet can file it.

use foliage::{
    Boxed, Ease, Elevation, Grove, Grow, Keystroke, Leaf, Location, Motion, Place, Pollen, Position,
    Rounding, Source, Stem, Timing, Tween, anchor, left, top,
};

use arboretum::sprout::{Hover, topmost};
use lichen::drawn;

use crate::arrange::{self, Body, Bounds};
use crate::board::{Board, Id, Kind, Link};
use crate::kit;
use crate::node::{Detail, NodeView};
use crate::theme::{self, Faces};
use crate::undo::History;
use crate::wires::{Lit, Rect, Wires};

/// What a drag on a card does.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tool {
    /// Moves it.
    Select,
    /// Draws a link from it.
    Connect,
}

/// What happened on the drawing this frame.
#[derive(Clone, Debug, PartialEq)]
pub enum Said {
    /// A card was picked out, or the pick was let go.
    Selected(Option<Id>),
    /// A card was tapped twice: its inspector is wanted.
    Inspect(Id),
    /// The board changed under a hand.
    Changed,
    /// A link was drawn.
    Linked(Link),
    /// A link could not be drawn, and why.
    Refused(&'static str),
    /// Delete was pressed on a card.
    Delete(Id),
    /// Keys pressed on a card or on the paper, for the sheet to make something of.
    Keys(Vec<Keystroke>),
}

/// The layers a sheet can show or hide: each kind of card, then the wires.
pub const LAYERS: usize = 6;
pub const WIRES: usize = 5;

/// A card's way home when the sheet is arranged: where it sets out from and where it goes.
type Path = (Id, (f32, f32), (f32, f32));
/// Room kept round a card, `(left, top, right, bottom)` on the sheet.
pub type Margins = (f32, f32, f32, f32);

pub struct Drawing {
    paper: Leaf,
    pub cards: Vec<(Id, NodeView)>,
    pub wires: Wires,
    port: Leaf,
    port_on: Option<Id>,
    marks: Leaf,
    marks_on: Option<Id>,
    hover_marks: Leaf,
    hover_on: Option<Id>,
    ring: Leaf,
    pub selected: Option<Id>,
    moving: Option<(Id, (f32, f32), bool)>,
    linking: Option<Id>,
    reach: Option<Position>,
    arranging: Option<(Tween, Vec<Path>)>,
    pub layers: [bool; LAYERS],
    pub scale: f32,
    pub clashing: Vec<(Id, Id)>,
    dirty: bool,
    faces: Faces,
}

impl Drawing {
    pub fn grow(grove: &mut Grove, paper: Leaf, faces: &Faces) -> Self {
        let marks_at = |pad: f32| {
            Location::new().xs(
                left(anchor().left() - pad.px()).right(anchor().right() + pad.px()),
                top(anchor().top() - pad.px()).bottom(anchor().bottom() + pad.px()),
            )
        };
        let marks = grove.branch(
            paper,
            Stem::new().intangible().elevate(Elevation::up(10)).visible(false).at(marks_at(3.0)),
        );
        kit::ticks(grove, marks, theme::line(), 1.5);
        let hover_marks = grove.branch(
            paper,
            Stem::new().intangible().elevate(Elevation::up(10)).visible(false).at(marks_at(2.0)),
        );
        kit::ticks(grove, hover_marks, theme::faint(), 1.0);
        // The port a link is drawn from: a small ring on a card's right edge, with room round it
        // for a finger.
        let port = grove.branch(
            paper,
            Stem::new()
                .interactive()
                .drags(foliage::Axes::Both)
                .elevate(Elevation::up(12))
                .visible(false)
                .at(Location::new().xs(
                    left(anchor().right() - 12.px()).width(24.px()),
                    top(anchor().center_y() - 12.px()).height(24.px()),
                )),
        );
        grove.branch(
            port,
            foliage::Panel::new()
                .color(theme::paper())
                .outline(1.5, theme::line())
                .rounding(foliage::Corners::all(Rounding::Full))
                .intangible()
                .at(Location::new().xs(
                    foliage::center_x(50.pct()).width(11.px()),
                    foliage::center_y(50.pct()).height(11.px()),
                )),
        );
        let ring = grove.branch(
            paper,
            kit::ring(2.0, theme::marked())
                .rounding(foliage::Corners::all(Rounding::Lg))
                .intangible()
                .elevate(Elevation::up(13))
                .visible(false)
                .at(marks_at(40.0)),
        );
        let wires = Wires::grow(grove, paper);
        Self {
            paper,
            cards: Vec::new(),
            wires,
            port,
            port_on: None,
            marks,
            marks_on: None,
            hover_marks,
            hover_on: None,
            ring,
            selected: None,
            moving: None,
            linking: None,
            reach: None,
            arranging: None,
            layers: [true; LAYERS],
            scale: 1.0,
            clashing: Vec::new(),
            dirty: true,
            faces: *faces,
        }
    }

    pub fn card(&self, id: Id) -> Option<&NodeView> {
        self.cards.iter().find(|(i, _)| *i == id).map(|(_, c)| c)
    }

    fn card_mut(&mut self, id: Id) -> Option<&mut NodeView> {
        self.cards.iter_mut().find(|(i, _)| *i == id).map(|(_, c)| c)
    }

    /// Where `id`'s card is drawn on the paper, in the paper's pixels; nothing if it is not shown.
    pub fn rect(&self, id: Id) -> Option<Rect> {
        self.card(id).filter(|c| c.shown).map(|c| c.drawn(self.scale))
    }

    /// Whether a card is being dragged, or a link drawn: a hand is on the drawing.
    pub fn handling(&self) -> bool {
        self.moving.is_some() || self.linking.is_some()
    }

    /// Grows a card for every node on the board that has none, and brings them in one after
    /// another; then the wires.
    pub fn populate(&mut self, grove: &mut Grove, board: &Board) {
        for (n, node) in board.nodes.iter().enumerate() {
            let attending = board.attendees(node.id).len();
            let mut card = NodeView::grow(grove, self.paper, node, attending, &self.faces);
            card.arrive(grove, 140 + n as u64 * 34);
            self.cards.push((node.id, card));
        }
        self.wires.sync(grove, board);
        self.aim(grove);
        self.wires.arrive(grove, 140 + board.nodes.len() as u64 * 34);
        self.relight(grove);
    }

    /// Brings every card and wire into line with the board: cards for nodes added, cards hidden
    /// for nodes gone, what each says, where each stands.
    pub fn refresh(&mut self, grove: &mut Grove, board: &Board) {
        for node in &board.nodes {
            let attending = board.attendees(node.id).len();
            if self.card(node.id).is_none() {
                let mut card = NodeView::grow(grove, self.paper, node, attending, &self.faces);
                let detail = self.detail();
                card.detail(grove, detail, self.scale);
                card.arrive(grove, 0);
                self.cards.push((node.id, card));
            }
        }
        let scale = self.scale;
        let layers = self.layers;
        for (id, card) in &mut self.cards {
            match board.get(*id) {
                Some(node) => {
                    card.refresh(grove, node, board.attendees(*id).len());
                    if card.at != node.at && self.arranging.is_none() {
                        card.place(grove, node.at, scale);
                    }
                    card.show(grove, layers[card.kind.index()]);
                }
                None => card.show(grove, false),
            }
        }
        if let Some(selected) = self.selected
            && board.get(selected).is_none()
        {
            self.select(grove, None);
        }
        self.wires.sync(grove, board);
        self.dirty = true;
        self.relight(grove);
    }

    fn detail(&self) -> Detail {
        if self.scale < theme::OVERVIEW { Detail::Brief } else { Detail::Full }
    }

    /// Reads the drawing at `scale`: every card placed again, and drawn in brief or in full.
    pub fn rescale(&mut self, grove: &mut Grove, scale: f32) {
        self.scale = scale;
        let detail = self.detail();
        for (_, card) in &mut self.cards {
            card.detail(grove, detail, scale);
            card.place(grove, card.at, scale);
        }
        self.dirty = true;
    }

    /// Picks out `id`'s card, or none: its wires lit and the rest quieted, a bead sent out along
    /// each of its own.
    pub fn select(&mut self, grove: &mut Grove, id: Option<Id>) {
        let id = id.filter(|id| self.card(*id).is_some_and(|c| c.shown));
        if self.selected == id {
            return;
        }
        self.selected = id;
        match id.and_then(|id| self.card(id)).map(|c| c.root) {
            Some(root) => {
                grove.anchor(self.marks, root);
                grove.visible(self.marks, true);
                self.marks_on = id;
            }
            None => {
                grove.visible(self.marks, false);
                self.marks_on = None;
            }
        }
        self.relight(grove);
        if let Some(id) = id {
            self.wires.pulse(grove, id);
        }
    }

    /// Lights every wire for the selection and the clashes, and shows those whose layer is on.
    pub fn relight(&mut self, grove: &mut Grove) {
        let selected = self.selected;
        let clashing = self.clashing.clone();
        let shown: Vec<(Id, bool)> = self.cards.iter().map(|(id, c)| (*id, c.shown)).collect();
        let is_shown = |id: Id| shown.iter().any(|(i, s)| *i == id && *s);
        let wires_on = self.layers[WIRES];
        self.wires.light(
            grove,
            |edge| {
                let mine = selected.is_some_and(|s| edge.from == s || edge.to == s);
                if clashing.contains(&(edge.from, edge.to)) && (mine || selected.is_none()) {
                    Lit::Clash
                } else if mine {
                    Lit::On
                } else if selected.is_some() {
                    Lit::Off
                } else {
                    Lit::Rest
                }
            },
            |edge| wires_on && is_shown(edge.from) && is_shown(edge.to),
        );
    }

    /// Shows or hides layer `n`.
    pub fn layer(&mut self, grove: &mut Grove, n: usize, on: bool) {
        self.layers[n] = on;
        for (_, card) in &mut self.cards {
            if card.kind.index() == n {
                card.show(grove, on);
            }
        }
        if let Some(selected) = self.selected
            && !self.card(selected).is_some_and(|c| c.shown)
        {
            self.select(grove, None);
        }
        self.dirty = true;
        self.relight(grove);
    }

    /// Quiets every card but those `found`, or none if there is no search.
    pub fn sift(&mut self, grove: &mut Grove, found: Option<&[Id]>) {
        for (id, card) in &mut self.cards {
            card.quiet(grove, found.is_some_and(|found| !found.contains(id)));
        }
    }

    /// Lights the meetings under the needle.
    pub fn activate(&mut self, grove: &mut Grove, active: &[Id]) {
        for (id, card) in &mut self.cards {
            if card.kind == Kind::Meeting {
                card.activate(grove, active.contains(id));
            }
        }
    }

    /// Rings `id`'s card in highlighter, closing in and fading: here it is.
    pub fn find(&mut self, grove: &mut Grove, id: Id, after: u64) {
        let Some(root) = self.card(id).map(|c| c.root) else {
            return;
        };
        let at = |pad: f32| {
            Location::new().xs(
                left(anchor().left() - pad.px()).right(anchor().right() + pad.px()),
                top(anchor().top() - pad.px()).bottom(anchor().bottom() + pad.px()),
            )
        };
        grove.anchor(self.ring, root);
        grove.visible(self.ring, true);
        grove.at(self.ring, at(46.0));
        grove.opacity(self.ring, 0.0);
        grove.animate(self.ring, Motion::Location(at(5.0)), Timing::ms(700).after(after).ease(Ease::Emphasis));
        grove.animate(self.ring, Motion::Opacity(1.0), Timing::ms(160).after(after));
        grove.animate(
            self.ring,
            Motion::Opacity(0.0),
            Timing::ms(900).after(after + 1300).ease(Ease::Accelerate),
        );
    }

    /// Puts a fresh card for `kind` on the board, at `at` on the sheet, and on the drawing.
    ///
    /// It goes where it is asked for, centred there, or -- where that is taken -- the nearest place
    /// clear of every other card, of the markup `reserve` says is drawn about them, and of the
    /// blocks, so that no two cards' lettering lies across each other's at any scale.
    #[allow(clippy::too_many_arguments)]
    pub fn add(
        &mut self,
        grove: &mut Grove,
        board: &mut Board,
        history: &mut History,
        kind: Kind,
        at: (f32, f32),
        bounds: &Bounds,
        reserve: &[(Id, Margins)],
    ) -> Id {
        history.keep(board, format!("added a {}", kind.name()));
        let size = room(kind);
        let taken: Vec<(f32, f32, f32, f32)> = board
            .nodes
            .iter()
            .map(|n| {
                let (w, h) = room(n.kind);
                let (l, t, r, b) = reserve.iter().find(|(id, _)| *id == n.id).map(|(_, m)| *m).unwrap_or_default();
                (n.at.0 - l, n.at.1 - t, n.at.0 + w + r, n.at.1 + h + b)
            })
            .collect();
        let at = arrange::clear((at.0 - size.0 / 2.0, at.1 - size.1 / 2.0), size, &taken, bounds);
        let id = board.add(kind, at);
        if !self.layers[kind.index()] {
            self.layer(grove, kind.index(), true);
        }
        self.refresh(grove, board);
        self.aim(grove);
        id
    }

    /// Takes `id` off the board and its card off the drawing.
    pub fn remove(&mut self, grove: &mut Grove, board: &mut Board, history: &mut History, id: Id) {
        let Some(code) = board.get(id).map(|n| n.code.clone()) else {
            return;
        };
        history.keep(board, format!("deleted {code}"));
        board.remove(id);
        self.refresh(grove, board);
    }

    /// Arranges the whole board inside `bounds`, gliding every card to where it goes. A card's
    /// lettering keeps its size whatever the scale, so it takes more of the sheet read small: it is
    /// given the most room it takes at any scale, so that what is arranged at one reads clear at
    /// every other -- and, all round it, the margins `reserve` asks for the markup drawn about it,
    /// `(left, top, right, bottom)` on the sheet.
    pub fn arrange(
        &mut self,
        grove: &mut Grove,
        board: &mut Board,
        history: &mut History,
        bounds: &Bounds,
        reserve: &[(Id, Margins)],
    ) {
        let index = |id: Id| board.nodes.iter().position(|n| n.id == id);
        let margins = |id: Id| reserve.iter().find(|(i, _)| *i == id).map(|(_, m)| *m).unwrap_or_default();
        let bodies: Vec<Body> = board
            .nodes
            .iter()
            .map(|n| {
                let (w, h) = room(n.kind);
                let (l, t, r, b) = margins(n.id);
                Body {
                    kind: n.kind,
                    at: (n.at.0 - l, n.at.1 - t),
                    size: (w + l + r, h + t + b),
                }
            })
            .collect();
        let edges: Vec<(usize, usize)> = board
            .edges
            .iter()
            .filter_map(|e| Some((index(e.from)?, index(e.to)?)))
            .collect();
        let placed = arrange::arrange(&bodies, &edges, bounds);
        history.keep(board, "arranged the sheet");
        let mut paths = Vec::new();
        for (node, to) in board.nodes.iter_mut().zip(placed) {
            let from = self.card(node.id).map(|c| c.at).unwrap_or(node.at);
            let (l, t, _, _) = margins(node.id);
            let to = (to.0 + l, to.1 + t);
            node.at = to;
            paths.push((node.id, from, to));
        }
        if let Some((tween, _)) = self.arranging.take() {
            grove.stop(tween);
        }
        let tween = grove.tween(0.0, 1.0, Timing::ms(1500).ease(Ease::Emphasis));
        self.arranging = Some((tween, paths));
    }

    /// Aims every wire at where its cards are drawn.
    pub fn aim(&mut self, grove: &mut Grove) {
        let rects: Vec<(Id, Option<Rect>)> = self.cards.iter().map(|(id, _)| (*id, self.rect(*id))).collect();
        self.wires.aim(grove, |id| rects.iter().find(|(i, _)| *i == id).and_then(|(_, r)| *r));
        self.dirty = false;
    }

    /// Where the pointer at `at` on the window is on the paper, in its pixels.
    fn on_paper(&self, grove: &Grove, at: Position) -> (f32, f32) {
        match drawn(grove, self.paper) {
            Some(paper) => (at.x - paper.left(), at.y - paper.top()),
            None => (at.x, at.y),
        }
    }

    /// The card the pointer at `at` is over, topmost first.
    fn under(&self, grove: &Grove, at: Position) -> Option<Id> {
        topmost(
            self.cards
                .iter()
                .filter(|(_, c)| c.shown)
                .filter_map(|(id, c)| Some((*id, drawn(grove, c.root)?))),
            at,
        )
    }

    /// Carries the drawing for a frame.
    pub fn frame(
        &mut self,
        grove: &mut Grove,
        pollen: &Pollen,
        board: &mut Board,
        history: &mut History,
        hover: &Hover,
        tool: Tool,
    ) -> Vec<Said> {
        let mut said = Vec::new();
        let mut keys = Vec::new();
        // An arrangement gliding home.
        if let Some((tween, paths)) = &self.arranging {
            let (tween, paths) = (*tween, paths.clone());
            if let Some(t) = pollen.tween(tween) {
                for (id, from, to) in &paths {
                    let at = (from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t);
                    let scale = self.scale;
                    if let Some(card) = self.card_mut(*id) {
                        card.place(grove, at, scale);
                    }
                }
                self.dirty = true;
            }
            if pollen.finished(tween) {
                self.arranging = None;
                said.push(Said::Changed);
            }
        }
        let ids: Vec<(Id, Leaf)> = self.cards.iter().filter(|(_, c)| c.shown).map(|(id, c)| (*id, c.root)).collect();
        for &(id, root) in &ids {
            keys.extend_from_slice(pollen.keys(root));
            for stroke in pollen.keys(root) {
                if matches!(stroke.key, foliage::Key::Delete | foliage::Key::Backspace) {
                    said.push(Said::Delete(id));
                }
            }
            if pollen.clicked(root) {
                self.select(grove, Some(id));
                said.push(Said::Selected(Some(id)));
                if pollen.taps(root) >= 2 {
                    said.push(Said::Inspect(id));
                }
            }
            if pollen.drag_started(root) {
                if let Some((tween, _)) = self.arranging.take() {
                    grove.finish(tween);
                }
                match tool {
                    Tool::Select => {
                        let origin = self.card(id).map(|c| c.at).unwrap_or_default();
                        self.moving = Some((id, origin, false));
                        if self.selected != Some(id) {
                            self.select(grove, Some(id));
                            said.push(Said::Selected(Some(id)));
                        }
                    }
                    Tool::Connect => self.linking = Some(id),
                }
            }
            if let Some(drag) = pollen.dragged(root) {
                if let Some((moving, origin, _)) = self.moving
                    && moving == id
                {
                    let scale = self.scale;
                    let (w, h) = self.card(id).map(|c| c.size()).unwrap_or((0.0, 0.0));
                    let at = (
                        (origin.0 + (drag.current.x - drag.start.x) / scale)
                            .clamp(theme::INNER, theme::SHEET.0 - theme::INNER - w / scale),
                        (origin.1 + (drag.current.y - drag.start.y) / scale)
                            .clamp(theme::INNER, theme::SHEET.1 - theme::INNER - h / scale),
                    );
                    if let Some(card) = self.card_mut(id) {
                        card.place(grove, at, scale);
                    }
                    self.moving = Some((id, origin, true));
                    self.dirty = true;
                }
                if self.linking == Some(id) {
                    self.stretch(grove, id, drag.current);
                }
            }
            if pollen.disengaged(root) {
                if let Some((moving, origin, moved)) = self.moving
                    && moving == id
                {
                    self.moving = None;
                    if moved {
                        self.settle(grove, board, history, id, origin);
                        said.push(Said::Changed);
                    }
                }
                if self.linking == Some(id) {
                    said.extend(self.link(grove, board, history, id, pollen));
                }
            }
        }
        // The port: a link drawn from a card's edge, whatever the tool.
        if pollen.drag_started(self.port) {
            self.linking = self.port_on;
        }
        if let Some(drag) = pollen.dragged(self.port)
            && let Some(from) = self.linking
        {
            self.stretch(grove, from, drag.current);
        }
        if pollen.disengaged(self.port)
            && let Some(from) = self.linking
        {
            said.extend(self.link(grove, board, history, from, pollen));
        }
        // Where the port and the hover marks stand.
        let over = hover.over().and_then(|leaf| self.cards.iter().find(|(_, c)| c.root == leaf).map(|(id, _)| *id));
        let on_port = hover.over() == Some(self.port);
        let port_on = match (tool, self.linking) {
            (_, Some(from)) => Some(from),
            (Tool::Connect, None) => None,
            (Tool::Select, None) if on_port => self.port_on,
            (Tool::Select, None) => over.or(self.selected),
        };
        if port_on != self.port_on {
            self.port_on = port_on;
            match port_on.and_then(|id| self.card(id)).map(|c| c.root) {
                Some(root) => {
                    grove.anchor(self.port, root);
                    grove.visible(self.port, true);
                }
                None => grove.visible(self.port, false),
            }
        }
        let hovered = over.filter(|id| Some(*id) != self.selected);
        if hovered != self.hover_on {
            self.hover_on = hovered;
            match hovered.and_then(|id| self.card(id)).map(|c| c.root) {
                Some(root) => {
                    grove.anchor(self.hover_marks, root);
                    grove.visible(self.hover_marks, true);
                }
                None => grove.visible(self.hover_marks, false),
            }
        }
        self.wires.frame(grove, pollen);
        if self.dirty {
            self.aim(grove);
        }
        if !keys.is_empty() {
            said.push(Said::Keys(keys));
        }
        said
    }

    /// A card let go: snapped to the grid, and written to the board.
    fn settle(&mut self, grove: &mut Grove, board: &mut Board, history: &mut History, id: Id, origin: (f32, f32)) {
        let scale = self.scale;
        let Some(card) = self.card_mut(id) else {
            return;
        };
        let at = (arrange::snap(card.at.0), arrange::snap(card.at.1));
        card.place(grove, at, scale);
        self.dirty = true;
        if at == origin {
            return;
        }
        let code = board.get(id).map(|n| n.code.clone()).unwrap_or_default();
        history.keep(board, format!("moved {code}"));
        if let Some(node) = board.get_mut(id) {
            node.at = at;
        }
    }

    /// The rubber band from `from`'s card to the pointer at `at`, lit for what is under it.
    fn stretch(&mut self, grove: &mut Grove, from: Id, at: Position) {
        let Some(r) = self.rect(from) else {
            return;
        };
        let start = (r.2, (r.1 + r.3) / 2.0);
        let end = self.on_paper(grove, at);
        self.reach = Some(at);
        let lit = match self.under(grove, at).filter(|id| *id != from) {
            Some(_) => Lit::On,
            None => Lit::Rest,
        };
        self.wires.stretch(grove, start, end, lit);
    }

    /// Lets a rubber band go over whatever is under it: a link drawn, or the band snapping back.
    fn link(&mut self, grove: &mut Grove, board: &mut Board, history: &mut History, from: Id, pollen: &Pollen) -> Vec<Said> {
        self.linking = None;
        let _ = pollen;
        let target = self.reach.take().and_then(|at| self.under(grove, at));
        let Some(to) = target.filter(|to| *to != from) else {
            self.wires.release(grove, true);
            return Vec::new();
        };
        match board.would_link(from, to) {
            Ok(edge) => {
                let (a, b) = (board.get(edge.from).map(|n| n.code.clone()), board.get(edge.to).map(|n| n.code.clone()));
                history.keep(board, format!("linked {} → {}", a.unwrap_or_default(), b.unwrap_or_default()));
                let _ = board.connect(from, to);
                self.wires.release(grove, false);
                self.wires.sync(grove, board);
                for (id, card) in &mut self.cards {
                    if let Some(node) = board.get(*id) {
                        card.refresh(grove, node, board.attendees(*id).len());
                    }
                }
                self.aim(grove);
                self.relight(grove);
                self.wires.pulse(grove, edge.from);
                vec![Said::Linked(edge.link), Said::Changed]
            }
            Err(refused) => {
                self.wires.release(grove, true);
                vec![Said::Refused(refused.words())]
            }
        }
    }
}

/// How much of the sheet a card of `kind` takes at the most, of every scale it is read at: its
/// lettering keeps its size whatever the scale, so read small it takes more.
fn room(kind: Kind) -> (f32, f32) {
    let (brief, full) = theme::least();
    let ((fw, fh), (bw, bh)) = (kind.footprint(), kind.brief());
    ((fw / full).max(bw / brief), (fh / full).max(bh / brief))
}
