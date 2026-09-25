//! Cutting a shape: a straight line across it, the part on one side, and the line drawn.
//!
//! A [`Cut`] is stated in the terms the outline was traced in -- fractions of the page -- because
//! it is drawn on the same sketch. [`Silhouette::cut`] keeps one side of it, [`Silhouette::crossings`]
//! says where the line enters and leaves the outline, and [`Silhouette::facing`] which way the side
//! it cut away lies. A [`Section`] is the line itself, dashed across the shape, drawn and taken
//! away.

use foliage::{
    Cap, Ease, Elevation, Grove, Grow, HAIRLINE, Leaf, Line, Motion, Palette, Place, Point, Source,
    Timing,
};

use crate::silhouette::Silhouette;

/// One dash of a section line, and the gap after it, in unit space.
const DASH: (f32, f32) = (0.030, 0.018);
/// How thick a section line is drawn.
const WEIGHT: f32 = HAIRLINE * 3.5;
/// How far past the outline a section line runs at each end, in unit space, so it reads as a cut
/// across the shape rather than a seam inside it.
const OVER: f32 = 0.05;
/// How long one dash takes to come or go, and how long the line takes to be drawn end to end.
const DASH_MS: u64 = 200;
const SWEEP: f32 = 320.0;

/// A straight line across a shape, and a point on the side of it to keep: three points of the
/// sketch, as fractions of the page it was traced on.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Cut {
    pub from: (f32, f32),
    pub to: (f32, f32),
    pub keep: (f32, f32),
}

impl Cut {
    /// The line through `from` and `to`, keeping the side `keep` is on.
    pub const fn new(from: (f32, f32), to: (f32, f32), keep: (f32, f32)) -> Self {
        Self { from, to, keep }
    }
}

impl Silhouette {
    /// The part of the shape on `cut`'s kept side, in the same unit space -- what is cut off, cut
    /// off, and what crosses the line cut to it. One edge of the clip at a time, which is all a
    /// straight cut is.
    pub fn cut(&self, cut: Cut) -> Silhouette {
        let (from, to, keep) = (self.at(cut.from), self.at(cut.to), self.at(cut.keep));
        let side =
            |(x, y): (f32, f32)| (to.0 - from.0) * (y - from.1) - (to.1 - from.1) * (x - from.0);
        let sign = side(keep).signum();
        let inside = |point: (f32, f32)| side(point) * sign >= 0.0;
        let crossing = |a: (f32, f32), b: (f32, f32)| {
            let (sa, sb) = (side(a), side(b));
            let at = sa / (sa - sb);
            (a.0 + (b.0 - a.0) * at, a.1 + (b.1 - a.1) * at)
        };
        let mut kept = Vec::with_capacity(self.outline.len() + 2);
        let mut previous = *self.outline.last().expect("an outline has vertices");
        for &point in &self.outline {
            match (inside(previous), inside(point)) {
                (true, true) => kept.push(point),
                (true, false) => kept.push(crossing(previous, point)),
                (false, true) => {
                    kept.push(crossing(previous, point));
                    kept.push(point);
                }
                (false, false) => {}
            }
            previous = point;
        }
        self.outlined(kept)
    }

    /// Where `cut`'s line enters the outline and where it last leaves it, in unit space: the two
    /// crossings furthest apart along the line. `None` for a line that misses it.
    pub fn crossings(&self, cut: Cut) -> Option<((f32, f32), (f32, f32))> {
        let (from, to) = (self.at(cut.from), self.at(cut.to));
        let side =
            |(x, y): (f32, f32)| (to.0 - from.0) * (y - from.1) - (to.1 - from.1) * (x - from.0);
        let along =
            |(x, y): (f32, f32)| (to.0 - from.0) * (x - from.0) + (to.1 - from.1) * (y - from.1);
        let mut first: Option<((f32, f32), f32)> = None;
        let mut last: Option<((f32, f32), f32)> = None;
        let mut previous = *self.outline.last()?;
        for &point in &self.outline {
            let (sa, sb) = (side(previous), side(point));
            if (sa >= 0.0) != (sb >= 0.0) {
                let at = sa / (sa - sb);
                let crossing = (
                    previous.0 + (point.0 - previous.0) * at,
                    previous.1 + (point.1 - previous.1) * at,
                );
                let reach = along(crossing);
                if first.is_none_or(|(_, least)| reach < least) {
                    first = Some((crossing, reach));
                }
                if last.is_none_or(|(_, most)| reach > most) {
                    last = Some((crossing, reach));
                }
            }
            previous = point;
        }
        Some((first?.0, last?.0))
    }

    /// Which way the side `cut` cuts away lies, as a direction of unit length in unit space: the
    /// line's perpendicular, turned away from what it keeps. What is laid off the shape beyond a
    /// cut -- the room the cut made -- is laid this way.
    pub fn facing(&self, cut: Cut) -> (f32, f32) {
        let (from, to, keep) = (self.at(cut.from), self.at(cut.to), self.at(cut.keep));
        let (nx, ny) = (from.1 - to.1, to.0 - from.0);
        let (nx, ny) = match nx * (keep.0 - from.0) + ny * (keep.1 - from.1) > 0.0 {
            true => (-nx, -ny),
            false => (nx, ny),
        };
        let length = nx.hypot(ny).max(f32::EPSILON);
        (nx / length, ny / length)
    }
}

/// A cut's line, dashed across a shape, drawn and taken away.
pub struct Section {
    dashes: Vec<Leaf>,
}

impl Section {
    /// The line of `cut` across `shape`, dashed off `root` -- the box the shape is grown into --
    /// from where it enters the outline to where it leaves, and a little past both. Unseen. No
    /// dashes, for a cut that misses the shape.
    pub fn grow(grove: &mut Grove, root: Leaf, shape: &Silhouette, cut: Cut) -> Self {
        let Some((a, b)) = shape.crossings(cut) else {
            return Self { dashes: Vec::new() };
        };
        let length = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
        let over = ((b.0 - a.0) / length * OVER, (b.1 - a.1) / length * OVER);
        let (a, b) = ((a.0 - over.0, a.1 - over.1), (b.0 + over.0, b.1 + over.1));
        let length = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
        let count = (length / (DASH.0 + DASH.1)).floor().max(1.0);
        let stride = length / count;
        let aspect = shape.aspect();
        let at = |along: f32| {
            let t = along / length;
            (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)
        };
        let spot = |(x, y): (f32, f32)| Point::new((x * 100.0).pct(), (y / aspect * 100.0).pct());
        let dashes = (0..count as usize)
            .map(|n| {
                let from = n as f32 * stride;
                grove.branch(
                    root,
                    Line::new()
                        .between(spot(at(from)), spot(at(from + stride - DASH.1)))
                        .weight(WEIGHT)
                        .color(Palette::Ink)
                        .cap(Cap::Round)
                        .opacity(0.0)
                        .elevate(Elevation::up(2)),
                )
            })
            .collect();
        Self { dashes }
    }

    /// Draws the line, dash by dash from one end, or takes it away from the other.
    pub fn draw(&self, grove: &mut Grove, drawn: bool) {
        let count = self.dashes.len().max(1) as f32;
        for (n, &dash) in self.dashes.iter().enumerate() {
            let front = match drawn {
                true => n as f32 / count,
                false => 1.0 - n as f32 / count,
            };
            grove.animate(
                dash,
                Motion::Opacity(if drawn { 1.0 } else { 0.0 }),
                Timing::ms(DASH_MS)
                    .after((front * SWEEP) as u64)
                    .ease(Ease::Decelerate),
            );
        }
    }
}
