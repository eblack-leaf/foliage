//! A sheet that slides in from an edge, over a scrim.

use super::escaped::Escaped;
use foliage::{
    Boxed, Color, Ease, Elevation, Grove, Grow, Leaf, Location, Motion, Palette, Panel, Place,
    Pollen, Source, Stem, Timing, Tween, bottom, left, right, top,
};

/// How long the sheet takes to come in, and to go.
const IN_MS: u64 = 380;
const OUT_MS: u64 = 260;
/// How dark the scrim is at its darkest.
const SCRIM: f32 = 0.42;

/// Which edge a drawer comes in from.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Edge {
    Right,
    Bottom,
}

/// Why a drawer closed itself.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Closed {
    /// The scrim behind it was pressed.
    Scrim,
    /// Escape was pressed while it was open.
    Escape,
}

/// A sheet that slides in from an edge over a darkened scrim, with the page behind it out of reach
/// while it is open.
///
/// It closes itself on a press of the scrim and on Escape. Escape never reaches an app -- it takes
/// focus away, and that is all -- so the sheet takes focus when it opens, as the
/// [`focus_scope`](Place::focus_scope) it is, and focus going from somewhere to nowhere while it is
/// open is read as Escape by [`Escaped`] -- unless a pointer came up, or what held focus went out of
/// reach, as a press armed inert does. Every part grown in the sheet's [`body`](Drawer::body) that
/// is not a press is intangible, so a stray press lands on the sheet and focuses it.
pub struct Drawer {
    page: Leaf,
    scrim: Leaf,
    sheet: Leaf,
    body: Leaf,
    edge: Edge,
    size: f32,
    open: bool,
    going: Option<Tween>,
    escape: Escaped,
}

impl Drawer {
    /// A drawer under `under` -- which must not be inside `page` -- coming in from `edge`, `size`
    /// pixels deep (never more than the window), taking `page` out of reach while it is open.
    /// Closed.
    pub fn grow(grove: &mut Grove, under: Leaf, page: Leaf, edge: Edge, size: f32) -> Self {
        let scrim = grove.branch(
            under,
            Panel::new()
                .color(Color::rgba(0.02, 0.02, 0.04, SCRIM))
                .interactive()
                .elevate(Elevation::up(20))
                .visible(false)
                .opacity(0.0)
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(0.px()).bottom(100.pct()),
                )),
        );
        let sheet = grove.branch(
            under,
            Panel::new()
                .color(Palette::Surface)
                .interactive()
                .focus_scope()
                .elevate(Elevation::up(21))
                .visible(false)
                .at(away(edge, size)),
        );
        let body = grove.branch(
            sheet,
            Stem::new()
                .intangible()
                .elevate(Elevation::up(1))
                .font_size(lichen::caption())
                .at(Location::new().xs(
                    left(20.px()).right(100.pct() - 20.px()),
                    top(20.px()).bottom(100.pct() - 20.px()),
                )),
        );
        Self {
            page,
            scrim,
            sheet,
            body,
            edge,
            size,
            open: false,
            going: None,
            escape: Escaped::new(),
        }
    }

    /// Where to grow what the drawer holds: inside its sheet, inset from its edges.
    pub fn body(&self) -> Leaf {
        self.body
    }

    /// The sheet, which takes focus while the drawer is open.
    pub fn sheet(&self) -> Leaf {
        self.sheet
    }

    /// Whether it is open, or opening.
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Opens the drawer. Nothing, if it is open.
    pub fn open(&mut self, grove: &mut Grove) {
        if self.open {
            return;
        }
        self.open = true;
        if let Some(going) = self.going.take() {
            grove.stop(going);
        }
        grove.disable(self.page);
        grove.visible(self.scrim, true);
        grove.visible(self.sheet, true);
        grove.animate(self.scrim, Motion::Opacity(1.0), Timing::ms(IN_MS).ease(Ease::Decelerate));
        grove.animate(
            self.sheet,
            Motion::Location(here(self.edge, self.size)),
            Timing::ms(IN_MS).ease(Ease::Emphasis),
        );
        grove.focus(self.sheet);
        self.escape.forget();
    }

    /// Closes the drawer. Nothing, if it is closed.
    pub fn close(&mut self, grove: &mut Grove) {
        if !self.open {
            return;
        }
        self.open = false;
        grove.enable(self.page);
        grove.animate(self.scrim, Motion::Opacity(0.0), Timing::ms(OUT_MS).ease(Ease::Accelerate));
        grove.animate(
            self.sheet,
            Motion::Location(away(self.edge, self.size)),
            Timing::ms(OUT_MS).ease(Ease::Accelerate),
        );
        self.going = Some(grove.timer(Timing::ms(OUT_MS)));
        if grove.focused().is_some() {
            grove.unfocus();
        }
    }

    /// Watches for the scrim being pressed and for Escape, and closes on either. Reports why it
    /// closed, the frame it did.
    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) -> Option<Closed> {
        if let Some(going) = self.going
            && pollen.finished(going)
        {
            self.going = None;
            grove.visible(self.scrim, false);
            grove.visible(self.sheet, false);
        }
        if !self.open {
            return None;
        }
        if pollen.clicked(self.scrim) {
            self.close(grove);
            return Some(Closed::Scrim);
        }
        let escaped = self.escape.frame(grove);
        if escaped {
            self.close(grove);
            return Some(Closed::Escape);
        }
        None
    }
}

/// Where the sheet stands open.
fn here(edge: Edge, size: f32) -> Location {
    match edge {
        Edge::Right => Location::new().xs(
            right(100.pct()).width(size.px()).at_most(100.pct()),
            top(0.px()).bottom(100.pct()),
        ),
        Edge::Bottom => Location::new().xs(
            left(0.px()).right(100.pct()),
            bottom(100.pct()).height(size.px()).at_most(92.pct()),
        ),
    }
}

/// Where the sheet waits, just past its edge.
fn away(edge: Edge, size: f32) -> Location {
    match edge {
        Edge::Right => Location::new().xs(
            left(100.pct() + 24.px()).width(size.px()).at_most(100.pct()),
            top(0.px()).bottom(100.pct()),
        ),
        Edge::Bottom => Location::new().xs(
            left(0.px()).right(100.pct()),
            top(100.pct() + 24.px()).height(size.px()).at_most(92.pct()),
        ),
    }
}
