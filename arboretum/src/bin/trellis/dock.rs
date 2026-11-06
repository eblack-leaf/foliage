//! The two instruments down the left: the layers, which is the drawing's legend with a switch on
//! every line of it, and the key plan, which is the whole sheet in small with where you are on it.
//!
//! Wide, they stand in a column of their own. Narrow, there is no room for a column, so they fold
//! away together and come up from the foot when the rail's layers press asks for them -- with the
//! sheet's scale, which has no room in a narrow rail either.

use foliage::{
    Area, Axes, Boxed, Cap, Elevation, Grove, Grow, Layout, Leaf, Location, Motion, Panel, Place, Pollen,
    Point, Polygon, Position, Source, Stem, Timing, center_y, content, left, top,
};
use lichen::{Chip, Switch};

use arboretum::sprout::Dashes;

use crate::board::{Board, Id, Kind, Link};
use crate::drawing::{LAYERS, WIRES};
use crate::icons::Icons;
use crate::kit;
use crate::theme::{self, Faces, SHEET};

/// How tall a row of the legend is.
const ROW: f32 = 28.0;
/// How far down the first row stands, under the heading.
const FIRST: f32 = 32.0;

/// What the dock asks of the sheet.
#[derive(Clone, Debug, PartialEq)]
pub enum Asked {
    Layer(usize, bool),
    /// The key plan was pressed or dragged: bring this point of the sheet, at full scale, into the
    /// middle of the view.
    Look((f32, f32)),
    Zoom(i32),
}

pub struct Dock {
    legend: Leaf,
    plan: Leaf,
    switches: Vec<Switch>,
    symbols: Vec<Leaf>,
    lines: Vec<(Leaf, Leaf)>,
    lines_head: Leaf,
    lines_rule: Leaf,
    map: Leaf,
    /// Every card in miniature, and the scale the sheet is read at, which their size follows.
    dots: Vec<(Id, Leaf)>,
    drawn: f32,
    view: Leaf,
    zoom: (Chip, Chip),
    scale: Leaf,
    layout: Layout,
    /// Whether it is up, on a window too narrow for it to stand all the time.
    pub open: bool,
    dashes: Dashes,
    /// The title block and the revision block, drawn small.
    blocks: [Leaf; 2],
}

/// Where a box on the sheet, in its full-scale units, stands on the key plan.
fn block_at((l, t, r, b): (f32, f32, f32, f32)) -> Location {
    Location::new().xs(
        left((l / SHEET.0 * 100.0).pct()).right((r / SHEET.0 * 100.0).pct()),
        top((t / SHEET.1 * 100.0).pct()).bottom((b / SHEET.1 * 100.0).pct()),
    )
}

impl Dock {
    pub fn grow(grove: &mut Grove, under: Leaf, faces: &Faces, icons: &Icons) -> Self {
        let legend = kit::plate(grove, under, Elevation::up(30));
        kit::heading(grove, legend, faces, "LAYERS", 12.0, 11.0);
        let names = ["people", "meetings", "specs", "milestones", "notes", "wires"];
        let mut switches = Vec::new();
        let mut symbols = Vec::new();
        for name in names.iter() {
            let mut switch = Switch::grow(
                grove,
                legend,
                Location::new().xs(left(0.px()).width(Switch::width(name)), top(0.px()).height(ROW.px())),
                Elevation::up(1),
                name,
            );
            switch.set(grove, true);
            switches.push(switch);
            let symbol = grove.branch(
                legend,
                foliage::Stem::new()
                    .intangible()
                    .elevate(Elevation::up(1))
                    .at(Location::new().xs(left(0.px()).width(16.px()), top(0.px()).height(16.px()))),
            );
            symbols.push(symbol);
        }
        // The legend's own symbols, drawn as the cards are.
        let ink = theme::ink();
        grove.branch(symbols[0], kit::ring(1.25, ink).intangible().at(Location::new()));
        grove.branch(
            symbols[1],
            Panel::new().color(theme::clear()).outline(1.0, ink).intangible().at(Location::new()),
        );
        grove.branch(
            symbols[1],
            Panel::new().color(ink).intangible().at(Location::new().xs(left(0.px()).width(100.pct()), top(0.px()).height(5.px()))),
        );
        grove.branch(
            symbols[2],
            Panel::new().color(theme::clear()).outline(1.0, ink).intangible().at(Location::new()),
        );
        for y in [5.0, 7.0] {
            kit::rule(grove, symbols[2], Location::new().xs(left(0.px()).width(100.pct()), top(y.px()).height(1.px())), ink);
        }
        grove.branch(symbols[3], Polygon::new().sides(4.0).color(ink).intangible().at(Location::new()));
        grove.branch(
            symbols[3],
            Polygon::new()
                .sides(4.0)
                .color(theme::plate())
                .intangible()
                .elevate(Elevation::up(1))
                .at(Location::new().xs(foliage::center_x(50.pct()).width(70.pct()), center_y(50.pct()).height(70.pct()))),
        );
        let mut dashes = Dashes::grow(grove, legend, Area::new(14.0, 12.0), 5.0, 0.0, 1.0, theme::soft(), Elevation::up(2));
        dashes.over(grove, symbols[4]);
        dashes.draw(grove, true);
        kit::rule(
            grove,
            symbols[5],
            Location::new().xs(left(0.px()).width(100.pct()), center_y(50.pct()).height(1.5.px())),
            ink,
        );
        // The key to the line types.
        let lines_rule = kit::rule(
            grove,
            legend,
            Location::new().xs(left(12.px()).width(10.px()), top(0.px()).height(1.px())),
            theme::rule(),
        );
        let lines_head = kit::heading(grove, legend, faces, "LINES", 12.0, 0.0);
        let mut lines = Vec::new();
        for link in Link::ALL {
            let sample = sample(grove, legend, link);
            let name = kit::letter(
                grove,
                legend,
                if link == Link::Annotates { "notes" } else { link.name() },
                faces.regular,
                theme::SUB,
                theme::soft(),
                Location::new().xs(left(0.px()).width(content()), top(0.px()).height(1.letters())),
            );
            lines.push((sample, name));
        }
        // The key plan.
        let plan = kit::plate(grove, under, Elevation::up(30));
        kit::heading(grove, plan, faces, "KEY PLAN", 12.0, 11.0);
        let map = grove.branch(
            plan,
            Panel::new()
                .color(theme::paper())
                .outline(1.0, theme::rule())
                .interactive()
                .drags(Axes::Both)
                .elevate(Elevation::up(1))
                .at(Location::new().xs(left(12.px()).width(100.px()), top(30.px()).height(60.px()))),
        );
        let blocks = [theme::title_block(1.0), theme::revision_block(1.0)].map(|(l, t, r, b)| {
            grove.branch(
                map,
                Panel::new()
                    .color(theme::clear())
                    .outline(1.0, theme::faint())
                    .intangible()
                    .elevate(Elevation::up(1))
                    .at(block_at((l, t, r, b))),
            )
        });
        let view = grove.branch(
            map,
            Panel::new()
                .color(theme::clear())
                .outline(1.25, theme::marked())
                .intangible()
                .elevate(Elevation::up(3))
                .at(Location::new()),
        );
        let m = lichen::measure();
        let spot = Location::new().xs(left(0.px()).width(m.height.px()), top(0.px()).height(m.height.px()));
        let zoom = (
            Chip::mark(grove, plan, spot.clone(), Elevation::up(1), icons.zoom_out),
            Chip::mark(grove, plan, spot, Elevation::up(1), icons.zoom_in),
        );
        let scale = kit::letter(
            grove,
            plan,
            "100%",
            faces.bold,
            theme::TITLE,
            theme::ink(),
            Location::new().xs(left(0.px()).width(content()), top(0.px()).height(1.letters())),
        );
        Self {
            legend,
            plan,
            switches,
            symbols,
            lines,
            lines_head,
            lines_rule,
            map,
            dots: Vec::new(),
            drawn: 1.0,
            view,
            zoom,
            scale,
            layout: Layout::Xl,
            open: false,
            dashes,
            blocks,
        }
    }

    /// The plates, which take every press that misses what is on them.
    pub fn plates(&self) -> [Leaf; 2] {
        [self.legend, self.plan]
    }

    /// Whether the dock folds away on this layout.
    pub fn folds(layout: Layout) -> bool {
        theme::dock(layout) == 0.0
    }

    /// Lays the dock out for `layout`: a column down the left, or two plates over the scale bar
    /// that come and go.
    pub fn place(&mut self, grove: &mut Grove, layout: Layout) {
        self.layout = layout;
        let folds = Self::folds(layout);
        let width = theme::dock(layout);
        // The legend: a column of switches wide, two columns of them narrow.
        let columns = if folds { 2 } else { 1 };
        let rows = LAYERS.div_ceil(columns);
        for (n, (switch, symbol)) in self.switches.iter().zip(&self.symbols).enumerate() {
            let (col, row) = (n / rows, n % rows);
            let y = FIRST + row as f32 * ROW;
            let x = match col {
                0 => 12.0,
                _ => 12.0 + 168.0,
            };
            grove.at(
                *symbol,
                Location::new().xs(left(x.px()).width(16.px()), top((y + (ROW - 16.0) / 2.0).px()).height(16.px())),
            );
            let name = ["people", "meetings", "specs", "milestones", "notes", "wires"][n];
            grove.at(
                switch.leaf(),
                Location::new().xs(left((x + 26.0).px()).width(Switch::width(name)), top(y.px()).height(ROW.px())),
            );
        }
        let after = FIRST + rows as f32 * ROW + 8.0;
        grove.at(
            self.lines_rule,
            Location::new().xs(left(12.px()).right(100.pct() - 12.px()), top(after.px()).height(1.px())),
        );
        grove.at(
            self.lines_head,
            Location::new().xs(left(12.px()).width(content()), top((after + 10.0).px()).height(1.letters())),
        );
        let key_top = after + 30.0;
        let per = if folds { 3 } else { 2 };
        let col_w = if folds { 112.0 } else { 104.0 };
        for (n, (sample, name)) in self.lines.iter().enumerate() {
            let (col, row) = (n % per, n / per);
            let x = 12.0 + col as f32 * col_w;
            let y = key_top + row as f32 * 20.0;
            grove.at(*sample, Location::new().xs(left(x.px()).width(30.px()), top((y + 2.0).px()).height(10.px())));
            grove.at(*name, Location::new().xs(left((x + 38.0).px()).width(content()), top(y.px()).height(1.letters())));
        }
        let key_rows = Link::ALL.len().div_ceil(per);
        let legend_h = key_top + key_rows as f32 * 20.0 + 6.0;
        // The key plan: the sheet in small, the scale beside it where the rail has none.
        let map_w = if folds { 196.0 } else { width - 24.0 };
        let map_h = (map_w * SHEET.1 / SHEET.0).round();
        let plan_h = 30.0 + map_h + 12.0;
        grove.at(self.map, Location::new().xs(left(12.px()).width(map_w.px()), top(30.px()).height(map_h.px())));
        let m = lichen::measure();
        let zx = 12.0 + map_w + 16.0;
        grove.at(
            self.zoom.0.leaf(),
            Location::new().xs(left(zx.px()).width(m.height.px()), top(30.px()).height(m.height.px())),
        );
        grove.at(
            self.zoom.1.leaf(),
            Location::new().xs(left((zx + m.height + m.gap).px()).width(m.height.px()), top(30.px()).height(m.height.px())),
        );
        grove.at(
            self.scale,
            Location::new().xs(left(zx.px()).width(content()), top((30.0 + m.height + 12.0).px()).height(1.letters())),
        );
        for leaf in [self.zoom.0.leaf(), self.zoom.1.leaf(), self.scale] {
            grove.visible(leaf, folds);
        }
        if folds {
            let bottom = theme::BOTTOM;
            grove.at(
                self.plan,
                Location::new().xs(
                    left(theme::GUTTER.px()).right(100.pct() - theme::GUTTER.px()),
                    foliage::bottom(100.pct() - bottom.px()).height(plan_h.px()),
                ),
            );
            grove.at(
                self.legend,
                Location::new().xs(
                    left(theme::GUTTER.px()).right(100.pct() - theme::GUTTER.px()),
                    foliage::bottom(100.pct() - (bottom + plan_h + theme::GUTTER).px()).height(legend_h.px()),
                ),
            );
            for plate in self.plates() {
                grove.visible(plate, self.open);
            }
        } else {
            grove.at(
                self.legend,
                Location::new().xs(
                    left(theme::GUTTER.px()).width(width.px()),
                    top(theme::TOP.px()).height(legend_h.px()),
                ),
            );
            grove.at(
                self.plan,
                Location::new().xs(
                    left(theme::GUTTER.px()).width(width.px()),
                    top((theme::TOP + legend_h + theme::GUTTER).px()).height(plan_h.px()),
                ),
            );
            for plate in self.plates() {
                grove.visible(plate, true);
                grove.opacity(plate, 1.0);
                grove.shift(plate, Position::default());
            }
        }
    }

    /// Brings the folded dock up from the foot, or puts it away. Nothing where it does not fold.
    pub fn show(&mut self, grove: &mut Grove, open: bool) {
        if self.open == open {
            return;
        }
        self.open = open;
        if !Self::folds(self.layout) {
            return;
        }
        for (n, plate) in self.plates().into_iter().enumerate() {
            match open {
                true => {
                    grove.visible(plate, true);
                    grove.opacity(plate, 0.0);
                    grove.shift(plate, Position::new(0.0, 18.0));
                    let timing = Timing::ms(260).after(n as u64 * 50).ease(foliage::Ease::Decelerate);
                    grove.animate(plate, Motion::Opacity(1.0), timing);
                    grove.animate(plate, Motion::Shift(Position::default()), timing);
                }
                false => grove.visible(plate, false),
            }
        }
    }

    /// Writes the scale where the dock shows it.
    pub fn scale(&mut self, grove: &mut Grove, scale: f32) {
        self.drawn = scale;
        grove.text(self.scale, format!("{}%", (scale * 100.0).round()));
        // The blocks keep their size on the window, so they take more of the sheet read smaller.
        let [title, revisions] = self.blocks;
        grove.at(title, block_at(theme::title_block(scale)));
        grove.at(revisions, block_at(theme::revision_block(scale)));
    }

    /// Draws every card on the key plan in miniature: a block of graphite as large on the plan as
    /// the card is on the sheet at the scale it is read at, where it stands, and only if it is
    /// shown. A key plan is the drawing small, so its cards read as the drawing's do.
    pub fn plot(&mut self, grove: &mut Grove, board: &Board, shown: impl Fn(Id) -> bool, at: impl Fn(Id) -> Option<(f32, f32)>) {
        for node in &board.nodes {
            if !self.dots.iter().any(|(id, _)| *id == node.id) {
                let fill = match node.kind {
                    Kind::Note => theme::faint(),
                    _ => theme::line(),
                };
                let dot = grove.branch(
                    self.map,
                    Panel::new().color(fill).intangible().elevate(Elevation::up(2)).at(Location::new()),
                );
                self.dots.push((node.id, dot));
            }
        }
        for (id, dot) in &self.dots {
            let node = board.get(*id);
            let visible = node.is_some() && shown(*id);
            grove.visible(*dot, visible);
            if let (Some(node), Some((x, y))) = (node, at(*id)) {
                let size = match self.drawn < theme::OVERVIEW {
                    true => node.kind.brief(),
                    false => node.kind.footprint(),
                };
                let (w, h) = (size.0 / self.drawn, size.1 / self.drawn);
                let pct = |v: f32, of: f32| (v / of * 100.0).pct();
                grove.at(
                    *dot,
                    Location::new().xs(
                        left(pct(x, SHEET.0)).width(pct(w, SHEET.0)),
                        top(pct(y, SHEET.1)).height(pct(h, SHEET.1)),
                    ),
                );
            }
        }
    }

    /// Draws where the view is on the key plan: `(left, top, right, bottom)` of the sheet, at full
    /// scale, that can be seen.
    pub fn view(&self, grove: &mut Grove, seen: (f32, f32, f32, f32)) {
        let (w, h) = SHEET;
        let clamp = |v: f32, m: f32| (v / m * 100.0).clamp(0.0, 100.0);
        grove.at(
            self.view,
            Location::new().xs(
                left(clamp(seen.0, w).pct()).right(clamp(seen.2, w).pct()),
                top(clamp(seen.1, h).pct()).bottom(clamp(seen.3, h).pct()),
            ),
        );
    }

    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) -> Vec<Asked> {
        let mut asked = Vec::new();
        for (n, switch) in self.switches.iter_mut().enumerate() {
            if switch.frame(grove, pollen) {
                asked.push(Asked::Layer(n, switch.on()));
            }
        }
        let at = pollen.clicked_at(self.map).or_else(|| pollen.dragged(self.map).map(|d| d.current));
        if let (Some(at), Some(map)) = (at, lichen::drawn(grove, self.map)) {
            let x = ((at.x - map.left()) / map.width().max(1.0)).clamp(0.0, 1.0) * SHEET.0;
            let y = ((at.y - map.top()) / map.height().max(1.0)).clamp(0.0, 1.0) * SHEET.1;
            asked.push(Asked::Look((x, y)));
        }
        if self.zoom.0.pressed(pollen) {
            asked.push(Asked::Zoom(-1));
        }
        if self.zoom.1.pressed(pollen) {
            asked.push(Asked::Zoom(1));
        }
        let _ = WIRES;
        let _ = &self.dashes;
        asked
    }

    /// Sets switch `n` without its being pressed.
    pub fn set(&mut self, grove: &mut Grove, n: usize, on: bool) {
        if let Some(switch) = self.switches.get_mut(n) {
            switch.set(grove, on);
        }
    }
}

/// A short sample of the line a link is drawn in, for the key: a box 30 wide and 10 tall, drawn by
/// hand rather than as a wire, since a wire spaces its dashes along its whole length and a sample
/// has almost none.
fn sample(grove: &mut Grove, under: Leaf, link: Link) -> Leaf {
    let stem = grove.branch(
        under,
        Stem::new()
            .intangible()
            .elevate(Elevation::up(1))
            .at(Location::new().xs(left(0.px()).width(30.px()), top(0.px()).height(10.px()))),
    );
    let ink = theme::ink();
    let stroke = |grove: &mut Grove, a: f32, b: f32, weight: f32, cap: Cap| {
        grove.branch(
            stem,
            foliage::Line::new()
                .between(Point::new(a.px(), 5.5.px()), Point::new(b.px(), 5.5.px()))
                .weight(weight)
                .color(ink)
                .cap(cap)
                .intangible(),
        );
    };
    let head = |grove: &mut Grove| {
        grove.branch(
            stem,
            Polygon::new()
                .sides(3.0)
                .rotation(-std::f32::consts::FRAC_PI_2)
                .color(ink)
                .intangible()
                .at(Location::new().xs(left(22.px()).width(9.px()), center_y(50.pct()).height(9.px()))),
        );
    };
    let path = |grove: &mut Grove, points: &[(f32, f32)], weight: f32| {
        for pair in points.windows(2) {
            grove.branch(
                stem,
                foliage::Line::new()
                    .between(Point::new(pair[0].0.px(), pair[0].1.px()), Point::new(pair[1].0.px(), pair[1].1.px()))
                    .weight(weight)
                    .color(ink)
                    .cap(Cap::Round)
                    .intangible(),
            );
        }
    };
    match link {
        // Ruled square: across, down, across.
        Link::Owns => path(grove, &[(0.0, 2.5), (13.0, 2.5), (17.0, 8.5), (30.0, 8.5)], 1.25),
        // The French curve.
        Link::Attends => path(grove, &[(0.0, 2.5), (8.0, 3.0), (12.0, 4.5), (18.0, 6.5), (22.0, 8.0), (30.0, 8.5)], 1.0),
        Link::Depends => {
            stroke(grove, 0.0, 24.0, 1.25, Cap::Butt);
            head(grove);
        }
        Link::Reviews => {
            for a in [0.0, 12.0, 24.0] {
                stroke(grove, a, a + 7.0, 1.25, Cap::Butt);
            }
        }
        Link::Annotates => {
            stroke(grove, 0.0, 24.0, 1.0, Cap::Butt);
            head(grove);
        }
    }
    stem
}
