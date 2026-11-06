//! The drawing itself: the paper under the instruments, its grid, its border and zones, the title
//! block and the revision block -- and how the paper is moved about and read closer or farther.
//!
//! The paper lies in a region that scrolls both ways. A region takes a drag along one axis at a
//! time, so the paper's own table takes the drag instead, on both, and writes where the region is
//! on each axis itself, coasting a little after a fling. Everything printed on the sheet is placed
//! in fractions of the paper, so reading it at another scale is one resize; only what is placed in
//! pixels -- the cards and their wires -- is placed again.

use foliage::{
    Axes, Boxed, Cap, Elevation, Grove, Grow, Layout, Leaf, Line, Location, Panel, Place, Point,
    Motion, Pollen, Position, Sap, Scroll, ScrollTo, Source, Stem, Timing, Tween, Vein, center_x, center_y,
    content, left, right, top,
};
use lichen::{Chip, measure};

use arboretum::sprout::App;

use crate::api;
use crate::icons::Icons;
use crate::kit::{self, letter, rule};
use crate::theme::{self, CELL, Faces, INNER, MAJOR_EVERY, OUTER, SHEET, ZONES};

/// How fast a fling has to be to coast, and how quickly a coast dies away.
const COAST_FROM: f32 = 120.0;
const COAST_HALF_LIFE: f32 = 0.18;

/// The title block: who drew the sheet, what it is, and where it stands.
pub struct TitleBlock {
    pub scale: Leaf,
    pub rev: Leaf,
    pub status: Leaf,
    pub ways: Vec<(App, Chip)>,
}

/// The revision block: the last few revisions, latest at the foot.
pub struct RevisionBlock {
    rows: Vec<(Leaf, Leaf, Leaf)>,
}

/// A glide under way: its tween, and the offsets it runs from and to.
type Glide = (Tween, (f32, f32), (f32, f32));

pub struct Sheet {
    pub viewport: Leaf,
    /// What a drag on the paper takes, under everything printed on it.
    pub table: Leaf,
    /// What everything on the drawing is grown in, in sheet pixels at the current scale.
    pub paper: Leaf,
    minor: Vec<Leaf>,
    pub scale: f32,
    margins: (f32, f32, f32, f32),
    grab: Option<(f32, f32)>,
    velocity: (f32, f32),
    coasting: bool,
    glide: Option<Glide>,
    pub title: TitleBlock,
    pub revisions: RevisionBlock,
    plotted: Vec<Leaf>,
}

/// What the paper did this frame.
#[derive(Default)]
pub struct Moved {
    /// It was tapped where nothing is drawn.
    pub tapped: bool,
    /// It was dragged, or coasted, or glided: where it shows moved.
    pub panned: bool,
    /// A drag on it began.
    pub grabbed: bool,
}

impl Sheet {
    pub fn grow(grove: &mut Grove, window: Leaf, faces: &Faces, icons: &Icons) -> Self {
        let viewport = grove.branch(
            window,
            Stem::new()
                .scrolls(Scroll::new(Axes::Both).contain(Axes::Both))
                .elevate(Elevation::up(1))
                .at(Location::new()),
        );
        let table = grove.branch(
            viewport,
            Panel::new()
                .color(theme::paper())
                .interactive()
                .drags(Axes::Both)
                .at(Location::new()),
        );
        let paper = grove.branch(
            viewport,
            Stem::new().intangible().elevate(Elevation::up(1)).at(Location::new()),
        );
        let (w, h) = SHEET;
        let mut plotted = Vec::new();
        // The grid, inside the frame, every cell and heavier every fifth.
        let mut minor = Vec::new();
        let (across, down) = (((w - 2.0 * INNER) / CELL) as usize, ((h - 2.0 * INNER) / CELL) as usize);
        for i in 1..across {
            let x = (INNER + CELL * i as f32) / w * 100.0;
            let heavy = i % MAJOR_EVERY == 0;
            let line = grove.branch(
                paper,
                Line::new()
                    .between(
                        Point::new(x.pct() + 0.5.px(), (INNER / h * 100.0).pct()),
                        Point::new(x.pct() + 0.5.px(), ((h - INNER) / h * 100.0).pct()),
                    )
                    .weight(1.0)
                    .color(theme::oklch(if heavy { theme::MAJOR } else { theme::GRID }))
                    .intangible()
                    .elevate(Elevation::up(if heavy { 1 } else { 0 })),
            );
            if !heavy {
                minor.push(line);
            }
            plotted.push(line);
        }
        for j in 1..down {
            let y = (INNER + CELL * j as f32) / h * 100.0;
            let heavy = j % MAJOR_EVERY == 0;
            let line = grove.branch(
                paper,
                Line::new()
                    .between(
                        Point::new((INNER / w * 100.0).pct(), y.pct() + 0.5.px()),
                        Point::new(((w - INNER) / w * 100.0).pct(), y.pct() + 0.5.px()),
                    )
                    .weight(1.0)
                    .color(theme::oklch(if heavy { theme::MAJOR } else { theme::GRID }))
                    .intangible()
                    .elevate(Elevation::up(if heavy { 1 } else { 0 })),
            );
            if !heavy {
                minor.push(line);
            }
            plotted.push(line);
        }
        // The border: a heavy line round the sheet and a light one round the drawing, with the
        // zones lettered in the band between.
        let frame = |grove: &mut Grove, inset: f32, weight: f32| {
            grove.branch(
                paper,
                kit::ring(weight, theme::line())
                    .rounding(foliage::Corners::all(foliage::Rounding::None))
                    .intangible()
                    .elevate(Elevation::up(2))
                    .at(Location::new().xs(
                        left((inset / w * 100.0).pct()).right(((w - inset) / w * 100.0).pct()),
                        top((inset / h * 100.0).pct()).bottom(((h - inset) / h * 100.0).pct()),
                    )),
            )
        };
        plotted.push(frame(grove, OUTER, 2.0));
        plotted.push(frame(grove, INNER, 1.0));
        let band = (OUTER + INNER) / 2.0;
        let (cols, rows) = ZONES;
        for c in 0..cols {
            let x0 = INNER + (w - 2.0 * INNER) * c as f32 / cols as f32;
            let mid = INNER + (w - 2.0 * INNER) * (c as f32 + 0.5) / cols as f32;
            for y in [band, h - band] {
                plotted.push(letter(
                    grove,
                    paper,
                    format!("{}", c + 1),
                    faces.bold,
                    theme::ZONE,
                    theme::faint(),
                    Location::new().xs(
                        center_x((mid / w * 100.0).pct()).width(content()),
                        center_y((y / h * 100.0).pct()).height(1.letters()),
                    ),
                ));
            }
            if c > 0 {
                for (a, b) in [(OUTER, INNER), (h - INNER, h - OUTER)] {
                    plotted.push(tick(grove, paper, (x0, a), (x0, b)));
                }
            }
        }
        for r in 0..rows {
            let y0 = INNER + (h - 2.0 * INNER) * r as f32 / rows as f32;
            let mid = INNER + (h - 2.0 * INNER) * (r as f32 + 0.5) / rows as f32;
            let name = ((b'A' + r as u8) as char).to_string();
            for x in [band, w - band] {
                plotted.push(letter(
                    grove,
                    paper,
                    name.clone(),
                    faces.bold,
                    theme::ZONE,
                    theme::faint(),
                    Location::new().xs(
                        center_x((x / w * 100.0).pct()).width(content()),
                        center_y((mid / h * 100.0).pct()).height(1.letters()),
                    ),
                ));
            }
            if r > 0 {
                for (a, b) in [(OUTER, INNER), (w - INNER, w - OUTER)] {
                    plotted.push(tick(grove, paper, (a, y0), (b, y0)));
                }
            }
        }
        let title = title_block(grove, paper, faces, icons);
        let revisions = revision_block(grove, paper, faces);
        Self {
            viewport,
            table,
            paper,
            minor,
            scale: 1.0,
            margins: (0.0, 0.0, 0.0, 0.0),
            grab: None,
            velocity: (0.0, 0.0),
            coasting: false,
            glide: None,
            title,
            revisions,
            plotted,
        }
    }

    /// Draws the sheet in as a plotter would: the grid across then down, the border, then the
    /// zones -- a few milliseconds a stroke, while the drawing itself is still loading.
    pub fn plot(&self, grove: &mut Grove) {
        for (n, &leaf) in self.plotted.iter().enumerate() {
            grove.opacity(leaf, 0.0);
            let timing = Timing::ms(260).after(120 + n as u64 * 4).ease(foliage::Ease::Decelerate);
            grove.animate(leaf, Motion::Opacity(1.0), timing);
        }
    }

    /// Lays the paper out for `layout`: margins round it wide enough that every corner of the
    /// drawing can be brought out from under the instruments.
    pub fn place(&mut self, grove: &mut Grove, layout: Layout) {
        let left_margin = match theme::dock(layout) {
            0.0 => theme::GUTTER,
            dock => theme::GUTTER * 2.0 + dock,
        };
        let right_margin = match theme::docked(layout) {
            Some(width) => theme::GUTTER * 2.0 + width,
            None => theme::GUTTER,
        };
        self.margins = (left_margin, theme::TOP, right_margin, theme::BOTTOM);
        self.resize(grove);
    }

    fn resize(&mut self, grove: &mut Grove) {
        let (l, t, r, b) = self.margins;
        let (w, h) = (SHEET.0 * self.scale, SHEET.1 * self.scale);
        grove.at(
            self.paper,
            Location::new().xs(left(l.px()).width(w.px()), top(t.px()).height(h.px())),
        );
        grove.at(
            self.table,
            Location::new().xs(
                left(0.px()).width((l + w + r).px()).at_least(100.pct()),
                top(0.px()).height((t + h + b).px()).at_least(100.pct()),
            ),
        );
    }

    /// Where the drawing's top left corner is on the region's content.
    pub fn margin(&self) -> (f32, f32) {
        (self.margins.0, self.margins.1)
    }

    /// Where the region is scrolled to.
    pub fn offset(&self, grove: &Grove) -> (f32, f32) {
        match grove.tap(self.viewport, Vein::Offset) {
            Some(Sap::Position(p)) => (p.x, p.y),
            _ => (0.0, 0.0),
        }
    }

    /// Where a point on the window is on the sheet, at full scale.
    pub fn to_sheet(&self, grove: &Grove, at: Position) -> (f32, f32) {
        let (ox, oy) = self.offset(grove);
        let (mx, my) = self.margin();
        ((at.x + ox - mx) / self.scale, (at.y + oy - my) / self.scale)
    }

    /// Scrolls the region to `to`, on both axes, at once.
    pub fn scroll_to(&mut self, grove: &mut Grove, to: (f32, f32)) {
        grove.scroll(self.viewport, ScrollTo::px(to.0.max(0.0).round()).on(Axes::Horizontal));
        grove.scroll(self.viewport, ScrollTo::px(to.1.max(0.0).round()).on(Axes::Vertical));
    }

    /// Scrolls the region so that `at` on the sheet, at full scale, stands at `window` on the
    /// window, gliding there over `ms`.
    pub fn bring(&mut self, grove: &mut Grove, at: (f32, f32), window: (f32, f32), ms: u64) {
        let (mx, my) = self.margin();
        let to = (at.0 * self.scale + mx - window.0, at.1 * self.scale + my - window.1);
        self.glide_to(grove, to, ms);
    }

    /// Glides the paper the least it takes for a card at `at` on the sheet, `size` across on the
    /// window, to stand inside `clear` -- and not at all if it does already.
    pub fn reveal(&mut self, grove: &mut Grove, at: (f32, f32), size: (f32, f32), clear: (f32, f32, f32, f32), ms: u64) {
        const ROOM: f32 = 20.0;
        let (ox, oy) = self.offset(grove);
        let (mx, my) = self.margin();
        let (x, y) = (at.0 * self.scale + mx - ox, at.1 * self.scale + my - oy);
        let (l, t, r, b) = (clear.0 + ROOM, clear.1 + ROOM, clear.2 - ROOM, clear.3 - ROOM);
        let by = |near: f32, far: f32, low: f32, high: f32| {
            if near < low {
                near - low
            } else if far > high {
                (far - high).min(near - low)
            } else {
                0.0
            }
        };
        let (dx, dy) = (by(x, x + size.0, l, r), by(y, y + size.1, t, b));
        if dx.abs() < 1.0 && dy.abs() < 1.0 {
            return;
        }
        self.glide_to(grove, (ox + dx, oy + dy), ms);
    }

    fn glide_to(&mut self, grove: &mut Grove, to: (f32, f32), ms: u64) {
        if let Some((tween, _, _)) = self.glide.take() {
            grove.stop(tween);
        }
        self.coasting = false;
        let from = self.offset(grove);
        let tween = grove.tween(0.0, 1.0, Timing::ms(ms).ease(foliage::Ease::Emphasis));
        self.glide = Some((tween, from, to));
    }

    /// Reads the sheet at `scale`, keeping the point of the sheet at `around` on the window where
    /// it is.
    pub fn zoom(&mut self, grove: &mut Grove, scale: f32, around: (f32, f32)) {
        if (scale - self.scale).abs() < 1e-3 {
            return;
        }
        let at = self.to_sheet(grove, Position::new(around.0, around.1));
        self.scale = scale;
        self.resize(grove);
        let (mx, my) = self.margin();
        self.scroll_to(grove, (at.0 * scale + mx - around.0, at.1 * scale + my - around.1));
        // The minor grid is only noise at half scale.
        for &line in &self.minor {
            grove.visible(line, scale > 0.6);
        }
        grove.text(self.title.scale, theme::scale_words(scale));
    }

    /// Carries the paper for a frame: a drag pans it, a fling coasts, a glide runs on.
    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) -> Moved {
        let mut moved = Moved::default();
        if pollen.clicked(self.table) {
            moved.tapped = true;
        }
        if pollen.drag_started(self.table) {
            self.grab = Some(self.offset(grove));
            self.coasting = false;
            self.velocity = (0.0, 0.0);
            if let Some((tween, _, _)) = self.glide.take() {
                grove.stop(tween);
            }
            moved.grabbed = true;
        }
        let dt = grove.frame_time().as_secs_f32().clamp(1.0 / 240.0, 0.1);
        if let Some(drag) = pollen.dragged(self.table)
            && let Some(grab) = self.grab
        {
            let to = (grab.0 - (drag.current.x - drag.start.x), grab.1 - (drag.current.y - drag.start.y));
            self.scroll_to(grove, to);
            // The speed of the hand, smoothed over the last few frames.
            let v = (-drag.delta.x / dt, -drag.delta.y / dt);
            self.velocity = (self.velocity.0 * 0.6 + v.0 * 0.4, self.velocity.1 * 0.6 + v.1 * 0.4);
            moved.panned = true;
        }
        if self.grab.is_some() && pollen.disengaged(self.table) {
            self.grab = None;
            self.coasting = self.velocity.0.hypot(self.velocity.1) > COAST_FROM;
        }
        if self.coasting {
            let at = self.offset(grove);
            self.scroll_to(grove, (at.0 + self.velocity.0 * dt, at.1 + self.velocity.1 * dt));
            let decay = 0.5f32.powf(dt / COAST_HALF_LIFE);
            self.velocity = (self.velocity.0 * decay, self.velocity.1 * decay);
            if self.velocity.0.hypot(self.velocity.1) < 30.0 {
                self.coasting = false;
            }
            grove.again();
            moved.panned = true;
        }
        if let Some((tween, from, to)) = self.glide {
            if let Some(t) = pollen.tween(tween) {
                self.scroll_to(grove, (from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t));
                moved.panned = true;
            }
            if pollen.finished(tween) {
                self.glide = None;
            }
        }
        moved
    }

    /// Whether the paper is being held, or is still moving from being let go.
    pub fn moving(&self) -> bool {
        self.grab.is_some() || self.coasting || self.glide.is_some()
    }

    /// Writes the revision the drawing is at, and the line under it.
    pub fn revised(&mut self, grove: &mut Grove, revision: u32, history: &[(u32, String)]) {
        grove.text(self.title.rev, api::letter(revision));
        let shown = history.len().min(self.revisions.rows.len());
        let from = history.len() - shown;
        for (n, &(rev, word, by)) in self.revisions.rows.iter().enumerate() {
            match history.get(from + n) {
                Some((r, what)) if n < shown => {
                    grove.text(rev, api::letter(*r));
                    grove.text(word, theme::clip(what, 30));
                    grove.text(by, "m.o.");
                }
                _ => {
                    grove.text(rev, "");
                    grove.text(word, "");
                    grove.text(by, "");
                }
            }
        }
    }

    /// Says where the drawing stands, on the title block's status line.
    pub fn status(&self, grove: &mut Grove, words: &str) {
        grove.text(self.title.status, words);
    }

    /// Goes where a reference drawing in the title block leads.
    pub fn signpost(&self, grove: &mut Grove, pollen: &Pollen) {
        for (app, chip) in &self.title.ways {
            if chip.pressed(pollen) {
                grove.navigate(app.href());
            }
        }
    }
}

fn tick(grove: &mut Grove, paper: Leaf, a: (f32, f32), b: (f32, f32)) -> Leaf {
    let (w, h) = SHEET;
    grove.branch(
        paper,
        Line::new()
            .between(
                Point::new((a.0 / w * 100.0).pct() + 0.5.px(), (a.1 / h * 100.0).pct() + 0.5.px()),
                Point::new((b.0 / w * 100.0).pct() + 0.5.px(), (b.1 / h * 100.0).pct() + 0.5.px()),
            )
            .weight(1.0)
            .color(theme::line())
            .cap(Cap::Butt)
            .intangible()
            .elevate(Elevation::up(2)),
    )
}

/// A box of fixed size in the drawing's corner, `(right, bottom)` or `(right, top)` of the frame.
fn cornered(size: (f32, f32), bottom: bool) -> Location {
    let (w, h) = SHEET;
    let r = ((w - INNER) / w * 100.0).pct();
    let vertical = match bottom {
        true => foliage::bottom(((h - INNER) / h * 100.0).pct()).height(size.1.px()),
        false => top((INNER / h * 100.0).pct()).height(size.1.px()),
    };
    Location::new().xs(right(r).width(size.0.px()), vertical)
}

/// One cell of a block: a label in small capitals over a value.
fn cell(grove: &mut Grove, block: Leaf, faces: &Faces, (x, y): (f32, f32), label: &str, value: &str, px: u32) -> Leaf {
    letter(
        grove,
        block,
        label,
        faces.regular,
        theme::LABEL,
        theme::faint(),
        Location::new().xs(left((x + 8.0).px()).width(content()), top((y + 6.0).px()).height(1.letters())),
    );
    letter(
        grove,
        block,
        value,
        faces.bold,
        px,
        theme::line(),
        Location::new().xs(left((x + 8.0).px()).width(content()), top((y + 18.0).px()).height(1.letters())),
    )
}

fn hairline(grove: &mut Grove, block: Leaf, across: Option<(f32, f32, f32)>, down: Option<(f32, f32, f32)>) {
    if let Some((x0, x1, y)) = across {
        rule(
            grove,
            block,
            Location::new().xs(left(x0.px()).width((x1 - x0).px()), top(y.px()).height(1.px())),
            theme::line(),
        );
    }
    if let Some((x, y0, y1)) = down {
        rule(
            grove,
            block,
            Location::new().xs(left(x.px()).width(1.px()), top(y0.px()).height((y1 - y0).px())),
            theme::line(),
        );
    }
}

fn title_block(grove: &mut Grove, paper: Leaf, faces: &Faces, icons: &Icons) -> TitleBlock {
    let (w, h) = theme::TITLE_BLOCK;
    let block = grove.branch(
        paper,
        Panel::new()
            .color(theme::paper())
            .outline(1.5, theme::line())
            .intangible()
            .elevate(Elevation::up(3))
            .font_size(lichen::caption())
            .at(cornered((w, h), true)),
    );
    // The studio and the drawing number.
    letter(
        grove,
        block,
        "FERNWOOD STUDIO",
        faces.bold,
        theme::TITLE,
        theme::line(),
        Location::new().xs(left(10.px()).width(content()), top(10.px()).height(1.letters())),
    );
    letter(
        grove,
        block,
        "orchard launch · planning sheet",
        faces.regular,
        theme::SUB,
        theme::faint(),
        Location::new().xs(left(10.px()).width(content()), top(30.px()).height(1.letters())),
    );
    let split = 262.0;
    letter(
        grove,
        block,
        "DWG NO.",
        faces.regular,
        theme::LABEL,
        theme::faint(),
        Location::new().xs(left((split + 10.0).px()).width(content()), top(8.px()).height(1.letters())),
    );
    letter(
        grove,
        block,
        "TR-001",
        faces.bold,
        theme::NUMBER,
        theme::line(),
        Location::new().xs(left((split + 10.0).px()).width(content()), top(22.px()).height(1.letters())),
    );
    hairline(grove, block, Some((0.0, w, 54.0)), Some((split, 0.0, 54.0)));
    // Who drew it, who checked it, the scale, and the revision.
    let quarter = w / 4.0;
    cell(grove, block, faces, (0.0, 54.0), "DRAWN", "m.okafor", theme::TITLE);
    cell(grove, block, faces, (quarter, 54.0), "CHECKED", "j.reyes", theme::TITLE);
    let scale = cell(grove, block, faces, (quarter * 2.0, 54.0), "SCALE", "1:1", theme::TITLE);
    let rev = cell(grove, block, faces, (quarter * 3.0, 54.0), "REV", "C", theme::TITLE);
    for n in 1..4 {
        hairline(grove, block, None, Some((quarter * n as f32, 54.0, 94.0)));
    }
    hairline(grove, block, Some((0.0, w, 94.0)), None);
    // Where it stands now, and which sheet of how many.
    let status = cell(grove, block, faces, (0.0, 94.0), "STATUS", "fetching TR-001…", theme::SUB);
    cell(grove, block, faces, (quarter * 3.0, 94.0), "SHEET", "1 / 1", theme::TITLE);
    hairline(grove, block, Some((0.0, w, 130.0)), Some((quarter * 3.0, 94.0, 130.0)));
    // The drawings this one refers to: the rest of the portfolio.
    letter(
        grove,
        block,
        "REF.",
        faces.regular,
        theme::LABEL,
        theme::faint(),
        Location::new().xs(left(8.px()).width(content()), top(146.px()).height(1.letters())),
    );
    let m = measure();
    let mut x = 40.0;
    let mut ways = Vec::new();
    for app in [App::Hub, App::Conservatory, App::Orchard] {
        let name = app.name();
        let mark = if app == App::Hub { icons.home } else { icons.arrow_right };
        let chip = Chip::grow(
            grove,
            block,
            Location::new().xs(
                left(x.px()).width(Chip::width(name)),
                top((130.0 + (h - 130.0 - m.height) / 2.0).px()).height(m.height.px()),
            ),
            Elevation::up(1),
            mark,
            name,
        );
        x += theme::chip_px(name) + 6.0;
        ways.push((app, chip));
    }
    TitleBlock { scale, rev, status, ways }
}

fn revision_block(grove: &mut Grove, paper: Leaf, faces: &Faces) -> RevisionBlock {
    let (w, h) = theme::REVISIONS;
    let block = grove.branch(
        paper,
        Panel::new()
            .color(theme::paper())
            .outline(1.5, theme::line())
            .intangible()
            .elevate(Elevation::up(3))
            .at(cornered((w, h), false)),
    );
    let columns = [(0.0, "REV"), (40.0, "DESCRIPTION"), (w - 52.0, "BY")];
    for (x, name) in columns {
        letter(
            grove,
            block,
            name,
            faces.regular,
            theme::LABEL,
            theme::faint(),
            Location::new().xs(left((x + 8.0).px()).width(content()), top(7.px()).height(1.letters())),
        );
    }
    hairline(grove, block, Some((0.0, w, 24.0)), Some((40.0, 0.0, h)));
    hairline(grove, block, None, Some((w - 52.0, 0.0, h)));
    let rows = (0..5)
        .map(|n| {
            let y = 24.0 + 8.0 + n as f32 * 19.0;
            let at = |x: f32| Location::new().xs(left((x + 8.0).px()).width(content()), top(y.px()).height(1.letters()));
            let rev = letter(grove, block, "", faces.bold, theme::SUB, theme::line(), at(0.0));
            let word = letter(grove, block, "", faces.regular, theme::SUB, theme::line(), at(40.0));
            let by = letter(grove, block, "", faces.regular, theme::SUB, theme::faint(), at(w - 52.0));
            (rev, word, by)
        })
        .collect();
    RevisionBlock { rows }
}
