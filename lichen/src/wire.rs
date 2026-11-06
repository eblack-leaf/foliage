//! A connector between two points: its route, how it is stroked, an arrowhead, and a bead sent along
//! it.
//!
//! The geometry is pure and stands on its own -- [`route`], [`length`], [`along`], [`dashes`] -- so
//! it is tested without a surface, and a [`Wire`] is only those numbers written into a fixed pool
//! of strokes.

use std::f32::consts::FRAC_PI_2;

use foliage::{
    Boxed, Cap, Ease, Elevation, Fill, Grove, Grow, Leaf, Line, Location, Motion, Place, Point,
    Polygon, Pollen, Shape, Source, Timing, Tween, left, top,
};

/// A point on the surface a wire is grown into, in pixels.
pub type Spot = (f32, f32);

/// How a wire gets from one end to the other.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Route {
    /// The shortest way.
    Straight,
    /// Across, down, and across again, its corners chamfered: what a drawing's leader does.
    Elbow,
    /// A cubic easing out of one end and into the other, both ways horizontal.
    Curve,
}

/// How a wire is stroked.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Stroke {
    Solid,
    Dashed,
    Dotted,
}

/// How far in from a corner an elbow's chamfer starts, at most.
const CHAMFER: f32 = 10.0;
/// How many straight pieces a curve is drawn in.
const PIECES: usize = 16;
/// How many dashes or dots a broken stroke is drawn in, whatever its length.
const DASHED: usize = 9;
const DOTTED: usize = 14;
/// How much of each step a dash takes.
const DUTY: f32 = 0.58;

/// The points a wire of `route` passes through from `from` to `to`, ends included, with no point
/// repeated.
pub fn route(route: Route, from: Spot, to: Spot) -> Vec<Spot> {
    let points = match route {
        Route::Straight => vec![from, to],
        Route::Elbow => elbow(from, to),
        Route::Curve => curve(from, to),
    };
    let mut out: Vec<Spot> = Vec::with_capacity(points.len());
    for point in points {
        if out.last().is_none_or(|last| distance(*last, point) > 0.01) {
            out.push(point);
        }
    }
    if out.len() == 1 {
        out.push(to);
    }
    out
}

fn elbow(from: Spot, to: Spot) -> Vec<Spot> {
    let mid = (from.0 + to.0) / 2.0;
    let rise = to.1 - from.1;
    let reach = (mid - from.0).abs();
    let c = CHAMFER.min(reach).min(rise.abs() / 2.0);
    if c < 0.5 {
        // Level, or as good as: across and no corners.
        return vec![from, (mid, from.1), (mid, to.1), to];
    }
    let across = (mid - from.0).signum();
    let down = rise.signum();
    vec![
        from,
        (mid - across * c, from.1),
        (mid, from.1 + down * c),
        (mid, to.1 - down * c),
        (mid + across * c, to.1),
        to,
    ]
}

fn curve(from: Spot, to: Spot) -> Vec<Spot> {
    let reach = ((to.0 - from.0).abs() * 0.5).max(36.0);
    let lean = if to.0 >= from.0 { 1.0 } else { -1.0 };
    let c1 = (from.0 + lean * reach, from.1);
    let c2 = (to.0 - lean * reach, to.1);
    (0..=PIECES)
        .map(|n| {
            let t = n as f32 / PIECES as f32;
            let u = 1.0 - t;
            let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            (
                a * from.0 + b * c1.0 + c * c2.0 + d * to.0,
                a * from.1 + b * c1.1 + c * c2.1 + d * to.1,
            )
        })
        .collect()
}

fn distance(a: Spot, b: Spot) -> f32 {
    ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt()
}

/// How long a run of points is, end to end.
pub fn length(points: &[Spot]) -> f32 {
    points.windows(2).map(|pair| distance(pair[0], pair[1])).sum()
}

/// Where along `points` a fraction `t` of the way is, and which way the run is heading there, as a
/// unit vector.
pub fn along(points: &[Spot], t: f32) -> (Spot, Spot) {
    let total = length(points);
    let mut left = t.clamp(0.0, 1.0) * total;
    let mut heading = (1.0, 0.0);
    for pair in points.windows(2) {
        let piece = distance(pair[0], pair[1]);
        if piece <= 0.0 {
            continue;
        }
        heading = ((pair[1].0 - pair[0].0) / piece, (pair[1].1 - pair[0].1) / piece);
        if left <= piece {
            let f = left / piece;
            let at = (
                pair[0].0 + (pair[1].0 - pair[0].0) * f,
                pair[0].1 + (pair[1].1 - pair[0].1) * f,
            );
            return (at, heading);
        }
        left -= piece;
    }
    (*points.last().unwrap_or(&(0.0, 0.0)), heading)
}

/// `count` dashes evenly along `points`, each `duty` of its step, as their two ends. A dash that
/// runs round a corner is drawn as the chord across it.
pub fn dashes(points: &[Spot], count: usize, duty: f32) -> Vec<(Spot, Spot)> {
    let step = 1.0 / count.max(1) as f32;
    (0..count)
        .map(|n| {
            let start = n as f32 * step;
            (along(points, start).0, along(points, start + step * duty).0)
        })
        .collect()
}

/// What a wire is drawn with.
#[derive(Copy, Clone, Debug)]
pub struct Look {
    pub route: Route,
    pub stroke: Stroke,
    pub weight: f32,
    pub fill: Fill,
    pub arrow: bool,
}

/// A wire, grown: a fixed pool of strokes re-aimed whenever its ends move, an arrowhead, and a
/// bead.
///
/// The pool is sized by the route and the stroke, so aiming is only rewriting where each stroke
/// runs -- a wire dragged about at a frame's pace never grows or prunes anything. Every part of it
/// is intangible: a line's hit box is the rectangle round it, and a wire across a sheet would
/// otherwise take every press on the paper.
pub struct Wire {
    look: Look,
    strokes: Vec<Leaf>,
    lit: usize,
    head: Option<Leaf>,
    bead: Leaf,
    pulse: Option<Tween>,
    points: Vec<Spot>,
    radius: f32,
}

impl Wire {
    /// A wire under `under` looking like `look`, `lift` in front, aimed nowhere yet. Points it is
    /// aimed at are in `under`'s own pixels.
    pub fn grow(grove: &mut Grove, under: Leaf, look: Look, lift: Elevation) -> Self {
        let pool = match look.stroke {
            Stroke::Solid => match look.route {
                Route::Straight => 1,
                Route::Elbow => 5,
                Route::Curve => PIECES,
            },
            Stroke::Dashed => DASHED,
            Stroke::Dotted => DOTTED,
        };
        let strokes = (0..pool)
            .map(|_| {
                grove.branch(
                    under,
                    Line::new()
                        .between(Point::new(0.px(), 0.px()), Point::new(0.px(), 0.px()))
                        .weight(look.weight)
                        .color(look.fill)
                        .cap(Cap::Round)
                        .intangible()
                        .elevate(lift)
                        .visible(false),
                )
            })
            .collect();
        let radius = (look.weight * 2.6).max(4.0);
        let head = look.arrow.then(|| {
            grove.branch(
                under,
                Polygon::new()
                    .sides(3.0)
                    .rounding(0.15)
                    .color(look.fill)
                    .intangible()
                    .elevate(lift)
                    .visible(false)
                    .at(dot(0.0, 0.0, radius)),
            )
        });
        let bead = grove.branch(
            under,
            Polygon::circle()
                .color(look.fill)
                .intangible()
                .elevate(lift)
                .opacity(0.0)
                .at(dot(0.0, 0.0, radius * 0.9)),
        );
        Self {
            look,
            strokes,
            lit: 0,
            head,
            bead,
            pulse: None,
            points: Vec::new(),
            radius,
        }
    }

    /// Where the wire runs now.
    pub fn points(&self) -> &[Spot] {
        &self.points
    }

    /// Aims the wire from `from` to `to`. Nothing, if it runs there already.
    pub fn aim(&mut self, grove: &mut Grove, from: Spot, to: Spot) {
        let points = route(self.look.route, from, to);
        if points == self.points {
            return;
        }
        // The stroke stops short of an arrowhead, so the point of the head is the end.
        let drawn = match self.head {
            Some(_) => {
                let (end, heading) = along(&points, 1.0);
                let mut cut = points.clone();
                let back = self.radius * 1.2;
                if let Some(last) = cut.last_mut()
                    && length(&points) > back * 2.0
                {
                    *last = (end.0 - heading.0 * back, end.1 - heading.1 * back);
                }
                cut
            }
            None => points.clone(),
        };
        let runs: Vec<(Spot, Spot)> = match self.look.stroke {
            Stroke::Solid => drawn.windows(2).map(|pair| (pair[0], pair[1])).collect(),
            Stroke::Dashed => dashes(&drawn, DASHED, DUTY),
            Stroke::Dotted => dashes(&drawn, DOTTED, 0.02),
        };
        for (n, &stroke) in self.strokes.iter().enumerate() {
            match runs.get(n) {
                Some(&(a, b)) => {
                    // A dot is a stroke too short to see the length of, which its round cap makes
                    // round.
                    let b = if distance(a, b) < 0.4 { (a.0 + 0.4, a.1) } else { b };
                    grove.between(stroke, Point::new(a.0.px(), a.1.px()), Point::new(b.0.px(), b.1.px()));
                    if n >= self.lit {
                        grove.visible(stroke, true);
                    }
                }
                None if n < self.lit => grove.visible(stroke, false),
                None => {}
            }
        }
        self.lit = runs.len().min(self.strokes.len());
        if let Some(head) = self.head {
            let (end, heading) = along(&points, 1.0);
            let r = self.radius;
            let centre = (end.0 - heading.0 * r, end.1 - heading.1 * r);
            grove.at(head, dot(centre.0, centre.1, r));
            grove.reshape(
                head,
                Shape {
                    sides: 3.0,
                    rounding: 0.15,
                    rotation: heading.1.atan2(heading.0) - FRAC_PI_2,
                },
            );
            grove.visible(head, true);
        }
        self.points = points;
    }

    /// Recolours the whole wire over `ms`.
    pub fn tint(&mut self, grove: &mut Grove, fill: impl Into<Fill>, ms: u64) {
        let fill = fill.into();
        self.look.fill = fill;
        let timing = Timing::ms(ms).ease(Ease::Decelerate);
        for &stroke in &self.strokes {
            grove.animate(stroke, Motion::from(fill), timing);
        }
        if let Some(head) = self.head {
            grove.animate(head, Motion::from(fill), timing);
        }
        grove.animate(self.bead, Motion::from(fill), timing);
    }

    /// Fades the whole wire to `opacity` over `ms`: lit, dimmed, or gone.
    pub fn fade(&mut self, grove: &mut Grove, opacity: f32, ms: u64) {
        let timing = Timing::ms(ms).ease(Ease::Decelerate);
        for &stroke in &self.strokes {
            grove.animate(stroke, Motion::Opacity(opacity), timing);
        }
        if let Some(head) = self.head {
            grove.animate(head, Motion::Opacity(opacity), timing);
        }
    }

    /// Shows the wire or hides it outright.
    pub fn show(&mut self, grove: &mut Grove, shown: bool) {
        for (n, &stroke) in self.strokes.iter().enumerate() {
            grove.visible(stroke, shown && n < self.lit);
        }
        if let Some(head) = self.head {
            grove.visible(head, shown && !self.points.is_empty());
        }
        if !shown {
            self.halt(grove);
        }
    }

    /// Sends a bead from one end to the other over `ms`.
    pub fn pulse(&mut self, grove: &mut Grove, ms: u64, after: u64) {
        if self.points.len() < 2 {
            return;
        }
        self.halt(grove);
        let start = self.points[0];
        grove.at(self.bead, dot(start.0, start.1, self.radius * 0.9));
        grove.animate(self.bead, Motion::Opacity(1.0), Timing::ms(120).after(after));
        self.pulse = Some(grove.tween(0.0, 1.0, Timing::ms(ms).after(after).ease(Ease::Emphasis)));
    }

    /// Whether a bead is on its way.
    pub fn pulsing(&self) -> bool {
        self.pulse.is_some()
    }

    /// Carries a bead along.
    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) {
        let Some(pulse) = self.pulse else {
            return;
        };
        if let Some(t) = pollen.tween(pulse) {
            let (at, _) = along(&self.points, t);
            grove.at(self.bead, dot(at.0, at.1, self.radius * 0.9));
        }
        if pollen.finished(pulse) {
            self.pulse = None;
            grove.animate(self.bead, Motion::Opacity(0.0), Timing::ms(220));
        }
    }

    /// Takes the wire down.
    pub fn prune(self, grove: &mut Grove) {
        for stroke in self.strokes {
            grove.prune(stroke);
        }
        if let Some(head) = self.head {
            grove.prune(head);
        }
        grove.prune(self.bead);
    }

    fn halt(&mut self, grove: &mut Grove) {
        if let Some(pulse) = self.pulse.take() {
            grove.stop(pulse);
            grove.opacity(self.bead, 0.0);
        }
    }
}

/// A square box `2r` across centred on `(x, y)`, in the trunk's pixels.
fn dot(x: f32, y: f32, r: f32) -> Location {
    Location::new().xs(
        left((x - r).px()).width((2.0 * r).px()),
        top((y - r).px()).height((2.0 * r).px()),
    )
}

#[cfg(test)]
mod tests {
    use super::{Route, along, dashes, length, route};

    fn near(a: (f32, f32), b: (f32, f32)) -> bool {
        (a.0 - b.0).abs() < 1e-3 && (a.1 - b.1).abs() < 1e-3
    }

    #[test]
    fn every_route_starts_and_ends_where_it_was_aimed() {
        for kind in [Route::Straight, Route::Elbow, Route::Curve] {
            let points = route(kind, (10.0, 20.0), (210.0, 140.0));
            assert!(near(points[0], (10.0, 20.0)), "{kind:?}");
            assert!(near(*points.last().unwrap(), (210.0, 140.0)), "{kind:?}");
        }
    }

    #[test]
    fn an_elbow_is_orthogonal_but_for_its_chamfers() {
        let points = route(Route::Elbow, (0.0, 0.0), (200.0, 100.0));
        assert_eq!(points.len(), 6);
        // Across, chamfer, down, chamfer, across.
        assert_eq!(points[0].1, points[1].1);
        assert_eq!(points[2].0, points[3].0);
        assert_eq!(points[4].1, points[5].1);
        // A level elbow is a straight run with no corners to cut.
        let level = route(Route::Elbow, (0.0, 50.0), (100.0, 50.0));
        assert!(level.iter().all(|p| p.1 == 50.0));
    }

    #[test]
    fn walking_along_a_run_measures_it() {
        let points = vec![(0.0, 0.0), (100.0, 0.0), (100.0, 100.0)];
        assert_eq!(length(&points), 200.0);
        let (mid, heading) = along(&points, 0.5);
        assert!(near(mid, (100.0, 0.0)));
        assert!(near(heading, (1.0, 0.0)));
        let (three_quarters, heading) = along(&points, 0.75);
        assert!(near(three_quarters, (100.0, 50.0)));
        assert!(near(heading, (0.0, 1.0)));
        assert!(near(along(&points, 2.0).0, (100.0, 100.0)));
    }

    #[test]
    fn dashes_are_spread_evenly_and_stop_short() {
        let points = vec![(0.0, 0.0), (90.0, 0.0)];
        let cut = dashes(&points, 9, 0.5);
        assert_eq!(cut.len(), 9);
        assert!(near(cut[0].0, (0.0, 0.0)));
        assert!(near(cut[0].1, (5.0, 0.0)));
        assert!(near(cut[8].0, (80.0, 0.0)));
    }

    #[test]
    fn a_curve_leaves_and_arrives_level() {
        let points = route(Route::Curve, (0.0, 0.0), (300.0, 200.0));
        let (_, out) = along(&points, 0.0);
        let (_, into) = along(&points, 1.0);
        assert!(out.1.abs() < 0.35, "leaves at {out:?}");
        assert!(into.1.abs() < 0.35, "arrives at {into:?}");
    }
}
