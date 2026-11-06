//! Things laid over a page: an [`Overlay`], which comes up over it where it stands, and a
//! [`Drawer`], which slides in from an edge over a scrim.
//!
//! Both take the page out of reach while they are open, hold focus inside themselves as the
//! [`focus_scope`](Place::focus_scope) they are, and close on `Escape`, which foliage reports as
//! [`escaped`](Pollen::escaped) wherever focus was. Both take every press that
//! lands on them, so a stray press inside one focuses it rather than nothing: what is grown in one
//! that is not a press should be intangible.
//!
//! What either holds is the app's. They close themselves on `Escape` and on a press of the scrim,
//! and say so, the frame they do; whatever closing means to the app -- a request let go, a preview
//! put to sleep -- follows from that, and from the app's own presses, through its own close, which
//! closing either of these again leaves alone.

use foliage::{
    Boxed, Color, Ease, Elevation, Grove, Grow, Leaf, Location, Motion, Palette, Panel, Place,
    Pollen, Source, Stem, Timing, Tween, bottom, left, right, top,
};

use crate::chip::caption;

/// How long an overlay takes to come up, and to go.
const UP_MS: u64 = 280;
const DOWN_MS: u64 = 220;
/// How long a drawer takes to come in, and to go.
const IN_MS: u64 = 380;
const OUT_MS: u64 = 260;
/// How dark a drawer's scrim is at its darkest.
const SCRIM: f32 = 0.42;
/// How far a drawer's body stands in from its sheet's edges.
const INSET: f32 = 20.0;

/// Why an overlay or a drawer closed itself.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Closed {
    /// The scrim behind a drawer was pressed.
    Scrim,
    /// `Escape` was pressed while it was open.
    Escape,
}

/// Which edge a drawer comes in from.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Edge {
    Right,
    Bottom,
}

/// What anything laid over a page does, whatever it looks like coming and going: the page out of
/// reach while it is open, focus held inside it, `Escape` heard, and the time it takes to go kept,
/// so it is hidden once it has.
struct Laid {
    page: Leaf,
    open: bool,
    going: Option<Tween>,
    /// Opened since `Escape` was last asked about: the frame it opened in may carry an `Escape`
    /// pressed before it was there, which is not for it.
    fresh: bool,
}

impl Laid {
    fn new(page: Leaf) -> Self {
        Self {
            page,
            open: false,
            going: None,
            fresh: false,
        }
    }

    /// Opens. Whether it was closed. Focus is the caller's to move, once what holds it is shown:
    /// foliage drops focus sent to anything hidden.
    fn open(&mut self, grove: &mut Grove) -> bool {
        if self.open {
            return false;
        }
        self.open = true;
        self.fresh = true;
        if let Some(going) = self.going.take() {
            grove.stop(going);
        }
        grove.disable(self.page);
        true
    }

    /// Closes, taking `ms` to go. Whether it was open.
    fn close(&mut self, grove: &mut Grove, ms: u64) -> bool {
        if !self.open {
            return false;
        }
        self.open = false;
        grove.enable(self.page);
        self.going = Some(grove.timer(Timing::ms(ms)));
        if grove.focused().is_some() {
            grove.unfocus();
        }
        true
    }

    /// Whether it finished going this frame, and what it is laid on can be hidden.
    fn gone(&mut self, pollen: &Pollen) -> bool {
        match self.going {
            Some(going) if pollen.finished(going) => {
                self.going = None;
                true
            }
            _ => false,
        }
    }

    /// Whether any of it is on screen: open, or not yet gone.
    fn shown(&self) -> bool {
        self.open || self.going.is_some()
    }

    /// Whether `Escape` was pressed this frame while it was open.
    ///
    /// The first time it is asked after opening is passed over. The frame it opened in may carry
    /// an `Escape` pressed before it was there -- the one that closed a drawer whose closing
    /// opened it, say -- and that one is not for it.
    fn escaped(&mut self, pollen: &Pollen) -> bool {
        let fresh = core::mem::replace(&mut self.fresh, false);
        self.open && !fresh && pollen.escaped()
    }
}

/// A sheet that comes up over a page where it stands -- a viewing room, a product's detail, a
/// checkout -- with the page out of reach behind it while it is open.
///
/// The caller says where the sheet stands, which need not be the whole window: under a bar that
/// stays in reach, say. It is in the scheme's surface; what more it wears, and everything on it,
/// is grown on [`sheet`](Overlay::sheet). It comes up and goes by fading.
pub struct Overlay {
    laid: Laid,
    sheet: Leaf,
    hold: Leaf,
}

impl Overlay {
    /// An overlay under `under` -- which must not be inside `page` -- its sheet standing where `at`
    /// says, `lift` above what is under it, taking `page` out of reach while it is open. Closed.
    pub fn grow(grove: &mut Grove, under: Leaf, page: Leaf, at: Location, lift: Elevation) -> Self {
        let sheet = grove.branch(
            under,
            Panel::new()
                .color(Palette::Surface)
                .interactive()
                .focus_scope()
                .elevate(lift)
                .visible(false)
                .opacity(0.0)
                .at(at),
        );
        Self {
            laid: Laid::new(page),
            sheet,
            hold: sheet,
        }
    }

    /// The sheet: where everything the overlay holds is grown, and what takes every press that
    /// lands on none of it.
    pub fn sheet(&self) -> Leaf {
        self.sheet
    }

    /// Says what takes focus when it opens: the sheet, unless said -- a scrolling body that keys
    /// should reach, say.
    pub fn hold(&mut self, leaf: Leaf) {
        self.hold = leaf;
    }

    /// Whether it is open, or coming up.
    pub fn is_open(&self) -> bool {
        self.laid.open
    }

    /// Whether any of it is on screen: open, or still going. Once it is not, what it held can be
    /// let go of.
    pub fn is_shown(&self) -> bool {
        self.laid.shown()
    }

    /// Brings the overlay up. Nothing, if it is open.
    pub fn open(&mut self, grove: &mut Grove) {
        if !self.laid.open(grove) {
            return;
        }
        grove.visible(self.sheet, true);
        grove.focus(self.hold);
        grove.animate(self.sheet, Motion::Opacity(1.0), Timing::ms(UP_MS).ease(Ease::Decelerate));
    }

    /// Takes the overlay down. Nothing, if it is closed.
    pub fn close(&mut self, grove: &mut Grove) {
        if !self.laid.close(grove, DOWN_MS) {
            return;
        }
        grove.animate(self.sheet, Motion::Opacity(0.0), Timing::ms(DOWN_MS).ease(Ease::Accelerate));
    }

    /// Watches for `Escape`, and closes on it. Reports that it closed, the frame it did. Call once
    /// a frame, open or not: it hides the sheet once it has gone.
    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) -> Option<Closed> {
        if self.laid.gone(pollen) {
            grove.visible(self.sheet, false);
        }
        if self.laid.escaped(pollen) {
            self.close(grove);
            return Some(Closed::Escape);
        }
        None
    }
}

/// A sheet that slides in from an edge over a darkened scrim, with the page behind it out of reach
/// while it is open: a bag, a tool's options, a list to pick from.
///
/// It closes itself on a press of the scrim and on `Escape`. Everything it holds is grown in its
/// [`body`](Drawer::body), inset from the sheet's edges.
pub struct Drawer {
    laid: Laid,
    scrim: Leaf,
    sheet: Leaf,
    body: Leaf,
    edge: Edge,
    size: f32,
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
                .font_size(caption())
                .at(Location::new().xs(
                    left(INSET.px()).right(100.pct() - INSET.px()),
                    top(INSET.px()).bottom(100.pct() - INSET.px()),
                )),
        );
        Self {
            laid: Laid::new(page),
            scrim,
            sheet,
            body,
            edge,
            size,
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
        self.laid.open
    }

    /// Whether any of it is on screen: open, or still going.
    pub fn is_shown(&self) -> bool {
        self.laid.shown()
    }

    /// Opens the drawer. Nothing, if it is open.
    pub fn open(&mut self, grove: &mut Grove) {
        if !self.laid.open(grove) {
            return;
        }
        grove.visible(self.scrim, true);
        grove.visible(self.sheet, true);
        grove.focus(self.sheet);
        grove.animate(self.scrim, Motion::Opacity(1.0), Timing::ms(IN_MS).ease(Ease::Decelerate));
        grove.animate(
            self.sheet,
            Motion::Location(here(self.edge, self.size)),
            Timing::ms(IN_MS).ease(Ease::Emphasis),
        );
    }

    /// Closes the drawer. Nothing, if it is closed.
    pub fn close(&mut self, grove: &mut Grove) {
        if !self.laid.close(grove, OUT_MS) {
            return;
        }
        grove.animate(self.scrim, Motion::Opacity(0.0), Timing::ms(OUT_MS).ease(Ease::Accelerate));
        grove.animate(
            self.sheet,
            Motion::Location(away(self.edge, self.size)),
            Timing::ms(OUT_MS).ease(Ease::Accelerate),
        );
    }

    /// Watches for the scrim being pressed and for `Escape`, and closes on either. Reports why it
    /// closed, the frame it did. Call once a frame, open or not: it hides itself once it has gone.
    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) -> Option<Closed> {
        if self.laid.gone(pollen) {
            grove.visible(self.scrim, false);
            grove.visible(self.sheet, false);
        }
        if !self.laid.open {
            return None;
        }
        if pollen.clicked(self.scrim) {
            self.close(grove);
            return Some(Closed::Scrim);
        }
        if self.laid.escaped(pollen) {
            self.close(grove);
            return Some(Closed::Escape);
        }
        None
    }
}

/// Where a drawer's sheet stands open.
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

/// Where a drawer's sheet waits, just past its edge.
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
