//! The wires between the cards, each drawn in the line type its link is. What is structure --
//! who owns a spec, what waits on what -- is ruled square, as a drawing's leaders are: an owner's
//! solid line, a dependency's line with its arrow. What is a matter of the calendar is drawn with
//! the French curve: an attendee's fine line, a review's long dashes. A note's leader runs straight
//! to what it is about. Plus the rubber band a new link is drawn with.
//!
//! A broken line is drawn in a fixed number of dashes whatever its length, so only the links that
//! run a middling distance are broken; the many attendances, some of which cross the whole sheet,
//! are drawn whole, fine and quiet.

use foliage::{Color, Elevation, Fill, Grove, Grow, Leaf, Line, Motion, Place, Point, Pollen, Source, Timing, Tween};

use arboretum::sprout::wire::{Spot, dashes, length};
use arboretum::sprout::{Look, Route, Stroke, Wire};

use crate::board::{Board, Edge, Id, Link};
use crate::theme;

/// A card's box on the sheet, in the sheet's pixels: `(left, top, right, bottom)`.
pub type Rect = (f32, f32, f32, f32);

/// How a wire is lit.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Lit {
    /// As drawn.
    Rest,
    /// One of the selected card's: full ink.
    On,
    /// Someone else's, while a card is selected.
    Off,
    /// An attendance that double-books someone: red pencil.
    Clash,
}

/// How quiet a wire at rest is drawn, so the cards stay what the eye lands on: the calendar's
/// curves quieter than the structure's rules.
fn rest(link: Link) -> f32 {
    match link {
        Link::Attends | Link::Reviews => 0.42,
        _ => 0.7,
    }
}

/// The colour and opacity a wire of `link` takes when `lit`.
fn shade(lit: Lit, link: Link) -> (Color, f32) {
    match lit {
        Lit::Rest => (theme::line(), rest(link)),
        Lit::On => (theme::line(), 1.0),
        Lit::Off => (theme::line(), 0.16),
        Lit::Clash => (theme::pencil(), 1.0),
    }
}

/// What a wire of `link` is drawn with.
pub fn look(link: Link, fill: Fill) -> Look {
    let (route, stroke, weight, arrow) = match link {
        Link::Owns => (Route::Elbow, Stroke::Solid, 1.25, false),
        Link::Attends => (Route::Curve, Stroke::Solid, 1.0, false),
        Link::Depends => (Route::Elbow, Stroke::Solid, 1.25, true),
        Link::Reviews => (Route::Curve, Stroke::Dashed, 1.25, false),
        Link::Annotates => (Route::Straight, Stroke::Solid, 1.0, true),
    };
    Look {
        route,
        stroke,
        weight,
        fill,
        arrow,
    }
}

/// Where a wire of `link` from card `a` to card `b` leaves and arrives: side to side where one
/// stands clear of the other across, top to bottom where they are stacked; a note's leader runs
/// straight between the nearest sides.
pub fn ends(a: Rect, b: Rect, link: Link) -> (Spot, Spot) {
    let (acx, acy) = ((a.0 + a.2) / 2.0, (a.1 + a.3) / 2.0);
    let (bcx, bcy) = ((b.0 + b.2) / 2.0, (b.1 + b.3) / 2.0);
    const CLEAR: f32 = 16.0;
    if link == Link::Annotates {
        let (dx, dy) = (bcx - acx, bcy - acy);
        let across = (dx.abs() - (a.2 - a.0 + b.2 - b.0) / 2.0) > (dy.abs() - (a.3 - a.1 + b.3 - b.1) / 2.0);
        return match (across, dx >= 0.0, dy >= 0.0) {
            (true, true, _) => ((a.2, acy), (b.0, bcy)),
            (true, false, _) => ((a.0, acy), (b.2, bcy)),
            (false, _, true) => ((acx, a.3), (bcx, b.1)),
            (false, _, false) => ((acx, a.1), (bcx, b.3)),
        };
    }
    if b.0 >= a.2 + CLEAR {
        ((a.2, acy), (b.0, bcy))
    } else if b.2 <= a.0 - CLEAR {
        ((a.0, acy), (b.2, bcy))
    } else if b.1 >= a.3 {
        ((acx, a.3), (bcx, b.1))
    } else if b.3 <= a.1 {
        ((acx, a.1), (bcx, b.3))
    } else {
        ((a.2, acy), (b.2, bcy))
    }
}

pub struct EdgeView {
    pub edge: Edge,
    wire: Run,
    lit: Lit,
    shown: bool,
}

/// How far apart a hidden line's dashes stand, and how much of that each dash takes: the pitch
/// of the notes' own dashed outlines.
const PITCH: f32 = 11.5;
const DASH: f32 = 0.64;
/// The most dashes a hidden line is drawn in; a longer one opens its pitch out.
const HIDDEN: usize = 56;

/// A wire as the sheet draws it. Most are the sprout's own; a review is a hidden line, dashed at
/// a fixed pitch whatever its length, which the sprout's fixed pool of dashes cannot do -- so the
/// sprout's wire runs unseen beside it, for its route and its bead.
struct Run {
    wire: Wire,
    dashes: Vec<Leaf>,
    lit: usize,
    shown: bool,
}

impl Run {
    fn grow(grove: &mut Grove, paper: Leaf, link: Link) -> Self {
        let lift = Elevation::up(5);
        let mut look = look(link, theme::line().into());
        let hidden = look.stroke == Stroke::Dashed;
        if hidden {
            look.stroke = Stroke::Solid;
        }
        let mut wire = Wire::grow(grove, paper, look, lift);
        let dashes = match hidden {
            false => Vec::new(),
            true => {
                wire.fade(grove, 0.0, 0);
                (0..HIDDEN)
                    .map(|_| {
                        grove.branch(
                            paper,
                            Line::new()
                                .between(Point::new(0.px(), 0.px()), Point::new(0.px(), 0.px()))
                                .weight(look.weight)
                                .color(theme::line())
                                .intangible()
                                .elevate(lift)
                                .visible(false),
                        )
                    })
                    .collect()
            }
        };
        Self {
            wire,
            dashes,
            lit: 0,
            shown: true,
        }
    }

    fn aim(&mut self, grove: &mut Grove, from: Spot, to: Spot) {
        self.wire.aim(grove, from, to);
        if self.dashes.is_empty() {
            return;
        }
        let points = self.wire.points();
        let count = ((length(points) / PITCH).round() as usize).clamp(2, HIDDEN);
        let runs = dashes(points, count, DASH);
        for (n, &dash) in self.dashes.iter().enumerate() {
            match runs.get(n) {
                Some(&(a, b)) => {
                    grove.between(dash, Point::new(a.0.px(), a.1.px()), Point::new(b.0.px(), b.1.px()));
                    if n >= self.lit && self.shown {
                        grove.visible(dash, true);
                    }
                }
                None if n < self.lit => grove.visible(dash, false),
                None => {}
            }
        }
        self.lit = runs.len();
    }

    fn tint(&mut self, grove: &mut Grove, fill: Color, ms: u64) {
        self.wire.tint(grove, fill, ms);
        let timing = Timing::ms(ms).ease(foliage::Ease::Decelerate);
        for &dash in &self.dashes {
            grove.animate(dash, Motion::from(Fill::from(fill)), timing);
        }
    }

    fn fade(&mut self, grove: &mut Grove, opacity: f32, ms: u64) {
        if self.dashes.is_empty() {
            self.wire.fade(grove, opacity, ms);
            return;
        }
        let timing = Timing::ms(ms).ease(foliage::Ease::Decelerate);
        for &dash in &self.dashes {
            grove.animate(dash, Motion::Opacity(opacity), timing);
        }
    }

    fn show(&mut self, grove: &mut Grove, shown: bool) {
        self.shown = shown;
        self.wire.show(grove, shown);
        for (n, &dash) in self.dashes.iter().enumerate() {
            grove.visible(dash, shown && n < self.lit);
        }
    }

    fn pulse(&mut self, grove: &mut Grove, ms: u64, after: u64) {
        self.wire.pulse(grove, ms, after);
    }

    fn pulsing(&self) -> bool {
        self.wire.pulsing()
    }

    fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) {
        self.wire.frame(grove, pollen);
    }

    fn prune(self, grove: &mut Grove) {
        self.wire.prune(grove);
        for dash in self.dashes {
            grove.prune(dash);
        }
    }
}

pub struct Wires {
    paper: Leaf,
    pub views: Vec<EdgeView>,
    band: Wire,
    band_lit: Option<Lit>,
    retract: Option<(Tween, Spot, Spot)>,
    arriving: Option<Tween>,
}

impl Wires {
    pub fn grow(grove: &mut Grove, paper: Leaf) -> Self {
        let band = Wire::grow(
            grove,
            paper,
            Look {
                route: Route::Straight,
                stroke: Stroke::Dashed,
                weight: 1.5,
                fill: theme::line().into(),
                arrow: true,
            },
            Elevation::up(11),
        );
        Self {
            paper,
            views: Vec::new(),
            band,
            band_lit: None,
            retract: None,
            arriving: None,
        }
    }

    /// Grows a wire for every edge on the board that has none, and takes down every wire whose
    /// edge is gone. New wires are unseen until they are aimed.
    pub fn sync(&mut self, grove: &mut Grove, board: &Board) {
        let mut n = 0;
        while n < self.views.len() {
            if board.edges.contains(&self.views[n].edge) {
                n += 1;
            } else {
                let view = self.views.remove(n);
                view.wire.prune(grove);
            }
        }
        for edge in &board.edges {
            if self.views.iter().any(|v| v.edge == *edge) {
                continue;
            }
            let mut wire = Run::grow(grove, self.paper, edge.link);
            wire.fade(grove, rest(edge.link), 0);
            self.views.push(EdgeView {
                edge: *edge,
                wire,
                lit: Lit::Rest,
                shown: true,
            });
        }
    }

    /// Aims every wire at the cards it joins, where `rect` says each card is drawn. A wire whose
    /// ends did not move is left as it is.
    pub fn aim(&mut self, grove: &mut Grove, rect: impl Fn(Id) -> Option<Rect>) {
        for view in &mut self.views {
            let (Some(a), Some(b)) = (rect(view.edge.from), rect(view.edge.to)) else {
                continue;
            };
            let (from, to) = ends(a, b, view.edge.link);
            view.wire.aim(grove, from, to);
        }
    }

    /// Lights every wire for `lit`, and shows only those `shown` allows.
    pub fn light(&mut self, grove: &mut Grove, lit: impl Fn(&Edge) -> Lit, shown: impl Fn(&Edge) -> bool) {
        for view in &mut self.views {
            let show = shown(&view.edge);
            if show != view.shown {
                view.shown = show;
                view.wire.show(grove, show);
            }
            let now = lit(&view.edge);
            if now == view.lit {
                continue;
            }
            let (fill, opacity) = shade(now, view.edge.link);
            if view.lit == Lit::Clash || now == Lit::Clash {
                view.wire.tint(grove, fill, 260);
            }
            // While the wires are still to come in, only the colour is set; they arrive lit.
            if self.arriving.is_none() {
                view.wire.fade(grove, opacity, 260);
            }
            view.lit = now;
        }
    }

    /// Sends a bead out along every wire of `id`, one after another.
    pub fn pulse(&mut self, grove: &mut Grove, id: Id) {
        let mut n = 0;
        for view in &mut self.views {
            if view.shown && (view.edge.from == id || view.edge.to == id) {
                view.wire.pulse(grove, 720, n * 70);
                n += 1;
            }
        }
    }

    /// Brings the wires in after the cards, a few at a time.
    pub fn arrive(&mut self, grove: &mut Grove, after: u64) {
        for view in &mut self.views {
            view.wire.fade(grove, 0.0, 0);
        }
        self.arriving = Some(grove.timer(Timing::ms(after)));
    }

    /// Carries the beads along.
    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) {
        if let Some(timer) = self.arriving
            && pollen.finished(timer)
        {
            self.arriving = None;
            for view in &mut self.views {
                view.wire.fade(grove, shade(view.lit, view.edge.link).1, 600);
            }
        }
        for view in &mut self.views {
            if view.wire.pulsing() {
                view.wire.frame(grove, pollen);
            }
        }
        if let Some((tween, from, end)) = self.retract {
            if let Some(t) = pollen.tween(tween) {
                let to = (end.0 + (from.0 - end.0) * t, end.1 + (from.1 - end.1) * t);
                self.band.aim(grove, from, to);
            }
            if pollen.finished(tween) {
                self.retract = None;
                self.band.show(grove, false);
            }
        }
    }

    /// Pulls the rubber band from `from` to `to`, lit for whether letting go there would link.
    pub fn stretch(&mut self, grove: &mut Grove, from: Spot, to: Spot, lit: Lit) {
        if let Some((tween, _, _)) = self.retract.take() {
            grove.stop(tween);
        }
        if self.band_lit != Some(lit) {
            let fill = match lit {
                Lit::On => theme::line(),
                Lit::Clash => theme::pencil(),
                _ => theme::faint(),
            };
            self.band.tint(grove, fill, 120);
            self.band_lit = Some(lit);
        }
        self.band.show(grove, true);
        self.band.aim(grove, from, to);
    }

    /// Lets the rubber band go: snapped back to where it was drawn from, or simply gone.
    pub fn release(&mut self, grove: &mut Grove, snap_back: bool) {
        let points = self.band.points().to_vec();
        match (snap_back, points.first(), points.last()) {
            (true, Some(&from), Some(&end)) => {
                let tween = grove.tween(0.0, 1.0, Timing::ms(240).ease(foliage::Ease::Accelerate));
                self.retract = Some((tween, from, end));
            }
            _ => self.band.show(grove, false),
        }
        self.band_lit = None;
    }
}

#[cfg(test)]
mod tests {
    use super::ends;
    use crate::board::Link;

    #[test]
    fn a_wire_runs_side_to_side_between_cards_clear_of_each_other() {
        let a = (0.0, 0.0, 100.0, 50.0);
        let b = (300.0, 200.0, 400.0, 260.0);
        assert_eq!(ends(a, b, Link::Owns), ((100.0, 25.0), (300.0, 230.0)));
        assert_eq!(ends(b, a, Link::Owns), ((300.0, 230.0), (100.0, 25.0)));
    }

    #[test]
    fn stacked_cards_are_joined_top_to_bottom() {
        let a = (0.0, 0.0, 200.0, 80.0);
        let b = (20.0, 200.0, 220.0, 280.0);
        assert_eq!(ends(a, b, Link::Depends), ((100.0, 80.0), (120.0, 200.0)));
        assert_eq!(ends(b, a, Link::Depends), ((120.0, 200.0), (100.0, 80.0)));
    }

    #[test]
    fn a_leader_takes_the_nearest_sides() {
        let note = (500.0, 0.0, 680.0, 60.0);
        let spec = (0.0, 20.0, 200.0, 100.0);
        assert_eq!(ends(note, spec, Link::Annotates), ((500.0, 30.0), (200.0, 60.0)));
        let below = (0.0, 300.0, 200.0, 380.0);
        let above = (40.0, 0.0, 220.0, 60.0);
        assert_eq!(ends(above, below, Link::Annotates), ((130.0, 60.0), (100.0, 300.0)));
    }
}
